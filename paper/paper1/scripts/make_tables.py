#!/usr/bin/env python3
"""Fill the generated tables and inline values in paper/paper1/paper.md.

Every number in the paper's tables, and every number in the prose that is
wrapped in a value marker, comes from the published benchmark JSON or the
pinned corpus files; none is typed by hand. Run from the repository root:

    python3 paper/paper1/scripts/make_tables.py          # rewrite paper.md
    python3 paper/paper1/scripts/make_tables.py --check  # fail if stale

Markers in paper.md:

    <!-- table:NAME -->  ...generated...  <!-- /table:NAME -->
    <!-- v:NAME -->...generated...<!-- /v -->

Standard library only.
"""
import argparse
import collections
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PAPER = ROOT / "paper/paper1/paper.md"
GOLD_DIR = ROOT / "bench/results/2026-10-10"
PLACED_DIR = ROOT / "bench/results/2026-10-10-r2"
# Gold rescored on the fixed corpus (acm_ccs from arXivOld, msc MSC 2020 only).
# Its runs and skips replace the 2026-10-10 ones of the same name.
GOLD_FIX_DIR = ROOT / "bench/results/2026-10-10-r3"
CORPUS = ROOT / "bench/corpus/corpus.jsonl"
HOLDOUT = ROOT / "bench/corpus/holdout.jsonl"

# arXiv top-level groups other than physics; every other primary-category
# archive (astro-ph, cond-mat, hep-*, quant-ph, ...) belongs to physics.
NON_PHYSICS = {"cs", "econ", "eess", "math", "q-bio", "q-fin", "stat"}


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def rows(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def f3(x):
    return f"{x:.3f}"


def pct(x):
    # Round half up on the decimal text of x, so that 0.8765 prints as 87.7%
    # (binary formatting prints 87.6%). Two decimals below 0.1% so that small
    # nonzero shares do not print as 0.0%.
    from decimal import Decimal, ROUND_HALF_UP
    p = Decimal(repr(float(x))) * 100
    nd = 2 if 0 < p < Decimal("0.1") else 1
    return f"{p.quantize(Decimal(1).scaleb(-nd), rounding=ROUND_HALF_UP)}%"


def delta(x):
    # Round before formatting so that tiny negatives do not print as -0.000.
    return f"{round(x, 3) + 0.0:+.3f}"


def count(x):
    return f"{x:,}"


def table(header, body, align=None):
    align = align or ["---"] + ["---:"] * (len(header) - 1)
    out = ["| " + " | ".join(header) + " |", "| " + " | ".join(align) + " |"]
    out += ["| " + " | ".join(str(c) for c in r) + " |" for r in body]
    return "\n".join(out)


def ranks(values):
    """Average ranks (1 = smallest), ties share their mean rank."""
    order = sorted(range(len(values)), key=lambda i: values[i])
    r = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for k in range(i, j + 1):
            r[order[k]] = (i + j) / 2 + 1
        i = j + 1
    return r


def spearman(x, y):
    rx, ry = ranks(x), ranks(y)
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = (sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry)) ** 0.5
    return num / den if den else float("nan")


def kendall_tau_b(x, y):
    conc = disc = tx = ty = 0
    n = len(x)
    for i in range(n):
        for j in range(i + 1, n):
            dx, dy = x[i] - x[j], y[i] - y[j]
            if dx == 0 and dy == 0:
                continue
            if dx == 0:
                tx += 1
            elif dy == 0:
                ty += 1
            elif (dx > 0) == (dy > 0):
                conc += 1
            else:
                disc += 1
    den = ((conc + disc + tx) * (conc + disc + ty)) ** 0.5
    return (conc - disc) / den if den else float("nan")


def score_row(key, run, summary):
    s = run["structure"]
    return [
        f"`{run['scheme']}`",
        summary["version"],
        pct(run["coverage"]),
        f3(run["cost_norm"]),
        f3(run["revenue_norm"]),
        f3(run["skill"]),
        f"{s['effective_classes']:.1f}",
        f"{s['mean_depth']:.2f}",
        s["max_depth"],
        s["max_branching"],
        f3(s["gini"]),
        f"{s['nav_log_a1']:.1f}",
    ]


