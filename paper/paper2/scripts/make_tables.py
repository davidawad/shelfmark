#!/usr/bin/env python3
"""Render paper/paper2/paper.md from paper.src.md and the committed results.

Every number in the paper comes from JSON on this branch; nothing is typed by
hand. The template uses two kinds of placeholder:

    {{key}}         a formatted value from the `values` dict built below
    {{table:name}}  a Markdown table built below

Usage, from the repository root:

    python3 paper/paper2/scripts/make_tables.py          # write paper.md and tables/
    python3 paper/paper2/scripts/make_tables.py --check  # fail if they are stale

Inputs (all committed):
    bench/results/2026-10-10-learned/results.json   learned runs and tables
    bench/results/2026-10-10-learned/insert-report.json
    bench/results/2026-10-10-learned/learned-holdout.json
    bench/results/2026-10-10-r2/results.json        Paper 1 placed runs
    bench/results/2026-10-10/results.json           Paper 1 gold runs
    schemes/learned/learned.json                    the scheme
    schemes/learned/data/centroids.json.gz          works per leaf class
    bench/corpus/corpus.jsonl                       primary categories

The script also cross-checks the inputs (J recomputed from its two terms,
placed rows equal to the Paper 1 rerun, the fit report equal to the scheme
row) and stops on any mismatch.
"""

import argparse
import gzip
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PAPER = ROOT / "paper" / "paper2"
LEARNED = ROOT / "bench" / "results" / "2026-10-10-learned"
R2 = ROOT / "bench" / "results" / "2026-10-10-r2"
P1 = ROOT / "bench" / "results" / "2026-10-10"
SCHEME = ROOT / "schemes" / "learned"


def load(path):
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", encoding="utf-8") as f:
        return json.load(f)


def f4(x):
    return f"{x + 0.0:.4f}".replace("-0.0000", "0.0000")


def f2(x):
    return f"{x:.2f}"


def int_(x):
    return f"{int(x):,}"


def pct(x, digits=1):
    return f"{100 * x:.{digits}f}%"


def check(cond, msg):
    if not cond:
        sys.exit(f"make_tables: inconsistent inputs: {msg}")


def close(a, b, tol=1e-9):
    return abs(a - b) <= tol * max(1.0, abs(a), abs(b))


def row(table, scheme, placements_prefix):
    hits = [
        r
        for r in table
        if r["scheme"] == scheme and r["placements"].startswith(placements_prefix)
    ]
    check(len(hits) == 1, f"{len(hits)} rows for {scheme!r} / {placements_prefix!r}")
    return hits[0]


SCORE_COLS = [
    ("scheme", "scheme", str),
    ("placements", "placements", str),
    ("coverage", "coverage", f4),
    ("cost_norm", "cost_norm", f4),
    ("skill", "skill", f4),
    ("J", "objective_J", f4),
    ("nav log a=1", "nav_log_a1", f2),
    ("classes used", "effective_classes", lambda x: f"{x:.1f}"),
    ("mean depth", "mean_depth", f2),
    ("max branching", "max_branching", str),
]
RIGHT = {"coverage", "cost_norm", "skill", "J", "nav log a=1", "classes used", "mean depth", "max branching"}


def md_table(header, rows, right=()):
    out = ["| " + " | ".join(header) + " |"]
    out.append("| " + " | ".join("---:" if h in right else "---" for h in header) + " |")
    out += ["| " + " | ".join(r) + " |" for r in rows]
    return "\n".join(out)


def score_table(rows, ref=None, ref_label=None):
    header = [c[0] for c in SCORE_COLS]
    rows = sorted(rows, key=lambda r: (r["objective_J"] is None, r["objective_J"] or 0.0))
    body = [[fmt(r[key]) for _, key, fmt in SCORE_COLS] for r in rows]
    if ref is not None:
        body.append([ref_label, "-", "1.0000", f4(ref), "1.0000", "-", "-", "-", "-", "2"])
    return md_table(header, body, RIGHT)


