#!/usr/bin/env python3
"""Every table and test of report 9, regenerated from committed results.

Reads only committed files: `bench/results/2026-10-10-proposal/` (written by
`kb_proposal_runs.py` and `ol_work_numbers.py`), `2026-10-10-ddc`,
`2026-10-10-embed` and `2026-10-10-covered`. Each table sits in the report
between `<!-- table:NAME -->` and `<!-- /table:NAME -->`. With no option the
script rebuilds every table and exits 1 if any differs from the report;
`--write` rewrites them in place, and rewrites the LCC-to-FDC crosswalk
`bench/results/2026-10-10-proposal/crosswalk/lcc-fdc.json` that the
check compares with its inputs.

Tests are paired: every run of a universe and graph is scored on the same
1,000 item draws (seed 20261010), so the difference of two runs is computed
draw by draw. The interval is the 2.5% and 97.5% percentile of the paired
differences and the two-sided p is 2 min(share <= 0, share >= 0), shown as
"<0.002" when no draw crosses zero (the resolution of 1,000 draws). Holm's
step-down adjustment is applied within each table's family of tests.

    python3 -I research/reports/scripts/kb_proposal_tables.py [--write]
"""
import argparse
import gzip
import hashlib
import importlib.util
import json
import math
import re
import sys
from collections import Counter
from functools import cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REPORT = ROOT / "research/reports/9-kb-structure-proposal.md"
P = ROOT / "bench/results/2026-10-10-proposal"
DDC_R = ROOT / "bench/results/2026-10-10-ddc"
METRICS_R = ROOT / "bench/results/2026-10-11-metrics/books/results.json"
MODELS = ["minilm", "bge-small", "nomic"]
KS = [10, 20, 50]
LAMBDA0 = 0.002
LAMBDA_RANGE = (0.0005, 0.005)
LAMBDAS = [0.0, 0.0005, 0.002, 0.005, 0.01]
B = 1000

_spec = importlib.util.spec_from_file_location("ddc_numbers", ROOT / "crates/bench/scripts/ddc_numbers.py")
ddc = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ddc)


# ---- loading -----------------------------------------------------------
@cache
def boot(universe, model="minilm", k=20):
    return json.loads((P / f"bootstrap/{universe}-{model}-k{k}.json").read_text())


@cache
def draws(universe, model="minilm", k=20):
    return json.loads(gzip.decompress((P / f"bootstrap/{universe}-{model}-k{k}.draws.json.gz").read_bytes()))


@cache
def scores(name):
    return json.loads((P / f"scores/{name}.json").read_text())


def jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def point(universe, run, metric, model="minilm", k=20):
    return boot(universe, model, k)["runs"][run]["point"][metric]


def ci(universe, run, metric, model="minilm", k=20):
    return boot(universe, model, k)["runs"][run]["ci"][metric]


def series(universe, run, metric, model="minilm", k=20, lam=None):
    """Per-draw values; metric "J" is cost_norm + lam * nav_log_a1."""
    r = draws(universe, model, k)["runs"][run]
    if metric == "J":
        return [c + lam * n for c, n in zip(r["cost_norm"], r["nav_log_a1"])]
    return r[metric]


def j_point(universe, run, lam, model="minilm", k=20):
    return point(universe, run, "cost_norm", model, k) + lam * point(universe, run, "nav_log_a1", model, k)


# ---- statistics --------------------------------------------------------
def percentile(sorted_values, q):
    pos = q * (len(sorted_values) - 1)
    lo, hi = math.floor(pos), math.ceil(pos)
    return sorted_values[lo] + (sorted_values[hi] - sorted_values[lo]) * (pos - lo)


def paired(a, b, pa, pb):
    """Point difference, 95% interval and two-sided p of a - b from paired draws."""
    d = sorted(x - y for x, y in zip(a, b))
    below, above = sum(x <= 0 for x in d), sum(x >= 0 for x in d)
    p = min(1.0, 2 * min(below, above) / len(d))
    return {"diff": pa - pb, "lo": percentile(d, 0.025), "hi": percentile(d, 0.975), "p": p}


def holm(tests):
    """Adds `p_holm` to each test dict (p = 0 counted as 1/B)."""
    order = sorted(range(len(tests)), key=lambda i: max(tests[i]["p"], 1 / B))
    running, m = 0.0, len(tests)
    for rank, i in enumerate(order):
        running = max(running, min(1.0, (m - rank) * max(tests[i]["p"], 1 / B)))
        tests[i]["p_holm"] = running
    return tests


def wilson(k, n, z=1.959964):
    phat = k / n
    centre = (phat + z * z / (2 * n)) / (1 + z * z / n)
    half = z * math.sqrt(phat * (1 - phat) / n + z * z / (4 * n * n)) / (1 + z * z / n)
    return centre - half, centre + half


# ---- formatting --------------------------------------------------------
def f(x, d=4):
    return f"{x:.{d}f}"


def sf(x, d=4):
    return f"{x:+.{d}f}"


def interval(lo, hi, d=4, signed=False):
    g = sf if signed else f
    return f"[{g(lo, d)}, {g(hi, d)}]"


def pv(p):
    return "<0.002" if p == 0 else f"{p:.3f}"


def table(header, rows, align=None):
    align = align or ["---"] + ["---:"] * (len(header) - 1)
    lines = ["| " + " | ".join(header) + " |", "| " + " | ".join(align) + " |"]
    lines += ["| " + " | ".join(str(c) for c in r) + " |" for r in rows]
    return "\n".join(lines)


