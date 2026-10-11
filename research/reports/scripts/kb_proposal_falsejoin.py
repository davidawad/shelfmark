"""Report 9 tables for the false-join correction and the two external checks.

Loaded by `kb_proposal_tables.py`, which calls `register(module)` with itself
so these tables share its statistics and formatting (paired percentile
intervals, two-sided bootstrap p, Holm within each table). Inputs, all
committed:

- `bench/results/2026-10-11-falsejoin/bootstrap/*.json.gz` and `.draws.json.gz`
  (`crates/bench/scripts/falsejoin_rescore.py`): every run on the same 1,000
  paired item draws with cost_norm, the two false-join forms and nav_log_a1;
- `docs/validity/falsejoin-results.json` and
  `bench/results/2026-10-11-falsejoin/external/agreement.json`
  (`crates/bench/scripts/falsejoin_validity.py`): agreement of every metric
  variant with the stored judge labels and with the external signals, and the
  external separation of every run on resampled pairs.

The corrected objective is J' = C + β · FJs + λ · N, with FJs the specificity
form of the false-join term, β = 1 (PRIMARY) and λ as in §1. The outside-mass
form (FJ) is reported next to it.
"""
import gzip
import json
import math
import random
from functools import cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FJ = ROOT / "bench/results/2026-10-11-falsejoin"
VALIDITY = ROOT / "docs/validity/falsejoin-results.json"
AGREEMENT = FJ / "external/agreement.json"
BETA = 1.0
FORMS = {"FJs": "fjs_norm", "FJ": "fj_norm"}
SWEEP = [0.25, 0.5, 1.0, 2.0, 4.0]
BETA_RANGE = (0.5, 2.0)
NULL_PERMUTATIONS = 20
K = None


def register(module):
    """Bind the shared helpers of kb_proposal_tables.py and return this module's tables."""
    global K
    K = module
    return {"fj-validity": t_validity, "fj-books-main": t_books_main, "fj-section-lcc": t_section_lcc,
            "fj-beta-sweep": t_beta_sweep, "fj-null": t_null, "fj-cuts": t_cuts, "fj-papers": t_papers,
            "fj-holdout": t_holdout, "ext-books": t_ext_books, "ext-papers": t_ext_papers}


# ---- loading -----------------------------------------------------------
@cache
def boot(universe, model="minilm", k=20):
    return json.loads(gzip.decompress((FJ / f"bootstrap/{universe}-{model}-k{k}.json.gz").read_bytes()))


@cache
def draws(universe, model="minilm", k=20):
    return json.loads(gzip.decompress((FJ / f"bootstrap/{universe}-{model}-k{k}.draws.json.gz").read_bytes()))


@cache
def validity():
    return json.loads(VALIDITY.read_text())


@cache
def agreement():
    return json.loads(AGREEMENT.read_text())


def value(universe, run, form, beta, model="minilm", k=20, lam=0.0):
    """Point value of C + β · form + λ · N (form None: C alone)."""
    p = boot(universe, model, k)["runs"][run]["point"]
    return p["cost_norm"] + (beta * p[FORMS[form]] if form else 0.0) + lam * p["nav_log_a1"]


def series(universe, run, form, beta, model="minilm", k=20, lam=0.0):
    r = draws(universe, model, k)["runs"][run]
    fj = r[FORMS[form]] if form else [0.0] * len(r["cost_norm"])
    return [c + beta * x + lam * n for c, x, n in zip(r["cost_norm"], fj, r["nav_log_a1"])]


def test(universe, a, b, form, beta=BETA, model="minilm", k=20, lam=0.0):
    return K.paired(series(universe, a, form, beta, model, k, lam), series(universe, b, form, beta, model, k, lam),
                    value(universe, a, form, beta, model, k, lam), value(universe, b, form, beta, model, k, lam))


def cells(t, d=4):
    return [K.sf(t["diff"], d), K.interval(t["lo"], t["hi"], d, signed=True), K.pv(t["p_holm"])]


def beta_star(universe, a, b, form, model, k):
    """β at which C + β · form is equal for a and b, or None when one dominates."""
    p, q = boot(universe, model, k)["runs"][a]["point"], boot(universe, model, k)["runs"][b]["point"]
    dc, dj = p["cost_norm"] - q["cost_norm"], p[FORMS[form]] - q[FORMS[form]]
    return -dc / dj if dj and -dc / dj > 0 else None


def ci_cell(universe, run, metric, model="minilm", k=20, d=4):
    r = boot(universe, model, k)["runs"][run]
    return f"{K.f(r['point'][metric], d)} {K.interval(*r['ci'][metric], d)}"


