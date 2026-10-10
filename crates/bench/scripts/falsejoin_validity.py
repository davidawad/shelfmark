#!/usr/bin/env python3
"""Validity of the false-join metric variants (crates/bench/src/falsejoin.rs).

Subcommands, all deterministic for fixed inputs:

  judge     Re-run the judge agreement of docs/validity on the SAME 953 pairs
            and the SAME stored judge labels (no relabelling) for every metric
            variant: Dasgupta alone, each false-join form alone, and
            C + beta * FJ over the beta sweep. Writes
            docs/validity/falsejoin-results.{md,json}.
  sample    Draw the external pair samples (bibliographic coupling and
            co-citation for the arXiv papers, shared LCSH headings for the
            3,000 books) and store each pair's cosine under each embedding
            model, so `external` needs no vectors. Writes
            bench/results/2026-10-11-falsejoin/external/pairs-<signal>.jsonl.gz.
  external  Agreement of every metric variant with the external signals, per
            pair (AUC and kappa against an external error label) and per
            scheme (Spearman rank correlation across runs between the metric
            and the external separation of the run's tree). Writes
            .../external/agreement.{md,json}.

Per-pair reading of a metric variant (as in docs/validity/README.md, one
reasonable reading of "the metric charges this pair"): a graph edge's
standardised Dasgupta term is z+ = w |leaves(LCA)| / n divided by the run's
mean per edge; a pair with cosine below the floor tau has the standardised
false-join term z- = d f(LCA) divided by the run's mean per negative pair,
with d = tau - cos and f = (n - |leaves|) / n (outside mass) or
ln(n / |leaves|) / ln n (specificity). The variant's score is m (z+ + beta z-),
with m the run's mean Dasgupta charge per edge (Dasgupta alone: m z+, the
pair's raw cost contribution, so the original AUC is reproduced; a false-join
form alone: m z-). It flags the pair when the score exceeds m, the charge of
one average pair of its kind. Only the Python standard library is used.
"""
import argparse
import gzip
import importlib.util
import json
import math
import random
from array import array
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
FJ = ROOT / "bench/results/2026-10-11-falsejoin"
EXT = FJ / "external"
DOCS = ROOT / "docs/validity"
VECTORS = ROOT / "bench/corpus/cache/falsejoin/vectors"
SCHEME_CACHE = ROOT / "bench/corpus/cache/schemes"
SEED = 20261010
BOOT = 2000
BOOT_EXT = 1000
BETAS = [0.25, 0.5, 1.0, 2.0, 4.0, 8.0]
PRIMARY_BETA = 1.0
MODELS = ["minilm", "bge-small", "nomic"]
PER_CLASS = 4000
SPECIFIC_DF = 30
SIGNALS = {"coupling": "corpus", "cocitation": "corpus", "lcsh": "books"}
CORPORA = {"books": ROOT / "bench/corpus/holdout-books.jsonl", "corpus": ROOT / "bench/corpus/corpus.jsonl"}
VECTOR_NAME = {"books": "holdout-books", "corpus": "corpus"}

_spec = importlib.util.spec_from_file_location("validity_check", HERE / "validity_check.py")
vc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(vc)
kappa, spearman, jsonl, rng_for = vc.kappa, vc.spearman, vc.jsonl, vc.rng_for


def variants():
    """(name, form, beta): beta None is the term alone; form None is Dasgupta alone."""
    out = [("dasgupta", None, 0.0), ("fj", "outside", None), ("fjs", "specific", None)]
    for form, short in (("outside", "fj"), ("specific", "fjs")):
        out += [(f"c+{b:g}{short}", form, b) for b in BETAS]
    return out


def run_metric(point, form, beta):
    """A run's value of a variant from its rescored point estimates (lower is better)."""
    if form is None:
        return point["cost_norm"]
    fj = point["fj_norm" if form == "outside" else "fjs_norm"]
    return fj if beta is None else point["cost_norm"] + beta * fj


