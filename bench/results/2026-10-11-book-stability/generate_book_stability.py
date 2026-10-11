#!/usr/bin/env python3
"""Book-side rank stability of report 9's claims (sibling of ../2026-10-11-stability).

Scores every report 9 book run on the 3,000-book set and every holdout run on
the 500-book title-only holdout, for 3 embedding models x k in {10, 20, 50} x
3 graph thresholds (54 settings). Each setting is one `shelfmark-bench
bootstrap --negatives` call: 1,000 paired item draws, seed 20261010, the same
negative sample as bench/results/2026-10-11-falsejoin. The threshold sweep is
the one of generate_stability.py: subtract the 25th or 50th percentile of the
positive edge weights from every edge and drop edges that reach zero (q00 is
the published graph).

Steps, run from the repository root:

  score    the 54 bootstrap calls (at most 2 at a time); a compact summary per
           setting goes to settings/, the raw output to --raw (not committed).
           A setting whose summary exists is skipped unless --force.
  publish  results.json and results.md from settings/.

    python3 -I bench/results/2026-10-11-book-stability/generate_book_stability.py all \
        --harness <target>/release/shelfmark-bench
"""
import argparse
import gzip
import hashlib
import itertools
import json
import math
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[2]
R = ROOT / "bench/results"
FJ = R / "2026-10-11-falsejoin"
SCHEME_CACHE = ROOT / "bench/corpus/cache/schemes"
MODELS = ("minilm", "bge-small", "nomic")
KS = (10, 20, 50)
QS = ("q00", "q25", "q50")
UNIVERSES = {"books": "bench/corpus/holdout-books.jsonl", "holdout": "bench/corpus/holdout.jsonl",
             "corpus": "bench/corpus/corpus.jsonl"}
SEED = 20261010
RESAMPLES = 1000
LAMBDA = 0.002
BETA = 1.0
RHO = 0.8
TOL = 1e-9
DRAW_METRICS = "cost_norm,fjs_norm,nav_log_a1,coverage"
# Objectives, all lower-is-better: C, J = C + λN, C + β·FJs and J′ = C + β·FJs + λN.
OBJECTIVES = ("C", "J", "C+FJs", "J'")
PRIMARY = ("C", "J'")
COVERAGE_FLOOR = 0.80
FLOORS = (0.70, 0.80, 0.85, 0.90, 0.95)
# Placed runs whose scheme may ship inside the MIT/Apache binary: LCC (US
# government work), FDC and BK (CC0), and the project's own learned scheme.
# UNESCO (CC BY-SA, share-alike) and ILC2 (licence unsettled) may not.
BUNDLEABLE = ("fdc-placed", "lcc-placed", "lcc-lcsh-placed", "lcc-full-placed", "bk-placed", "learned-placed")
# Report 9's minimax-regret structures (`fj-cuts`), over λ ∈ [0.0005, 0.005] and β ∈ [0.5, 2].
REGRET_RUNS = ("ddc-section", "ddc-d2", "fdc-d3", "lcc-gold")
LAMBDAS = tuple(0.0005 * i for i in range(1, 11))
BETAS = (0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0)


def objective(name, c, fjs, nav):
    return {"C": c, "J": c + LAMBDA * nav, "C+FJs": c + BETA * fjs, "J'": c + BETA * fjs + LAMBDA * nav}[name]


