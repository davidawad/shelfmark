#!/usr/bin/env python3
"""Publish placed runs of several classifier profiles side by side.

Each tier is one `run_benchmark.py --placed-only --tag <tier> --profile <id>
--bootstrap N` run in the shared cache. This writes, under `--out`:

- `<run>.<tier>.json`: every scored run, as the harness wrote it;
- `placements/<tier>/<corpus>-<scheme>.jsonl` (gzipped when over 20 MB);
- `graphs/<corpus>-k20.manifest.json`, pointing at the baseline's graphs;
- `results.json` and `results.md`: per-tier metrics with bootstrap 95%
  intervals, paired lexical-vs-semantic differences, and the coverage/skill
  correlation of each tier.

Bootstrap replicates are paired: every run over one corpus used the same
seed, so replicate r of every scheme and tier saw the same items.
"""
import argparse
import gzip
import json
import shutil
import statistics
from pathlib import Path

from publish_results import git, load, sha256_file

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
LIMIT = 20 * 1024 * 1024
# Metrics shown with intervals in the per-tier tables: (column, field, digits).
METRICS = [
    ("coverage", "coverage", 4),
    ("cost_norm", "cost_norm", 4),
    ("revenue_norm", "revenue_norm", 4),
    ("skill", "skill", 4),
    ("classes used", "effective_classes", 1),
    ("mean depth", "mean_depth", 2),
    ("gini", "gini", 3),
    ("nav log a=1", "nav_log_a1", 2),
    ("nav scan", "nav_scan", 1),
]


def value(r, field):
    return r[field] if field in r else r["structure"][field]


def ci(r, field):
    return r["bootstrap"]["ci95"][field]


def pct(values, q):
    s = sorted(values)
    pos = (len(s) - 1) * q
    lo, hi = int(pos), min(int(pos) + 1, len(s) - 1)
    return s[lo] + (s[hi] - s[lo]) * (pos - lo)


def interval(values):
    return [pct(values, 0.025), pct(values, 0.975)]


def ranks(xs):
    """Average ranks (1-based), ties sharing the mean rank."""
    order = sorted(range(len(xs)), key=lambda i: xs[i])
    out = [0.0] * len(xs)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and xs[order[j + 1]] == xs[order[i]]:
            j += 1
        for k in range(i, j + 1):
            out[order[k]] = (i + j) / 2 + 1
        i = j + 1
    return out


def pearson(x, y):
    try:
        return statistics.correlation(x, y)
    except statistics.StatisticsError:  # a constant series
        return None


def spearman(x, y):
    return pearson(ranks(x), ranks(y))


def correlation(runs):
    """Coverage/skill correlation over `runs`, with paired bootstrap intervals."""
    if len(runs) < 3:
        return None
    cov = [r["coverage"] for r in runs]
    skill = [r["skill"] for r in runs]
    out = {"n": len(runs), "spearman": spearman(cov, skill), "pearson": pearson(cov, skill)}
    reps = [(r["bootstrap"]["replicates"]["coverage"], r["bootstrap"]["replicates"]["skill"])
            for r in runs]
    for name, fn in (("spearman", spearman), ("pearson", pearson)):
        per_round = [fn([c[i] for c, _ in reps], [s[i] for _, s in reps])
                     for i in range(len(reps[0][0]))]
        per_round = [v for v in per_round if v is not None]
        out[f"{name}_ci95"] = interval(per_round) if per_round else None
        out[f"{name}_undefined_rounds"] = len(reps[0][0]) - len(per_round)
    return out


def gain_correlation(base, runs):
    """Correlation of the coverage gain with the skill gain of `runs` over `base`
    (paired run by run), with paired bootstrap intervals."""
    if len(runs) < 3:
        return None

    def gains(field, rep=None):
        if rep is None:
            return [r[field] - b[field] for b, r in zip(base, runs)]
        return [r["bootstrap"]["replicates"][field][rep] - b["bootstrap"]["replicates"][field][rep]
                for b, r in zip(base, runs)]

    cov, skill = gains("coverage"), gains("skill")
    out = {"n": len(runs), "spearman": spearman(cov, skill), "pearson": pearson(cov, skill)}
    rounds = len(base[0]["bootstrap"]["replicates"]["coverage"])
    for name, fn in (("spearman", spearman), ("pearson", pearson)):
        per_round = [fn(gains("coverage", i), gains("skill", i)) for i in range(rounds)]
        per_round = [v for v in per_round if v is not None]
        out[f"{name}_ci95"] = interval(per_round) if per_round else None
        out[f"{name}_undefined_rounds"] = rounds - len(per_round)
    return out


