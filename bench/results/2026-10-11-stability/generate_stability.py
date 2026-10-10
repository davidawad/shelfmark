#!/usr/bin/env python3
"""Recompute scheme rankings over embedding, k, threshold, and classifier settings.

Run from the repository root after building shelfmark-bench. Results are
deterministic apart from no randomized score component: the catalogue's graph
triplet sampler uses its fixed default seed. Bootstrap intervals resample the
13 schemes as paired ranking units.
"""
import argparse
import gzip
import itertools
import json
import math
import random
import statistics
import subprocess
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
MODELS = ("minilm", "bge-small", "nomic")
KS = (10, 20, 50)
TIERS = ("lexical", "shortlist")
OBJECTIVES = ("dasgupta_cost_norm", "moseley_wang_revenue", "skill",
              "triplet_accuracy", "shelf_order_locality")
THRESHOLDS = ("q00", "q25", "q50")
BOOT = 200
SEED = 20261011


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def graph_path(model, k):
    if k == 20:
        if model == "minilm":
            return ROOT / "bench/results/2026-10-10/graphs/corpus-k20.json.gz", True
        return ROOT / f"bench/results/2026-10-11-metrics/embed/graphs/{model}-corpus-k20.json", False
    return ROOT / f"bench/results/2026-10-10-embed/graphs/{model}-corpus-k{k}.json.gz", True


def ranks(values):
    order = sorted(range(len(values)), key=lambda i: (values[i], i))
    out = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        rank = (i + j) / 2 + 1
        for z in range(i, j + 1):
            out[order[z]] = rank
        i = j + 1
    return out


def pearson(x, y):
    mx, my = statistics.fmean(x), statistics.fmean(y)
    dx, dy = [v - mx for v in x], [v - my for v in y]
    den = math.sqrt(sum(v * v for v in dx) * sum(v * v for v in dy))
    return sum(a * b for a, b in zip(dx, dy)) / den if den else 1.0


def kendall(x, y):
    con = dis = 0
    for i in range(len(x)):
        for j in range(i + 1, len(x)):
            a = (x[i] > x[j]) - (x[i] < x[j])
            b = (y[i] > y[j]) - (y[i] < y[j])
            if a and b:
                con += a * b
    # Tau-b ties use the square root of untied pair totals.
    tx = sum((x[i] == x[j]) for i in range(len(x)) for j in range(i + 1, len(x)))
    ty = sum((y[i] == y[j]) for i in range(len(y)) for j in range(i + 1, len(y)))
    total = len(x) * (len(x) - 1) / 2
    den = math.sqrt((total - tx) * (total - ty))
    return con / den if den else 1.0


def percentile(values, p):
    values = sorted(values)
    pos = (len(values) - 1) * p
    lo = int(pos)
    hi = min(lo + 1, len(values) - 1)
    return values[lo] + (values[hi] - values[lo]) * (pos - lo)