# Report 9's book claims. Pair claims: (a, b, sign of a - b each objective
# claims; 0 = the report claims a tie). Rankings: run sets whose order the
# report relies on.
BOOK_PAIRS = [
    ("dewey-vs-lcc", "Dewey sections beat LCC (§4.2, §6.1)", "ddc-section", "lcc-gold",
     {"C": -1, "J": -1, "C+FJs": 0, "J'": 0}),
    ("section-vs-fdc-d3", "Dewey sections beat FDC depth 3 (§5.1, §6.2)", "ddc-section", "fdc-d3",
     {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("full-vs-section", "Full-depth Dewey gains cost but loses on J (§6.1)", "ddc-gold", "ddc-section",
     {"C": -1, "J": 1, "C+FJs": -1, "J'": 1}),
    ("cut-3-vs-2", "Three FDC levels beat two at λ = 0.002 (§6.2)", "fdc-d3", "fdc-d2",
     {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("cut-3-vs-full", "Three FDC levels beat the full FDC number at λ = 0.002 (§6.2)", "fdc-d3", "fdc-ddc",
     {"C": 1, "J": -1, "C+FJs": 1, "J'": -1}),
    ("dewey-cut-3-vs-2", "Dewey section beats two Dewey digits at λ = 0.002 (§6.2)", "ddc-section", "ddc-d2",
     {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("learned-vs-section", "The learned scheme beats Dewey sections (§4.4)", "learned-own", "ddc-section",
     {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("hybrid1-vs-fdc-d3", "One nested learned level lowers cost and J (§4.4)", "fdc-learned1", "fdc-d3",
     {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("hybrid3-vs-fdc-d3", "Three nested learned levels lower cost but raise J (§4.4)", "fdc-learned3", "fdc-d3",
     {"C": -1, "J": 1, "C+FJs": -1, "J'": 1}),
    ("fdc-placed-vs-lcc-placed", "Through the same classifier, FDC beats LCC (bead rule input)", "fdc-placed",
     "lcc-placed", {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("fdc-placed-vs-lcc-lcsh-placed", "Through the same classifier, FDC beats the bundled LCC + LCSH",
     "fdc-placed", "lcc-lcsh-placed", {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
]
HOLDOUT_PAIRS = [
    ("holdout-work-vs-lcc", "Title-only holdout cannot rank FDC against LCC (§5.2)", "fdc-work", "lcc-gold",
     {"C": 0, "J": 0, "C+FJs": 0, "J'": 0}),
    ("holdout-pipeline", "Holdout pipelines tie (§5.2)", "fdc-pipeline", "lcc-pipeline",
     {"C": 0, "J": 0, "C+FJs": 0, "J'": 0}),
    ("holdout-fdc-placed-vs-lcc-placed", "Through the same classifier, FDC beats LCC (holdout)", "fdc-placed",
     "lcc-placed", {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("holdout-fdc-placed-vs-lcc-lcsh-placed", "Through the same classifier, FDC beats the bundled LCC + LCSH "
     "(holdout)", "fdc-placed", "lcc-lcsh-placed", {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
]
CORPUS_PAIRS = [
    ("corpus-fdc-placed-vs-lcc-lcsh-placed", "Through the same classifier, FDC beats the bundled LCC + LCSH "
     "(papers)", "fdc-placed", "lcc-lcsh-placed", {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
    ("corpus-fdc-placed-vs-lcc-placed", "Through the same classifier, FDC beats LCC (papers)", "fdc-placed",
     "lcc-placed", {"C": -1, "J": -1, "C+FJs": -1, "J'": -1}),
]
PAIRS = {"books": BOOK_PAIRS, "holdout": HOLDOUT_PAIRS, "corpus": CORPUS_PAIRS}
CUTS = ["fdc-d1", "fdc-d2", "fdc-d3", "fdc-ddc", "ddc-d1", "ddc-d2", "ddc-section", "ddc-gold"]
STRUCTURES = ["ddc-section", "ddc-d2", "ddc-gold", "fdc-d3", "fdc-d2", "fdc-ddc", "lcc-gold", "lcc-full-gold"]
PLACED = ["fdc-placed", "lcc-placed", "lcc-lcsh-placed", "lcc-full-placed", "unesco-thesaurus-placed", "ilc2-placed",
          "bk-placed", "learned-placed"]
BOOK_RANKINGS = [
    ("rank-all", "All 37 book runs (composite ranking, §4.1, §4.5)", None),
    ("rank-structures", "Gold-numbered structures: Dewey/FDC cuts and LCC (§6.1, §6.2)", STRUCTURES),
    ("rank-cuts", "Cut depth: 1, 2, 3 levels and full, FDC and Dewey (§6.2)", CUTS),
    ("rank-learned", "Learned scheme and hybrids against their backbone (§4.4)",
     ["learned-own", "fdc-learned1", "fdc-learned2", "fdc-learned3", "fdc-d3", "ddc-section"]),
    ("rank-pipelines", "Availability pipelines, FDC and LCC (§5.3)",
     [f"{s}-pipe-{p}" for s in ("fdc", "lcc") for p in (20, 40, 60, 80, 90)]),
    ("rank-placed", "Placed through the lexical classifier (bead rule)", PLACED),
]
HOLDOUT_RANKINGS = [
    ("holdout-rank-all", "All holdout runs common to every graph", None),
    ("holdout-rank-placed", "Holdout, placed through the lexical classifier (bead rule)", PLACED),
]
CORPUS_RANKINGS = [
    ("corpus-rank-placed", "Papers, placed through the lexical classifier (bead rule)", None),
]
RANKINGS = {"books": BOOK_RANKINGS, "holdout": HOLDOUT_RANKINGS, "corpus": CORPUS_RANKINGS}


def rel(path):
    return str(Path(path).resolve().relative_to(ROOT))


def read_json(path):
    raw = Path(path).read_bytes()
    return json.loads(gzip.decompress(raw) if str(path).endswith(".gz") else raw)


def graph_source(universe, model, k):
    if universe == "books":
        if k == 20:
            return R / f"2026-10-11-metrics/books/graphs/{model}-books-k20.json"
        return R / f"2026-10-10-proposal/graphs/{model}-books-k{k}.json.gz"
    if k == 20:
        return R / f"2026-10-11-metrics/embed/graphs/{model}-{universe}-k20.json"
    return R / f"2026-10-10-embed/graphs/{model}-{universe}-k{k}.json.gz"


def negatives_source(universe, model, k):
    # Holdout and paper negatives exist for k = 20 only. They are pairs with cosine below
    # τ (the mean pair cosine, the same at every k), which are never kNN edges
    # at any k, so the k = 20 sample is valid for k = 10 and 50; score() checks τ.
    return FJ / f"negatives/{universe}-{model}-k{k if universe == 'books' else 20}.json.gz"


def run_specs(universe, model):
    """The runs of the published false-join bootstrap of this universe and model at
    k = 20, plus Dewey cuts on the holdout; fetch-and-build schemes from the cache."""
    spec = read_json(FJ / f"bootstrap/{universe}-{model}-k20.json.gz")["runs_spec"]
    if universe == "corpus":  # papers: the placed runs only, for the bead rule
        spec = [r for r in spec if r["key"].endswith("-placed")]
    runs = []
    for r in spec:
        scheme = r["scheme"]
        if scheme.startswith("build_schemes.sh:"):
            scheme = rel(SCHEME_CACHE / scheme.split(":", 1)[1]) if (SCHEME_CACHE / scheme.split(":", 1)[1]).exists() \
                else None
            if scheme is None:
                raise SystemExit(f"{r['key']}: run crates/bench/scripts/build_schemes.sh {r['scheme'].split(':')[1][:-5]}")
        runs.append({"key": r["key"], "scheme": scheme, "placements": r["placements"]})
    # The scheme the CLI bundles (LCC with LCSH vocabulary), placed by the same
    # classifier; its class tree is lcc.json's.
    runs.append({"key": "lcc-lcsh-placed", "scheme": "data/schemes/lcc/lcc.json",
                 "placements": rel(OUT / f"placements/{universe}/lcc-lcsh-placed.jsonl")})
    if universe == "holdout":
        keys = {r["key"] for r in runs}
        for key in ("ddc-section", "ddc-gold"):
            if key not in keys:
                runs.append({"key": key, "scheme": "bench/results/2026-10-10-ddc/schemes/ddc.json",
                             "placements": f"bench/results/2026-10-10-ddc/placements/holdout/{key}.jsonl"})
    return runs


def percentile(values, p):
    values = sorted(values)
    pos = (len(values) - 1) * p
    lo = int(pos)
    hi = min(lo + 1, len(values) - 1)
    return values[lo] + (values[hi] - values[lo]) * (pos - lo)


def threshold(graph, q):
    weights = sorted(e["weight"] for e in graph["edges"] if e["weight"] > 0)
    offset = {"q00": 0.0, "q25": percentile(weights, .25), "q50": percentile(weights, .50)}[q]
    if offset == 0.0:
        return graph, 0.0
    edges = [dict(e, weight=e["weight"] - offset) for e in graph["edges"] if e["weight"] > offset]
    return dict(graph, tau=graph.get("tau", 0.0) + offset, edges=[e for e in edges if e["weight"] > 0]), offset


def setting_name(universe, model, k, q):
    return f"{universe}-{model}-k{k}-{q}"


def summarise(boot, draws, pairs):
    """Point and 95% CI of every objective per run, and paired deltas of the claim pairs."""
    runs, per_draw = {}, {}
    for key, run in boot["runs"].items():
        p = run["point"]
        d = draws["runs"][key]
        series = {o: [objective(o, c, f, n) for c, f, n in zip(d["cost_norm"], d["fjs_norm"], d["nav_log_a1"])]
                  for o in OBJECTIVES}
        per_draw[key] = series
        runs[key] = {"coverage": p["coverage"], "cost_norm": p["cost_norm"], "fjs_norm": p["fjs_norm"],
                     "nav_log_a1": p["nav_log_a1"], "skill": p["skill"], "skill_ci": run["ci"]["skill"],
                     "point": {o: objective(o, p["cost_norm"], p["fjs_norm"], p["nav_log_a1"]) for o in OBJECTIVES},
                     "ci": {o: [percentile(series[o], .025), percentile(series[o], .975)] for o in OBJECTIVES}}
    deltas = {}
    placed = [k for k in PLACED if k in runs]
    pairs = list(pairs) + [(f"placed:{a}|{b}", None, a, b, None) for a, b in itertools.combinations(placed, 2)]
    for cid, _, a, b, _ in pairs:
        if a not in runs or b not in runs:
            continue
        deltas[cid] = {}
        for o in OBJECTIVES:
            diff = [x - y for x, y in zip(per_draw[a][o], per_draw[b][o])]
            below = sum(v < 0 for v in diff) / len(diff)
            above = sum(v > 0 for v in diff) / len(diff)
            deltas[cid][o] = {"diff": runs[a]["point"][o] - runs[b]["point"][o],
                              "ci": [percentile(diff, .025), percentile(diff, .975)],
                              "p": min(1.0, 2 * min(below, above) + 1 / len(diff))}
    return runs, deltas


def score_one(a, universe, model, k, q):
    name = setting_name(universe, model, k, q)
    out = OUT / f"settings/{name}.json"
    if out.exists() and not a.force:
        return name, "kept"
    pairs = PAIRS[universe]
    specs = run_specs(universe, model)
    source, negfile = graph_source(universe, model, k), negatives_source(universe, model, k)
    graph, neg = read_json(source), read_json(negfile)
    if abs(graph["tau"] - neg["tau"]) > 1e-8:
        raise SystemExit(f"{name}: graph τ {graph['tau']} differs from the negative floor {neg['tau']}")
    varied, offset = threshold(graph, q)
    raw = Path(a.raw)
    raw.mkdir(parents=True, exist_ok=True)
    if a.from_raw and (raw / f"{name}.draws.json.gz").exists():
        boot, draws = read_json(raw / f"{name}.boot.json.gz"), read_json(raw / f"{name}.draws.json.gz")
    else:
        boot, draws = bootstrap(a, universe, varied, neg, specs, raw, name)
    return name, write_setting(universe, model, k, q, boot, draws, specs, source, negfile, offset, varied, pairs)


def bootstrap(a, universe, varied, neg, specs, raw, name):
    with tempfile.TemporaryDirectory(prefix="book-stability-", dir="/work/artifacts") as tmp:
        tmp = Path(tmp)
        (tmp / "graph.json").write_text(json.dumps(varied, separators=(",", ":")))
        (tmp / "negatives.json").write_text(json.dumps(neg, separators=(",", ":")))
        (tmp / "runs.json").write_text(json.dumps(specs))
        subprocess.run([a.harness, "bootstrap", "--corpus", UNIVERSES[universe], "--graph", tmp / "graph.json",
                        "--runs", tmp / "runs.json", "--negatives", tmp / "negatives.json",
                        "--resamples", str(a.resamples), "--seed", str(SEED), "--threads", str(a.threads),
                        "--output", tmp / "boot.json", "--draws-out", tmp / "draws.json",
                        "--draw-metrics", DRAW_METRICS], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
        boot, draws = read_json(tmp / "boot.json"), read_json(tmp / "draws.json")
        for f in ("boot", "draws"):
            (raw / f"{name}.{f}.json.gz").write_bytes(gzip.compress((tmp / f"{f}.json").read_bytes(), mtime=0))
    return boot, draws


def write_setting(universe, model, k, q, boot, draws, specs, source, negfile, offset, varied, pairs):
    name = setting_name(universe, model, k, q)
    out = OUT / f"settings/{name}.json"
    checked = 0
    if q == "q00":
        published = FJ / f"bootstrap/{universe}-{model}-k{k}.json.gz"
        if published.exists():
            prior = read_json(published)["runs"]
            for key, run in prior.items():
                if key not in boot["runs"]:
                    continue
                for m in ("cost_norm", "fjs_norm", "nav_log_a1"):
                    if abs(run["point"][m] - boot["runs"][key]["point"][m]) > TOL:
                        raise SystemExit(f"{name} {key}: {m} differs from {rel(published)}")
                checked += 1
    runs, deltas = summarise(boot, draws, pairs)
    doc = {"setting": {"universe": universe, "model": model, "k": k, "threshold": q},
           "graph": rel(source), "graph_sha256": hashlib.sha256(Path(source).read_bytes()).hexdigest(),
           "threshold_offset": offset, "edges": len(varied["edges"]), "negatives": rel(negfile),
           "resamples": boot["resamples"], "seed": boot["seed"],
           "published_points_reproduced": checked,
           "runs_spec": specs, "runs": runs, "deltas": deltas}
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(doc, indent=1, sort_keys=True) + "\n")
    return f"{len(runs)} runs, {len(varied['edges'])} edges, {checked} published points reproduced"


def score(a):
    todo = [(u, m, k, q) for u in UNIVERSES for m in MODELS for k in KS for q in QS
            if not a.universe or u == a.universe]
    with ThreadPoolExecutor(max_workers=min(a.jobs, 2)) as pool:
        for i, (name, status) in enumerate(pool.map(lambda t: score_one(a, *t), todo), 1):
            print(f"[{i}/{len(todo)}] {name}: {status}", flush=True)


# ---------- publish ----------

def ranks(values):
    order = sorted(range(len(values)), key=lambda i: values[i])
    out = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for z in range(i, j + 1):
            out[order[z]] = (i + j) / 2 + 1
        i = j + 1
    return out


def spearman(x, y):
    rx, ry = ranks(x), ranks(y)
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = math.sqrt(sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry))
    return num / den if den else 1.0


def ranking_check(settings, members, o):
    names = sorted(settings)
    common = set.intersection(*(set(settings[s]["runs"]) for s in names))
    members = sorted(common if members is None else [m for m in members if m in common])
    vectors = {s: [settings[s]["runs"][m]["point"][o] for m in members] for s in names}
    pairs = [(spearman(vectors[x], vectors[y]), x, y) for x, y in itertools.combinations(names, 2)]
    worst = min(pairs)
    leaders = {}
    for s in names:
        lead = members[min(range(len(members)), key=lambda i: vectors[s][i])]
        leaders[lead] = leaders.get(lead, 0) + 1
    return {"members": members, "settings": len(names), "pairs": len(pairs),
            "min_spearman": worst[0], "min_pair": [worst[1], worst[2]],
            "pairs_below_threshold": sum(r < RHO for r, _, _ in pairs),
            "median_spearman": percentile([r for r, _, _ in pairs], .5),
            "passes": worst[0] >= RHO, "leader_counts": dict(sorted(leaders.items(), key=lambda kv: -kv[1]))}


def pair_check(settings, cid, claimed, o):
    rows = [(s, settings[s]["deltas"][cid][o]) for s in sorted(settings) if cid in settings[s]["deltas"]]
    n = len(rows)
    neg = sum(r["diff"] < 0 for _, r in rows)
    sig_neg = sum(r["ci"][1] < 0 for _, r in rows)
    sig_pos = sum(r["ci"][0] > 0 for _, r in rows)
    if claimed:
        match = neg if claimed < 0 else n - neg
        sig_for, sig_against = (sig_neg, sig_pos) if claimed < 0 else (sig_pos, sig_neg)
        verdict = ("STABLE" if match == n and sig_against == 0 else
                   "MOSTLY" if match / n >= .8 and sig_against == 0 else "NOT STABLE")
    else:
        match = n - sig_neg - sig_pos  # a tie claim matches where the CI covers zero
        sig_for, sig_against = None, sig_neg + sig_pos
        verdict = "STABLE" if sig_against == 0 else "NOT STABLE"
    diffs = [r["diff"] for _, r in rows]
    return {"claimed_sign": claimed, "settings": n, "share_sign_matches": match / n,
            "a_lower_settings": neg, "significant_a_lower": sig_neg, "significant_a_higher": sig_pos,
            "significant_for_claim": sig_for, "significant_against_claim": sig_against,
            "diff_range": [min(diffs), max(diffs)], "verdict": verdict,
            "by_setting": {s: r for s, r in rows}}


def bead_rule(settings, floor, allowed=None):
    """Per setting: placed runs with coverage ≥ floor; the highest skill wins, and
    runs whose paired skill difference to it is not significant (its 95% CI covers
    zero) tie with it, broken by the lowest nav_log_a1. Skill shares each draw's
    reference with C, so a skill difference is significant exactly when the C
    difference is."""
    out = {}
    for s in sorted(settings):
        runs, deltas = settings[s]["runs"], settings[s]["deltas"]
        eligible = [k for k in PLACED if k in runs and runs[k]["coverage"] >= floor
                    and (allowed is None or k in allowed)]
        if not eligible:
            out[s] = {"eligible": [], "leader": None, "tied": [], "winner": None}
            continue
        leader = max(eligible, key=lambda k: runs[k]["skill"])

        def tied(k):
            if k == leader:
                return True
            d = deltas.get(f"placed:{leader}|{k}") or deltas.get(f"placed:{k}|{leader}")
            return d["C"]["ci"][0] <= 0 <= d["C"]["ci"][1]
        ties = [k for k in eligible if tied(k)]
        winner = min(ties, key=lambda k: (runs[k]["nav_log_a1"], k))
        out[s] = {"eligible": eligible, "leader": leader, "tied": ties, "winner": winner,
                  "skill": {k: [runs[k]["skill"]] + runs[k]["skill_ci"] for k in eligible},
                  "nav_log_a1": {k: runs[k]["nav_log_a1"] for k in eligible},
                  "J'": {k: runs[k]["point"]["J'"] for k in eligible}}
    return out


def counts(rows, field):
    out = {}
    for r in rows.values():
        out[r[field] or "none"] = out.get(r[field] or "none", 0) + 1
    return dict(sorted(out.items(), key=lambda kv: -kv[1]))


def regret(settings, members, only=None):
    """Max regret of each structure over settings × λ × β of J′ = C + β·FJs + λ·N."""
    worst = {m: 0.0 for m in members}
    for s, d in settings.items():
        if only and d["setting"]["threshold"] not in only:
            continue
        r = d["runs"]
        for lam, beta in itertools.product(LAMBDAS, BETAS):
            j = {m: r[m]["cost_norm"] + beta * r[m]["fjs_norm"] + lam * r[m]["nav_log_a1"] for m in members}
            best = min(j.values())
            for m in members:
                worst[m] = max(worst[m], j[m] - best)
    return worst


def publish(_a):
    universes = {}
    for u in UNIVERSES:
        files = sorted((OUT / "settings").glob(f"{u}-*.json"))
        universes[u] = {f.stem.removeprefix(f"{u}-"): json.loads(f.read_text()) for f in files}
    claims = []
    for u in UNIVERSES:
        settings = universes[u]
        if not settings:
            continue
        for cid, text, a, b, signs in PAIRS[u]:
            claims.append({"id": cid, "universe": u, "claim": text, "kind": "pair", "a": a, "b": b,
                           "objectives": {o: pair_check(settings, cid, signs[o], o) for o in OBJECTIVES}})
        for cid, text, members in RANKINGS[u]:
            claims.append({"id": cid, "universe": u, "claim": text, "kind": "ranking",
                           "objectives": {o: ranking_check(settings, members, o) for o in OBJECTIVES}})
    bead = {}
    for u, settings in universes.items():
        if not settings:
            continue
        bead[u] = {"rows": bead_rule(settings, COVERAGE_FLOOR),
                   "rows_bundleable": bead_rule(settings, COVERAGE_FLOOR, BUNDLEABLE),
                   "floors": {f"{f:.2f}": {"all": counts(bead_rule(settings, f), "winner"),
                                           "bundleable": counts(bead_rule(settings, f, BUNDLEABLE), "winner")}
                              for f in FLOORS}}
        bead[u]["winners"] = counts(bead[u]["rows"], "winner")
        bead[u]["leaders"] = counts(bead[u]["rows"], "leader")
        bead[u]["winners_bundleable"] = counts(bead[u]["rows_bundleable"], "winner")
        bead[u]["coverage"] = {k: v["coverage"] for k, v in next(iter(settings.values()))["runs"].items()
                               if k in PLACED}
    books = universes["books"]
    regrets = {"members": list(REGRET_RUNS), "lambda": [LAMBDAS[0], LAMBDAS[-1]], "beta": [BETAS[0], BETAS[-1]],
               "published_graphs_only": regret(books, REGRET_RUNS, {"q00"}),
               "all_settings": regret(books, REGRET_RUNS),
               "all_settings_with_lcc_full": regret(books, REGRET_RUNS + ("lcc-full-gold",))} if books else {}
    result = {
        "title": "Book-side rank stability of report 9's claims",
        "settings": {"models": list(MODELS), "k": list(KS), "thresholds": list(QS),
                     "threshold_definition": "subtract the 25th or 50th percentile of positive edge weights from "
                                             "every kNN edge and drop edges that reach zero; q00 is the published graph",
                     "universes": dict(UNIVERSES),
                     "per_universe": {u: len(universes[u]) for u in universes}},
        "objectives": {"C": "cost_norm", "J": f"cost_norm + {LAMBDA} * nav_log_a1",
                       "C+FJs": f"cost_norm + {BETA:g} * fjs_norm",
                       "J'": f"cost_norm + {BETA:g} * fjs_norm + {LAMBDA} * nav_log_a1"},
        "bootstrap": {"resamples": RESAMPLES, "seed": SEED, "intervals": "95% percentile, paired item draws",
                      "p": "two-sided bootstrap share, 2 * min(share below 0, share above 0) + 1/resamples"},
        "thresholds_for_claims": {"spearman": RHO,
                                  "pair_verdict": "STABLE: the claimed sign in every setting and no significant "
                                                  "reversal; MOSTLY: the claimed sign in at least 80% of settings and "
                                                  "no significant reversal; tie claims are STABLE when no setting's "
                                                  "95% CI excludes zero"},
        "coverage_floor": COVERAGE_FLOOR, "bundleable": list(BUNDLEABLE),
        "claims": claims, "bead_rule": bead, "regret": regrets,
        "points": {u: {s: {k: {"coverage": r["coverage"], "skill": r["skill"], "nav_log_a1": r["nav_log_a1"],
                               **r["point"]} for k, r in d["runs"].items()} for s, d in universes[u].items()}
                   for u in universes},
        "reproduced": {u: sum(d["published_points_reproduced"] for d in universes[u].values()) for u in universes},
    }
    (OUT / "results.json").write_text(json.dumps(result, indent=1, sort_keys=True) + "\n")
    write_markdown(result)
    print(f"published {len(claims)} claims")


def f4(v):
    return f"{v:+.4f}"


def fmt_counts(c):
    return ", ".join(f"`{k}` {v}" if k != "none" else f"none {v}" for k, v in c.items())


def write_markdown(res):
    n = res["settings"]["per_universe"]
    L = ["# Book-side rank stability of report 9's claims", "",
         "Generated by `generate_book_stability.py publish`; method in `REPRODUCE.md`. Settings: 3 embedding "
         "models × k ∈ {10, 20, 50} × 3 graph thresholds (q00 = published graph, q25, q50), "
         f"{n.get('books', 0)} each for the 3,000 books, the 500-book title-only holdout "
         f"({n.get('holdout', 0)}) and, for the placed runs only, the 10,000 papers ({n.get('corpus', 0)}). "
         "Objectives (lower is better): C = cost_norm; J = C + 0.002·N; C + FJs; J′ = C + FJs + 0.002·N, "
         "with N = nav_log_a1 and FJs the specificity false-join term at β = 1. Intervals are 95% paired "
         "item-bootstrap percentiles (1,000 draws, seed 20261010). Every unthresholded setting reproduces the "
         f"published false-join points to 1e-9 ({sum(res['reproduced'].values())} run × setting checks).", "",
         "A ranking claim passes when the minimum Spearman ρ over all setting pairs is ≥ 0.8. A pair claim is "
         "**STABLE** when the paired difference has the claimed sign in every setting and no setting's 95% CI "
         "excludes zero the other way; **MOSTLY** when the sign matches in ≥ 80% of settings with no significant "
         "reversal; otherwise **NOT STABLE**. A tie claim is stable when no setting's CI excludes zero. "
         "`lcc-lcsh-placed` is the scheme the `shelfmark` binary bundles (the LCC outline plus LCSH vocabulary) "
         "placed by the same classifier as the other `-placed` runs (`placements/`).", ""]
    for u in UNIVERSES:
        claims = [c for c in res["claims"] if c["universe"] == u]
        if not claims:
            continue
        L += [f"## {u}: pair claims", "",
              "| claim | a − b | objective | claimed | sign matches | sig. for / against | Δ range | verdict |",
              "| --- | --- | --- | :---: | ---: | ---: | --- | --- |"]
        for c in claims:
            if c["kind"] != "pair":
                continue
            for o in OBJECTIVES:
                x = c["objectives"][o]
                sign = {-1: "a < b", 1: "a > b", 0: "tie"}[x["claimed_sign"]]
                sig = (f"{x['significant_for_claim']} / {x['significant_against_claim']}" if x["claimed_sign"]
                       else f"– / {x['significant_against_claim']}")
                L.append(f"| {c['claim']} | `{c['a']}` − `{c['b']}` | {o} | {sign} | "
                         f"{round(x['share_sign_matches'] * x['settings'])}/{x['settings']} | {sig} | "
                         f"[{f4(x['diff_range'][0])}, {f4(x['diff_range'][1])}] | {x['verdict']} |")
        L += ["", f"## {u}: ranking claims", "",
              "| claim | runs | objective | min ρ | pairs < 0.8 | median ρ | leader (settings) | result |",
              "| --- | ---: | --- | ---: | ---: | ---: | --- | --- |"]
        for c in claims:
            if c["kind"] != "ranking":
                continue
            for o in OBJECTIVES:
                x = c["objectives"][o]
                lead = ", ".join(f"`{k}` ({v})" for k, v in list(x["leader_counts"].items())[:3])
                L.append(f"| {c['claim']} | {len(x['members'])} | {o} | {x['min_spearman']:.3f} | "
                         f"{x['pairs_below_threshold']}/{x['pairs']} | {x['median_spearman']:.3f} | {lead} | "
                         f"{'PASS' if x['passes'] else 'NOT CLEARED'} |")
        L.append("")
    if res["regret"]:
        g = res["regret"]
        L += ["## Minimax regret of the gold-numbered structures", "",
              "Max regret of J′ = C + β·FJs + λ·N over λ ∈ [0.0005, 0.005] (step 0.0005) and β ∈ [0.5, 2] "
              "(step 0.25), as in report 9's `fj-cuts`, on the 3,000 books. Lower is better.", "",
              "| structure | 9 published graphs (q00) | all 27 settings | all 27, with `lcc-full-gold` |",
              "| --- | ---: | ---: | ---: |"]
        for m in g["members"] + ["lcc-full-gold"]:
            a = g["published_graphs_only"].get(m)
            b = g["all_settings"].get(m)
            L.append(f"| `{m}` | {'' if a is None else f'{a:.4f}'} | {'' if b is None else f'{b:.4f}'} | "
                     f"{g['all_settings_with_lcc_full'][m]:.4f} |")
        L.append("")
    L += [f"## Bead rule: highest placed skill, coverage ≥ {res['coverage_floor']:.2f}, ties by nav_log_a1", "",
          "Placed runs only: every scheme placed by the same lexical classifier on the same items. Skill is "
          "(1 − C) / (1 − C_ref) against the setting's average-linkage reference, so within a setting it orders "
          "runs as C does. A run ties with the highest-skill run when their paired difference's 95% CI covers "
          "zero; the tied run with the lowest nav_log_a1 wins. *Bundleable* keeps the schemes the binary may "
          "ship: " + ", ".join(f"`{k}`" for k in res["bundleable"]) + ".", "",
          "| universe | coverage of each placed run | highest skill | winner | winner, bundleable only |",
          "| --- | --- | --- | --- | --- |"]
    for u, b in res["bead_rule"].items():
        cov = ", ".join(f"`{k}` {v:.3f}" for k, v in sorted(b["coverage"].items(), key=lambda kv: -kv[1]))
        L.append(f"| {u} | {cov} | {fmt_counts(b['leaders'])} | {fmt_counts(b['winners'])} | "
                 f"{fmt_counts(b['winners_bundleable'])} |")
    L += ["", "Winners by coverage floor (settings won):", "", "| universe | floor | winner | winner, bundleable only |",
          "| --- | ---: | --- | --- |"]
    for u, b in res["bead_rule"].items():
        for f, w in b["floors"].items():
            L.append(f"| {u} | {f} | {fmt_counts(w['all'])} | {fmt_counts(w['bundleable'])} |")
    L += ["", "Per setting, bundleable only (skill with its 95% CI; tied runs in brackets):", "",
          "| universe | setting | highest skill | skill | tied | winner | nav bits | J′ |",
          "| --- | --- | --- | --- | --- | --- | ---: | ---: |"]
    for u, b in res["bead_rule"].items():
        for s, r in b["rows_bundleable"].items():
            if not r["leader"]:
                L.append(f"| {u} | {s} | none above the floor | | | | | |")
                continue
            sk = r["skill"][r["leader"]]
            w = r["winner"]
            jp = r["J'"][w]
            L.append(f"| {u} | {s} | `{r['leader']}` | {sk[0]:.3f} [{sk[1]:.3f}, {sk[2]:.3f}] | "
                     f"{', '.join(f'`{k}`' for k in r['tied'] if k != r['leader']) or '–'} | `{w}` | "
                     f"{r['nav_log_a1'][w]:.2f} | {jp:.4f} |")
    L += ["", "Per-setting points, CIs and paired deltas are in `settings/`; every claim's per-setting deltas are "
          "in `results.json` (`claims[].objectives[].by_setting`).", ""]
    (OUT / "results.md").write_text("\n".join(L))


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("step", choices=["score", "summarise", "publish", "all"])
    p.add_argument("--harness", default="/cargo-target/release/shelfmark-bench")
    p.add_argument("--threads", type=int, default=7)
    p.add_argument("--jobs", type=int, default=2, help="concurrent scoring jobs (at most 2)")
    p.add_argument("--resamples", type=int, default=RESAMPLES)
    p.add_argument("--raw", default="/work/artifacts/book-stability/raw")
    p.add_argument("--universe", choices=list(UNIVERSES))
    p.add_argument("--force", action="store_true")
    a = p.parse_args()
    a.from_raw = a.step == "summarise"
    if a.from_raw:
        a.force = True
    for step in (["score", "publish"] if a.step in ("all", "summarise") else [a.step]):
        globals()[step](a)


if __name__ == "__main__":
    main()