def test_cells(t, d=4):
    return [sf(t["diff"], d), interval(t["lo"], t["hi"], d, signed=True), pv(t["p"]), pv(t["p_holm"])]


def run_test(universe, a, b, metric, model="minilm", k=20, lam=LAMBDA0):
    if metric == "J":
        pa, pb = j_point(universe, a, lam, model, k), j_point(universe, b, lam, model, k)
    else:
        pa, pb = point(universe, a, metric, model, k), point(universe, b, metric, model, k)
    return paired(series(universe, a, metric, model, k, lam), series(universe, b, metric, model, k, lam), pa, pb)


# ---- tables ------------------------------------------------------------
NAMES = {
    "ddc-gold": "Dewey, librarian number, full depth", "ddc-section": "Dewey, cut to section (3 digits)",
    "fdc-ddc": "FDC from Dewey, full FDC depth", "fdc-d3": "FDC from Dewey, depth 3",
    "fdc-d2": "FDC from Dewey, depth 2", "fdc-d1": "FDC from Dewey, depth 1",
    "lcc-gold": "LCC, librarian call number", "learned-own": "learned scheme, own insertion",
    "fdc-placed": "FDC, lexical@2 classifier", "lcc-placed": "LCC, lexical@2 classifier",
    "unesco-thesaurus-placed": "UNESCO thesaurus, lexical@2 classifier",
}


def t_availability():
    rows = []
    edition = jsonl(ROOT / "bench/corpus/holdout-ddc.jsonl")
    k = sum(bool(ddc.first_number(r)) for r in edition)
    lo, hi = wilson(k, len(edition))
    rows.append(["holdout (500 editions)", "edition record", "Dewey number", f"{k}/{len(edition)}",
                 f(k / len(edition), 3), interval(lo, hi, 3), "bench/corpus/holdout-ddc.jsonl"])
    for sample, label in (("holdout", "holdout (500 works)"), ("books50", "50 well-known books")):
        works = jsonl(P / f"availability/{sample}-works.jsonl")
        n = len(works)
        for what, test in (("Dewey number", lambda w: any(ddc.ddc_number(v) for v in w["ddc"])),
                           ("LC call number", lambda w: bool(w["lcc"])),
                           ("both", lambda w: any(ddc.ddc_number(v) for v in w["ddc"]) and bool(w["lcc"]))):
            k = sum(test(w) for w in works)
            lo, hi = wilson(k, n)
            rows.append([label, "work (all editions)", what, f"{k}/{n}", f(k / n, 3), interval(lo, hi, 3),
                         f"bench/results/2026-10-10-proposal/availability/{sample}-works.jsonl"])
    placed = jsonl(P / "placements/holdout/fdc-work.jsonl")
    k = sum(bool(r["classes"]) for r in placed)
    lo, hi = wilson(k, len(placed))
    rows.append(["holdout (500 works)", "edition, else work consensus", "FDC class reached", f"{k}/{len(placed)}",
                 f(k / len(placed), 3), interval(lo, hi, 3),
                 "bench/results/2026-10-10-proposal/placements/holdout/fdc-work.jsonl"])
    return table(["sample", "level", "number", "count", "share", "95% Wilson interval", "source"], rows,
                 ["---", "---", "---", "---:", "---:", "---:", "---"])


BOOK_MAIN = ["ddc-gold", "ddc-section", "fdc-ddc", "fdc-d3", "fdc-d2", "lcc-gold", "learned-own",
             "fdc-placed", "lcc-placed", "unesco-thesaurus-placed"]


def t_books_main():
    rows = []
    for run in BOOK_MAIN:
        rows.append([f"`{run}`", NAMES[run], f(point("books", run, "coverage"), 3),
                     f(point("books", run, "cost_norm")) + " " + interval(*ci("books", run, "cost_norm")),
                     f(point("books", run, "skill"), 3) + " " + interval(*ci("books", run, "skill"), 3),
                     f(point("books", run, "nav_log_a1"), 2), f(point("books", run, "mean_depth"), 2),
                     f(point("books", run, "effective_classes"), 1),
                     f(j_point("books", run, LAMBDA0))])
    return table(["run", "structure", "coverage", "cost_norm [95% CI]", "skill [95% CI]", "nav bits",
                  "mean depth", "effective classes", f"J (λ = {LAMBDA0})"], rows,
                 ["---", "---"] + ["---:"] * 7)