def auc(scores, positive):
    """Mann-Whitney AUC by average ranks (ties count one half); None if a class is empty."""
    n1 = sum(positive)
    n0 = len(positive) - n1
    if not n1 or not n0:
        return None
    order = sorted(range(len(scores)), key=lambda i: scores[i])
    ranks = [0.0] * len(scores)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and scores[order[j + 1]] == scores[order[i]]:
            j += 1
        for k in range(i, j + 1):
            ranks[order[k]] = (i + j) / 2 + 1
        i = j + 1
    r1 = sum(r for r, p in zip(ranks, positive) if p)
    return (r1 - n1 * (n1 + 1) / 2) / (n1 * n0)


def interval(values):
    v = sorted(x for x in values if x is not None)
    if not v:
        return None
    return [v[int(0.025 * len(v))], v[min(len(v) - 1, int(0.975 * len(v)))]]


def load_boot(name):
    return json.loads(gzip.decompress((FJ / f"bootstrap/{name}.json.gz").read_bytes()))


def means(boot, key):
    """Mean charge per graph edge and per negative pair of each false-join form."""
    p = boot["runs"][key]["point"]
    neg = boot["negatives"]
    return {"outside": p["fj_norm"] * neg["dissimilarity"] / neg["pairs"],
            "specific": p["fjs_norm"] * neg["dissimilarity"] / neg["pairs"]}


def pair_terms(n, mass, cos, tau, w, mean_edge, mu):
    """Standardised Dasgupta term and false-join terms of one pair."""
    zpos = (w * mass / n) / mean_edge if w > 0 else 0.0
    d = max(0.0, tau - cos)
    zout = d * (n - mass) / n / mu["outside"] if mu["outside"] > 0 else 0.0
    zspec = d * math.log(n / mass) / math.log(n) / mu["specific"] if mu["specific"] > 0 else 0.0
    return zpos, {"outside": zout, "specific": zspec}


def score(zpos, zneg, form, beta, scale):
    """The variant's pair score in units of `scale`, the run's mean edge charge."""
    if form is None:
        return scale * zpos
    return scale * (zneg[form] if beta is None else zpos + beta * zneg[form])


# ---------------------------------------------------------------- judge


def judge(_a):
    rows = jsonl(DOCS / "sample.jsonl")
    meta = json.loads((DOCS / "sample-meta.json").read_text())
    boot = load_boot("corpus-minilm-k20")
    if boot["graph_sha256"] != meta["graph_sha256"]:
        raise SystemExit("the rescored corpus graph is not the validity sample's graph")
    tau, n = meta["tau"], 10000
    mu = {key: means(boot, key) for key in meta["runs"]}
    var = variants()
    for r in rows:
        zpos, zneg = pair_terms(n, r["lca_mass"], r["cos"], tau, r["w_sparse"], r["mean_edge_contribution"],
                                mu[r["run"]])
        r["_scale"] = r["mean_edge_contribution"]
        r["_scores"] = [score(zpos, zneg, f, b, r["_scale"]) for _, f, b in var]
        r["_bad"] = r["labels"]["pass1"]["scheme_decision"] == "incorrect"
    # The Dasgupta variant must reproduce the stored metric flag.
    if any((r["_scores"][0] > r["_scale"]) != r["metric_flag"] for r in rows):
        raise SystemExit("Dasgupta variant does not reproduce the stored metric_flag")

    def stats(sample):
        bad = [r["_bad"] for r in sample]
        out = []
        for v in range(len(var)):
            s = [r["_scores"][v] for r in sample]
            flag = [x > r["_scale"] for x, r in zip(s, sample)]
            out.append({"kappa": kappa(bad, flag), "auc": auc(s, bad), "flagged": sum(flag) / len(s)})
        return out

    def boot_stats(sample, tag):
        by = {}
        for r in sample:
            by.setdefault(r["stratum"], []).append(r)
        rng = rng_for("falsejoin-boot", tag)
        reps = []
        for _ in range(BOOT):
            reps.append(stats([rng.choice(v) for s in sorted(by) for v in [by[s]] for _ in v]))
        return reps

    res = {"variants": [{"name": nm, "form": f, "beta": b} for nm, f, b in var], "groups": {},
           "method": "per-pair score z+ + beta z- (see script docstring); flag when > 1; 95% percentile "
                     f"bootstrap, {BOOT} replicates, pairs resampled within strata; delta = variant - dasgupta "
                     "on the same replicate", "tau": tau, "means": mu}
    groups = [("all runs", rows)] + [(k, [r for r in rows if r["run"] == k]) for k in meta["runs"]]
    groups += [(f"stratum {s}", [r for r in rows if r["stratum"] == s]) for s in vc.STRATA]
    for name, sample in groups:
        point = stats(sample)
        reps = boot_stats(sample, name)
        g = []
        for v, (vn, _, _) in enumerate(var):
            entry = dict(point[v])
            for m in ("kappa", "auc"):
                entry[f"{m}_ci"] = interval([rep[v][m] for rep in reps])
                entry[f"d{m}_ci"] = interval([None if rep[v][m] is None or rep[0][m] is None
                                              else rep[v][m] - rep[0][m] for rep in reps])
                entry[f"d{m}"] = None if point[v][m] is None or point[0][m] is None else point[v][m] - point[0][m]
            g.append(entry)
        res["groups"][name] = g
        print(f"{name}: dasgupta kappa {fmt(point[0]['kappa'])} auc {fmt(point[0]['auc'])}; "
              f"c+1fjs kappa {fmt(point[var.index(('c+1fjs', 'specific', 1.0))]['kappa'])}", flush=True)
    # Across schemes: run-level metric values against the judge's error rates.
    keys = list(meta["runs"])
    c_bad = [sum(r["_bad"] for r in rows if r["run"] == k and r["stratum"] == "C") /
             sum(1 for r in rows if r["run"] == k and r["stratum"] == "C") for k in keys]
    all_bad = [sum(r["_bad"] for r in rows if r["run"] == k) / sum(1 for r in rows if r["run"] == k) for k in keys]
    res["cross_run"] = {}
    for vn, f, b in var:
        vals = [run_metric(boot["runs"][k]["point"], f, b) for k in keys]
        res["cross_run"][vn] = {"values": dict(zip(keys, vals)), "spearman_C": spearman(vals, c_bad),
                                "spearman_all": spearman(vals, all_bad)}
    res["judge_error"] = {"C": dict(zip(keys, c_bad)), "all": dict(zip(keys, all_bad))}
    (DOCS / "falsejoin-results.json").write_text(json.dumps(res, indent=1, sort_keys=True) + "\n")
    write_judge_md(res, keys)