def build():
    res = load(LEARNED / "results.json")
    ins_report = load(LEARNED / "insert-report.json")
    ins_scheme = load(LEARNED / "learned-holdout.json")
    r2 = load(R2 / "results.json")
    p1 = load(P1 / "results.json")
    scheme = load(SCHEME / "learned.json")
    centroids = load(SCHEME / "data" / "centroids.json.gz")

    obj = res["objective"]
    lam, a = obj["lambda"], obj["branch_cost_a"]
    build_ = res["learned"]["build"]
    params = build_["config"]["params"]
    shape = build_["shape"]
    stages = {s["name"]: s for s in build_["stages"]}
    tables = res["tables"]
    corpus, test20, split, holdout = (tables[k] for k in ("corpus", "test20", "split", "holdout"))

    # --- cross-checks ---------------------------------------------------
    check(lam == params["lambda"] and a == params["branch_cost"], "objective vs build params")
    for name, t in tables.items():
        for r in t:
            if r["objective_J"] is not None:
                check(
                    close(r["objective_J"], r["cost_norm"] + lam * r["nav_log_a1"], 1e-9),
                    f"J != cost_norm + lambda * nav in {name} / {r['scheme']}",
                )
    own = row(corpus, "learned", "own")
    check(close(own["cost_norm"], build_["objective"]["dasgupta"]), "build D vs harness cost_norm")
    check(close(own["nav_log_a1"], build_["objective"]["nav"], 1e-9), "build N vs harness nav")
    check(res["baselines"]["paper1_rerun_matches_published"] is True, "Paper 1 rerun did not match")
    for r in corpus:
        if r["placements"].startswith("shelfmark") and r["scheme"] != "learned":
            pub = r2["results"][f"{r['scheme']}-placed"]
            check(close(r["cost_norm"], pub["cost_norm"]), f"placed {r['scheme']} vs r2")
            check(close(r["coverage"], pub["coverage"]), f"placed coverage {r['scheme']} vs r2")
        if r["placements"].startswith("gold"):
            pub = p1["results"][f"{r['scheme']}-gold"]
            check(close(r["cost_norm"], pub["cost_norm"]), f"gold {r['scheme']} vs 2026-10-10")
    for r in holdout:
        if r["placements"].startswith("shelfmark") and r["scheme"] != "learned":
            pub = r2["results"][f"{r['scheme']}-placed-holdout"]
            check(close(r["cost_norm"], pub["cost_norm"]), f"placed holdout {r['scheme']} vs r2")
    check(len(scheme["classes"]) == shape["classes"], "scheme class count")
    check(sum(leaf["n"] for leaf in centroids["leaves"]) == build_["inputs"]["n"], "centroid sizes")
    check(ins_report["stable"] and res["insertion"]["split"]["stable"], "insertion not stable")
    check(len(ins_scheme["classes"]) == ins_report["classes_after"], "inserted scheme size")

    # --- values -----------------------------------------------------------
    v = {}
    n = build_["inputs"]["n"]
    v["n"] = int_(n)
    with open(ROOT / "bench" / "corpus" / "corpus.jsonl", encoding="utf-8") as f:
        primaries = [json.loads(line)["categories"][0] for line in f if line.strip()]
    check(len(primaries) == n, "corpus rows")
    v["cs_share"] = pct(sum(1 for c in primaries if c.split(".")[0] == "cs") / n)
    v["n_holdout"] = int_(holdout[0]["n"])
    v["n_test"] = int_(test20[0]["n"])
    v["n_train"] = int_(split[0]["n"])
    v["test_pct"] = f"{100 * res['split']['fraction']:.0f}"
    v["train_pct"] = f"{100 * (1 - res['split']['fraction']):.0f}"
    v["split_seed"] = res["split"]["seed"]
    v["edges"] = int_(build_["inputs"]["edges"])
    v["k"] = str(r2["harness"]["k"])
    v["lambda"] = f"{lam:g}"
    v["a"] = f"{a:g}"
    v["lambda_pct"] = f"{100 * lam:g}%"
    for key in ("min_children", "max_children", "max_depth", "min_leaf"):
        v[key] = str(params[key])
    v["seed"] = str(build_["config"]["seed"])
    v["dims"] = str(centroids["dims"])
    v["model"] = r2["model"]["model_id"]
    v["model_rev"] = r2["model"]["weights_revision"][:12]
    v["model_sha"] = r2["model"]["weights_sha256"][:12]
    v["corpus_sha"] = res["corpus"]["corpus.jsonl"][:12]
    v["holdout_sha"] = res["corpus"]["holdout.jsonl"][:12]
    v["graph_sha"] = res["graphs"]["corpus"]["graph_sha256"][:12]
    v["scheme_sha"] = res["learned"]["scheme_sha256"][:12]
    v["tools_commit"] = res["tools"]["commit"][:12]
    v["classifier"] = r2["classifier"]["profile"]
    v["n_paper1_schemes"] = str(sum(1 for s in r2["schemes"].values() if s["status"] == "ok"))
    v["n_paper1_placed"] = str(len(r2["results"]))
    v["n_paper1_gold"] = str(sum(1 for k in p1["results"] if k.endswith("-gold")))

    v["ref_corpus"] = f4(res["graphs"]["corpus"]["ref_cost_norm"])
    v["ref_holdout"] = f4(res["graphs"]["holdout"]["ref_cost_norm"])
    v["ref_test"] = f4(res["split"]["test_ref_cost_norm"])

    al, rot, col, ls = (stages[s] for s in ("average_linkage", "rotation_local_search", "collapse", "order_preserving_local_search"))
    check(close(al["dasgupta"], res["graphs"]["corpus"]["ref_cost_norm"]), "average linkage vs harness reference")
    v["al_D"] = f"{al['dasgupta']:.6f}"
    v["rotations"] = str(rot["detail"]["rotations"])
    v["col_D"], v["col_N"], v["col_J"] = f"{col['dasgupta']:.6f}", f"{col['nav']:.4f}", f"{col['total']:.6f}"
    v["col_dp_J"] = f"{col['detail']['dp_total']:.6f}"
    v["col_classes"] = str(col["detail"]["classes"])
    v["ls_D"], v["ls_N"], v["ls_J"] = f"{ls['dasgupta']:.6f}", f"{ls['nav']:.4f}", f"{ls['total']:.6f}"
    v["ls_shifts"] = str(ls["detail"]["stats"]["shifts"])
    v["ls_moves"] = str(ls["detail"]["stats"]["structure_moves"])
    v["ls_rounds"] = str(ls["detail"]["stats"]["rounds"])
    v["fit_seconds"] = f"{sum(s['seconds'] for s in build_['stages']):.1f}"

    v["classes"] = str(shape["classes"])
    v["leaf_classes"] = str(shape["leaf_classes"])
    v["internal_classes"] = str(shape["internal_classes"])
    by_depth = shape["classes_by_depth"]
    v["classes_by_depth"] = " / ".join(str(by_depth[d]) for d in sorted(by_depth, key=int))
    v["root_children"] = str(shape["children"]["root"])
    v["children_max"] = str(shape["children"]["max"])
    v["children_mean"] = f"{shape['children']['mean']:.1f}"
    v["leaf_min"], v["leaf_median"], v["leaf_max"] = (str(shape["leaf_size"][k]) for k in ("min", "median", "max"))
    v["mean_class_depth"] = f2(shape["mean_item_class_depth"])

    # corpus
    arx_gold = row(corpus, "arxiv", "gold")
    lp = row(corpus, "learned", "shelfmark")
    paper1_placed = [r for r in corpus if r["placements"].startswith("shelfmark") and r["scheme"] != "learned"]
    best = min(paper1_placed, key=lambda r: r["cost_norm"])
    v["own_cost"], v["own_skill"], v["own_J"], v["own_nav"] = f4(own["cost_norm"]), f4(own["skill"]), f4(own["objective_J"]), f2(own["nav_log_a1"])
    v["own_skill_pct"] = pct(own["skill"])
    v["arxiv_gold_cost"], v["arxiv_gold_J"], v["arxiv_gold_nav"] = f4(arx_gold["cost_norm"]), f4(arx_gold["objective_J"]), f2(arx_gold["nav_log_a1"])
    v["lp_cost"], v["lp_J"], v["lp_cov"], v["lp_skill"] = f4(lp["cost_norm"]), f4(lp["objective_J"]), pct(lp["coverage"], 2), f4(lp["skill"])
    v["lp_nav"] = f2(lp["nav_log_a1"])
    v["lp_eff"], v["own_eff"] = f"{lp['effective_classes']:.1f}", f"{own['effective_classes']:.1f}"
    v["lp_maxb"], v["own_maxb"] = str(lp["max_branching"]), str(own["max_branching"])
    v["best_p1"], v["best_p1_cost"], v["best_p1_J"], v["best_p1_skill"] = best["scheme"], f4(best["cost_norm"]), f4(best["objective_J"]), f4(best["skill"])
    worst = max(paper1_placed, key=lambda r: r["cost_norm"])
    v["worst_p1"], v["worst_p1_cost"] = worst["scheme"], f4(worst["cost_norm"])
    v["n_placed_rows_corpus"] = str(len(paper1_placed))

    # held-out 20%
    ho = row(test20, "learned (fit on the other 80%)", "own insertion")
    full_on_test = row(test20, "learned (full fit)", "own")
    lp_test = row(test20, "learned (full fit)", "shelfmark")
    arx_test = row(test20, "arxiv", "gold")
    best_test = min((r for r in test20 if r["placements"].startswith("shelfmark") and not r["scheme"].startswith("learned")), key=lambda r: r["cost_norm"])
    v["ho_cost"], v["ho_skill"], v["ho_J"] = f4(ho["cost_norm"]), f4(ho["skill"]), f4(ho["objective_J"])
    v["full_test_cost"], v["full_test_J"] = f4(full_on_test["cost_norm"]), f4(full_on_test["objective_J"])
    v["lp_test_cost"] = f4(lp_test["cost_norm"])
    v["arx_test_cost"], v["arx_test_J"] = f4(arx_test["cost_norm"]), f4(arx_test["objective_J"])
    v["best_test"], v["best_test_cost"] = best_test["scheme"], f4(best_test["cost_norm"])
    s80 = res["insertion"]["split"]
    v["s80_classes"], v["s80_splits"] = str(s80["classes_before"]), str(s80["splits"])
    v["s80_classes_after"] = str(s80["classes_after"])
    tr, whole, fullw = split
    check(tr["placements"].startswith("own, in-sample, 80%"), "split table order")
    v["tr_cost"], v["tr_J"] = f4(tr["cost_norm"]), f4(tr["objective_J"])
    v["whole_cost"], v["whole_J"] = f4(whole["cost_norm"]), f4(whole["objective_J"])
    v["fullw_cost"], v["fullw_J"] = f4(fullw["cost_norm"]), f4(fullw["objective_J"])

    # holdout
    hown = row(holdout, "learned", "own")
    hlp = row(holdout, "learned", "shelfmark")
    lcc_gold = row(holdout, "lcc", "gold")
    lccf_gold = row(holdout, "lcc-full", "gold")
    hbest = min((r for r in holdout if r["placements"].startswith("shelfmark") and r["scheme"] != "learned"), key=lambda r: r["cost_norm"])
    v["h_cost"], v["h_skill"], v["h_J"] = f4(hown["cost_norm"]), f4(hown["skill"]), f4(hown["objective_J"])
    v["h_mean_depth"] = f2(hown["mean_depth"])
    v["h_max_depth"] = str(hown["max_depth"])
    v["lcc_gold_cost"], v["lcc_gold_cov"] = f4(lcc_gold["cost_norm"]), pct(lcc_gold["coverage"])
    v["lccf_gold_cost"] = f4(lccf_gold["cost_norm"])
    v["hlp_cost"], v["hlp_cov"] = f4(hlp["cost_norm"]), pct(hlp["coverage"])
    stats = res["runs"]["learned-placed-holdout"]["run"]["placement_stats"]
    v["hlp_placed"] = str(stats["placed"])
    check(stats["placed"] / stats["rows"] == hlp["coverage"], "holdout placed count vs coverage")
    v["hbest"], v["hbest_cost"], v["hbest_cov"] = hbest["scheme"], f4(hbest["cost_norm"]), pct(hbest["coverage"])

    # insertion
    ip = ins_report["params"]
    v["ins_split_after"], v["ins_max_cos"], v["ins_min_child"] = str(ip["split_after"]), f"{ip['max_cos']:g}", str(ip["min_child"])
    v["ins_scheme"] = ins_report["scheme"]
    v["ins_before"], v["ins_after"] = str(ins_report["classes_before"]), str(ins_report["classes_after"])
    sp = ins_report["splits"]
    v["ins_splits"] = str(len(sp))
    v["ins_distinct"] = str(sum(1 for s in sp if s["cause"] == "distinct"))
    v["ins_capacity"] = str(sum(1 for s in sp if s["cause"] == "capacity"))
    v["ins_new_classes"] = str(ins_report["classes_after"] - ins_report["classes_before"])
    check(2 * len(sp) == ins_report["classes_after"] - ins_report["classes_before"], "two classes per split")
    first_leaf = sp[0]["leaf"]
    labels = {c["code"]: c["label"] for c in scheme["classes"]}
    v["ins_first_leaf"] = first_leaf
    v["ins_first_new_label"] = {c["code"]: c["label"] for c in ins_scheme["classes"]}[sp[0]["new"]]
    v["ins_first_label"] = labels[first_leaf]
    v["ins_first_after"] = str(sp[0]["after_inserts"])
    roots = sorted({s["leaf"] for s in sp if s["leaf"] in labels})
    v["ins_fitted_leaves_split"] = str(len(roots))
    v["ins_fitted_leaves_list"] = ", ".join(f"`{c}` ({labels[c]})" for c in roots)
    new_depths = [c["code"].count(".") + 1 for c in ins_scheme["classes"] if c["code"] not in labels]
    v["ins_max_class_depth"] = str(max(new_depths))

    # sensitivity sweep
    sweep = res["sweep"]
    v["sweep_fits"] = str(len(sweep))
    a_values = sorted({s["branch_cost"] for s in sweep})
    v["sweep_a_values"] = ", ".join(f"{x:g}" for x in a_values)
    v["sweep_lambdas"] = ", ".join(f"{x:g}" for x in sorted({s["lambda"] for s in sweep}))
    v["sweep_depths"] = ", ".join(str(x) for x in sorted({s["max_depth"] for s in sweep}))
    non_binary = [s for s in sweep if s["root_children"] != 2]
    v["sweep_binary_root"] = str(len(sweep) - len(non_binary))
    v["sweep_nonbinary_desc"] = "; ".join(
        f"depth bound {s['max_depth']}, lambda {s['lambda']:g}, a {s['branch_cost']:g}: {s['root_children']}"
        for s in non_binary
    ) or "none"
    v["sweep_nonbinary_min_lambda"] = f"{min(s['lambda'] for s in non_binary):g}"
    v["sweep_nonbinary_count"] = str(len(non_binary))
    v["sweep_max_root"] = str(max(s["root_children"] for s in sweep))
    check(any(s["max_depth"] == params["max_depth"] and s["lambda"] == lam and s["branch_cost"] == a and close(s["J"], own["objective_J"], 1e-9) for s in sweep), "committed fit not in sweep")

    # --- tables -----------------------------------------------------------
    t = {}
    t["stages"] = md_table(
        ["stage", "D (cost_norm)", "N (nav, a = 1)", "J", "classes", "seconds"],
        [
            ["average linkage (binary, one work per leaf)", f"{al['dasgupta']:.6f}", "-", "-", "-", f"{al['seconds']:.2f}"],
            [f"rotation local search ({rot['detail']['rotations']} moves)", f"{rot['dasgupta']:.6f}", "-", "-", "-", f"{rot['seconds']:.2f}"],
            ["exact collapse (DP)", f"{col['dasgupta']:.6f}", f"{col['nav']:.4f}", f"{col['total']:.6f}", str(col["detail"]["classes"]), f"{col['seconds']:.2f}"],
            [f"order-preserving local search ({v['ls_shifts']} shifts, {v['ls_moves']} structure moves)", f"{ls['dasgupta']:.6f}", f"{ls['nav']:.4f}", f"{ls['total']:.6f}", str(ls["detail"]["classes"]), f"{ls['seconds']:.2f}"],
        ],
        {"D (cost_norm)", "N (nav, a = 1)", "J", "classes", "seconds"},
    )

    leaf_n = {leaf["code"]: leaf["n"] for leaf in centroids["leaves"]}
    def under(code):
        return [c for c in leaf_n if c == code or c.startswith(code + ".")]
    outline = []
    for c in scheme["classes"]:
        depth = c["code"].count(".") + 1
        if depth > 3:
            continue
        lv = under(c["code"])
        indent = "&nbsp;&nbsp;" * (depth - 1)
        outline.append([f"{indent}`{c['code']}`", c["label"], str(len(lv)), int_(sum(leaf_n[x] for x in lv))])
    t["outline"] = md_table(["code", "label (hand-written)", "leaf classes", "works"], outline, {"leaf classes", "works"})

    t["corpus"] = score_table(corpus, res["graphs"]["corpus"]["ref_cost_norm"], "average linkage (binary reference, not a classification)")
    t["test20"] = score_table(test20)
    t["split"] = score_table(split)
    t["holdout"] = score_table(holdout, res["graphs"]["holdout"]["ref_cost_norm"], "average linkage (binary reference, not a classification)")

    new_labels = {c["code"]: c["label"] for c in ins_scheme["classes"]}
    t["splits"] = md_table(
        ["after insert no.", "cause", "split leaf", "new class", "its label (automatic)", "new works in it", "centroid cosine"],
        [
            [str(s["after_inserts"]), s["cause"], f"`{s['leaf']}`", f"`{s['new']}`", new_labels[s["new"]], str(s["new_works"]), f"{s['centroid_cos']:.3f}"]
            for s in sp
        ],
        {"after insert no.", "new works in it", "centroid cosine"},
    )

    sw = [s for s in sweep if s["branch_cost"] == a]
    sw.sort(key=lambda s: (s["max_depth"], s["lambda"]))
    t["sweep"] = md_table(
        ["depth bound", "lambda", "J", "cost_norm", "nav (a=1)", "classes", "leaf classes", "root children", "median leaf"],
        [
            [str(s["max_depth"]), f"{s['lambda']:g}", f4(s["J"]), f4(s["dasgupta"]), f2(s["nav"]), str(s["classes"]), str(s["leaf_classes"]), str(s["root_children"]), str(s["median_leaf"])]
            for s in sw
        ],
        {"depth bound", "lambda", "J", "cost_norm", "nav (a=1)", "classes", "leaf classes", "root children", "median leaf"},
    )

    # --- later results shared with Paper 1 (paper/common/extra_results.py):
    # the independent-embedding rescoring, the false-join term and its
    # validity check, and the 27-setting book comparison.
    sys.path.insert(0, str(ROOT / "paper" / "common"))
    import extra_results
    xv, xt = extra_results.collect(ROOT)
    v.update(xv)
    for name in SHARED_TABLES:
        tab = xt[name]
        t[name] = md_table(tab["header"], [[str(c) for c in r] for r in tab["rows"]],
                           set(tab["header"][1:]) - LEFT_SHARED.get(name, set()))
    return v, t


