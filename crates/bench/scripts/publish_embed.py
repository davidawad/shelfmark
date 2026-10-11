#!/usr/bin/env python3
"""Publish the cross-embedding robustness run (embed_benchmark.py) as a results directory.

Writes results.json (graphs and manifests, references, every run with its
bootstrap intervals, rankings, ranking stability, the learned scheme's edge
with paired bootstrap intervals, and the check of the all-MiniLM-L6-v2 k = 20
runs against the published ones), results.md tables, the per-run JSON, the
placements, the refitted learned schemes and the gzipped graphs.
"""
import argparse
import gzip
import hashlib
import json
import math
import shutil
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
MODELS = ["minilm", "bge-small", "nomic"]
KS = [10, 20, 50]
BASE = "minilm-k20"
UNIVERSES = ["corpus", "holdout", "split"]
LAMBDA = 0.005  # J = cost_norm + LAMBDA * nav_log_a1, the learned objective
REFIT = ["bge-small", "nomic"]
MODEL_LABEL = {"minilm": "all-MiniLM-L6-v2", "bge-small": "bge-small-en-v1.5", "nomic": "nomic-embed-text-v1.5"}
METRICS = ["cost_norm", "revenue_norm", "skill", "coverage", "mean_depth", "normalized_entropy",
           "effective_classes", "gini", "largest_class_share", "nav_scan", "nav_log_a0", "nav_log_a1",
           "nav_log_a2"]