def paired_delta(a, b, field):
    """b - a, with the paired bootstrap interval of the difference."""
    ra, rb = a["bootstrap"]["replicates"][field], b["bootstrap"]["replicates"][field]
    return {"delta": b[field] - a[field], "ci95": interval([y - x for x, y in zip(ra, rb)])}


def publish_placements(src, dst):
    """Copy a placements file, gzipped when it is over the size limit."""
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.stat().st_size <= LIMIT:
        shutil.copyfile(src, dst)
        return dst
    gz = dst.with_name(dst.name + ".gz")
    with open(src, "rb") as f, gzip.GzipFile(gz, "wb", mtime=0) as g:
        shutil.copyfileobj(f, g)
    return gz


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--out", required=True)
    p.add_argument("--tier", action="append", required=True, metavar="TAG=PROFILE",
                   help="a tagged run and its profile; the first is the baseline tier")
    p.add_argument("--harness", required=True)
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--runtime", required=True, help="JSON file of wall-clock stage timings")
    p.add_argument("--graphs-from", required=True,
                   help="published results directory holding the graphs")
    p.add_argument("--check-against", metavar="DIR",
                   help="published results of the baseline tier's profile; its scores must match")
    p.add_argument("--notes", help="Markdown file whose text opens results.md")
    a = p.parse_args()
    cache, out = Path(a.cache), Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    tiers = dict(t.split("=", 1) for t in a.tier)
    base = next(iter(tiers))

    results = {tag: {} for tag in tiers}
    files = {}
    for tag in tiers:
        for path in sorted((cache / "runs" / tag).glob("*.json")):
            if path.stem.startswith(("reference-", "failures")):
                continue
            r = load(path)
            stats = r["run"]["placement_stats"]
            src = Path(stats["output"])
            pub = publish_placements(src, out / "placements" / tag / f"{src.parent.name}.jsonl")
            stats["output"] = str(pub.relative_to(out))
            stats["placements_sha256"] = sha256_file(src)
            (out / f"{path.stem}.{tag}.json").write_text(json.dumps(r, indent=2) + "\n")
            results[tag][path.stem] = r
            files[(tag, path.stem)] = src
    keys = sorted(set.intersection(*(set(v) for v in results.values())))

    graphs = {}
    (out / "graphs").mkdir(exist_ok=True)
    for corpus in ("corpus", "holdout"):
        manifest = load(cache / "graphs" / f"{corpus}-k20.manifest.json")
        manifest["published"] = str((Path(a.graphs_from) / "graphs" / f"{corpus}-k20.json.gz")
                                    .resolve().relative_to(ROOT))
        (out / "graphs" / f"{corpus}-k20.manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n")
        manifest["reference"] = load(cache / "runs" / base / f"reference-{corpus}.json")
        graphs[corpus] = manifest

    status = {line.split("\t")[0]: line.rstrip("\n").split("\t")
              for line in (cache / "schemes/status.tsv").open(encoding="utf-8")}
    schemes = {}
    for name, row in status.items():
        doc = load(cache / "schemes" / f"{name}.json")
        schemes[name] = {"status": row[1], "id": f"scheme:{doc['name']}@{doc['version']}",
                         "classes": len(doc["classes"]), "file_sha256": row[2]}

    comparison = {}
    for key in keys:
        r0 = results[base][key]
        row = {"scheme": r0["scheme"], "corpus": r0["run"]["corpus"]}
        for tag in list(tiers)[1:]:
            row[tag] = {f: paired_delta(r0, results[tag][key], f)
                        for f in ("coverage", "cost_norm", "skill")}
        comparison[key] = row

    def subset(tag, corpus):
        return [results[tag][k] for k in keys if corpus in (None, results[tag][k]["run"]["corpus"])]

    correlations = {tag: {scope: correlation(subset(tag, corpus))
                          for scope, corpus in (("corpus", "corpus"), ("holdout", "holdout"),
                                                ("all", None))}
                    for tag in tiers}
    gain_correlations = {tag: {scope: gain_correlation(subset(base, corpus), subset(tag, corpus))
                               for scope, corpus in (("corpus", "corpus"), ("holdout", "holdout"),
                                                     ("all", None))}
                         for tag in list(tiers)[1:]}
    # Does each tier rank the schemes by skill the way the baseline tier does?
    agreement = {tag: {scope: spearman([r["skill"] for r in subset(base, corpus)],
                                       [r["skill"] for r in subset(tag, corpus)])
                       for scope, corpus in (("corpus", "corpus"), ("holdout", "holdout"))}
                 for tag in list(tiers)[1:]}
    identical = {}
    tags = list(tiers)
    for i, t1 in enumerate(tags):
        for t2 in tags[i + 1:]:
            same = [k for k in keys if files[(t1, k)].read_bytes() == files[(t2, k)].read_bytes()]
            identical[f"{t1}={t2}"] = {"identical_runs": len(same), "runs": len(keys)}
    check = None
    if a.check_against:
        old = load(Path(a.check_against) / "results.json")["results"]
        diffs = {k: {f: [old[k][f], results[base][k][f]] for f in ("coverage", "cost_norm", "skill")
                     if old[k][f] != results[base][k][f]}
                 for k in keys if k in old}
        check = {"against": str(Path(a.check_against).resolve().relative_to(ROOT)),
                 "runs": len(diffs), "differing": {k: d for k, d in diffs.items() if d}}

    any_run = results[base][keys[0]]
    combined = {
        "date": out.name,
        "corpus": {"corpus.jsonl": sha256_file(ROOT / "bench/corpus/corpus.jsonl"),
                   "holdout.jsonl": sha256_file(ROOT / "bench/corpus/holdout.jsonl")},
        "model": {k: graphs["corpus"][k] for k in ("model_id", "runtime", "weights_repository",
                                                   "weights_revision", "weights_sha256")},
        "graphs": graphs,
        "harness": {"commit": git("rev-parse", "HEAD"), "binary_sha256": sha256_file(a.harness),
                    "k": 20},
        "classifier": {"commit": git("rev-parse", "HEAD"),
                       "binary_sha256": sha256_file(a.shelfmark),
                       "features": ["semantic-shortlist"]},
        "tiers": {tag: {"profile": prof} for tag, prof in tiers.items()},
        "bootstrap": {k: any_run["bootstrap"][k] for k in ("resamples", "seed", "method")},
        "placement": "one isolated store per scheme and tier with an empty rules file; "
                     "low-confidence placements kept; fallback placements (no candidate) "
                     "counted as unplaced",
        "schemes": schemes,
        "failures": {tag: load(cache / "runs" / tag / "failures.json") for tag in tiers},
        "runtime": load(a.runtime),
        "results": {tag: {k: {f: v[f] for f in ("scheme", "version", "n", "coverage", "cost_norm",
                                                "revenue_norm", "ref_cost_norm", "skill")}
                          | {"corpus": v["run"]["corpus"],
                             "placement_source": v["run"]["placement_source"],
                             "ci95": v["bootstrap"]["ci95"],
                             "bootstrap_bias": {f: m - value(v, f)
                                                for f, m in v["bootstrap"]["mean"].items()},
                             "file": f"{k}.{tag}.json",
                             "placements": v["run"]["placement_stats"]["output"]}
                          for k, v in sorted(results[tag].items())}
                    for tag in tiers},
        "comparison": comparison,
        "coverage_skill_correlation": correlations,
        "coverage_gain_skill_gain_correlation": gain_correlations,
        "skill_rank_agreement_with_" + base: agreement,
        "identical_placements": identical,
        "baseline_check": check,
    }
    (out / "results.json").write_text(json.dumps(combined, indent=2) + "\n")
    notes = Path(a.notes).read_text(encoding="utf-8").strip() if a.notes else None
    (out / "results.md").write_text(render(combined, results, keys, tiers, notes))
    print(json.dumps({"runs": len(keys), "tiers": list(tiers), "identical": identical,
                      "baseline_check": check and len(check["differing"])}))