def t_metrics_upgrade():
    """Query-weighted navigation and permutation nulls on all three book graphs."""
    runs = json.loads(METRICS_R.read_text(encoding="utf-8"))["runs"]
    names = ["ddc-section", "fdc-d3", "lcc-gold", "learned-own"]
    rows = []
    def value_ci(value, ci, digits=2):
        return f"{value:.{digits}f} [{ci[0]:.{digits}f}, {ci[1]:.{digits}f}]"
    for model in MODELS:
        for name in names:
            r = runs[f"{model}-k20/{name}.json"]
            m = r["metrics_upgrade"]
            u = m["query_workloads"]["uniform"]
            c = m["confidence_intervals"]
            null = r["permutation_null"]["cost_norm"]
            skill_null = r["permutation_null"]["skill"]
            rows.append([
                model, f"`{name}`", value_ci(r["coverage"], r["bootstrap"]["ci"]["coverage"], 3),
                value_ci(u["placed_only"]["menu_accesses"], c["uniform.menu_accesses.placed"]),
                value_ci(u["including_unplaced"]["menu_accesses"], c["uniform.menu_accesses.all"]),
                value_ci(u["including_unplaced"]["items_scanned"], c["uniform.items_scanned.all"]),
                value_ci(m["information_bounds"]["entropy_bits"], c["information.entropy_bits"]),
                value_ci(m["information_bounds"]["alphabetic_optimum_bits"], c["information.alphabetic_optimum_bits"]),
                value_ci(m["information_bounds"]["nav_over_alphabetic"], c["information.nav_over_alphabetic"]),
                *[value_ci(m["branch_constant_sweep"][f"a={a}"], c[f"branch_constant.a{a}"]) for a in (0, 1, 2, 4)],
                value_ci(null["mean"], null["confidence_intervals"]["mean"]),
                value_ci(null["sd"], null["confidence_intervals"]["sd"]),
                value_ci(null["z"], null["confidence_intervals"]["z"]),
                value_ci(null["percentile"], null["confidence_intervals"]["percentile"], 3),
                value_ci(skill_null["mean"], skill_null["confidence_intervals"]["mean"]),
                value_ci(skill_null["sd"], skill_null["confidence_intervals"]["sd"]),
                value_ci(skill_null["z"], skill_null["confidence_intervals"]["z"]),
                value_ci(skill_null["percentile"], skill_null["confidence_intervals"]["percentile"], 3),
            ])
    return table(["embedding", "scheme", "coverage", "menu accesses: placed", "menu accesses: all",
                  "items scanned: all", "entropy (bits)", "alphabetic optimum (bits)", "nav / bound",
                  "log2 menu cost a=0", "a=1", "a=2", "a=4",
                  "null cost mean", "null cost SD", "null cost z", "null cost percentile",
                  "null skill mean", "null skill SD", "null skill z", "null skill percentile"], rows,
                 ["---", "---"] + ["---:"] * 19)


PAIRS_BOOKS = [("fdc-ddc", "lcc-gold"), ("fdc-d3", "lcc-gold"), ("fdc-d3", "ddc-section"),
               ("fdc-d3", "fdc-ddc"), ("fdc-d3", "fdc-d2"), ("fdc-d3", "fdc-placed"),
               ("learned-own", "fdc-d3")]


def t_paired_books():
    tests = [(a, b, m, run_test("books", a, b, m)) for a, b in PAIRS_BOOKS for m in ("cost_norm", "J")]
    holm([t for *_, t in tests])
    rows = []
    for a, b, m, t in tests:
        rel = t["diff"] / point("books", b, "cost_norm") if m == "cost_norm" else None
        rows.append([f"`{a}` − `{b}`", "cost_norm" if m == "cost_norm" else f"J (λ = {LAMBDA0})",
                     *test_cells(t), f"{100 * rel:+.1f}%" if rel is not None else ""])
    return table(["comparison", "score", "Δ", "95% CI", "p", "p (Holm)", "Δ / cost of b"], rows,
                 ["---", "---"] + ["---:"] * 5)


CATALOGUE = [("triplet.accuracy", "vertex.triplet.accuracy", "triplet accuracy (higher better)"),
             ("order.locality", "vertex.order.locality", "shelf-order locality (higher better)"),
             ("tsd.nats", "vertex.tsd.nats", "tree-sampling divergence, nats (higher better)")]


def t_catalogue_books():
    runs = ["fdc-d3", "fdc-ddc", "ddc-section", "lcc-gold", "learned-own", "fdc-placed"]
    rows = []
    for key, label in [("dasgupta.cost_norm", "cost_norm (lower better)")] + [(k, l) for k, _, l in CATALOGUE] + [
            ("hospitality.array", "hospitality in array"), ("insertion.splits_per_1000", "splits per 1,000 inserts"),
            ("notation.length", "notation length, characters"), ("search.scheme_bits", "expected search bits")]:
        rows.append([label] + [f(scores(f"books-{r}")["catalogue"]["scores"][key], 4 if "cost" in key or
                                "accuracy" in key or "locality" in key else 3) for r in runs])
    return table(["score"] + [f"`{r}`" for r in runs], rows)


def catalogue_test(a, b, boot_key, score_key, prefix="books-"):
    sa, sb = scores(f"{prefix}{a}")["catalogue"], scores(f"{prefix}{b}")["catalogue"]
    return paired(sa["boot"][boot_key], sb["boot"][boot_key], sa["scores"][score_key], sb["scores"][score_key])


def t_catalogue_tests():
    tests = []
    for a, b in (("ddc-section", "lcc-gold"), ("fdc-d3", "lcc-gold"), ("learned-own", "ddc-section")):
        for key, boot_key, label in [("dasgupta.cost_norm", "vertex.dasgupta.cost_norm", "cost_norm")] + CATALOGUE:
            tests.append((a, b, label.split(" (")[0], catalogue_test(a, b, boot_key, key)))
    holm([t for *_, t in tests])
    return table(["comparison", "score", "Δ", "95% CI (vertex)", "p", "p (Holm)"],
                 [[f"`{a}` − `{b}`", s, *test_cells(t)] for a, b, s, t in tests], ["---", "---"] + ["---:"] * 4)


DEPTH_RUNS = ["fdc-d1", "fdc-d2", "fdc-d3", "fdc-ddc", "fdc-m5", "fdc-m10", "fdc-m25", "fdc-m50",
              "ddc-d1", "ddc-d2", "ddc-section", "ddc-d4", "ddc-gold"]


def regret(runs, model="minilm", k=20, steps=451):
    """Max over λ in LAMBDA_RANGE of J(run) − min J; returns {run: regret}."""
    lo, hi = LAMBDA_RANGE
    worst = dict.fromkeys(runs, 0.0)
    for i in range(steps):
        lam = lo + (hi - lo) * i / (steps - 1)
        js = {r: j_point("books", r, lam, model, k) for r in runs}
        best = min(js.values())
        for r in runs:
            worst[r] = max(worst[r], js[r] - best)
    return worst