PUBLISHED_MAX_GRAPH_MB = 20


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def same_content(gz, want):
    """True when `gz` exists and decompresses to a file with sha256 `want` (keep its bytes)."""
    if not gz.exists():
        return False
    h = hashlib.sha256()
    with gzip.open(gz, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest() == want


def conditions():
    return [f"{m}-k{k}" for m in MODELS for k in KS]


# -- statistics ---------------------------------------------------------------
def ranks(values):
    """Average ranks, 1 = best (largest value)."""
    order = sorted(range(len(values)), key=lambda i: -values[i])
    out = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for t in range(i, j + 1):
            out[order[t]] = (i + j) / 2 + 1
        i = j + 1
    return out


def spearman(a, b):
    ra, rb = ranks(a), ranks(b)
    ma, mb = sum(ra) / len(ra), sum(rb) / len(rb)
    num = sum((x - ma) * (y - mb) for x, y in zip(ra, rb))
    den = math.sqrt(sum((x - ma) ** 2 for x in ra) * sum((y - mb) ** 2 for y in rb))
    return num / den if den else float("nan")


def kendall(a, b):
    """Kendall tau-b."""
    conc = disc = ties_a = ties_b = 0
    for i in range(len(a)):
        for j in range(i + 1, len(a)):
            da, db = a[i] - a[j], b[i] - b[j]
            if da == 0 and db == 0:
                continue
            if da == 0:
                ties_a += 1
            elif db == 0:
                ties_b += 1
            elif (da > 0) == (db > 0):
                conc += 1
            else:
                disc += 1
    den = math.sqrt((conc + disc + ties_a) * (conc + disc + ties_b))
    return (conc - disc) / den if den else float("nan")


def percentile(sorted_values, q):
    pos = q * (len(sorted_values) - 1)
    lo, hi = math.floor(pos), math.ceil(pos)
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * (pos - lo)


def interval(values):
    v = sorted(x for x in values if math.isfinite(x))
    return [percentile(v, 0.025), percentile(v, 0.975)] if v else [float("nan")] * 2


# -- loading --------------------------------------------------------------------
class Runs:
    def __init__(self, work):
        self.work = work
        self.runs = {}  # (cond, universe, key) -> result
        self.refs = {}
        for cond in conditions():
            for u in UNIVERSES:
                d = work / "runs" / cond / u
                self.refs[(cond, u)] = json.loads((d / "reference.json").read_text())
                for f in sorted(d.glob("*.json")):
                    if f.name.endswith(".reps.json") or f.name == "reference.json":
                        continue
                    self.runs[(cond, u, f.stem)] = json.loads(f.read_text())
        self._reps = {}

    def reps(self, cond, u, key):
        k = (cond, u, key)
        if k not in self._reps:
            values = json.loads((self.work / "runs" / cond / u / f"{key}.reps.json").read_text())["values"]
            values["J"] = [c + LAMBDA * nav for c, nav in zip(values["cost_norm"], values["nav_log_a1"])]
            self._reps[k] = values
        return self._reps[k]

    def keys(self, u):
        return sorted({k for (c, uu, k) in self.runs if uu == u and c == BASE})


def j_value(r):
    return r["cost_norm"] + LAMBDA * r["structure"]["nav_log_a1"]


def row(r):
    """Compact run record: point values and 95% intervals."""
    s = r["structure"]
    point = {"cost_norm": r["cost_norm"], "revenue_norm": r["revenue_norm"], "skill": r["skill"],
             "coverage": r["coverage"], "J": j_value(r)}
    point.update({m: s[m] for m in METRICS if m in s})
    ci = dict(r["bootstrap"]["ci"])
    return {"scheme": r["scheme"], "version": r["version"], "track": r["track"], "n": r["n"],
            "placement_source": r["run"]["placement_source"], "placements": r["run"]["placements"],
            "point": point, "ci95": ci, "classes_used": s["effective_classes"],
            "max_branching": s["max_branching"], "hashes": r["hashes"]}


def is_learned(key):
    return key.startswith("learned")


def is_refit(key):
    return any(key.startswith(f"learned-{m}") for m in REFIT)


# -- analyses -------------------------------------------------------------------
def stability(runs, universe, include):
    """Pairwise rank correlations of the skill (and J) rankings across conditions."""
    keys = [k for k in runs.keys(universe) if include(k)]
    out = {"keys": keys, "metrics": {}}
    for metric in ("skill", "J"):
        sign = 1 if metric == "skill" else -1  # higher skill, lower J is better
        vals = {c: [sign * (runs.runs[(c, universe, k)]["skill"] if metric == "skill"
                            else j_value(runs.runs[(c, universe, k)])) for k in keys] for c in conditions()}
        pairs = {}
        for i, a in enumerate(conditions()):
            for b in conditions()[i + 1:]:
                pairs[f"{a}|{b}"] = {"spearman": spearman(vals[a], vals[b]), "kendall": kendall(vals[a], vals[b])}
        vs_base = {}
        for c in conditions():
            if c == BASE:
                continue
            # Paired bootstrap: the same resample of works in both conditions.
            ra = [runs.reps(BASE, universe, k)[metric] for k in keys]
            rb = [runs.reps(c, universe, k)[metric] for k in keys]
            boot_s, boot_k = [], []
            for r in range(len(ra[0])):
                xa = [sign * x[r] for x in ra]
                xb = [sign * x[r] for x in rb]
                boot_s.append(spearman(xa, xb))
                boot_k.append(kendall(xa, xb))
            key = f"{BASE}|{c}" if f"{BASE}|{c}" in pairs else f"{c}|{BASE}"
            vs_base[c] = dict(pairs[key], spearman_ci95=interval(boot_s), kendall_ci95=interval(boot_k))
        rank_of = {c: dict(zip(keys, ranks(vals[c]))) for c in conditions()}
        out["metrics"][metric] = {
            "pairs": pairs, "vs_base": vs_base, "ranks": rank_of,
            "min_spearman": min(p["spearman"] for p in pairs.values()),
            "min_kendall": min(p["kendall"] for p in pairs.values()),
        }
    return out


EDGES = [
    # (name, universe, learned key, competitors, description)
    ("split-heldout", "split", "learned-heldout", lambda k: not is_learned(k),
     "MiniLM 80% fit, held-out works inserted with MiniLM vectors, vs every Paper 1 scheme on the same 2,000 works"),
    ("corpus-own", "corpus", "learned-own", lambda k: not is_learned(k),
     "MiniLM full fit, own placements (in-sample works), vs every Paper 1 scheme"),
    ("holdout-own", "holdout", "learned-own", lambda k: not is_learned(k),
     "MiniLM fit, 500 books inserted with MiniLM vectors, vs every Paper 1 scheme"),
    ("corpus-placed", "corpus", "learned-placed", lambda k: k.endswith("-placed") and not is_learned(k),
     "learned scheme placed by shelfmark lexical@2 vs every Paper 1 placed scheme"),
]
for _m in REFIT:
    EDGES.append((f"split-heldout-{_m}", "split", f"learned-{_m}-heldout", lambda k: not is_learned(k),
                  f"{_m} 80% fit, held out ({_m} insertion vectors), vs every Paper 1 scheme"))


def edges(runs):
    out = {}
    for name, u, lkey, comp, desc in EDGES:
        per = {}
        for c in conditions():
            others = [k for k in runs.keys(u) if comp(k)]
            lr = runs.runs[(c, u, lkey)]
            best_skill = max(others, key=lambda k: runs.runs[(c, u, k)]["skill"])
            best_j = min(others, key=lambda k: j_value(runs.runs[(c, u, k)]))
            lrep = runs.reps(c, u, lkey)
            ds = [a - b for a, b in zip(lrep["skill"], runs.reps(c, u, best_skill)["skill"])]
            dj = [b - a for a, b in zip(lrep["J"], runs.reps(c, u, best_j)["J"])]
            per[c] = {
                "learned_skill": lr["skill"], "learned_skill_ci95": lr["bootstrap"]["ci"]["skill"],
                "learned_J": j_value(lr),
                "best_other_by_skill": best_skill, "best_other_skill": runs.runs[(c, u, best_skill)]["skill"],
                "skill_margin": lr["skill"] - runs.runs[(c, u, best_skill)]["skill"],
                "skill_margin_ci95": interval(ds),
                "skill_margin_share_positive": sum(d > 0 for d in ds) / len(ds),
                "best_other_by_J": best_j, "best_other_J": j_value(runs.runs[(c, u, best_j)]),
                "J_margin": j_value(runs.runs[(c, u, best_j)]) - j_value(lr),
                "J_margin_ci95": interval(dj),
                "learned_rank_by_skill": 1 + sum(runs.runs[(c, u, k)]["skill"] > lr["skill"] for k in others),
            }
        survives = all(v["skill_margin_ci95"][0] > 0 and v["J_margin_ci95"][0] > 0 for v in per.values())
        out[name] = {"universe": u, "learned": lkey, "description": desc, "conditions": per,
                     "survives_every_condition": survives,
                     "min_skill_margin": min(v["skill_margin"] for v in per.values()),
                     "min_skill_margin_ci_low": min(v["skill_margin_ci95"][0] for v in per.values())}
    return out


def cross_fit(runs):
    """Each learned fit (MiniLM, BGE, nomic) scored on each model's k = 20 graph."""
    fits = {"minilm": ("learned-own", "learned-own", "learned-heldout")}
    fits.update({m: (f"learned-{m}-own", f"learned-{m}-own", f"learned-{m}-heldout") for m in REFIT})
    out = {}
    for fit, (ck, hk, sk) in fits.items():
        for m in MODELS:
            c = f"{m}-k20"
            out[f"{fit}@{c}"] = {
                "fit": fit, "scored_on": c,
                "corpus_skill": runs.runs[(c, "corpus", ck)]["skill"],
                "corpus_skill_ci95": runs.runs[(c, "corpus", ck)]["bootstrap"]["ci"]["skill"],
                "corpus_J": j_value(runs.runs[(c, "corpus", ck)]),
                "split_skill": runs.runs[(c, "split", sk)]["skill"],
                "split_skill_ci95": runs.runs[(c, "split", sk)]["bootstrap"]["ci"]["skill"],
                "split_J": j_value(runs.runs[(c, "split", sk)]),
                "holdout_skill": runs.runs[(c, "holdout", hk)]["skill"],
                "holdout_skill_ci95": runs.runs[(c, "holdout", hk)]["bootstrap"]["ci"]["skill"],
            }
    return out


def published_checks(runs):
    """The all-MiniLM-L6-v2 k = 20 runs against the published cost_norm of the same placements."""
    rd = ROOT / "bench/results"
    p1 = json.loads((rd / "2026-10-10/results.json").read_text())["results"]
    r2 = json.loads((rd / "2026-10-10-r2/results.json").read_text())["results"]
    r3 = json.loads((rd / "2026-10-10-r3/results.json").read_text())["results"]
    lr = json.loads((rd / "2026-10-10-learned/results.json").read_text())["runs"]
    want = {}
    for key, v in r2.items():
        if key.endswith("-placed-holdout"):
            want[("holdout", key.removesuffix("-holdout"))] = ("2026-10-10-r2", v["cost_norm"])
        else:
            want[("corpus", key)] = ("2026-10-10-r2", v["cost_norm"])
    for name, u in (("arxiv", "corpus"), ("jel", "corpus"), ("lcc", "holdout"), ("lcc-full", "holdout")):
        want[(u, f"{name}-gold")] = ("2026-10-10", p1[f"{name}-gold"]["cost_norm"])
    want[("corpus", "msc-gold")] = ("2026-10-10-r3", r3["msc-gold"]["cost_norm"])
    learned_map = {("corpus", "learned-own"): "learned-own", ("holdout", "learned-own"): "learned-own-holdout",
                   ("corpus", "learned-placed"): "learned-placed",
                   ("holdout", "learned-placed"): "learned-placed-holdout",
                   ("split", "learned-heldout"): "learned-split-heldout", ("split", "learned-own"): "test-learned-own"}
    for (u, key), src in learned_map.items():
        want[(u, key)] = ("2026-10-10-learned", lr[src]["cost_norm"])
    for name in lr:
        if name.startswith("test-") and name != "test-learned-own":
            want[("split", name.removeprefix("test-"))] = ("2026-10-10-learned", lr[name]["cost_norm"])
    checks = {}
    for (u, key), (src, value) in sorted(want.items()):
        got = runs.runs.get((BASE, u, key))
        if got is None:
            continue
        checks[f"{u}/{key}"] = {"published": value, "source": src, "rescored": got["cost_norm"],
                                "equal": abs(got["cost_norm"] - value) <= 1e-12}
    return checks


# -- publishing -------------------------------------------------------------------
def fmt(x, d=4):
    return "-" if x is None or (isinstance(x, float) and not math.isfinite(x)) else f"{x:.{d}f}"


def fmt_ci(v, ci, d=4):
    return f"{fmt(v, d)} [{fmt(ci[0], d)}, {fmt(ci[1], d)}]"


def tables(runs, stab, edge, cross, checks, graphs):
    md = []
    md.append("## Graphs\n")
    md.append("| model | graph | k | n | edges | tau | average-linkage cost_norm [95% CI] |")
    md.append("| --- | --- | ---: | ---: | ---: | ---: | --- |")
    for c in conditions():
        for u in ("corpus", "holdout"):
            g = graphs[f"{c}/{u}"]
            ref = runs.refs[(c, u)]
            md.append(f"| {MODEL_LABEL[g['model']]} | {u} | {g['graph_k']} | {g['n']} | {g['edges']} | "
                      f"{g['tau']:.4f} | {fmt_ci(ref['ref_cost_norm'], ref['ci'])} |")
        ref = runs.refs[(c, "split")]
        md.append(f"| {MODEL_LABEL[c.rsplit('-k', 1)[0]]} | 20% split (induced) | {c.rsplit('-k', 1)[1]} | "
                  f"{ref['n']} | - | - | {fmt_ci(ref['ref_cost_norm'], ref['ci'])} |")
    md.append("")
    md.append("## Ranking stability\n")
    md.append("Rankings by skill (equivalently cost_norm within one graph) and by J. Rows: Paper 1 "
              "schemes (gold and lexical@2 placed) plus the learned scheme fitted on MiniLM; the refits are "
              "left out. Spearman rho and Kendall tau-b of each condition's ranking against all-MiniLM-L6-v2 "
              "k = 20, with paired item-bootstrap 95% intervals (each resample ranks the same resampled works "
              "on both graphs), and the minimum over all 36 pairs of conditions.\n")
    for u in UNIVERSES:
        s = stab[u]
        md.append(f"### {u} ({len(s['keys'])} rows)\n")
        md.append("| condition | Spearman (skill) | Kendall (skill) | Spearman (J) | Kendall (J) |")
        md.append("| --- | --- | --- | --- | --- |")
        for c in conditions():
            if c == BASE:
                continue
            a, b = s["metrics"]["skill"]["vs_base"][c], s["metrics"]["J"]["vs_base"][c]
            md.append(f"| {c} | {fmt_ci(a['spearman'], a['spearman_ci95'], 3)} | "
                      f"{fmt_ci(a['kendall'], a['kendall_ci95'], 3)} | {fmt_ci(b['spearman'], b['spearman_ci95'], 3)}"
                      f" | {fmt_ci(b['kendall'], b['kendall_ci95'], 3)} |")
        sk, sj = s["metrics"]["skill"], s["metrics"]["J"]
        md.append(f"| min over all pairs | {fmt(sk['min_spearman'], 3)} | {fmt(sk['min_kendall'], 3)} | "
                  f"{fmt(sj['min_spearman'], 3)} | {fmt(sj['min_kendall'], 3)} |")
        md.append("")
        md.append("Rank by skill under each condition:\n")
        md.append("| row | " + " | ".join(conditions()) + " |")
        md.append("| --- |" + " ---: |" * len(conditions()))
        for k in sorted(s["keys"], key=lambda k: sk["ranks"][BASE][k]):
            md.append(f"| {k} | " + " | ".join(f"{sk['ranks'][c][k]:g}" for c in conditions()) + " |")
        md.append("")
    md.append("## The learned scheme's edge\n")
    md.append("Margin = learned skill minus the best other row's skill on the same graph (positive: learned "
              "ahead); J margin = best other J minus learned J (positive: learned ahead). Intervals are paired "
              "item-bootstrap 95% intervals of the difference.\n")
    for name, e in edge.items():
        md.append(f"### {name}: {e['description']}\n")
        md.append("| condition | learned skill [95% CI] | best other (skill) | skill margin [95% CI] | "
                  "best other (J) | J margin [95% CI] |")
        md.append("| --- | --- | --- | --- | --- | --- |")
        for c, v in e["conditions"].items():
            md.append(f"| {c} | {fmt_ci(v['learned_skill'], v['learned_skill_ci95'])} | "
                      f"{v['best_other_by_skill']} {fmt(v['best_other_skill'])} | "
                      f"{fmt_ci(v['skill_margin'], v['skill_margin_ci95'])} | {v['best_other_by_J']} "
                      f"{fmt(v['best_other_J'])} | {fmt_ci(v['J_margin'], v['J_margin_ci95'])} |")
        md.append(f"\nSurvives every condition (both margins' intervals above 0): "
                  f"**{'yes' if e['survives_every_condition'] else 'no'}**.\n")
    md.append("## Cross-fit: each learned fit scored on each model's k = 20 graph\n")
    md.append("| fit | scored on | corpus skill (in-sample works) | split skill (held out) | split J | "
              "holdout skill |")
    md.append("| --- | --- | --- | --- | ---: | --- |")
    for v in cross.values():
        md.append(f"| {v['fit']} | {v['scored_on']} | {fmt_ci(v['corpus_skill'], v['corpus_skill_ci95'])} | "
                  f"{fmt_ci(v['split_skill'], v['split_skill_ci95'])} | {fmt(v['split_J'])} | "
                  f"{fmt_ci(v['holdout_skill'], v['holdout_skill_ci95'])} |")
    md.append("")
    md.append("## Every run\n")
    for u in UNIVERSES:
        md.append(f"### {u}\n")
        md.append("| condition | row | coverage [95% CI] | cost_norm [95% CI] | MW revenue_norm [95% CI] | "
                  "skill [95% CI] | J | eff. classes [95% CI] | mean depth | nav log a=1 [95% CI] | "
                  "nav scan [95% CI] |")
        md.append("| --- | --- | --- | --- | --- | --- | ---: | --- | ---: | --- | --- |")
        for c in conditions():
            keys = sorted(runs.keys(u), key=lambda k: -runs.runs[(c, u, k)]["skill"])
            for k in keys:
                r = runs.runs[(c, u, k)]
                ci, s = r["bootstrap"]["ci"], r["structure"]
                md.append(f"| {c} | {k} | {fmt_ci(r['coverage'], ci['coverage'])} | "
                          f"{fmt_ci(r['cost_norm'], ci['cost_norm'])} | "
                          f"{fmt_ci(r['revenue_norm'], ci['revenue_norm'])} | {fmt_ci(r['skill'], ci['skill'])} | "
                          f"{fmt(j_value(r))} | {fmt_ci(s['effective_classes'], ci['effective_classes'], 1)} | "
                          f"{fmt(s['mean_depth'], 2)} | {fmt_ci(s['nav_log_a1'], ci['nav_log_a1'], 2)} | "
                          f"{fmt_ci(s['nav_scan'], ci['nav_scan'], 1)} |")
        md.append("")
    md.append("## Check against the published all-MiniLM-L6-v2 k = 20 results\n")
    equal = sum(v["equal"] for v in checks.values())
    md.append(f"{equal} of {len(checks)} rescored runs equal the published cost_norm exactly.\n")
    bad = {k: v for k, v in checks.items() if not v["equal"]}
    if bad:
        md.append("| run | published | source | rescored |")
        md.append("| --- | ---: | --- | ---: |")
        for k, v in bad.items():
            md.append(f"| {k} | {v['published']:.6f} | {v['source']} | {v['rescored']:.6f} |")
        md.append("")
    return "\n".join(md) + "\n"


def graph_info(cache, work, out, copy):
    info = {}
    for m in MODELS:
        for k in KS:
            c = f"{m}-k{k}"
            for u in ("corpus", "holdout"):
                if c == BASE:
                    src = cache / "graphs" / f"{u}-k20.json"
                    manifest = json.loads((ROOT / "bench/results/2026-10-10/graphs" /
                                           f"{u}-k20.manifest.json").read_text())
                    manifest["published"] = f"../2026-10-10/graphs/{u}-k20.json.gz"
                else:
                    src = cache / "embed2" / f"{m}-{u}-k{k}.json"
                    manifest = json.loads(src.with_name(f"{m}-{u}-k{k}.manifest.json").read_text())
                    gz = out / "graphs" / f"{m}-{u}-k{k}.json.gz"
                    if copy and not same_content(gz, manifest["graph_sha256"]):
                        gz.parent.mkdir(parents=True, exist_ok=True)
                        with open(src, "rb") as f, gzip.GzipFile(gz, "wb", mtime=0) as g:
                            shutil.copyfileobj(f, g)
                    size = gz.stat().st_size / 1e6
                    if size <= PUBLISHED_MAX_GRAPH_MB:
                        manifest["published"] = f"graphs/{gz.name}"
                    else:
                        gz.unlink()
                        manifest["published"] = None
                    manifest["gz_mb"] = round(size, 2)
                    (out / "graphs" / f"{m}-{u}-k{k}.manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
                g = json.loads(src.read_text())
                manifest.update(model=m, edges=len(g["edges"]))
                info[f"{c}/{u}"] = manifest
    return info


def copy_artifacts(cache, work, out):
    """Placements (embedding-free inputs) and the refitted learned schemes."""
    pl = out / "placements"
    pl.mkdir(parents=True, exist_ok=True)
    files = {}

    def put(src, name):
        dst = pl / name
        shutil.copy(src, dst)
        if dst.stat().st_size > 20e6:
            with open(dst, "rb") as f, gzip.GzipFile(str(dst) + ".gz", "wb", mtime=0) as g:
                shutil.copyfileobj(f, g)
            dst.unlink()
            dst = Path(str(dst) + ".gz")
        files[name] = {"sha256": sha256(dst), "file": f"placements/{dst.name}"}

    for f in sorted((cache / "placed").glob("*/placements.jsonl")):
        put(f, f"{f.parent.name}-placed.jsonl")
    for f in sorted((work / "placements").glob("*-gold.jsonl")):
        put(f, f.name)
    lw = cache / "learned-bench"
    put(lw / "corpus-placements.jsonl", "corpus-learned-own.jsonl")
    put(lw / "holdout-placements.jsonl", "holdout-learned-own.jsonl")
    put(lw / "split" / "test-placements.jsonl", "split-learned-heldout.jsonl")
    shutil.copy(lw / "learned-holdout.json", pl / "learned-holdout.json")
    shutil.copy(lw / "split" / "split-inserted.json", pl / "learned-split80-inserted.json")
    for m in REFIT:
        w = work / "refit" / m
        put(w / "corpus-placements.jsonl", f"corpus-learned-{m}-own.jsonl")
        put(w / "holdout-placements.jsonl", f"holdout-learned-{m}-own.jsonl")
        put(w / "split-test-placements.jsonl", f"split-learned-{m}-heldout.jsonl")
        rd = out / "refit" / m
        rd.mkdir(parents=True, exist_ok=True)
        for name in (f"learned-{m}.json", f"learned-{m}-holdout.json", f"learned-{m}-split80-inserted.json",
                     "build-report.json", "insert-report.json", "split-build-report.json",
                     "split-insert-report.json"):
            shutil.copy(w / name, rd / name)
    return files


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--out", default=str(ROOT / "bench/results/2026-10-10-embed"))
    p.add_argument("--harness", required=True)
    p.add_argument("--learned", required=True)
    p.add_argument("--runtime", help="hand-written JSON object of stage timings")
    p.add_argument("--notes", help="markdown written before the generated tables of results.md")
    a = p.parse_args()
    cache, out = Path(a.cache), Path(a.out)
    work = cache / "embed-bench"
    out.mkdir(parents=True, exist_ok=True)
    runs = Runs(work)
    graphs = graph_info(cache, work, out, copy=True)
    placements = copy_artifacts(cache, work, out)
    rows_with = lambda k: not is_refit(k)
    stab = {u: stability(runs, u, rows_with) for u in UNIVERSES}
    stab_p1 = {u: stability(runs, u, lambda k: not is_learned(k)) for u in UNIVERSES}
    edge = edges(runs)
    cross = cross_fit(runs)
    checks = published_checks(runs)
    run_dir = out / "runs"
    for (c, u, k), r in runs.runs.items():
        d = run_dir / c / u
        d.mkdir(parents=True, exist_ok=True)
        (d / f"{k}.json").write_text(json.dumps(r, indent=2) + "\n")
    results = {
        "date": "2026-10-10",
        "corpus": {"corpus_sha256": sha256(ROOT / "bench/corpus/corpus.jsonl"),
                   "holdout_sha256": sha256(ROOT / "bench/corpus/holdout.jsonl"),
                   "split_test_ids": "../2026-10-10-learned/split/test-ids.txt"},
        "conditions": conditions(), "base": BASE,
        "graphs": graphs,
        "references": {f"{c}/{u}": v for (c, u), v in runs.refs.items()},
        "bootstrap": {"resamples": 1000, "seed": 20261010, "confidence": 0.95,
                      "method": "percentile; works resampled with replacement on each fixed graph and tree, "
                                "the same resamples for every row of a universe and every condition (paired)"},
        "objective_J": f"cost_norm + {LAMBDA} * nav_log_a1",
        "tools": {"harness_sha256": sha256(a.harness), "learned_scheme_sha256": sha256(a.learned)},
        "runtime": json.loads(Path(a.runtime).read_text()) if a.runtime else None,
        "stability": stab, "stability_paper1_only": stab_p1,
        "edge": edge, "cross_fit": cross, "published_check": checks,
        "placements": placements,
        "runs": {f"{c}/{u}/{k}": row(r) for (c, u, k), r in sorted(runs.runs.items())},
    }
    (out / "results.json").write_text(json.dumps(results, indent=1) + "\n")
    notes = Path(a.notes).read_text() if a.notes else "# Cross-embedding robustness, 2026-10-10-embed\n\n"
    (out / "results.md").write_text(notes.rstrip() + "\n\n" + tables(runs, stab, edge, cross, checks, graphs))
    print(json.dumps({"runs": len(runs.runs), "checks_equal": sum(v["equal"] for v in checks.values()),
                      "checks": len(checks),
                      "edge_survives": {k: v["survives_every_condition"] for k, v in edge.items()}}, indent=1))


if __name__ == "__main__":
    main()