SCORE_HEADER = [
    "scheme", "version", "coverage", "cost_norm", "revenue_norm", "skill",
    "eff. classes", "mean depth", "max depth", "max fan-out", "Gini",
    "nav_log (a=1)",
]


def build():
    gold = load(GOLD_DIR / "results.json")
    placed = load(PLACED_DIR / "results.json")
    fix = load(GOLD_FIX_DIR / "results.json")
    runs_g = {k: load(GOLD_DIR / v["file"]) for k, v in gold["results"].items()}
    runs_g.update({k: load(GOLD_FIX_DIR / v["file"]) for k, v in fix["results"].items()})
    gold["results"].update(fix["results"])
    gold["skipped"].update(fix["skipped"])
    runs_p = {k: load(PLACED_DIR / v["file"]) for k, v in placed["results"].items()}
    corpus, holdout = rows(CORPUS), rows(HOLDOUT)
    tables, values = {}, {}

    # Sanity: the r2 run must reuse the 2026-10-10 graphs and scheme files.
    for g in ("corpus", "holdout"):
        assert gold["graphs"][g]["graph_sha256"] == placed["graphs"][g]["graph_sha256"], g
    for name, s in placed["schemes"].items():
        assert s["file_sha256"] == gold["schemes"][name]["file_sha256"], name
    # The r3 rescore must sit on the same graphs and scheme files, and the
    # pinned corpus is the fixed one it was scored on.
    assert fix["baseline"]["results"] == "bench/results/2026-10-10/results.json"
    for g in ("corpus", "holdout"):
        assert fix["graphs"][g]["graph_sha256"] == gold["graphs"][g]["graph_sha256"], g
    for name, s in fix["schemes"].items():
        assert s["file_sha256"] == gold["schemes"][name]["file_sha256"], name
    assert sha256(CORPUS) == fix["corpus"]["corpus.jsonl"]
    assert sha256(HOLDOUT) == gold["corpus"]["holdout.jsonl"] == fix["corpus"]["holdout.jsonl"]

    # Corpus composition.
    groups = collections.Counter()
    for r in corpus:
        a = r["categories"][0].split(".")[0]
        groups[a if a in NON_PHYSICS else "physics"] += 1
    n = len(corpus)
    tables["corpus"] = table(
        ["arXiv top-level group (primary category)", "papers", "share"],
        [[g, count(c), pct(c / n)] for g, c in sorted(groups.items())]
        + [["**total**", count(n), pct(1.0)]],
    )
    values["n_corpus"] = count(n)
    values["n_holdout"] = count(len(holdout))
    values["share_physical_math_cs"] = pct(
        (groups["cs"] + groups["math"] + groups["physics"]) / n)
    values["share_2026_ids"] = pct(sum(1 for r in corpus if r["id"].startswith("26")) / n)
    values["n_msc_rows"] = count(sum(1 for r in corpus if r["msc"]))
    values["n_acm_rows"] = count(sum(1 for r in corpus if r["acm_ccs"]))
    values["n_jel_rows"] = count(sum(1 for r in corpus if r["jel"]))
    values["holdout_title_words"] = f"{sum(len(r['title'].split()) for r in holdout) / len(holdout):.1f}"
    values["holdout_with_text"] = count(sum(1 for r in holdout if r["abstract_text"].strip()))
    letters = collections.Counter()
    for r in holdout:
        first = next((c.strip() for c in r["lc_classifications"] if c.strip()), "")
        letters[first[:1] if first[:1].isalpha() else ""] += 1
    nh = len(holdout)
    values["holdout_p"] = count(letters["P"])
    values["holdout_p_share"] = pct(letters["P"] / nh)
    values["holdout_empty"] = count(letters[""])
    values["holdout_letter_count"] = count(sum(1 for k in letters if k))
    values["holdout_other_top"] = ", ".join(
        f"{k} {v}" for k, v in letters.most_common() if k and k != "P")

    # Graphs.
    for g in ("corpus", "holdout"):
        m = gold["graphs"][g]
        values[f"tau_{g}"] = f3(m["tau"])
        values[f"ref_{g}"] = f3(m["reference"]["ref_cost_norm"])
    values["model"] = gold["model"]["model_id"]
    values["k"] = str(gold["harness"]["k"])

    # Schemes.
    body = []
    for name, s in sorted(placed["schemes"].items()):
        lic = s.get("license", "see scheme README")
        body.append([f"`{name}`", s["version"], count(s["classes"]), lic])
    tables["schemes"] = table(["scheme", "version", "classes", "licence as recorded by the build"],
                              body, ["---", "---", "---:", "---"])
    values["n_schemes"] = str(len(placed["schemes"]))

    # Gold results.
    def score_table(keys, runs, summary):
        ordered = sorted(keys, key=lambda k: runs[k]["cost_norm"])
        return table(SCORE_HEADER, [score_row(k, runs[k], summary[k]) for k in ordered])

    gold_corpus = [k for k, v in gold["results"].items() if v["track"] == "A" and v["corpus"] == "corpus"]
    gold_hold = [k for k, v in gold["results"].items() if v["track"] == "A" and v["corpus"] == "holdout"]
    tables["gold_corpus"] = score_table(gold_corpus, runs_g, gold["results"])
    tables["gold_holdout"] = score_table(gold_hold, runs_g, gold["results"])

    placed_corpus = [k for k, v in placed["results"].items() if v["corpus"] == "corpus"]
    placed_hold = [k for k, v in placed["results"].items() if v["corpus"] == "holdout"]
    tables["placed_corpus"] = score_table(placed_corpus, runs_p, placed["results"])
    tables["placed_holdout"] = score_table(placed_hold, runs_p, placed["results"])

    # Navigation: scan model next to the log model.
    nav_body = []
    for label, keys, runs in (("gold", gold_corpus + gold_hold, runs_g),
                              ("placed", placed_corpus + placed_hold, runs_p)):
        for k in sorted(keys, key=lambda k: (runs[k]["run"]["corpus"], runs[k]["scheme"])):
            s = runs[k]["structure"]
            nav_body.append([f"`{runs[k]['scheme']}`", label, runs[k]["run"]["corpus"],
                             s["p50_depth"], s["p95_depth"], s["root_fanout"],
                             f"{s['nav_log_a0']:.1f}", f"{s['nav_log_a1']:.1f}",
                             f"{s['nav_log_a2']:.1f}", f"{s['nav_scan']:.0f}"])
    tables["navigation"] = table(
        ["scheme", "labels", "corpus", "p50 depth", "p95 depth", "root fan-out",
         "nav_log a=0", "a=1", "a=2", "nav_scan"],
        nav_body, ["---", "---", "---"] + ["---:"] * 7)

    # Placement statistics (r2).
    pb = []
    for k in sorted(placed_corpus + placed_hold, key=lambda k: (runs_p[k]["run"]["corpus"], runs_p[k]["scheme"])):
        st = runs_p[k]["run"]["placement_stats"]
        terms = ", ".join(f"\"{t}\"" for t in st["top_class_terms"])
        pb.append([f"`{runs_p[k]['scheme']}`", runs_p[k]["run"]["corpus"], count(st["rows"]),
                   count(st["placed"]), count(st["flagged"]), count(st["fallback"]),
                   count(st["classes_used"]), f"`{st['top_class']}`", pct(st["top_class_share"]), terms])
    tables["placement"] = table(
        ["scheme", "corpus", "rows", "placed", "flagged (<0.8)", "unplaced", "classes used",
         "top class", "top share", "top matched term"],
        pb, ["---", "---"] + ["---:"] * 5 + ["---", "---:", "---"])

    # Stopword fix: before (lexical@1) and after (lexical@2).
    cb = []
    before_skill, after_skill = [], []
    for k, c in sorted(placed["comparison"].items(), key=lambda kv: (kv[1]["corpus"], kv[1]["scheme"])):
        b, a = c["before"], c["after"]
        cb.append([f"`{c['scheme']}`", c["corpus"],
                   f"{pct(b['top_class_share'])} `{b['top_class']}`",
                   f"{pct(a['top_class_share'])} `{a['top_class']}`",
                   f"{pct(b['coverage'])} → {pct(a['coverage'])}",
                   f"{f3(b['cost_norm'])} → {f3(a['cost_norm'])}",
                   f"{f3(b['skill'])} → {f3(a['skill'])}",
                   delta(a["skill"] - b["skill"])])
        if c["corpus"] == "corpus":
            before_skill.append(b["skill"])
            after_skill.append(a["skill"])
    tables["stopword"] = table(
        ["scheme", "corpus", "top class before", "top class after", "coverage",
         "cost_norm", "skill", "Δ skill"],
        cb, ["---", "---", "---", "---", "---:", "---:", "---:", "---:"])
    comp = placed["comparison"]
    for name in ("ilc2", "ipc", "lcc-full", "unesco-thesaurus"):
        c = comp[f"{name}-placed"]
        values[f"sw_{name}_before"] = pct(c["before"]["top_class_share"])
        values[f"sw_{name}_after"] = pct(c["after"]["top_class_share"])
        values[f"sw_{name}_skill_before"] = f3(c["before"]["skill"])
        values[f"sw_{name}_skill_after"] = f3(c["after"]["skill"])
    values["sw_kendall"] = f"{kendall_tau_b(before_skill, after_skill):.2f}"
    values["sw_n"] = str(len(before_skill))
    changed = sum(1 for b, a in zip(before_skill, after_skill) if abs(a - b) >= 0.0005)
    values["sw_changed"] = str(changed)

    # Classifier bias: same scheme, same items, gold vs placed.
    pairs = [("arxiv", "corpus", "arxiv-gold", "arxiv-placed"),
             ("msc", "corpus", "msc-gold", "msc-placed"),
             ("jel", "corpus", "jel-gold", "jel-placed"),
             ("lcc", "holdout", "lcc-gold", "lcc-placed-holdout"),
             ("lcc-full", "holdout", "lcc-full-gold", "lcc-full-placed-holdout")]
    bb = []
    for scheme, corpus_name, gk, pk in pairs:
        g, p = runs_g[gk], runs_p[pk]
        bb.append([f"`{scheme}`", corpus_name, pct(g["coverage"]), pct(p["coverage"]),
                   f3(g["skill"]), f3(p["skill"]), delta(p["skill"] - g["skill"])])
        values[f"gold_{scheme}_skill"] = f3(g["skill"])
        values[f"gold_{scheme}_cov"] = pct(g["coverage"])
        values[f"placed_{scheme}_{corpus_name}_skill"] = f3(p["skill"])
        values[f"placed_{scheme}_{corpus_name}_cov"] = pct(p["coverage"])
    tables["bias"] = table(
        ["scheme", "corpus", "gold coverage", "placed coverage", "gold skill",
         "placed skill", "placed − gold"],
        bb, ["---", "---"] + ["---:"] * 5)

    # Placed corpus run: headline values and coverage dependence.
    pc = sorted(placed_corpus, key=lambda k: -runs_p[k]["skill"])
    best, second = runs_p[pc[0]], runs_p[pc[1]]
    values["placed_best"] = best["scheme"]
    values["placed_best_skill"] = f3(best["skill"])
    values["placed_second"] = second["scheme"]
    values["placed_second_skill"] = f3(second["skill"])
    worst = runs_p[pc[-1]]
    values["placed_worst"] = worst["scheme"]
    values["placed_worst_skill"] = f3(worst["skill"])
    cov = [runs_p[k]["coverage"] for k in placed_corpus]
    sk = [runs_p[k]["skill"] for k in placed_corpus]
    values["rho_cov_skill"] = f"{spearman(cov, sk):.2f}"
    values["n_placed_corpus"] = str(len(placed_corpus))
    hi = [k for k in placed_corpus if runs_p[k]["coverage"] >= 0.95]
    values["n_highcov"] = str(len(hi))
    hs = [runs_p[k]["skill"] for k in hi]
    values["highcov_skill_min"] = f3(min(hs))
    values["highcov_skill_max"] = f3(max(hs))
    for k in placed_corpus + placed_hold:
        r = runs_p[k]
        values[f"p_{k}_skill"] = f3(r["skill"])
        values[f"p_{k}_cov"] = pct(r["coverage"])
        values[f"p_{k}_eff"] = f"{r['structure']['effective_classes']:.0f}"
    for k in gold_corpus + gold_hold:
        r = runs_g[k]
        values[f"g_{k}_cost"] = f3(r["cost_norm"])
        values[f"g_{k}_eff"] = f"{r['structure']['effective_classes']:.0f}"
        values[f"g_{k}_depth"] = f"{r['structure']['mean_depth']:.2f}"
        values[f"g_{k}_navscan"] = f"{r['structure']['nav_scan']:.0f}"
    flag_share = [runs_p[k]["run"]["placement_stats"]["flagged"]
                  / max(1, runs_p[k]["run"]["placement_stats"]["placed"]) for k in placed_corpus]
    values["n_flagged_over90"] = str(sum(1 for x in flag_share if x > 0.9))
    arx = runs_p["arxiv-placed"]["run"]["placement_stats"]
    values["arxiv_placed_top_share"] = pct(arx["top_class_share"])
    values["arxiv_placed_top_class"] = arx["top_class"]
    values["lccfull_placed_navscan"] = f"{runs_p['lcc-full-placed']['structure']['nav_scan']:.0f}"
    values["ref_seconds_corpus"] = f"{gold['graphs']['corpus']['reference']['seconds']:.0f}"
    score_s = [r["run"]["score_seconds"] for r in list(runs_p.values()) + list(runs_g.values())]
    values["score_seconds_max"] = f"{max(score_s):.1f}"
    values["skipped_acm_note"] = gold["skipped"]["acm-ccs-gold"]["note"]

    # Edge count of the corpus graph is not in results.json; read the graph.
    import gzip
    for g in ("corpus", "holdout"):
        with gzip.open(GOLD_DIR / "graphs" / f"{g}-k20.json.gz", "rt", encoding="utf-8") as f:
            graph = json.load(f)
        values[f"edges_{g}"] = count(len(graph["edges"]))
    values["harness_commit"] = gold["harness"]["commit"][:12]
    values["classifier_commit"] = placed["classifier"]["commit"][:12]
    values["graph_corpus_sha"] = gold["graphs"]["corpus"]["graph_sha256"][:16]

    # Results added after the first draft: judge validity, false-join term,
    # rank stability, book comparison, semantic tier. See paper/common.
    sys.path.insert(0, str(ROOT / "paper/common"))
    import extra_results
    xv, xt = extra_results.collect(ROOT)
    values.update(xv)
    tables.update({k: extra_results.md_table(v) for k, v in xt.items()})
    return tables, values