FDC_DEPTHS = ["fdc-d1", "fdc-d2", "fdc-d3", "fdc-ddc", "fdc-m5", "fdc-m10", "fdc-m25", "fdc-m50"]


def t_depth():
    worst = regret(FDC_DEPTHS)
    rows = []
    for run in DEPTH_RUNS:
        cells = [f"`{run}`", f(point("books", run, "cost_norm")), f(point("books", run, "nav_log_a1"), 2),
                 f(point("books", run, "effective_classes"), 1)]
        cells += [f(j_point("books", run, lam)) for lam in LAMBDAS]
        cells.append(f(worst[run]) if run in worst else "")
        ls = lambda_star("books", run, "fdc-d3", "minilm", 20) if run != "fdc-d3" else None
        cells.append(f(ls) if ls else "")
        rows.append(cells)
    return table(["run", "cost_norm", "nav bits", "eff. classes"] + [f"J, λ = {lam}" for lam in LAMBDAS]
                 + ["max regret, FDC cuts", "λ where J = J(`fdc-d3`)"], rows)


def lambda_star(universe, a, b, model, k):
    """λ at which J(a) = J(b), or None when one dominates."""
    dc = point(universe, a, "cost_norm", model, k) - point(universe, b, "cost_norm", model, k)
    dn = point(universe, a, "nav_log_a1", model, k) - point(universe, b, "nav_log_a1", model, k)
    return -dc / dn if dn and -dc / dn > 0 else None


def t_embedding():
    tests, rows = [], []
    for model in MODELS:
        for k in KS:
            t1 = run_test("books", "fdc-d3", "lcc-gold", "cost_norm", model, k)
            t2 = run_test("books", "learned-own", "fdc-d3", "cost_norm", model, k)
            tests += [t1, t2]
            worst = regret(FDC_DEPTHS, model, k)
            pick = min(worst, key=worst.get)
            ls = lambda_star("books", "fdc-d3", "lcc-gold", model, k)
            rows.append([f"{model} k = {k}", f(point("books", "fdc-d3", "cost_norm", model, k)),
                         f(point("books", "lcc-gold", "cost_norm", model, k)), t1,
                         f(point("books", "learned-own", "cost_norm", model, k)), t2,
                         f"`{pick}`", f(ls) if ls else "never"])
    holm(tests)
    out = []
    for r in rows:
        t1, t2 = r[3], r[5]
        out.append(r[:3] + [sf(t1["diff"]), interval(t1["lo"], t1["hi"], signed=True), pv(t1["p_holm"]), r[4],
                            sf(t2["diff"]), interval(t2["lo"], t2["hi"], signed=True), pv(t2["p_holm"])] + r[6:])
    return table(["graph", "FDC d3", "LCC", "Δ FDC − LCC", "95% CI", "p (Holm)", "learned",
                  "Δ learned − FDC", "95% CI", "p (Holm)", "minimax depth", "λ* FDC = LCC"], out)


def t_pipelines():
    tests, rows = [], []
    for share in (20, 40, 60, 80, 90):
        a, b = f"fdc-pipe-{share}", f"lcc-pipe-{share}"
        t = run_test("books", a, b, "cost_norm")
        tj = run_test("books", a, b, "J")
        tests += [t, tj]
        rows.append([f"{share}%", f(point("books", a, "coverage"), 3), f(point("books", a, "cost_norm")),
                     f(point("books", b, "coverage"), 3), f(point("books", b, "cost_norm")), t, tj])
    for label, a, b in (("100% (gold)", "fdc-ddc", "lcc-gold"), ("0% (classifier only)", "fdc-placed", "lcc-placed")):
        t, tj = run_test("books", a, b, "cost_norm"), run_test("books", a, b, "J")
        tests += [t, tj]
        rows.append([label, f(point("books", a, "coverage"), 3), f(point("books", a, "cost_norm")),
                     f(point("books", b, "coverage"), 3), f(point("books", b, "cost_norm")), t, tj])
    holm(tests)
    out = [r[:5] + [sf(r[5]["diff"]), interval(r[5]["lo"], r[5]["hi"], signed=True), pv(r[5]["p_holm"]),
                    sf(r[6]["diff"]), pv(r[6]["p_holm"])] for r in rows]
    return table(["books with a librarian number", "FDC coverage", "FDC cost", "LCC coverage", "LCC cost",
                  "Δ cost FDC − LCC", "95% CI", "p (Holm)", f"Δ J (λ = {LAMBDA0})", "p (Holm)"], out)


def t_holdout():
    tests, rows = [], []
    for model in MODELS:
        for a, b in (("fdc-work", "lcc-gold"), ("fdc-pipeline", "lcc-pipeline"), ("fdc-work", "fdc-ddc")):
            t = run_test("holdout", a, b, "cost_norm", model, 20)
            tests.append(t)
            rows.append([model, f"`{a}` − `{b}`", f(point("holdout", a, "coverage", model), 3),
                         f(point("holdout", a, "cost_norm", model)), f(point("holdout", b, "coverage", model), 3),
                         f(point("holdout", b, "cost_norm", model)), t])
        a, b = (scores(f"holdout-both-{model}-{r}") for r in ("fdc-work", "lcc-gold"))
        t = paired(a["catalogue"]["boot"]["vertex.dasgupta.cost_norm"],
                   b["catalogue"]["boot"]["vertex.dasgupta.cost_norm"],
                   a["catalogue"]["scores"]["dasgupta.cost_norm"], b["catalogue"]["scores"]["dasgupta.cost_norm"])
        tests.append(t)
        n = a["n"]
        rows.append([model, f"`fdc-work` − `lcc-gold`, the {n} books both place", "1.000",
                     f(a["catalogue"]["scores"]["dasgupta.cost_norm"]), "1.000",
                     f(b["catalogue"]["scores"]["dasgupta.cost_norm"]), t])
    holm(tests)
    return table(["graph (k = 20)", "comparison", "coverage a", "cost a", "coverage b", "cost b", "Δ", "95% CI",
                  "p", "p (Holm)"], [r[:6] + test_cells(r[6]) for r in rows], ["---", "---"] + ["---:"] * 8)