def fmt(v, d=2):
    return "–" if v is None else f"{v:.{d}f}"


def fci(v, c, d=2, sign=False):
    f = (lambda x: "–" if x is None else f"{x:+.{d}f}") if sign else (lambda x: fmt(x, d))
    return f"{f(v)} [{f(c[0])}, {f(c[1])}]" if c else f(v)


def write_judge_md(res, keys):
    var = [v["name"] for v in res["variants"]]
    L = ["# False-join variants against the stored judge labels", "",
         "Generated by `crates/bench/scripts/falsejoin_validity.py judge` from `sample.jsonl` (the 953 pairs and "
         "pass-1 labels of `README.md`, unchanged) and the rescored corpus runs "
         "(`bench/results/2026-10-11-falsejoin/bootstrap/corpus-minilm-k20.json.gz`). "
         f"Brackets: 95% percentile bootstrap, {BOOT} replicates, pairs resampled within strata. "
         "Δ is the variant minus Dasgupta on the same replicate.", "",
         "Variants: `dasgupta` is the original metric; `fj` and `fjs` are the outside-mass and specificity "
         "false-join terms alone; `c+βfj` and `c+βfjs` are Dasgupta plus β times that term. A pair is flagged "
         "when its standardised score exceeds 1 (see the script docstring).", "",
         "## All runs pooled", "",
         "| variant | flagged | κ [95% CI] | Δκ [95% CI] | AUC [95% CI] | ΔAUC [95% CI] |",
         "| --- | ---: | ---: | ---: | ---: | ---: |"]
    for v, e in zip(var, res["groups"]["all runs"]):
        L.append(f"| `{v}` | {fmt(e['flagged'])} | {fci(e['kappa'], e['kappa_ci'])} | "
                 f"{fci(e['dkappa'], e['dkappa_ci'], sign=True)} | {fci(e['auc'], e['auc_ci'])} | "
                 f"{fci(e['dauc'], e['dauc_ci'], sign=True)} |")
    show = ["dasgupta", "fj", "fjs", "c+1fj", "c+1fjs", "c+4fjs"]
    L += ["", "## Per run and stratum: κ / AUC", "",
          "| group | " + " | ".join(f"`{v}`" for v in show) + " |", "| --- |" + " ---: |" * len(show)]
    for g, entries in res["groups"].items():
        cells = [f"{fmt(entries[var.index(v)]['kappa'])} / {fmt(entries[var.index(v)]['auc'])}" for v in show]
        L.append(f"| {g} | " + " | ".join(cells) + " |")
    L += ["", "## Per run: κ with 95% CI, before and after", "",
          "| run | `dasgupta` κ | `c+1fj` κ | `c+1fjs` κ | Δκ `c+1fjs` | `dasgupta` AUC | `c+1fjs` AUC | ΔAUC `c+1fjs` |",
          "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for g in ["all runs"] + keys:
        e = res["groups"][g]
        d, o, s = e[var.index("dasgupta")], e[var.index("c+1fj")], e[var.index("c+1fjs")]
        L.append(f"| {g} | {fci(d['kappa'], d['kappa_ci'])} | {fci(o['kappa'], o['kappa_ci'])} | "
                 f"{fci(s['kappa'], s['kappa_ci'])} | {fci(s['dkappa'], s['dkappa_ci'], sign=True)} | "
                 f"{fci(d['auc'], d['auc_ci'])} | {fci(s['auc'], s['auc_ci'])} | "
                 f"{fci(s['dauc'], s['dauc_ci'], sign=True)} |")
    L += ["", "## Across schemes", "",
          "Spearman ρ between each variant's run value (lower is better) and the judge's error rate, over the "
          "8 runs. Positive ρ: the variant ranks schemes as the judge does. Eight runs only; descriptive.", "",
          "| variant | ρ with judge error, stratum C | ρ with judge error, all strata |", "| --- | ---: | ---: |"]
    for v in var:
        c = res["cross_run"][v]
        L.append(f"| `{v}` | {fmt(c['spearman_C'])} | {fmt(c['spearman_all'])} |")
    (DOCS / "falsejoin-results.md").write_text("\n".join(L) + "\n")
    print("\n".join(L))


# ---------------------------------------------------------------- external signals


def gz_jsonl(path):
    return [json.loads(line) for line in gzip.decompress(Path(path).read_bytes()).decode().splitlines() if line]


def write_gz_jsonl(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    data = "".join(json.dumps(r, sort_keys=True) + "\n" for r in rows).encode()
    path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0))