TABLE_RE = re.compile(r"(<!-- table:([\w-]+) -->\n)(.*?)(\n<!-- /table:\2 -->)", re.DOTALL)
VALUE_RE = re.compile(r"(<!-- v:([\w.-]+) -->)(.*?)(<!-- /v -->)")


def render(text, tables, values):
    missing = []

    def fill_table(m):
        if m.group(2) not in tables:
            missing.append("table:" + m.group(2))
            return m.group(0)
        return m.group(1) + tables[m.group(2)] + m.group(4)

    def fill_value(m):
        if m.group(2) not in values:
            missing.append("v:" + m.group(2))
            return m.group(0)
        return m.group(1) + values[m.group(2)] + m.group(4)

    text = TABLE_RE.sub(fill_table, text)
    text = VALUE_RE.sub(fill_value, text)
    return text, missing


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="exit 1 if paper.md is stale")
    ap.add_argument("--paper", default=str(PAPER))
    a = ap.parse_args()
    tables, values = build()
    path = Path(a.paper)
    old = path.read_text(encoding="utf-8")
    new, missing = render(old, tables, values)
    if missing:
        print("unknown markers: " + ", ".join(sorted(set(missing))), file=sys.stderr)
        return 1
    if a.check:
        if new != old:
            print(f"{path} is stale; run {Path(__file__).name}", file=sys.stderr)
            return 1
        print(f"{path}: tables and values up to date")
        return 0
    path.write_text(new, encoding="utf-8")
    print(f"wrote {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