PAPER_RUNS = ["arxiv-gold", "fdc-arxiv-nested", "lcc-arxiv-nested", "fdc-arxiv", "lcc-arxiv", "learned-placed",
              "openalex-topics-placed", "fdc-placed"]


def t_papers():
    rows = []
    for run in PAPER_RUNS:
        rows.append([f"`{run}`"] + [f"{f(point('corpus', run, 'cost_norm', m))} / {f(point('corpus', run, 'nav_log_a1', m), 2)}"
                                    for m in MODELS] + [f(point("corpus", run, "coverage"), 3)])
    return table(["run", "minilm cost / nav", "bge-small cost / nav", "nomic cost / nav", "coverage"], rows)


def t_papers_tests():
    tests, rows = [], []
    for model in MODELS:
        for a, b in (("fdc-arxiv-nested", "arxiv-gold"), ("fdc-arxiv-nested", "lcc-arxiv-nested"),
                     ("fdc-arxiv-nested", "fdc-arxiv")):
            t = run_test("corpus", a, b, "cost_norm", model, 20)
            tj = run_test("corpus", a, b, "J", model, 20)
            tests += [t, tj]
            rows.append([model, f"`{a}` − `{b}`", t, tj])
    holm(tests)
    return table(["graph (k = 20)", "comparison", "Δ cost", "95% CI", "p (Holm)", f"Δ J (λ = {LAMBDA0})",
                  "95% CI", "p (Holm)"],
                 [[m, c, sf(t["diff"]), interval(t["lo"], t["hi"], signed=True), pv(t["p_holm"]), sf(tj["diff"]),
                   interval(tj["lo"], tj["hi"], signed=True), pv(tj["p_holm"])] for m, c, t, tj in rows],
                 ["---", "---"] + ["---:"] * 6)


def ancestor_at(code, parents, depth):
    chain = []
    while code:
        chain.append(code)
        code = parents.get(code)
    chain = chain[::-1]
    return chain[min(depth, len(chain)) - 1]


def t_classifier():
    rows = []
    src = DDC_R / "placements/books"
    for scheme, gold, placed, path in (("FDC", "fdc-ddc", "fdc-placed", ROOT / "schemes/fdc/fdc.json"),
                                       ("LCC", "lcc-gold", "lcc-placed", ROOT / "data/schemes/lcc/lcc.json")):
        parents = {c["code"]: c.get("parent") for c in json.loads(path.read_text())["classes"]}
        g = {r["id"]: r["classes"][0] for r in jsonl(src / f"{gold}.jsonl") if r["classes"]}
        p = {r["id"]: r["classes"][0] for r in jsonl(src / f"{placed}.jsonl") if r["classes"]}
        stats = json.loads((src / f"{placed}.stats.json").read_text())
        both = [i for i in p if i in g]
        for depth in (1, 2, 3):
            hits = sum(ancestor_at(p[i], parents, depth) == ancestor_at(g[i], parents, depth) for i in both)
            lo, hi = wilson(hits, len(both))
            rows.append([scheme, depth, f"{len(p)}/{len(g)}", f"{stats['placed'] - stats['flagged']}",
                         f"{hits}/{len(both)}", f(hits / len(both), 3), interval(lo, hi, 3)])
    return table(["scheme", "depth", "placed / books", "decided without flag", "same class as librarian",
                  "agreement", "95% Wilson interval"], rows, ["---"] + ["---:"] * 6)


def t_ddc_reference():
    d = json.loads((DDC_R / "bootstrap/books.json").read_text())
    want = {("ddc-gold", "lcc-gold"), ("ddc-gold", "fdc-ddc"), ("fdc-ddc", "lcc-gold"), ("ddc-gold", "ddc-section"),
            ("ddc-gold", "learned-own"), ("fdc-ddc", "fdc-placed")}
    rows = []
    for metric in ("cost_norm", "nav_log_a1"):
        for r in d["paired"][metric]:
            if (r["a"], r["b"]) in want:
                share = r["share_a_lower"]
                rows.append([f"`{r['a']}` − `{r['b']}`", metric, sf(r["diff"], 4 if metric == "cost_norm" else 2),
                             interval(*r["ci"], 4 if metric == "cost_norm" else 2, signed=True), f(share, 3)])
    return table(["comparison", "score", "Δ", "95% CI", "share of draws a lower"], rows, ["---", "---", "---:",
                                                                                           "---:", "---:"])