# Shared tables Paper 2 prints, and their left-aligned columns besides the first.
SHARED_TABLES = ["learned_edge", "learned_fj", "cross_fit"]
LEFT_SHARED = {
    "learned_fj": {"model", "best other by J'", "best other by C + FJ"},
}


PLACEHOLDER = re.compile(r"\{\{([A-Za-z0-9_:]+)\}\}")


def render(src, v, t):
    missing = []

    def sub(m):
        key = m.group(1)
        if key.startswith("table:"):
            name = key[len("table:"):]
            if name in t:
                return t[name]
        elif key in v:
            return v[key]
        missing.append(key)
        return m.group(0)

    out = PLACEHOLDER.sub(sub, src)
    if missing:
        sys.exit(f"make_tables: unknown placeholders: {', '.join(sorted(set(missing)))}")
    if "{{" in out or "}}" in out:
        sys.exit("make_tables: unrendered braces left in paper.md")
    header = (
        "<!-- GENERATED by paper/paper2/scripts/make_tables.py from paper.src.md "
        "and the committed results JSON. Do not edit; edit paper.src.md and rerun. -->\n\n"
    )
    if out.startswith("---\n"):
        end = out.index("\n---\n", 4) + len("\n---\n")
        return out[:end] + "\n" + header + out[end:].lstrip("\n")
    return header + out


def outputs(v, t):
    files = {PAPER / "paper.md": render((PAPER / "paper.src.md").read_text(encoding="utf-8"), v, t)}
    for name, table in t.items():
        files[PAPER / "tables" / f"{name}.md"] = table + "\n"
    files[PAPER / "tables" / "values.json"] = json.dumps(v, indent=1, sort_keys=True, ensure_ascii=False) + "\n"
    return files


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true", help="fail if paper.md or tables/ differ from a fresh render")
    args = ap.parse_args()
    v, t = build()
    files = outputs(v, t)
    stale = [p for p, text in files.items() if not p.exists() or p.read_text(encoding="utf-8") != text]
    if args.check:
        if stale:
            sys.exit("make_tables: stale: " + ", ".join(str(p.relative_to(ROOT)) for p in stale))
        print(f"make_tables: {len(files)} files up to date")
        return
    for p in stale:
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(files[p], encoding="utf-8")
    print(f"make_tables: wrote {len(stale)} of {len(files)} files")


if __name__ == "__main__":
    main()