def set_cosine(a, b, weight=None):
    if not a or not b:
        return 0.0
    if weight is None:
        return len(a & b) / math.sqrt(len(a) * len(b))
    num = sum(weight[h] ** 2 for h in a & b)
    return num / math.sqrt(sum(weight[h] ** 2 for h in a) * sum(weight[h] ** 2 for h in b))


def signal_sets(signal):
    """Item index -> set of features, the items the signal covers, and feature weights."""
    if signal == "lcsh":
        rows = gz_jsonl(EXT / "lcsh.jsonl.gz")
        sets = {i: set(r["headings"]) for i, r in enumerate(rows)}
        df = {}
        for s in sets.values():
            for h in s:
                df[h] = df.get(h, 0) + 1
        idf = {h: math.log(len(rows) / c) for h, c in df.items()}
        return sets, df, idf, [r["id"] for r in rows]
    rows = gz_jsonl(EXT / "openalex.jsonl.gz")
    field = "refs" if signal == "coupling" else "citers"
    sets = {i: set(r[field]) for i, r in enumerate(rows) if r[field]}
    return sets, None, None, [r["id"] for r in rows]


def related_pairs(sets, df):
    """Pairs sharing a feature (LCSH: a heading held by at most SPECIFIC_DF books)."""
    index = {}
    for i, s in sets.items():
        for h in s:
            if df is None or df[h] <= SPECIFIC_DF:
                index.setdefault(h, []).append(i)
    pairs = set()
    for members in index.values():
        members.sort()
        for x, i in enumerate(members):
            for j in members[x + 1:]:
                pairs.add((i, j))
    return sorted(pairs)