def num(x, d=4):
    return "n/a" if x is None else f"{x:.{d}f}"


def with_ci(x, lohi, d=4):
    return f"{num(x, d)} [{num(lohi[0], d)}, {num(lohi[1], d)}]"


BIAS_NOTE = ("† marks an interval that excludes the full-sample value. A resample holds "
             "about 63% distinct items, so classes with few items drop out of it; statistics "
             "of the set of used classes (classes used, gini, and the navigation costs, which "
             "depend on fan-out) are biased in resamples, and their intervals show resampling "
             "spread around a shifted centre. `results.json` gives every metric's bootstrap "
             "bias (`bootstrap_bias`, resample mean minus full-sample value). Coverage, "
             "cost_norm, revenue_norm and skill are averages over items and pairs and are not "
             "affected this way.")


def flagged(r, field, d):
    x, (lo, hi) = value(r, field), ci(r, field)
    return with_ci(x, (lo, hi), d) + ("" if lo <= x <= hi else " †")


def signed(delta, d=4):
    x, (lo, hi) = delta["delta"], delta["ci95"]
    return f"{x:+.{d}f} [{lo:+.{d}f}, {hi:+.{d}f}]"


def md_table(head, rows):
    return "\n".join(["| " + " | ".join(head) + " |",
                      "| " + " | ".join(["---"] * 2 + ["---:"] * (len(head) - 2)) + " |",
                      *("| " + " | ".join(r) + " |" for r in rows)])