# ---- validity ------------------------------------------------------------
VALIDITY_ROWS = ["dasgupta", "fj", "fjs", "c+0.25fjs", "c+1fj", "c+1fjs", "c+4fjs"]


def ci_text(e, key, d=2, signed=False):
    g = (lambda x: f"{x:+.{d}f}") if signed else (lambda x: f"{x:.{d}f}")
    v, c = e[key], e[f"{key}_ci"]
    return "–" if v is None else f"{g(v)} [{g(c[0])}, {g(c[1])}]"


def t_validity():
    """Judge agreement (same 953 pairs and labels as docs/validity) and external agreement of each variant."""
    judge = {v["name"]: e for v, e in zip(validity()["variants"], validity()["groups"]["all runs"])}
    ext = agreement()["signals"]
    rows = []
    for v in VALIDITY_ROWS:
        j = judge[v]
        row = [f"`{v}`", ci_text(j, "kappa"), ci_text(j, "auc")]
        for signal in ("coupling", "cocitation", "lcsh"):
            e = ext[signal]["models"]["minilm"]["variants"][v]
            row += [ci_text(e, "auc"), ci_text(e, "rho")]
        rows.append(row)
    return K.table(["variant", "judge κ", "judge AUC", "coupling: pair AUC", "coupling: scheme ρ",
                    "co-citation: pair AUC", "co-citation: scheme ρ", "LCSH: pair AUC", "LCSH: scheme ρ"], rows)


# ---- books ---------------------------------------------------------------
BOOK_RUNS = ["ddc-section", "ddc-d2", "fdc-d3", "ddc-gold", "lcc-gold", "learned-own", "fdc-placed", "lcc-placed"]


def t_books_main():
    rows = []
    for run in BOOK_RUNS:
        rows.append([f"`{run}`", K.f(K.point("books", run, "coverage"), 3), ci_cell("books", run, "cost_norm"),
                     ci_cell("books", run, "fjs_norm"), ci_cell("books", run, "fj_norm"),
                     ci_cell("books", run, "cfjs_b1"), K.f(value("books", run, "FJs", BETA, lam=K.LAMBDA0)),
                     K.f(value("books", run, "FJ", BETA, lam=K.LAMBDA0))])
    return K.table(["run", "coverage", "C [95% CI]", "FJs [95% CI]", "FJ [95% CI]", "C + FJs [95% CI]",
                    f"J' = C + FJs + {K.LAMBDA0}·N", f"C + FJ + {K.LAMBDA0}·N"], rows)


def t_section_lcc():
    """Dewey sections against LCC on all nine books graphs under the corrected objective."""
    tests, rows = [], []
    for model in K.MODELS:
        for k in K.KS:
            ts = [test("books", "ddc-section", "lcc-gold", "FJs", BETA, model, k),
                  test("books", "ddc-section", "lcc-gold", "FJs", BETA, model, k, K.LAMBDA0),
                  test("books", "ddc-section", "lcc-gold", "FJ", BETA, model, k)]
            tests += ts
            bs = beta_star("books", "ddc-section", "lcc-gold", "FJs", model, k)
            rows.append([f"{model} k = {k}", ts, K.f(bs, 2) if bs else "never"])
    K.holm(tests)
    return K.table(["graph", "Δ (C + FJs)", "95% CI", "p (Holm)", f"Δ J' (λ = {K.LAMBDA0})", "95% CI", "p (Holm)",
                    "Δ (C + FJ)", "95% CI", "p (Holm)", "β where C + β·FJs equal"],
                   [[r[0]] + [c for t in r[1] for c in cells(t)] + [r[2]] for r in rows])


def t_beta_sweep():
    """`ddc-section` − `lcc-gold` over β, both forms, k = 20. Descriptive: unadjusted p, because 30 tests
    cannot reach a Holm-adjusted p below 0.03 with 1,000 draws; the confirmatory family is fj-section-lcc."""
    rows = []
    for form in ("FJs", "FJ"):
        for beta in SWEEP:
            ts = [test("books", "ddc-section", "lcc-gold", form, beta, model, 20) for model in K.MODELS]
            rows.append([form, K.f(beta, 2)] + [c for t in ts for c in (K.sf(t["diff"]),
                                                                       K.interval(t["lo"], t["hi"], signed=True),
                                                                       K.pv(t["p"]))])
    return K.table(["form", "β"] + [h for m in K.MODELS for h in (f"{m} Δ", "95% CI", "p")], rows,
                   ["---"] + ["---:"] * 10)


