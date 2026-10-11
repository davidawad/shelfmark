"""results.md for covered_catalogue.py publish."""
import math
from pathlib import Path

import score_catalogue as sc
from covered_catalogue import DIRECTION, FAMILIES, MIN_ITEMS, PAIRS

fmt = sc.fmt
KEY_SCORES = [("cost_norm", "dasgupta.cost_norm"), ("TSD", "tsd.nats"), ("triplet", "triplet.accuracy"),
              ("locality", "order.locality"), ("kNN d1", "probe.knn_d1"), ("AMI fine", "gold.ami_fine"),
              ("DP fine", "gold.dendrogram_purity_fine"), ("silhouette d1", "embed.silhouette_d1")]
SIG_METRICS = ["dasgupta.cost_norm", "vertex.dasgupta.cost_norm", "tsd.nats", "vertex.tsd.nats",
               "triplet.accuracy", "vertex.triplet.accuracy", "order.locality", "vertex.order.locality",
               "probe.knn_d1", "gold.dendrogram_purity_fine", "gold.ari_fine"]


def score(r, key):
    return r["catalogue"]["scores"].get(key)


def ci_text(r, key, digits=3, ci_key=None):
    s = score(r, key)
    c = r["catalogue"]["ci"].get(ci_key or key)
    if s is None:
        return "-"
    return f"{s:.{digits}f} [{c[0]:.{digits}f}, {c[1]:.{digits}f}]" if c else f"{s:.{digits}f}"


def interval(c, digits=2):
    return "-" if not c else f"[{c[0]:+.{digits}f}, {c[1]:+.{digits}f}]"


def signed(v, digits=2):
    return "-" if v is None or math.isnan(v) else f"{v:+.{digits}f}"


def partial_text(value, ci):
    """Partial ρ and its interval; "-" when undefined (coverage fixes the ranking)."""
    if value is None or math.isnan(value):
        return "- (undefined)"
    return f"{signed(value)} {interval(ci)}"


def pair_lookup(paired, metric, a, b):
    """(diff a − b, ci, p) from the agree output, whichever order it holds."""
    for row in paired.get(metric, []):
        if (row["a"], row["b"]) == (a, b):
            return row["diff"], row["ci"], row["p"]
        if (row["a"], row["b"]) == (b, a):
            return -row["diff"], [-row["ci"][1], -row["ci"][0]], row["p"]
    return None


def diff_text(paired, metric, a, b, digits=4):
    got = pair_lookup(paired, metric, a, b)
    if not got:
        return "-"
    d, c, p = got
    star = "*" if p < 0.05 else ""
    ptext = "p<0.005" if p < 0.005 else f"p={p:.3f}"
    return f"{d:+.{digits}f} [{c[0]:+.{digits}f}, {c[1]:+.{digits}f}] {ptext}{star}"