def load_vectors(universe, model, n):
    path = VECTORS / f"{model}-{VECTOR_NAME[universe]}.f32"
    v = array("f")
    v.frombytes(path.read_bytes())
    return v, len(v) // n


def chosen(a):
    return {s: u for s, u in SIGNALS.items() if not a.signal or s == a.signal}


def sample(a):
    for signal, universe in chosen(a).items():
        sets, df, idf, ids = signal_sets(signal)
        n = len(ids)
        rng = rng_for("external", signal)
        rel = related_pairs(sets, df)
        related = rng.sample(rel, min(PER_CLASS, len(rel)))
        covered = sorted(sets)
        unrelated, seen = [], set()
        while len(unrelated) < PER_CLASS:
            i, j = sorted(rng.sample(covered, 2))
            if (i, j) not in seen and not (sets[i] & sets[j]):
                seen.add((i, j))
                unrelated.append((i, j))
        vecs = {m: load_vectors(universe, m, n) for m in MODELS}
        rows = []
        for kind, pairs in (("related", related), ("unrelated", unrelated)):
            for i, j in sorted(pairs):
                cos = {}
                for m, (v, dim) in vecs.items():
                    cos[m] = round(math.fsum(v[i * dim + k] * v[j * dim + k] for k in range(dim)), 6)
                rows.append({"i": i, "j": j, "id_a": ids[i], "id_b": ids[j], "kind": kind,
                             "strength": round(set_cosine(sets[i], sets[j], idf), 6), "cos": cos})
        write_gz_jsonl(EXT / f"pairs-{signal}.jsonl.gz", rows)
        meta = {"signal": signal, "universe": universe, "items_covered": len(covered), "items": n,
                "related_population": len(rel), "related_sampled": len(related),
                "unrelated_sampled": len(unrelated), "seed": SEED,
                "related_rule": ("share a heading held by at most %d books" % SPECIFIC_DF) if signal == "lcsh"
                else f"share at least one {'reference' if signal == 'coupling' else 'citing work'}",
                "unrelated_rule": "covered items sharing no feature at all"}
        (EXT / f"pairs-{signal}.meta.json").write_text(json.dumps(meta, indent=2, sort_keys=True) + "\n")
        print(json.dumps(meta), flush=True)