@cache
def null_ratio(run, model="minilm", k=20):
    """FJ and FJs of a run, and their means over seeded item-to-class permutations of its placements
    (class sizes and tree kept), on the committed negative sample."""
    neg = json.loads(gzip.decompress((FJ / f"negatives/books-{model}-k{k}.json.gz").read_bytes()))
    spec = next(s for s in boot("books", model, k)["runs_spec"] if s["key"] == run)
    parents = {c["code"]: c.get("parent") for c in json.loads((ROOT / spec["scheme"]).read_text())["classes"]}
    ids = [json.loads(line)["id"] for line in (ROOT / "bench/corpus/holdout-books.jsonl").open() if line.strip()]
    code = {r["id"]: K.first_code(r) for r in K.jsonl(ROOT / spec["placements"])}
    codes = [code.get(i) for i in ids]
    chain = {}
    for c in set(filter(None, codes)):
        out, x = [], c
        while x:
            out.append(x)
            x = parents[x]
        chain[c] = out[::-1]
    n = len(ids)
    mass = {}
    for c in filter(None, codes):
        for node in chain[c]:
            mass[node] = mass.get(node, 0) + 1
    pairs = [(e["left"], e["right"], e["weight"]) for e in neg["edges"]]
    total = sum(w for *_, w in pairs)

    def score(assign):
        o = s = 0.0
        for i, j, w in pairs:
            a, b = assign[i], assign[j]
            m = n
            if a and b:
                ca, cb = chain[a], chain[b]
                d = 0
                while d < min(len(ca), len(cb)) and ca[d] == cb[d]:
                    d += 1
                m = mass[ca[d - 1]] if d else n
            o += w * (n - m) / n
            s += w * math.log(n / m) / math.log(n)
        return o / total, s / total

    real = score(codes)
    rng = random.Random(f"{K.B}:{run}:null")
    shuffled, nulls = list(codes), []
    for _ in range(NULL_PERMUTATIONS):
        rng.shuffle(shuffled)
        nulls.append(score(shuffled))
    return real, tuple(sum(x[i] for x in nulls) / len(nulls) for i in (0, 1)), len({c[0] for c in chain.values()})


def t_null():
    rows = []
    for run in ("ddc-section", "fdc-d3", "ddc-d2", "lcc-gold", "learned-own"):
        (fj, fjs), (nfj, nfjs), mains = null_ratio(run)
        rows.append([f"`{run}`", mains, K.f(fjs), K.f(nfjs), K.f(fjs / nfjs, 3), K.f(fj), K.f(nfj), K.f(fj / nfj, 3)])
    return K.table(["run", "top-level classes", "FJs", "FJs, permuted", "FJs / permuted", "FJ", "FJ, permuted",
                    "FJ / permuted"], rows)


def regret(runs, model, k, steps=46):
    """Max over λ in K.LAMBDA_RANGE and β in BETA_RANGE of J'(run) − min J'."""
    worst = dict.fromkeys(runs, 0.0)
    (l0, l1), (b0, b1) = K.LAMBDA_RANGE, BETA_RANGE
    for i in range(steps):
        lam = l0 + (l1 - l0) * i / (steps - 1)
        for j in range(steps):
            beta = b0 + (b1 - b0) * j / (steps - 1)
            js = {r: value("books", r, "FJs", beta, model, k, lam) for r in runs}
            best = min(js.values())
            for r in runs:
                worst[r] = max(worst[r], js[r] - best)
    return worst


def t_cuts():
    """Minimax-regret cut under J' over λ in [0.0005, 0.005] and β in [0.5, 2], with LCC as a candidate."""
    rows = []
    runs = K.CUTS + ["lcc-gold"]
    overall = dict.fromkeys(runs, 0.0)
    for model in K.MODELS:
        for k in K.KS:
            worst = regret(runs, model, k)
            overall = {r: max(overall[r], worst[r]) for r in runs}
            pick = min(worst, key=worst.get)
            rows.append([f"{model} k = {k}", f"`{pick}`", K.f(worst["ddc-section"]), K.f(worst["ddc-d2"]),
                         K.f(worst["fdc-d3"]), K.f(worst["lcc-gold"]),
                         K.f(value("books", "ddc-section", "FJs", BETA, model, k, K.LAMBDA0)),
                         K.f(value("books", "ddc-d2", "FJs", BETA, model, k, K.LAMBDA0)),
                         K.f(value("books", "lcc-gold", "FJs", BETA, model, k, K.LAMBDA0))])
    pick = min(overall, key=overall.get)
    rows.append(["worst case over the nine graphs", f"`{pick}`", K.f(overall["ddc-section"]), K.f(overall["ddc-d2"]),
                 K.f(overall["fdc-d3"]), K.f(overall["lcc-gold"]), "", "", ""])
    return K.table(["graph", "minimax structure", "max regret `ddc-section`", "max regret `ddc-d2`",
                    "max regret `fdc-d3`", "max regret `lcc-gold`", "J' `ddc-section`", "J' `ddc-d2`",
                    "J' `lcc-gold`"], rows)