def t_hybrid():
    """FDC sections with the learned scheme's paths nested inside, against plain sections and the learned
    scheme, on each model's k = 20 books graph."""
    tests, rows = [], []
    for model in MODELS:
        for a, b in (("fdc-learned1", "fdc-d3"), ("fdc-learned2", "fdc-d3"), ("fdc-learned3", "fdc-d3"),
                     ("learned-own", "fdc-learned3")):
            t = run_test("books", a, b, "cost_norm", model, 20)
            tj = run_test("books", a, b, "J", model, 20)
            tests += [t, tj]
            rows.append([model, f"`{a}` − `{b}`", f(point("books", a, "cost_norm", model)),
                         f(point("books", a, "nav_log_a1", model), 2), t, tj])
    holm(tests)
    return table(["graph (k = 20)", "comparison", "cost a", "nav bits a", "Δ cost", "95% CI", "p (Holm)",
                  f"Δ J (λ = {LAMBDA0})", "p (Holm)"],
                 [r[:4] + [sf(r[4]["diff"]), interval(r[4]["lo"], r[4]["hi"], signed=True), pv(r[4]["p_holm"]),
                           sf(r[5]["diff"]), pv(r[5]["p_holm"])] for r in rows], ["---", "---"] + ["---:"] * 7)


def fold(item_id, folds=5):
    return int.from_bytes(hashlib.sha256(f"kb-crosswalk|{item_id}".encode()).digest()[:8], "big") % folds


def crosswalk(train, lcc_parents, fdc_parents, depth):
    """LCC class (and every ancestor) -> modal FDC class at `depth` over the training books."""
    votes = {}
    for lcc, fdc in train:
        target = ancestor_at(fdc, fdc_parents, depth)
        for a in chain_of(lcc, lcc_parents):
            votes.setdefault(a, Counter())[target] += 1
    return {a: min(c, key=lambda x: (-c[x], x)) for a, c in votes.items()}


def chain_of(code, parents):
    out = []
    while code:
        out.append(code)
        code = parents.get(code)
    return out


def predict(lcc, table_, lcc_parents):
    """The deepest ancestor of the LCC class that the crosswalk has seen."""
    return next((table_[a] for a in chain_of(lcc, lcc_parents) if a in table_), None)


def t_crosswalk():
    """LCC call number -> FDC class, learned from books carrying both numbers: 5-fold cross-validation on
    the 3,000 books, and trained on all 3,000 and tested on the 500-book holdout's works."""
    lp = {c["code"]: c.get("parent") for c in json.loads((ROOT / "data/schemes/lcc/lcc.json").read_text())["classes"]}
    fp = {c["code"]: c.get("parent") for c in json.loads((ROOT / "schemes/fdc/fdc.json").read_text())["classes"]}
    src, hold = DDC_R / "placements/books", DDC_R / "placements/holdout"
    lcc = {r["id"]: r["classes"][0] for r in jsonl(src / "lcc-gold.jsonl") if r["classes"]}
    fdc = {r["id"]: r["classes"][0] for r in jsonl(src / "fdc-ddc.jsonl") if r["classes"]}
    books = sorted(i for i in lcc if i in fdc)
    hl = {r["id"]: r["classes"][0] for r in jsonl(hold / "lcc-gold.jsonl") if r["classes"]}
    hf = {r["id"]: r["classes"][0] for r in jsonl(P / "placements/holdout/fdc-work.jsonl") if r["classes"]}
    held = sorted(i for i in hl if i in hf)
    rows = []
    for depth in (1, 2, 3):
        hits = 0
        for k in range(5):
            cw = crosswalk([(lcc[i], fdc[i]) for i in books if fold(i) != k], lp, fp, depth)
            hits += sum(predict(lcc[i], cw, lp) == ancestor_at(fdc[i], fp, depth) for i in books if fold(i) == k)
        lo, hi = wilson(hits, len(books))
        rows.append(["3,000 books, 5-fold", depth, f"{hits}/{len(books)}", f(hits / len(books), 3),
                     interval(lo, hi, 3)])
        cw = crosswalk([(lcc[i], fdc[i]) for i in books], lp, fp, depth)
        hits = sum(predict(hl[i], cw, lp) == ancestor_at(hf[i], fp, depth) for i in held)
        lo, hi = wilson(hits, len(held))
        rows.append(["500-book holdout works, trained on the 3,000", depth, f"{hits}/{len(held)}",
                     f(hits / len(held), 3), interval(lo, hi, 3)])
    return table(["test", "FDC depth", "same class as the Dewey number", "accuracy", "95% Wilson interval"],
                 rows, ["---"] + ["---:"] * 4)


CROSSWALK_TAU, CROSSWALK_MIN = 0.8, 5
CROSSWALK_FILE = P / "crosswalk/lcc-fdc.json"


def crosswalk_inputs():
    lp = {c["code"]: c.get("parent") for c in json.loads((ROOT / "data/schemes/lcc/lcc.json").read_text())["classes"]}
    fp = {c["code"]: c.get("parent") for c in json.loads((ROOT / "schemes/fdc/fdc.json").read_text())["classes"]}
    src, hold = DDC_R / "placements/books", DDC_R / "placements/holdout"
    lcc = {r["id"]: r["classes"][0] for r in jsonl(src / "lcc-gold.jsonl") if r["classes"]}
    fdc = {r["id"]: r["classes"][0] for r in jsonl(src / "fdc-ddc.jsonl") if r["classes"]}
    hl = {r["id"]: r["classes"][0] for r in jsonl(hold / "lcc-gold.jsonl") if r["classes"]}
    hf = {r["id"]: r["classes"][0] for r in jsonl(P / "placements/holdout/fdc-work.jsonl") if r["classes"]}
    return lp, fp, lcc, fdc, hl, hf


def crosswalk_votes(train, lcc_parents, fdc_parents):
    """{depth: {LCC class or ancestor: Counter of FDC classes at that depth}}."""
    votes = {d: {} for d in (1, 2, 3)}
    for lcc, fdc in train:
        for d in (1, 2, 3):
            target = ancestor_at(fdc, fdc_parents, d)
            for a in chain_of(lcc, lcc_parents):
                votes[d].setdefault(a, Counter())[target] += 1
    return votes