class Tree:
    """A run's induced tree in harness terms (virtual root depth 0; unplaced
    items hang off the root), from the committed scheme and placements."""

    def __init__(self, spec, ids):
        scheme = spec["scheme"]
        path = SCHEME_CACHE / scheme.split(":", 1)[1] if scheme.startswith("build_schemes.sh:") else ROOT / scheme
        self.parent = {c["code"]: c.get("parent") for c in json.loads(path.read_text())["classes"]}
        index = {x: k for k, x in enumerate(ids)}
        self.code = [None] * len(ids)
        for row in jsonl(ROOT / spec["placements"]):
            if row["classes"]:
                if len(row["classes"]) != 1:
                    raise SystemExit(f"{spec['key']}: fractional placements are not supported")
                c = row["classes"][0]
                self.code[index[row["id"]]] = c if isinstance(c, str) else c["code"]
        self.chain = {}
        for c in set(filter(None, self.code)):
            out, x = [], c
            while x:
                out.append(x)
                x = self.parent[x]
            self.chain[c] = tuple(reversed(out))
        self.n = len(ids)
        self.mass = {}
        for c in self.code:
            if c:
                for node in self.chain[c]:
                    self.mass[node] = self.mass.get(node, 0) + 1
        depths = sorted(len(self.chain[c]) for c in self.code if c)
        self.deep = max(2, depths[len(depths) // 2]) if depths else 2

    def lca(self, i, j):
        a, b = self.code[i], self.code[j]
        if not a or not b:
            return 0, self.n
        ca, cb = self.chain[a], self.chain[b]
        d = 0
        while d < min(len(ca), len(cb)) and ca[d] == cb[d]:
            d += 1
        return (d, self.mass[ca[d - 1]]) if d else (0, self.n)

    def deep_joined(self, i, j, depth):
        if not self.code[i] or not self.code[j]:
            return False
        return depth >= 2 and depth >= min(self.deep, len(self.chain[self.code[i]]), len(self.chain[self.code[j]]))


def graph_edges(boot):
    universe, model = boot["condition"]["universe"], boot["condition"]["model"]
    name = f"{model}-books-k20.json" if universe == "books" else f"{model}-{universe}-k20.json"
    sub = "books" if universe == "books" else "embed"
    g = json.loads((ROOT / f"bench/results/2026-10-11-metrics/{sub}/graphs/{name}").read_text())
    return g["tau"], {(e["left"], e["right"]): e["weight"] for e in g["edges"]}


def edge_mean(boot, key, edges):
    p = boot["runs"][key]["point"]
    return p["cost_norm"] * sum(edges.values()) / len(edges)


def draws(name):
    return json.loads(gzip.decompress((FJ / f"bootstrap/{name}.draws.json.gz").read_bytes()))


def weighted_auc(groups, positive, weight):
    """AUC from tie groups of entry indices sorted by ascending score, entry labels and entry
    weights (pair multiplicities of a resample): the weighted Mann-Whitney statistic, ties one half."""
    below = num = wp = wn = 0.0
    for g in groups:
        gp = sum(weight[i] for i in g if positive[i])
        gn = sum(weight[i] for i in g if not positive[i])
        num += gp * (below + 0.5 * gn)
        below += gn
        wp += gp
        wn += gn
    return num / (wp * wn) if wp and wn else None


def tie_groups(scores):
    order = sorted(range(len(scores)), key=lambda i: scores[i])
    groups, i = [], 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and scores[order[j + 1]] == scores[order[i]]:
            j += 1
        groups.append(order[i:j + 1])
        i = j + 1
    return groups


def weighted_kappa(table):
    """Cohen's kappa of a weighted 2x2 table {(a, b): weight}."""
    n = sum(table.values())
    if not n:
        return None
    po = (table.get((True, True), 0) + table.get((False, False), 0)) / n
    pa = (table.get((True, True), 0) + table.get((True, False), 0)) / n
    pb = (table.get((True, True), 0) + table.get((False, True), 0)) / n
    pe = pa * pb + (1 - pa) * (1 - pb)
    return None if pe >= 1 else (po - pe) / (1 - pe)


PAIR_MODEL = "minilm"
PAIR_REPS = 200


def resample_weights(groups, size, rng):
    w = [0] * size
    for members in groups:
        for _ in members:
            w[rng.choice(members)] += 1
    return w


def external(a):
    var = variants()
    res = {"variants": [{"name": nm, "form": f, "beta": b} for nm, f, b in var], "signals": {},
           "method": ("pair level: external error = a related pair split at LCA depth <= 1 (false split) or an "
                      "unrelated pair joined deep (LCA depth >= 2 and >= the run's deep threshold or either "
                      "item's class depth; false join); AUC of the variant's pair score and kappa of its flag "
                      "against that error, pooled over the universe's runs, on the MiniLM graph with "
                      f"{PAIR_REPS} replicates. Scheme level: the external separation of a run is the AUC with "
                      "which a smaller meeting class (|leaves(LCA)|) picks related over unrelated pairs; "
                      "Spearman rho across runs between it and minus the variant's run value, for each model. "
                      f"{BOOT_EXT} replicates: pairs resampled within related/unrelated (one resample stream per "
                      "signal, shared by every run and model), run values from the same-index paired "
                      "item-bootstrap draw.")}
    previous = json.loads((EXT / "agreement.json").read_text()) if (EXT / "agreement.json").exists() else None
    if a.signal and previous:
        res["signals"] = previous["signals"]
    for signal, universe in chosen(a).items():
        pairs = gz_jsonl(EXT / f"pairs-{signal}.jsonl.gz")
        ids = [json.loads(line)["id"] for line in CORPORA[universe].read_text().splitlines() if line.strip()]
        specs = load_boot(f"{universe}-minilm-k20")["runs_spec"]
        trees = {s["key"]: Tree(s, ids) for s in specs}
        keys = sorted(trees)
        res["signals"][signal] = external_signal(signal, universe, pairs, trees, keys, var)
        print(f"{signal}: done", flush=True)
    (EXT / "agreement.json").write_text(json.dumps(res, indent=1, sort_keys=True) + "\n")
    write_external_md(res)


def external_signal(signal, universe, pairs, trees, keys, var):
    related = [p["kind"] == "related" for p in pairs]
    idx = list(range(len(pairs)))
    strata = [[i for i in idx if related[i]], [i for i in idx if not related[i]]]
    # Tree side, the same for every model: LCA depth and mass, and the external error.
    lca = {k: [trees[k].lca(p["i"], p["j"]) for p in pairs] for k in keys}
    err = {k: [(rel and d <= 1) or (not rel and trees[k].deep_joined(p["i"], p["j"], d))
               for p, rel, (d, _) in zip(pairs, related, lca[k])] for k in keys}
    sep_groups = {k: tie_groups([-m for _, m in lca[k]]) for k in keys}
    ones = [1] * len(pairs)
    sep0 = {k: weighted_auc(sep_groups[k], related, ones) for k in keys}
    rng = rng_for("external-boot", signal)
    weights = [resample_weights(strata, len(pairs), rng) for _ in range(BOOT_EXT)]
    reps_sep = [{k: weighted_auc(sep_groups[k], related, w) for k in keys} for w in weights]
    out = {"universe": universe, "pairs": len(pairs), "runs": keys, "models": {},
           "separation": {k: {"point": sep0[k], "ci": interval([r[k] for r in reps_sep]),
                              "error_rate": sum(err[k]) / len(pairs)} for k in keys},
           "separation_draws": {k: [round(r[k], 6) for r in reps_sep] for k in keys}}
    for model in MODELS:
        name = f"{universe}-{model}-k20"
        boot = load_boot(name)
        dr = draws(name)
        ext0 = [sep0[k] for k in keys]
        rho = [spearman([-run_metric(boot["runs"][k]["point"], f, b) for k in keys], ext0) for _, f, b in var]
        reps_rho = []
        for r, rs in enumerate(reps_sep):
            point = {k: {m: dr["runs"][k][m][r] for m in ("cost_norm", "fj_norm", "fjs_norm")} for k in keys}
            ext = [rs[k] for k in keys]
            reps_rho.append([spearman([-run_metric(point[k], f, b) for k in keys], ext) for _, f, b in var])
        m = {"variants": {}}
        for v, (vn, _, _) in enumerate(var):
            m["variants"][vn] = {"rho": rho[v], "rho_ci": interval([r[v] for r in reps_rho]),
                                 "drho_ci": interval([None if r[v] is None or r[0] is None else r[v] - r[0]
                                                      for r in reps_rho])}
        if model == PAIR_MODEL:
            pair_level(m, pairs, boot, keys, lca, err, weights[:PAIR_REPS], var, model, universe)
        out["models"][model] = m
    return out


def pair_level(m, pairs, boot, keys, lca, err, weights, var, model, universe):
    """Pooled pair-level AUC and kappa of every variant against the external error, with intervals."""
    tau, edges = graph_edges(boot)
    n = sum(1 for line in CORPORA[universe].read_text().splitlines() if line.strip())
    entries = []  # (pair index, run key, scores per variant, mean edge charge)
    for key in keys:
        mu, mean_edge = means(boot, key), edge_mean(boot, key, edges)
        for i, (p, (_, mass)) in enumerate(zip(pairs, lca[key])):
            w = edges.get((p["i"], p["j"]), 0.0)
            zpos, zneg = pair_terms(n, mass, p["cos"][model], tau, w, mean_edge, mu)
            entries.append((i, err[key][i], [score(zpos, zneg, f, b, mean_edge) for _, f, b in var],
                            mean_edge))
    pos = [e[1] for e in entries]
    pair_of = [e[0] for e in entries]
    for v, (vn, _, _) in enumerate(var):
        scores = [e[2][v] for e in entries]
        groups = tie_groups(scores)
        # Per pair: the 2x2 counts (error, flag) over runs, so a resample's table is a weighted sum.
        cells = {}
        for (i, bad, s, mean_edge) in entries:
            key = (bad, s[v] > mean_edge)
            cells.setdefault(i, {}).setdefault(key, 0)
            cells[i][key] += 1

        def stats(w):
            ew = [w[i] for i in pair_of]
            table = {}
            for i, c in cells.items():
                for key, count in c.items():
                    table[key] = table.get(key, 0) + w[i] * count
            return weighted_auc(groups, pos, ew), weighted_kappa(table)

        a0, k0 = stats([1] * len(pairs))
        reps = [stats(w) for w in weights]
        m["variants"][vn].update({"auc": a0, "kappa": k0, "auc_reps": [r[0] for r in reps],
                                  "kappa_reps": [r[1] for r in reps]})
    base = m["variants"]["dasgupta"]
    for vn, e in m["variants"].items():
        for key in ("auc", "kappa"):
            e[f"{key}_ci"] = interval(e[f"{key}_reps"])
            e[f"d{key}_ci"] = interval([None if x is None or y is None else x - y
                                        for x, y in zip(e[f"{key}_reps"], base[f"{key}_reps"])])
    for e in m["variants"].values():
        del e["auc_reps"], e["kappa_reps"]


def write_external_md(res):
    L = ["# False-join variants against external, non-LLM signals", "",
         "Generated by `crates/bench/scripts/falsejoin_validity.py external`. Signals: bibliographic coupling "
         "and co-citation from OpenAlex `referenced_works` (arXiv papers), shared LCSH headings (3,000 books). "
         "Pair samples: `pairs-<signal>.jsonl.gz` and their `.meta.json`. Method: `agreement.json` `method`. "
         f"Brackets: 95% percentile bootstrap ({BOOT_EXT} replicates for ρ and separation, {PAIR_REPS} for "
         "pair-level AUC and κ); Δ is the variant minus Dasgupta on the same replicate. Pair-level agreement "
         f"is computed on the {PAIR_MODEL} graph; scheme-level ρ on each graph.", ""]
    show = ["dasgupta", "fj", "fjs", "c+0.25fj", "c+1fj", "c+4fj", "c+0.25fjs", "c+1fjs", "c+4fjs"]
    for signal, s in res["signals"].items():
        L += [f"## {signal} ({s['universe']}; {s['pairs']} pairs, {len(s['runs'])} runs)", "",
              f"| variant | pair AUC ({PAIR_MODEL}) | ΔAUC | pair κ | Δκ | "
              + " | ".join(f"scheme ρ ({m}) | Δρ" for m in MODELS) + " |",
              "| --- |" + " ---: |" * (4 + 2 * len(MODELS))]
        for v in show:
            e = s["models"][PAIR_MODEL]["variants"][v]
            d = s["models"][PAIR_MODEL]["variants"]["dasgupta"]
            row = [f"`{v}`", fci(e["auc"], e["auc_ci"]),
                   fci(None if v == "dasgupta" else e["auc"] - d["auc"], e["dauc_ci"], sign=True),
                   fci(e["kappa"], e["kappa_ci"]),
                   fci(None if v == "dasgupta" else e["kappa"] - d["kappa"], e["dkappa_ci"], sign=True)]
            for m in MODELS:
                x, y = s["models"][m]["variants"][v], s["models"][m]["variants"]["dasgupta"]
                row += [fci(x["rho"], x["rho_ci"]),
                        fci(None if v == "dasgupta" or x["rho"] is None else x["rho"] - y["rho"], x["drho_ci"],
                            sign=True)]
            L.append("| " + " | ".join(row) + " |")
        L += ["", f"### {signal}: external separation per run", "",
              "AUC with which a smaller meeting class picks related over unrelated pairs (higher is better), "
              "and the share of run × pair decisions that are external errors.", "",
              "| run | separation [95% CI] | external error rate |", "| --- | ---: | ---: |"]
        for k, e in sorted(s["separation"].items(), key=lambda kv: -kv[1]["point"]):
            L.append(f"| `{k}` | {fci(e['point'], e['ci'], 3)} | {fmt(e['error_rate'], 3)} |")
        L.append("")
    (EXT / "agreement.md").write_text("\n".join(L))
    print("\n".join(L[:40]))


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("step", choices=["judge", "sample", "external"])
    p.add_argument("--signal", choices=list(SIGNALS), help="only this external signal (others are kept)")
    a = p.parse_args()
    {"judge": judge, "sample": sample, "external": external}[a.step](a)


if __name__ == "__main__":
    main()