def family_section(doc, families):
    md = "## 1. Gold scores on the covered subset\n\n"
    md += ("Each gold run defines a subset: the items its source labels place in the scheme. Every run on "
           "that corpus (gold, shelfmark-placed and learned) is rescored restricted to the subset: items "
           "outside it are dropped, and so are the graph edges that touch them. Inside the subset a gold "
           "run covers every item, so its scores no longer measure coverage, and all runs share one graph "
           "and one item set: their bootstrap replicates are paired. \"coverage\" below is the share of "
           "subset items the run places.\n\n")
    for family, corpus in FAMILIES.items():
        n = doc["covered_items"].get(family)
        total = 10000 if corpus == "corpus" else 500
        md += f"### {family} subset ({n} of {total} items)\n\n"
        runs = families.get(family) or {}
        if n is None or n < MIN_ITEMS or not runs:
            md += f"Not scored: {n} labelled items, below the minimum of {MIN_ITEMS}.\n\n"
            continue
        edges = next(iter(runs.values()))["graph_edges"]
        md += f"Graph restricted to the subset: {edges} edges.\n\n"
        rows = sorted(runs.items(), key=lambda kr: score(kr[1], "dasgupta.cost_norm"))
        md += sc.table(rows, [
            ("coverage", lambda r: fmt(score(r, "coverage"))),
            ("cost_norm [edge CI]", lambda r: ci_text(r, "dasgupta.cost_norm")),
            ("[vertex CI]", lambda r: interval(r["catalogue"]["ci"].get("vertex.dasgupta.cost_norm"), 3)),
            ("TSD [vertex CI]", lambda r: ci_text(r, "tsd.nats", ci_key="vertex.tsd.nats")),
            ("triplet [vertex CI]", lambda r: ci_text(r, "triplet.accuracy", ci_key="vertex.triplet.accuracy")),
            ("locality [vertex CI]", lambda r: ci_text(r, "order.locality", ci_key="vertex.order.locality")),
            ("kNN d1", lambda r: fmt(score(r, "probe.knn_d1"))),
            ("AMI fine", lambda r: fmt(score(r, "gold.ami_fine"))),
            ("DP fine [CI]", lambda r: ci_text(r, "gold.dendrogram_purity_fine")),
        ])
        paired = doc["analysis"]["families"][family]["paired"]
        name = family.removesuffix("-gold")
        placed = f"{name}-placed" + ("" if corpus == "corpus" else "-holdout")
        compares = []
        if placed in runs:
            compares.append((family, placed))
        for other in FAMILIES:
            if other != family and other in runs and FAMILIES[other] == corpus:
                compares.append((family, other))
        if compares:
            md += ("\nPaired-bootstrap differences (first − second; 95% interval; two-sided bootstrap p, "
                   "* = p < 0.05). Lower cost is better; higher TSD and triplet are better.\n\n"
                   "| comparison | cost_norm (vertex) | cost_norm (edge) | TSD (vertex) | triplet (vertex) |\n"
                   "| --- | ---: | ---: | ---: | ---: |\n")
            for a, b in compares:
                md += (f"| {a} − {b} | {diff_text(paired, 'vertex.dasgupta.cost_norm', a, b)} | "
                       f"{diff_text(paired, 'dasgupta.cost_norm', a, b)} | "
                       f"{diff_text(paired, 'vertex.tsd.nats', a, b, 3)} | "
                       f"{diff_text(paired, 'vertex.triplet.accuracy', a, b, 3)} |\n")
        md += "\n"
    return md


def both_section(doc, both):
    md = ("### Gold against placed on the items both place\n\n"
          "The strictest equal footing: each gold run and its shelfmark-placed counterpart restricted to "
          "the items that both place (graph edges among them), so neither run leaves an item at the "
          "root. Differences are gold − placed, paired bootstrap; * = p < 0.05.\n\n"
          "| gold | placed | items | edges | cost gold | cost placed | Δ cost (vertex) | Δ TSD (vertex) | "
          "Δ triplet (vertex) | Δ locality (vertex) | AMI fine gold / placed |\n"
          "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n")
    for gold, placed in PAIRS.items():
        runs = both.get(gold) or {}
        n = doc["covered_items"].get(f"both-{gold}")
        if len(runs) != 2:
            md += f"| {gold} | {placed} | {n} | - | not scored (under {MIN_ITEMS} items) |||||||\n"
            continue
        paired = doc["analysis"]["both"][gold]
        g, p = runs[gold], runs[placed]
        md += (f"| {gold} | {placed} | {n} | {g['graph_edges']} | "
               f"{ci_text(g, 'dasgupta.cost_norm', ci_key='vertex.dasgupta.cost_norm')} | "
               f"{ci_text(p, 'dasgupta.cost_norm', ci_key='vertex.dasgupta.cost_norm')} | "
               f"{diff_text(paired, 'vertex.dasgupta.cost_norm', gold, placed)} | "
               f"{diff_text(paired, 'vertex.tsd.nats', gold, placed, 3)} | "
               f"{diff_text(paired, 'vertex.triplet.accuracy', gold, placed, 3)} | "
               f"{diff_text(paired, 'vertex.order.locality', gold, placed, 3)} | "
               f"{fmt(score(g, 'gold.ami_fine'))} / {fmt(score(p, 'gold.ami_fine'))} |\n")
    return md + "\n"