def crosswalk_rule(lcc, votes, lcc_parents):
    """Deepest FDC depth whose vote, at the deepest LCC ancestor with >= CROSSWALK_MIN books, has a modal
    share >= CROSSWALK_TAU; (depth, class) or None (review queue)."""
    for d in (3, 2, 1):
        for a in chain_of(lcc, lcc_parents):
            c = votes[d].get(a)
            if c and sum(c.values()) >= CROSSWALK_MIN:
                top, n = min(c.items(), key=lambda x: (-x[1], x[0]))
                if n / sum(c.values()) >= CROSSWALK_TAU:
                    return d, top
                break
    return None


def crosswalk_file():
    """The crosswalk trained on all 3,000 books, as committed for the placement rule of report 9."""
    lp, fp, lcc, fdc, _, _ = crosswalk_inputs()
    books = sorted(i for i in lcc if i in fdc)
    votes = crosswalk_votes([(lcc[i], fdc[i]) for i in books], lp, fp)
    entries = {}
    for d in (1, 2, 3):
        for a, c in sorted(votes[d].items()):
            total = sum(c.values())
            if total < CROSSWALK_MIN:
                continue
            top, n = min(c.items(), key=lambda x: (-x[1], x[0]))
            entries.setdefault(a, {})[str(d)] = {"fdc": top, "books": total, "share": round(n / total, 4)}
    return {"description": "LCC class (bench LCC scheme code) -> modal FDC class at FDC depth 1-3 over the "
                           "3,000 books carrying both a Dewey and an LC call number (report 9). Apply with "
                           "the rule: deepest depth whose entry, at the deepest LCC ancestor listed, has "
                           f"share >= {CROSSWALK_TAU}; else review.",
            "tau": CROSSWALK_TAU, "min_books": CROSSWALK_MIN, "sources": [
                "bench/results/2026-10-10-ddc/placements/books/lcc-gold.jsonl",
                "bench/results/2026-10-10-ddc/placements/books/fdc-ddc.jsonl"],
            "classes": entries}


def t_crosswalk_rule():
    lp, fp, lcc, fdc, hl, hf = crosswalk_inputs()
    books = sorted(i for i in lcc if i in fdc)
    held = sorted(i for i in hl if i in hf)
    rows = []

    def tally(outcomes, label, n):
        decided = [o for o in outcomes if o]
        ok = sum(o[1] for o in decided)
        lo, hi = wilson(ok, len(decided))
        depth = Counter(o[0] for o in decided)
        rows.append([label, f"{len(decided)}/{n}", f(len(decided) / n, 3), " / ".join(str(depth[d]) for d in (1, 2, 3)),
                     f"{ok}/{len(decided)}", f(ok / len(decided), 3), interval(lo, hi, 3)])

    def outcome(lcc_code, fdc_code, votes):
        r = crosswalk_rule(lcc_code, votes, lp)
        return r and (r[0], r[1] == ancestor_at(fdc_code, fp, r[0]))

    outcomes = []
    for k in range(5):
        votes = crosswalk_votes([(lcc[i], fdc[i]) for i in books if fold(i) != k], lp, fp)
        outcomes += [outcome(lcc[i], fdc[i], votes) for i in books if fold(i) == k]
    tally(outcomes, "3,000 books, 5-fold", len(books))
    votes = crosswalk_votes([(lcc[i], fdc[i]) for i in books], lp, fp)
    tally([outcome(hl[i], hf[i], votes) for i in held], "500-book holdout works, trained on the 3,000", len(held))
    if json.loads(CROSSWALK_FILE.read_text()) != crosswalk_file():
        sys.exit(f"{CROSSWALK_FILE.relative_to(ROOT)} drifted from its inputs (rerun with --write)")
    return table(["test", "decided", "share decided", "at FDC depth 1 / 2 / 3", "correct", "accuracy",
                  "95% Wilson interval"], rows, ["---"] + ["---:"] * 6)


CUTS = ["fdc-d1", "fdc-d2", "fdc-d3", "fdc-ddc", "ddc-d1", "ddc-d2", "ddc-section", "ddc-d4", "ddc-gold"]


def t_dewey_cut():
    """The librarian's Dewey number cut to its digits (main class / division / section) against the same
    number mapped into the FDC tree and cut at depth 3, on every books graph; minimax regret over all cuts."""
    tests, rows = [], []
    for model in MODELS:
        for k in KS:
            t = run_test("books", "ddc-section", "fdc-d3", "cost_norm", model, k)
            tj = run_test("books", "ddc-section", "fdc-d3", "J", model, k)
            tests += [t, tj]
            worst = regret(CUTS, model, k)
            pick = min(worst, key=worst.get)
            ls = lambda_star("books", "ddc-d2", "ddc-section", model, k)
            rows.append([f"{model} k = {k}", t, tj, f"`{pick}`", f(worst["ddc-section"]), f(worst["ddc-d2"]),
                         f(worst["fdc-d3"]), f(ls) if ls else "never"])
    holm(tests)
    return table(["graph", "Δ cost `ddc-section` − `fdc-d3`", "95% CI", "p (Holm)", f"Δ J (λ = {LAMBDA0})",
                  "95% CI", "p (Holm)", "minimax cut", "max regret `ddc-section`", "max regret `ddc-d2`",
                  "max regret `fdc-d3`", "λ where `ddc-d2` = `ddc-section`"],
                 [[r[0], sf(r[1]["diff"]), interval(r[1]["lo"], r[1]["hi"], signed=True), pv(r[1]["p_holm"]),
                   sf(r[2]["diff"]), interval(r[2]["lo"], r[2]["hi"], signed=True), pv(r[2]["p_holm"])] + r[3:]
                  for r in rows])