def render(c, results, keys, tiers, notes):
    tags = list(tiers)
    base, others = tags[0], tags[1:]
    order = sorted(keys, key=lambda k: (results[base][k]["run"]["corpus"] != "corpus",
                                        results[base][k]["scheme"]))
    ref = {k: g["reference"]["ref_cost_norm"] for k, g in c["graphs"].items()}
    b = c["bootstrap"]
    lines = [
        f"# Scheme benchmark, lexical and semantic classifier tiers, {c['date']}", "",
        "The shelfmark-placed part of the benchmark, run once per classifier profile on the "
        "same graphs, scheme files and average-linkage baselines: "
        + ", ".join(f"`{tag}` = `{p['profile']}`" for tag, p in c["tiers"].items()) + ". "
        "Placed scores measure a scheme *as reached by a classifier*; comparing tiers on the "
        "same scheme separates what the scheme's hierarchy is worth from how much of the corpus "
        "the classifier can file into it.", "",
        f"- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `{c['corpus']['corpus.jsonl']}`.",
        f"- Holdout: 500 Open Library editions (titles only), `holdout.jsonl` sha256 "
        f"`{c['corpus']['holdout.jsonl']}`.",
        f"- Graph: k = 20 mean-offset kNN cosine over `{c['model']['model_id']}` embeddings. "
        f"Average-linkage baseline cost_norm: corpus {ref['corpus']:.4f}, holdout "
        f"{ref['holdout']:.4f}.",
        f"- shelfmark and harness at commit `{c['classifier']['commit']}`, shelfmark built with "
        "feature `semantic-shortlist` (BGE-small-en-v1.5 int8 through ONNX Runtime).",
        f"- Intervals: {b['resamples']} item-bootstrap resamples, seed {b['seed']} ({b['method']}). "
        "Deltas are paired: the interval is of the per-resample difference.", "",
        *([notes, ""] if notes else []),
        "## Lexical vs semantic", "",
        "Skill is `(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the average-linkage tree "
        "fitted to the graph. Items without a placement attach to the root.", "",
    ]
    for tag in others:
        head = ["scheme", "corpus", f"coverage {base}", f"coverage {tag}", f"skill {base}",
                f"skill {tag}", "Δ skill", "Δ cost_norm"]
        rows = []
        for k in order:
            r0, r1, cmp = results[base][k], results[tag][k], c["comparison"][k][tag]
            rows.append([r0["scheme"], r0["run"]["corpus"], num(r0["coverage"]),
                         num(r1["coverage"]), with_ci(r0["skill"], ci(r0, "skill")),
                         with_ci(r1["skill"], ci(r1, "skill")), signed(cmp["skill"]),
                         signed(cmp["cost_norm"])])
        lines += [f"`{base}` → `{tag}`:", "", md_table(head, rows), ""]
    lines += ["## Coverage and skill", "",
              "Correlation of coverage with skill across the placed runs of each tier (corpus: "
              "13 schemes; holdout: the 6 general schemes). Intervals are over paired bootstrap "
              "replicates; a correlation is undefined in a replicate where every coverage is "
              "equal.", ""]
    rows = []
    for tag in tags:
        for scope in ("corpus", "holdout", "all"):
            x = c["coverage_skill_correlation"][tag][scope]
            if x:
                undefined = x["spearman_undefined_rounds"]
                rows.append([tag, scope, str(x["n"]),
                             with_ci(x["spearman"], x["spearman_ci95"] or [None, None], 3),
                             with_ci(x["pearson"], x["pearson_ci95"] or [None, None], 3),
                             str(undefined)])
    lines += [md_table(["tier", "runs", "n", "Spearman", "Pearson", "undefined rounds"], rows), ""]
    lines += ["Correlation of the coverage gain with the skill gain over `" + base + "`, run by "
              "run (a tier that files every item has constant coverage, so its own coverage/skill "
              "correlation is undefined; this asks instead whether the schemes where it adds the "
              "most coverage are the ones where skill rises most):", ""]
    rows = []
    for tag in others:
        for scope in ("corpus", "holdout", "all"):
            x = c["coverage_gain_skill_gain_correlation"][tag][scope]
            if x:
                rows.append([f"{base} → {tag}", scope, str(x["n"]),
                             with_ci(x["spearman"], x["spearman_ci95"] or [None, None], 3),
                             with_ci(x["pearson"], x["pearson_ci95"] or [None, None], 3),
                             str(x["spearman_undefined_rounds"])])
    lines += [md_table(["tiers", "runs", "n", "Spearman", "Pearson", "undefined rounds"], rows), ""]
    agree = c[f"skill_rank_agreement_with_{base}"]
    lines += ["Skill-rank agreement with `" + base + "` (Spearman over schemes): "
              + "; ".join(f"`{t}` corpus {num(v['corpus'], 3)}, holdout {num(v['holdout'], 3)}"
                          for t, v in agree.items()) + ".", ""]
    ident = "; ".join(f"`{k}`: {v['identical_runs']} of {v['runs']} placement files byte-identical"
                      for k, v in c["identical_placements"].items())
    lines += [f"Identical placements: {ident}.", ""]
    if c.get("baseline_check"):
        chk = c["baseline_check"]
        lines += [f"Baseline check: `{base}` coverage, cost_norm and skill against "
                  f"`{chk['against']}`: {len(chk['differing'])} of {chk['runs']} runs differ.", ""]
    for tag in tags:
        lines += [f"## `{tag}` ({c['tiers'][tag]['profile']}): all metrics", "",
                  "Value [95% interval]. `classes used` is exp of placement entropy; `nav log "
                  "a=1` is the expected `sum(1 + log2 fanout)` menu cost from root to item; `nav "
                  "scan` adds a linear menu scan and a scan of the final class shelf. "
                  + BIAS_NOTE, ""]
        rows = [[results[tag][k]["scheme"], results[tag][k]["run"]["corpus"]]
                + [flagged(results[tag][k], f, d) for _, f, d in METRICS] for k in order]
        lines += [md_table(["scheme", "corpus"] + [m for m, _, _ in METRICS], rows), ""]
        rows = []
        for k in order:
            s = results[tag][k]["run"]["placement_stats"]
            deciding = ", ".join(f"{t} {n}" for t, n in sorted(s["deciding_tiers"].items()))
            rows.append([s.get("scheme", results[tag][k]["scheme"]), results[tag][k]["run"]["corpus"],
                         str(s["placed"]), str(s["flagged"]), str(s["fallback"]), str(s["failed"]),
                         deciding, f"{s['top_class_share']:.1%} `{s['top_class']}`",
                         f"{s['classify_seconds']:.0f}"])
        lines += ["Placement counts (fallback = no candidate, counted as unplaced; `deciding` "
                  "is the tier that decided, or for a flagged item the tier of its top "
                  "candidate):", "",
                  md_table(["scheme", "corpus", "placed", "flagged", "fallback", "failed",
                            "deciding", "top class", "classify s"], rows), ""]
    fails = {t: f for t, f in c["failures"].items() if f}
    lines += ["## Failures", ""] + ([f"- `{t}` `{k}`: {v}" for t, f in fails.items()
                                     for k, v in f.items()] or ["None."])
    lines += ["", "## Runtime", ""] + [f"- {k}: {v}" for k, v in c["runtime"].items()] + [""]
    return "\n".join(lines)


if __name__ == "__main__":
    main()