def significance_section(doc):
    md = ("## 4. Bootstrap intervals and paired significance\n\n"
          "Two bootstraps for the graph scores. The edge bootstrap resamples graph edges; the vertex "
          "bootstrap resamples items and counts each edge `m_i · m_j` times. Edges that share an item are "
          "dependent, so the vertex intervals are the honest ones, above all on the 500-book holdout. "
          "Every run on one graph and item set draws the same resamples in every replicate, so a "
          "difference between two runs is bootstrapped replicate by replicate (paired).\n\n")
    md += "### Interval widths (mean over runs, all items)\n\n"
    md += "| group | runs | cost edge | cost vertex | ratio | triplet edge | triplet vertex | TSD edge | TSD vertex | locality edge | locality vertex |\n"
    md += "| --- | ---: |" + " ---: |" * 9 + "\n"
    for group in sc.GROUPS:
        rs = [r["all"] for r in doc["runs"].values() if r["all"]["run"]["group"] == group]

        def width(key, rs=rs):
            ws = [r["catalogue"]["ci"][key][1] - r["catalogue"]["ci"][key][0] for r in rs if key in r["catalogue"]["ci"]]
            return sum(ws) / len(ws) if ws else None
        ce, cv = width("dasgupta.cost_norm"), width("vertex.dasgupta.cost_norm")
        md += (f"| {group} | {len(rs)} | {fmt(ce, 4)} | {fmt(cv, 4)} | {fmt(cv / ce if ce else None, 1)} | "
               f"{fmt(width('triplet.accuracy'), 4)} | {fmt(width('vertex.triplet.accuracy'), 4)} | "
               f"{fmt(width('tsd.nats'), 3)} | {fmt(width('vertex.tsd.nats'), 3)} | "
               f"{fmt(width('order.locality'), 4)} | {fmt(width('vertex.order.locality'), 4)} |\n")
    md += ("\n### Significant pairwise differences\n\nPairs of runs whose paired 95% interval excludes 0 "
           "(p < 0.05), out of all pairs. No multiplicity correction; with this many pairs a few "
           "significant pairs are expected by chance at p close to 0.05.\n\n")
    blocks = [(g, doc["analysis"]["groups"][g]["paired"]) for g in sc.GROUPS if doc["analysis"]["groups"][g]["paired"]]
    blocks += [(f"{f} subset", v["paired"]) for f, v in doc["analysis"]["families"].items()]
    md += "| set | " + " | ".join(SIG_METRICS) + " |\n| --- |" + " ---: |" * len(SIG_METRICS) + "\n"
    for name, paired in blocks:
        cells = []
        for m in SIG_METRICS:
            rows = paired.get(m, [])
            cells.append(f"{sum(r['p'] < 0.05 for r in rows)}/{len(rows)}" if rows else "-")
        md += f"| {name} | " + " | ".join(cells) + " |\n"
    holdout = doc["analysis"]["groups"]["placed-holdout"]["paired"]
    runs = sorted({r["a"] for r in holdout["vertex.dasgupta.cost_norm"]} | {r["b"] for r in holdout["vertex.dasgupta.cost_norm"]},
                  key=lambda k: doc["runs"][k]["all"]["catalogue"]["scores"]["dasgupta.cost_norm"])
    md += ("\n### Book holdout: pairwise cost_norm differences (vertex bootstrap)\n\n"
           "Row − column, runs in cost order (best first); * = p < 0.05. Positive means the row "
           "run costs more.\n\n")
    md += "| | " + " | ".join(runs) + " |\n| --- |" + " ---: |" * len(runs) + "\n"
    for a in runs:
        cells = []
        for b in runs:
            got = pair_lookup(holdout, "vertex.dasgupta.cost_norm", a, b) if a != b else None
            cells.append("" if not got else f"{got[0]:+.4f}{'*' if got[2] < 0.05 else ''}")
        md += f"| {a} | " + " | ".join(cells) + " |\n"
    md += "\nThe same matrix from the edge bootstrap:\n\n"
    md += "| | " + " | ".join(runs) + " |\n| --- |" + " ---: |" * len(runs) + "\n"
    for a in runs:
        cells = []
        for b in runs:
            got = pair_lookup(holdout, "dasgupta.cost_norm", a, b) if a != b else None
            cells.append("" if not got else f"{got[0]:+.4f}{'*' if got[2] < 0.05 else ''}")
        md += f"| {a} | " + " | ".join(cells) + " |\n"
    return md + "\n"