# ---- papers and holdout --------------------------------------------------
def t_papers():
    tests, rows = [], []
    for model in K.MODELS:
        for a, b in (("fdc-arxiv-nested", "arxiv-gold"), ("fdc-arxiv-nested", "lcc-arxiv-nested"),
                     ("fdc-arxiv-nested", "fdc-arxiv")):
            ts = [test("corpus", a, b, "FJs", BETA, model), test("corpus", a, b, "FJs", BETA, model, 20, K.LAMBDA0)]
            tests += ts
            rows.append([model, f"`{a}` − `{b}`", K.f(value("corpus", a, "FJs", BETA, model)),
                         K.f(value("corpus", b, "FJs", BETA, model)), ts])
    K.holm(tests)
    return K.table(["graph (k = 20)", "comparison", "C + FJs a", "C + FJs b", "Δ (C + FJs)", "95% CI", "p (Holm)",
                    f"Δ J' (λ = {K.LAMBDA0})", "95% CI", "p (Holm)"],
                   [r[:4] + [c for t in r[4] for c in cells(t)] for r in rows], ["---", "---"] + ["---:"] * 8)


def t_holdout():
    tests, rows = [], []
    for model in K.MODELS:
        for a, b in (("fdc-work", "lcc-gold"), ("fdc-pipeline", "lcc-pipeline")):
            t = test("holdout", a, b, "FJs", BETA, model)
            tests.append(t)
            rows.append([model, f"`{a}` − `{b}`", K.f(value("holdout", a, "FJs", BETA, model)),
                         K.f(value("holdout", b, "FJs", BETA, model)), t])
    K.holm(tests)
    return K.table(["graph (k = 20)", "comparison", "C + FJs a", "C + FJs b", "Δ", "95% CI", "p (Holm)"],
                   [r[:4] + cells(r[4]) for r in rows], ["---", "---"] + ["---:"] * 5)


# ---- external signals ----------------------------------------------------
def sep(signal, run):
    s = agreement()["signals"][signal]
    return s["separation"][run]["point"], s["separation_draws"][run]


def sep_test(signal, a, b):
    (pa, da), (pb, db) = sep(signal, a), sep(signal, b)
    return K.paired(da, db, pa, pb)


def separation_rows(signal, runs):
    s = agreement()["signals"][signal]["separation"]
    return [[signal, f"`{r}`", f"{K.f(s[r]['point'], 3)} {K.interval(*s[r]['ci'], 3)}", K.f(s[r]["error_rate"], 3)]
            for r in runs]


EXT_BOOK_RUNS = ["ddc-section", "fdc-d3", "ddc-d2", "ddc-gold", "lcc-gold", "learned-own", "fdc-placed", "lcc-placed"]
EXT_BOOK_TESTS = [("ddc-section", "lcc-gold"), ("fdc-d3", "lcc-gold"), ("ddc-gold", "lcc-gold"),
                  ("ddc-d2", "ddc-section"), ("learned-own", "ddc-section")]
EXT_PAPER_RUNS = ["arxiv-gold", "fdc-arxiv-nested", "lcc-arxiv-nested", "fdc-arxiv", "lcc-arxiv", "learned-placed",
                  "openalex-topics-placed", "fdc-placed"]
EXT_PAPER_TESTS = [("fdc-arxiv-nested", "lcc-arxiv-nested"), ("fdc-arxiv-nested", "arxiv-gold"),
                   ("fdc-arxiv", "lcc-arxiv"), ("learned-placed", "arxiv-gold")]


def external_table(signals, runs, pairs):
    rows = [r for s in signals for r in separation_rows(s, runs)]
    tests = [(s, a, b, sep_test(s, a, b)) for s in signals for a, b in pairs]
    K.holm([t for *_, t in tests])
    return (K.table(["signal", "run", "separation [95% CI]", "external error rate"], rows,
                     ["---", "---", "---:", "---:"]) + "\n\n"
            + K.table(["signal", "comparison", "Δ separation", "95% CI", "p", "p (Holm)"],
                      [[s, f"`{a}` − `{b}`", *K.test_cells(t, 3)] for s, a, b, t in tests],
                      ["---", "---"] + ["---:"] * 4))


def t_ext_books():
    return external_table(["lcsh"], EXT_BOOK_RUNS, EXT_BOOK_TESTS)


def t_ext_papers():
    return external_table(["coupling", "cocitation"], EXT_PAPER_RUNS, EXT_PAPER_TESTS)