def settings_compare(a, b, schemes, draws):
    x, y = [a["ranks"][s] for s in schemes], [b["ranks"][s] for s in schemes]
    kt, sp = kendall(x, y), pearson(x, y)
    kboot, sboot = [], []
    for draw in draws:
        xx, yy = [x[i] for i in draw], [y[i] for i in draw]
        kboot.append(kendall(xx, yy))
        sboot.append(pearson(ranks(xx), ranks(yy)))
    return {"kendall_tau": kt, "kendall_ci95": [percentile(kboot, .025), percentile(kboot, .975)],
            "spearman_rho": sp, "spearman_ci95": [percentile(sboot, .025), percentile(sboot, .975)]}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bench-bin", default=str(ROOT / "target/release/shelfmark-bench"))
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--bootstrap", type=int, default=BOOT)
    ap.add_argument("--reuse-results", action="store_true",
                    help="refresh report-9 claim checks and Markdown from existing results.json")
    args = ap.parse_args()
    if args.reuse_results:
        result = load_json(OUT / "results.json")
        result["research_report_9_claim_checks"] = report9_claim_checks(
            result["pairwise_setting_agreement"], result["threshold_for_claims"])
        (OUT / "results.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        write_markdown(result)
        return
    bench = Path(args.bench_bin).resolve()
    corpus = ROOT / "bench/corpus/corpus.jsonl"
    schemes = sorted(load_json(ROOT / "bench/results/2026-10-10-semantic/results.json")["schemes"])
    semantic = load_json(ROOT / "bench/results/2026-10-10-semantic/results.json")["results"]
    reference_data = load_json(ROOT / "bench/results/2026-10-10-embed/results.json")["references"]
    cache_schemes = ROOT / "bench/corpus/cache/schemes"
    tasks, graph_meta = [], {}
    with tempfile.TemporaryDirectory(prefix="stability-", dir="/work/artifacts") as temp:
        temp = Path(temp)
        for model, k in itertools.product(MODELS, KS):
            source, zipped = graph_path(model, k)
            raw = gzip.decompress(source.read_bytes()) if zipped else source.read_bytes()
            graph = json.loads(raw)
            weights = sorted(edge["weight"] for edge in graph["edges"] if edge["weight"] > 0)
            quantiles = {"q00": 0.0, "q25": percentile(weights, .25), "q50": percentile(weights, .50)}
            refkey = f"{model}-k{k}/corpus"
            fixed_ref = reference_data[refkey]["ref_cost_norm"]
            graph_meta[f"{model}-k{k}"] = {"source": str(source.relative_to(ROOT)),
                                                 "source_sha256": __import__("hashlib").sha256(raw).hexdigest(),
                                                 "tau_offset": quantiles, "fixed_reference_cost_norm": fixed_ref}
            for qname, offset in quantiles.items():
                varied = dict(graph)
                varied["tau"] = graph.get("tau", 0.0) + offset
                varied["edges"] = [dict(e, weight=e["weight"] - offset)
                                   for e in graph["edges"] if e["weight"] > offset]
                varied["edges"] = [e for e in varied["edges"] if e["weight"] > 0]
                graphfile = temp / f"{model}-k{k}-{qname}.json"
                graphfile.write_text(json.dumps(varied, separators=(",", ":")), encoding="utf-8")
                for tier, scheme in itertools.product(TIERS, schemes):
                    placement = ROOT / f"bench/results/2026-10-10-semantic/placements/{tier}/corpus-{scheme}.jsonl"
                    key = f"{model}|k{k}|{qname}|{tier}"
                    tasks.append((key, model, k, qname, tier, scheme, graphfile, placement,
                                  cache_schemes / f"{scheme}.json", fixed_ref))

        def score(task):
            key, model, k, qname, tier, scheme, graphfile, placement, schemefile, fixed_ref = task
            cmd = [str(bench), "scores", "--corpus", str(corpus), "--scheme", str(schemefile),
                   "--graph", str(graphfile), "--placements", str(placement), "--reps", "1",
                   "--boot", "1", "--sample", "200"]
            proc = subprocess.run(cmd, check=True, capture_output=True, text=True)
            doc = json.loads(proc.stdout)
            points = doc["catalogue"]["scores"]
            cost = points["dasgupta.cost_norm"]
            values = {"dasgupta_cost_norm": cost,
                      "moseley_wang_revenue": 1.0 - cost,
                      "skill": (1.0 - cost) / (1.0 - fixed_ref),
                      "triplet_accuracy": points["triplet.accuracy"],
                      "shelf_order_locality": points["order.locality"]}
            return key, scheme, values

        rows = []
        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            for i, result in enumerate(pool.map(score, tasks), 1):
                key, scheme, values = result
                rows.append({"setting": key, "scheme": scheme, "scores": values})
                if i % 50 == 0:
                    print(f"scored {i}/{len(tasks)}", flush=True)

    matrix = {}
    for objective in OBJECTIVES:
        setting_names = sorted({r["setting"] for r in rows})
        for setting in setting_names:
            selected = [r for r in rows if r["setting"] == setting]
            value_map = {r["scheme"]: r["scores"][objective] for r in selected}
            oriented = {s: (-v if objective == "dasgupta_cost_norm" else v)
                        for s, v in value_map.items()}
            ordered = sorted(schemes, key=lambda s: (-oriented[s], s))
            matrix[f"{objective}|{setting}"] = {
                "objective": objective, "setting": setting,
                "scores": value_map, "ranking": ordered,
                "ranks": {s: ranks([oriented[x] for x in schemes])[i]
                          for i, s in enumerate(schemes)}}

    rng = random.Random(SEED)
    draws = [[rng.randrange(len(schemes)) for _ in schemes] for _ in range(args.bootstrap)]
    agreements, flips = [], []
    for objective in OBJECTIVES:
        settings = [v for k, v in matrix.items() if k.startswith(objective + "|")]
        settings.sort(key=lambda r: r["setting"])
        base = next((x for x in settings if x["setting"] == "minilm|k20|q00|lexical"), settings[0])
        base_order = base["ranking"]
        for left, right in itertools.combinations(settings, 2):
            comparison = settings_compare(left, right, schemes, draws)
            agreements.append({"objective": objective, "setting_a": left["setting"],
                               "setting_b": right["setting"], **comparison})
        for item in settings:
            if item is base:
                continue
            pos0, pos1 = {s: i + 1 for i, s in enumerate(base_order)}, {
                s: i + 1 for i, s in enumerate(item["ranking"])}
            moved = [s for s in schemes if pos0[s] != pos1[s]]
            flips.append({"objective": objective, "baseline": base["setting"],
                          "setting": item["setting"], "schemes_with_changed_rank": moved,
                          "pairwise_order_flips": [f"{a}>{b}" for a, b in itertools.combinations(schemes, 2)
                              if (pos0[a] < pos0[b]) != (pos1[a] < pos1[b])]})

    claim_checks = report9_claim_checks(agreements, .8)

    result = {"title": "Rank stability under controlled graph and classifier settings",
              "corpus": "10,000-item paper corpus", "schemes": schemes,
              "settings": {"embeddings": list(MODELS), "k": list(KS), "tau_sweep": list(THRESHOLDS),
                           "tau_definition": "subtract the selected positive edge-weight quantile from every kNN edge and drop nonpositive edges; q00 preserves the source graph",
                           "classifier_tiers": {"lexical": "profile:lexical@2", "shortlist": "profile:shortlist@2"},
                           "objectives": list(OBJECTIVES), "fixed_skill_reference": "unthresholded average-linkage reference cost for the same embedding and k; held fixed within each threshold sweep"},
              "graph_sources": graph_meta, "bootstrap": {"method": "paired scheme resampling, 200 draws, seed 20261011", "units": len(schemes)},
              "threshold_for_claims": .8, "rank_matrix": matrix,
              "pairwise_setting_agreement": agreements, "rank_flips": flips,
              "research_report_9_claim_checks": claim_checks,
              "score_rows": len(rows)}
    (OUT / "results.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    write_markdown(result)


def write_markdown(result):
    checks = result["research_report_9_claim_checks"]
    lines = ["# Rank stability report", "", f"Threshold for a robust ranking claim: Spearman ρ ≥ {result['threshold_for_claims']:.1f} for every pair of controlled settings within each objective.",
             "", "## Design", "", "The run scores all 13 schemes on the 10,000-item corpus for three embedding models, k ∈ {10, 20, 50}, three graph thresholds (the source graph, then the 25th and 50th percentile positive-edge offsets), and lexical@2 / shortlist@2 placements. Each score row supplies Dasgupta cost, Moseley–Wang revenue, skill, triplet accuracy, and shelf-order locality. Skill uses the unthresholded average-linkage reference for the same embedding and k, held fixed through the threshold sweep.",
             "", "The rank matrix and every pairwise setting comparison within each objective are in `results.json`. Confidence intervals resample the same 13 schemes as paired ranking units (200 draws, seed 20261011). Rank flips list each scheme whose position changes from the MiniLM k=20 q00 lexical baseline and every pair whose ordering reverses.", "", "## Ranking checks and report 9 audit", "", "The first five rows are paper-corpus stability diagnostics for the five requested objectives. The remaining rows map report 9's book-specific rank claims to this matrix and mark them not cleared where its book schemes, cut variants, or classifier outcomes are absent.", "", "| Claim | Objective | Minimum Spearman ρ across setting pairs | Threshold | Result |", "| --- | --- | ---: | ---: | --- |"]
    for c in checks:
        score = f"{c['minimum_spearman_across_setting_pairs']:.3f}" if c.get("minimum_spearman_across_setting_pairs") is not None else "n/a"
        outcome = "PASS" if c.get("passes") else "NOT CLEARED"
        lines.append(f"| {c['claim']} | `{c['objective']}` | {score} | 0.800 | {outcome} |")
    lines.extend(["", "The report-9 checks cover its scheme, cut-depth, secondary-score, classifier-pipeline, and learned/hybrid ranking claims. `Not in this matrix` means the claim's book-only candidates or composite objective are absent from the 13-scheme paper-corpus factorial; it is not evidence for the claim. Individual pair reversals appear in `rank_flips`.", "", "## Reproduction", "", "```sh", "export PATH=/usr/local/cargo/bin:$PATH", "cargo build --release -p shelfmark-bench", "python3 bench/results/2026-10-11-stability/generate_stability.py --bench-bin target/release/shelfmark-bench --jobs 8 --bootstrap 200", "```", "", "The script reads the published graph files, schemes, and tier placements. Temporary thresholded graphs are written under `/work/artifacts` and removed when scoring completes.", ""])
    (OUT / "results.md").write_text("\n".join(lines), encoding="utf-8")


def report9_claim_checks(agreements, threshold):
    checks = []
    for objective, claim in (("dasgupta_cost_norm", "Paper corpus: FDC versus LCC scheme-rank stability by Dasgupta cost"),
                             ("moseley_wang_revenue", "Paper corpus: Moseley–Wang revenue rank stability"),
                             ("skill", "Paper corpus: skill rank stability"),
                             ("triplet_accuracy", "Paper corpus: triplet-based scheme-rank stability"),
                             ("shelf_order_locality", "Paper corpus: shelf-order locality-rank stability")):
        subset = [a for a in agreements if a["objective"] == objective]
        minimum = min((a["spearman_rho"] for a in subset), default=1.0)
        checks.append({"claim": claim, "objective": objective,
                       "minimum_spearman_across_setting_pairs": minimum,
                       "threshold": threshold, "passes": minimum >= threshold,
                       "scope": "measured across all controlled settings"})
    for claim, objective, why in (
        ("Report 9 §§5.1 and 6.1: book FDC versus LCC scheme-choice claim", "dasgupta_cost_norm", "The matrix uses the paper corpus and full FDC/LCC schemes, not the report's 3,000-book set and FDC depth cuts."),
        ("Report 9 §4.3: book triplet and locality rankings", "triplet_accuracy / shelf_order_locality", "The matrix's paper corpus and semantic placements do not reproduce the report's book candidate set."),
        ("Sections 5.1 and 6.1: Dewey-section versus FDC-depth-three ranking", "dasgupta_cost_norm", "The requested 13-scheme paper corpus contains neither Dewey cut as a candidate scheme."),
        ("Section 6.1: full-depth Dewey versus section ranking", "dasgupta_cost_norm", "Full-depth Dewey and section cuts are absent from the candidate set."),
        ("Sections 4.4 and 6.1: learned scheme and hybrid rankings", "dasgupta_cost_norm", "The learned and hybrid book-only candidates are absent from the candidate set."),
        ("Sections 6.2 and 6.3: cut-depth and classifier-pipeline rankings", "skill", "These book-only cut candidates and librarian-share pipelines are not represented by the published paper placements."),
        ("Section 6.1: locality-weighted composite J ranking", "composite_J", "Composite J is outside the five requested objectives; the matrix reports locality as a separate objective."),
        ("Section 8: semantic-classifier book threshold", "coverage_and_agreement", "The semantic rerun covers the paper corpus; it does not provide classifier agreement on the 3,000-book set.")):
        checks.append({"claim": claim, "objective": objective,
                       "minimum_spearman_across_setting_pairs": None,
                       "threshold": threshold, "passes": False,
                       "scope": "not in this matrix", "reason": why})
    return checks


if __name__ == "__main__":
    main()