def coverage_section(doc, allr, own):
    md = ("## 2. Every score on all items and on the covered subset\n\n"
          "For every run, each catalogue score on all items and on the items the run places (its own "
          "covered subset; graph edges restricted to it). Runs that place every item have one value. "
          f"Covered subsets under {MIN_ITEMS} items are not scored (\"-\"). The key scores are below; "
          "the appendix and `results.json` hold every score.\n\n")
    for group, title in sc.GROUPS.items():
        rows = sorted(((k, r) for k, r in allr.items() if r["run"]["group"] == group),
                      key=lambda kr: score(kr[1], "dasgupta.cost_norm"))
        md += f"### {title}\n\nEach cell: all items → covered subset.\n\n"
        cols = [("coverage", lambda r: fmt(score(r, "coverage"))),
                ("covered n", lambda r: str(doc["covered_items"].get(r["run"]["key"], "-")))]
        for label, key in KEY_SCORES:
            def cell(r, key=key):
                o = own.get(r["run"]["key"])
                if score(r, "coverage") == 1.0:
                    o = r
                return f"{fmt(score(r, key))} → {fmt(score(o, key)) if o else '-'}"
            cols.append((label, cell))
        md += sc.table(rows, cols) + "\n"
    md += ("## 3. How much of each ranking is coverage\n\n"
           "Across the runs of a group: ρ(cov) is Spearman's ρ between the score and coverage, and ρ² "
           "the share of the score's rank variance that coverage explains. τ(all, covered) is Kendall's "
           "τ between the run ranking on all items and on each run's covered subset (runs whose covered "
           "subset was scored). The last two columns are the score's ρ with Dasgupta cost_norm, raw and "
           "partial on coverage. Intervals come from the bootstrap replicates of the scores (coverage is "
           "fixed), so scores without replicates have none.\n\n")
    for group, title in sc.GROUPS.items():
        ag = doc["analysis"]["groups"][group]["agree"]
        ctl = ag["control"]
        metrics = ag["metrics"]
        cost = metrics.index("dasgupta.cost_norm")
        md += f"### {title} ({len(ag['runs'])} runs)\n\n"
        md += "| score | better | ρ(cov) [CI] | ρ² | τ(all, covered) [CI] | runs | ρ with cost | partial ρ with cost [CI] |\n"
        md += "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n"
        order = sorted(range(len(metrics)), key=lambda i: -abs(ctl["rho"][i]))
        for i in order:
            m = metrics[i]
            if m == "coverage":
                continue
            cmp = ag.get("compare", {}).get(m, {})
            rho_ci = ctl["rho_ci"][i]
            part_ci = ctl["partial_ci"][i][cost]
            md += (f"| {m} | {DIRECTION[m]} | {signed(ctl['rho'][i])} {interval(rho_ci)} | "
                   f"{fmt(ctl['rho_sq'][i], 2)} | {signed(cmp.get('kendall'))} {interval(cmp.get('kendall_ci'))} | "
                   f"{cmp.get('runs', '-')} | {signed(ag['spearman'][i][cost])} | "
                   f"{partial_text(ctl['partial_spearman'][i][cost], part_ci) if i != cost else '-'} |\n")
        md += "\n"
    return md