def t_section_lcc():
    """Dewey sections against the librarians' LCC call numbers on every books graph."""
    tests, rows = [], []
    for model in MODELS:
        for k in KS:
            t = run_test("books", "ddc-section", "lcc-gold", "cost_norm", model, k)
            tj = run_test("books", "ddc-section", "lcc-gold", "J", model, k)
            tests += [t, tj]
            ls = lambda_star("books", "ddc-section", "lcc-gold", model, k)
            rows.append([f"{model} k = {k}", f(point("books", "ddc-section", "cost_norm", model, k)),
                         f(point("books", "lcc-gold", "cost_norm", model, k)), t, tj, f(ls) if ls else "never"])
    holm(tests)
    return table(["graph", "`ddc-section`", "`lcc-gold`", "Δ cost", "95% CI", "p (Holm)", f"Δ J (λ = {LAMBDA0})",
                  "95% CI", "p (Holm)", "λ where J equal"],
                 [r[:3] + [sf(r[3]["diff"]), interval(r[3]["lo"], r[3]["hi"], signed=True), pv(r[3]["p_holm"]),
                           sf(r[4]["diff"]), interval(r[4]["lo"], r[4]["hi"], signed=True), pv(r[4]["p_holm"]), r[5]]
                  for r in rows])


def t_locality_weight():
    """J − μ · locality: the μ above which LCC's better shelf-order locality outweighs its higher J."""
    rows = []
    lam_nav = {r: point("books", r, "nav_log_a1") for r in ("ddc-section", "fdc-d3", "lcc-gold")}
    for a in ("ddc-section", "fdc-d3"):
        sa, sb = scores(f"books-{a}")["catalogue"]["scores"], scores("books-lcc-gold")["catalogue"]["scores"]
        dj = (sa["dasgupta.cost_norm"] + LAMBDA0 * lam_nav[a]) - (sb["dasgupta.cost_norm"] + LAMBDA0 * lam_nav["lcc-gold"])
        dl = sa["order.locality"] - sb["order.locality"]
        rows.append([f"`{a}` − `lcc-gold`", sf(dj), sf(dl), f(dj / dl, 2) if dl < 0 < -dj else "never"])
    return table(["comparison", f"Δ J (λ = {LAMBDA0})", "Δ locality", "μ* (LCC preferred above)"], rows)


def t_captions():
    """How many of the Dewey sections the 3,000 books use have an FDC class (and caption) of the same number."""
    fdc = {c["code"] for c in json.loads((ROOT / "schemes/fdc/fdc.json").read_text())["classes"]}
    used = Counter(r["classes"][0] for r in jsonl(DDC_R / "placements/books/ddc-section.jsonl") if r["classes"])
    own = [s for s in used if s in fdc]
    return table(["sections used", "with an FDC class of the same number", "books in those sections"],
                 [[len(used), f"{len(own)} ({f(len(own) / len(used), 3)})",
                   f"{sum(used[s] for s in own)}/{sum(used.values())} ({f(sum(used[s] for s in own) / sum(used.values()), 3)})"]],
                 ["---:"] * 3)


TABLES = {
    "availability": t_availability, "books-main": t_books_main, "metrics-upgrade": t_metrics_upgrade,
    "paired-books": t_paired_books,
    "ddc-reference": t_ddc_reference, "catalogue-books": t_catalogue_books, "catalogue-tests": t_catalogue_tests,
    "depth": t_depth, "embedding": t_embedding, "pipelines": t_pipelines, "holdout": t_holdout,
    "papers": t_papers, "papers-tests": t_papers_tests, "classifier": t_classifier,
    "hybrid": t_hybrid, "crosswalk": t_crosswalk, "crosswalk-rule": t_crosswalk_rule,
    "dewey-cut": t_dewey_cut, "section-lcc": t_section_lcc,
    "locality-weight": t_locality_weight, "captions": t_captions,
}


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--write", action="store_true", help="rewrite the tables in the report")
    a = p.parse_args()
    if a.write:
        crosswalk_text = json.dumps(crosswalk_file(), indent=1, sort_keys=True) + "\n"
        if not CROSSWALK_FILE.exists() or CROSSWALK_FILE.read_text(encoding="utf-8") != crosswalk_text:
            CROSSWALK_FILE.parent.mkdir(parents=True, exist_ok=True)
            CROSSWALK_FILE.write_text(crosswalk_text, encoding="utf-8")
    text = REPORT.read_text(encoding="utf-8")
    drift, missing = [], []
    for name, fn in TABLES.items():
        pattern = re.compile(rf"(<!-- table:{re.escape(name)} -->\n)(.*?)(\n<!-- /table:{re.escape(name)} -->)", re.S)
        m = pattern.search(text)
        if not m:
            missing.append(name)
            continue
        new = fn()
        if m.group(2) != new:
            drift.append(name)
            text = text[:m.start(2)] + new + text[m.end(2):]
    if missing:
        sys.exit(f"report lacks table markers: {', '.join(missing)}")
    if a.write:
        REPORT.write_text(text, encoding="utf-8")
        print(f"wrote {len(TABLES)} tables ({len(drift)} changed)")
        return
    if drift:
        sys.exit(f"tables drifted from committed results: {', '.join(drift)}")
    print(f"all {len(TABLES)} tables match the committed results")


if __name__ == "__main__":
    main()