def dense_section(doc):
    md = ("## 5. Dense against kNN-sparse Dasgupta cost\n\n"
          "On deterministic random subsets of the papers (SplitMix64, seed 20261010) every pair gets the "
          "benchmark weight `max(0, cos − τ)` with the corpus graph's τ. The dense cost sums all pairs; "
          "the kNN graphs keep each item's k most similar subset items (symmetrised), and "
          "\"graph-restricted\" is the published k = 20 corpus graph restricted to the subset (the "
          "construction of the covered-subset runs). Each scheme's tree is induced on the subset. "
          "Agreement is between the dense and the sparse run ranking: over the shelfmark-placed runs, "
          "and over all runs (plus gold and learned-own). Intervals: 200 Poisson pair-bootstrap "
          "replicates on the first subset (dense and sparse share the pair weights in a replicate). "
          "\"subsets\" gives the min–max of the point Spearman ρ over the independent subsets.\n\n")
    for corpus, d in doc["dense"].items():
        md += f"### {'Papers' if corpus == 'corpus' else 'Book holdout (all 500 books)'}\n\n"
        md += ("| n | sparsifier | edges (dense pairs w>0) | placed: Spearman [CI] | Kendall [CI] | subsets ρ | "
               "all: Spearman [CI] | Kendall [CI] |\n| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n")
        for n, draws in d["sizes"].items():
            first = draws[0]
            for sel, ag in first["agreement"].items():
                p, a_ = ag["placed"], ag["all"]
                rhos = [x["agreement"][sel]["placed"]["spearman"] for x in draws]
                spread = f"{min(rhos):.2f}–{max(rhos):.2f}" if len(rhos) > 1 else "-"
                md += (f"| {n} | {sel} | {first['edges'][sel]} ({first['edges']['dense']}) | "
                       f"{p['spearman']:+.3f} {interval(p['spearman_ci'])} | {p['kendall']:+.3f} {interval(p['kendall_ci'])} | "
                       f"{spread} | {a_['spearman']:+.3f} {interval(a_['spearman_ci'])} | "
                       f"{a_['kendall']:+.3f} {interval(a_['kendall_ci'])} |\n")
        n, draws = list(d["sizes"].items())[-1]
        costs = draws[0]["costs"]
        sels = list(draws[0]["agreement"])
        md += f"\nCosts on the first subset of n = {n}, runs in dense-cost order:\n\n"
        md += "| run | coverage | dense [CI] | " + " | ".join(sels) + " |\n| --- | ---: | ---: |" + " ---: |" * len(sels) + "\n"
        for key, c in sorted(costs.items(), key=lambda kc: kc[1]["dense"]["cost_norm"]):
            md += (f"| {key} | {c['coverage']:.3f} | {c['dense']['cost_norm']:.4f} {interval(c['dense']['ci'], 4)} | "
                   + " | ".join(f"{c[s]['cost_norm']:.4f}" for s in sels) + " |\n")
        md += "\n"
    return md


def appendix(allr, own):
    md = "## Appendix: every score, all items / covered subset\n\n"
    for group, title in sc.GROUPS.items():
        keys = sorted((k for k, r in allr.items() if r["run"]["group"] == group),
                      key=lambda k: score(allr[k], "dasgupta.cost_norm"))
        names = sorted({m for k in keys for m in allr[k]["catalogue"]["scores"]})
        md += f"### {title}\n\n| score | " + " | ".join(keys) + " |\n| --- |" + " ---: |" * len(keys) + "\n"
        for m in names:
            cells = []
            for k in keys:
                a = allr[k]["catalogue"]["scores"].get(m)
                o = allr[k] if score(allr[k], "coverage") == 1.0 else own.get(k)
                b = o["catalogue"]["scores"].get(m) if o else None
                cells.append(f"{fmt(a)} / {fmt(b)}")
            md += f"| {m} | " + " | ".join(cells) + " |\n"
        md += "\n"
    return md


def markdown(a, doc, allr, own, families, both):
    notes = Path(a.notes).read_text(encoding="utf-8") if a.notes else ""
    md = "# Covered-subset scores, coverage analysis and dense-vs-sparse cost, 2026-10-10\n\n" + notes + "\n"
    md += family_section(doc, families)
    md += both_section(doc, both)
    md += coverage_section(doc, allr, own)
    md += significance_section(doc)
    md += dense_section(doc)
    md += appendix(allr, own)
    return md
