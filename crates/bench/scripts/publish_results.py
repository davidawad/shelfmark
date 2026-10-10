#!/usr/bin/env python3
"""Publish benchmark runs: per-run JSON, combined results.json and results.md."""
import argparse
import gzip
import json
import shutil
import subprocess
from pathlib import Path

from run_benchmark import PROFILE

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
REGISTRY = ROOT / "schemes/registry.json"
# Schemes scored for their gold labels but not listed in the registry.
EXTRA = {
    "lcc": "data/schemes/lcc (bundled outline)",
    "arxiv": "schemes/arxiv (bundled)",
    "msc": "schemes/msc (fetch and build)",
    "jel": "schemes/jel (fetch and build)",
}


def gold_note(name):
    """Why a gold source was empty, from the corpus field it reads."""
    if name != "acm-ccs":
        return None
    rows = [json.loads(line).get("acm_ccs") or []
            for line in (ROOT / "bench/corpus/corpus.jsonl").open(encoding="utf-8")]
    return (f"the corpus acm_ccs field ({sum(map(len, rows))} values on {sum(map(bool, rows))} "
            "papers) holds ACM CCS 1998 codes such as `I.2.6`, as submitted to arXiv; the scheme "
            "is CCS 2012, whose codes are paths of 8-digit concept IDs, and no CCS 1998 to 2012 "
            "crosswalk is used")


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True,
                          check=True).stdout.strip()


def sha256_file(path):
    import hashlib
    h = hashlib.sha256()
    h.update(Path(path).read_bytes())
    return h.hexdigest()


def fmt(x, digits=4):
    return f"{x:.{digits}f}" if isinstance(x, float) else str(x)


def table(rows, cols):
    head = "| " + " | ".join(c for c, _ in cols) + " |"
    rule = "| " + " | ".join("---" if i == 0 else "---:" for i, _ in enumerate(cols)) + " |"
    body = ["| " + " | ".join(fmt(get(r)) for _, get in cols) + " |" for r in rows]
    return "\n".join([head, rule, *body])


COLS = [
    ("scheme", lambda r: r["scheme"]),
    ("version", lambda r: r["version"]),
    ("coverage", lambda r: r["coverage"]),
    ("cost_norm", lambda r: r["cost_norm"]),
    ("revenue_norm", lambda r: r["revenue_norm"]),
    ("skill vs avg-link", lambda r: r["skill"]),
    ("classes used", lambda r: r["structure"]["effective_classes"]),
    ("mean depth", lambda r: r["structure"]["mean_depth"]),
    ("max depth", lambda r: r["structure"]["max_depth"]),
    ("max branching", lambda r: r["structure"]["max_branching"]),
    ("gini", lambda r: r["structure"]["gini"]),
    ("nav log a=1", lambda r: r["structure"]["nav_log_a1"]),
    ("nav scan", lambda r: r["structure"]["nav_scan"]),
]
COLUMN_NOTES = ("Column notes: `classes used` is the effective number of classes "
                "(exp of placement entropy); `nav log a=1` is the expected menu cost "
                "`sum(1 + log2 fanout)` from root to item; `nav scan` adds a linear menu scan "
                "and a scan of the final class shelf.")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--out", required=True)
    p.add_argument("--harness", required=True, help="harness binary used for scoring")
    p.add_argument("--runtime", required=True, help="JSON file of wall-clock stage timings")
    p.add_argument("--shelfmark", help="shelfmark binary used for the placed runs")
    p.add_argument("--baseline", help="published results directory whose graphs (and, without "
                   "--gold-only, gold results) this run reuses; its results are compared "
                   "against this run's")
    p.add_argument("--gold-only", action="store_true",
                   help="publish only gold runs (run_benchmark.py --gold-only)")
    p.add_argument("--notes", help="Markdown file whose text opens results.md, after the setup list")
    a = p.parse_args()
    cache, out = Path(a.cache), Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    runs = cache / "runs"

    results = {}
    for path in sorted(runs.glob("*.json")):
        if path.stem.startswith(("reference-", "failures")) or path.stem.endswith(".skipped"):
            continue
        result = load(path)
        stats = result["run"].get("placement_stats") or {}
        if "output" in stats:
            # Cache paths are machine-specific; keep them repo-relative.
            stats["output"] = str(Path(stats["output"]).resolve().relative_to(ROOT))
        (out / path.name).write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        results[path.stem] = result

    # Graphs under 20 MB are published gzipped next to their manifests,
    # unless they are the baseline's, which are referenced instead.
    graphs = {}
    baseline = Path(a.baseline) if a.baseline else None
    for corpus in ("corpus", "holdout"):
        src = cache / "graphs" / f"{corpus}-k20.json"
        manifest = load(cache / "graphs" / f"{corpus}-k20.manifest.json")
        if baseline:
            manifest["published"] = str((baseline / "graphs" / f"{corpus}-k20.json.gz")
                                        .resolve().relative_to(ROOT))
            manifest["reference"] = load(runs / f"reference-{corpus}.json")
            graphs[corpus] = manifest
            continue
        (out / "graphs").mkdir(exist_ok=True)
        if src.stat().st_size <= 20 * 1024 * 1024:
            with open(src, "rb") as f, gzip.GzipFile(out / "graphs" / f"{corpus}-k20.json.gz",
                                                     "wb", mtime=0) as g:
                shutil.copyfileobj(f, g)
            manifest["published"] = f"graphs/{corpus}-k20.json.gz"
        (out / "graphs" / f"{corpus}-k20.manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        manifest["reference"] = load(runs / f"reference-{corpus}.json")
        graphs[corpus] = manifest

    registry = {s["name"]: s for s in load(REGISTRY)["schemes"]}
    status = {line.split("\t")[0]: line.rstrip("\n").split("\t")
              for line in (cache / "schemes/status.tsv").open(encoding="utf-8")}
    schemes = {}
    for name, row in status.items():
        entry = {"status": row[1]}
        if row[1] == "ok":
            doc = load(cache / "schemes" / f"{name}.json")
            entry.update(id=f"scheme:{doc['name']}@{doc['version']}", version=doc["version"],
                         classes=len(doc["classes"]), file_sha256=row[2])
        else:
            entry["reason"] = row[2]
        reg = registry.get(name)
        if reg:
            kind = reg.get("distribution") or ("bundled" if (ROOT / "schemes" / reg["file"]).exists()
                                               else "built locally; file not committed")
            entry["source"] = f"schemes/registry.json ({kind})"
        else:
            entry["source"] = EXTRA.get(name, "unlisted")
        if reg:
            entry["license"] = reg.get("license")
        schemes[name] = entry

    skipped = {}
    for path in sorted(runs.glob("*.skipped.json")):
        info = load(path)
        if "output" in info.get("stats", {}):
            info["stats"]["output"] = str(Path(info["stats"]["output"]).resolve().relative_to(ROOT))
        note = gold_note(info["scheme"])
        if note:
            info["note"] = note
        key = path.name.removesuffix(".skipped.json")
        skipped[key] = info
        # A skipped run still gets its own file, so every expected result is accounted for.
        (out / f"{key}.json").write_text(json.dumps(
            {"skipped": True, "reason": skip_reason(info)} | info, indent=2) + "\n", encoding="utf-8")
    if not a.gold_only:
        skipped.update({f"{n}-placed": {"reason": s["reason"]}
                        for n, s in schemes.items() if s["status"] != "ok"})
    failures = load(runs / "failures.json")

    combined = {
        "date": out.name,
        "corpus": {"corpus.jsonl": sha256_file(ROOT / "bench/corpus/corpus.jsonl"),
                   "holdout.jsonl": sha256_file(ROOT / "bench/corpus/holdout.jsonl")},
        "model": {k: graphs["corpus"][k] for k in ("model_id", "runtime", "weights_repository",
                                                   "weights_revision", "weights_sha256")},
        "graphs": graphs,
        "harness": {
            "commit": git("log", "-1", "--format=%H", "--", "crates/bench/src",
                          "crates/bench/Cargo.toml"),
            "binary_sha256": sha256_file(a.harness),
            "k": 20,
        },
        "classifier": {"commit": git("rev-parse", "HEAD"), "profile": PROFILE}
                      | ({"binary_sha256": sha256_file(a.shelfmark)} if a.shelfmark else {}),
        "placement": {"gold": "source labels, normalized to scheme codes (see REPRODUCE.md)"}
                     | ({} if a.gold_only else
                        {"placed": f"shelfmark {PROFILE}, one isolated store per scheme with "
                                   "an empty rules file; fallback placements (no candidate) are "
                                   "counted as unplaced"}),
        "schemes": schemes,
        "skipped": skipped,
        "failures": failures,
        "runtime": load(a.runtime),
        "results": {k: {f: v[f] for f in ("scheme", "version", "track", "n", "coverage",
                                          "cost_norm", "revenue_norm", "ref_cost_norm", "skill")}
                    | {"corpus": v["run"]["corpus"], "placement_source": v["run"]["placement_source"],
                       "file": f"{k}.json"}
                    for k, v in results.items()},
    }
    if a.gold_only:
        del combined["classifier"]  # gold placements do not use it
    if baseline and a.gold_only:
        before = load(baseline / "results.json")
        combined["baseline"] = {
            "results": str((baseline / "results.json").resolve().relative_to(ROOT)),
            "corpus": before["corpus"],
            "placed": "not rerun: this run scores only the source labels",
        }
        combined["comparison"] = compare_gold(before, results, skipped)
    elif baseline:
        before = load(baseline / "results.json")
        combined["baseline"] = {
            "results": str((baseline / "results.json").resolve().relative_to(ROOT)),
            "gold": "not rerun: gold placements do not depend on the classifier, so the "
                    "baseline's gold results stand",
            "placement_source": next(iter({v["placement_source"] for v in before["results"].values()
                                           if v["placement_source"] != "gold"})),
        }
        combined["comparison"] = compare(baseline, before, results)
    (out / "results.json").write_text(json.dumps(combined, indent=2) + "\n", encoding="utf-8")
    notes = Path(a.notes).read_text(encoding="utf-8").strip() if a.notes else None
    (out / "results.md").write_text(render_md(combined, results, notes), encoding="utf-8")
    print(json.dumps({"results": len(results), "skipped": sorted(skipped),
                      "failures": sorted(failures)}))


def compare(baseline, before, results):
    """Coverage, scores and the largest class share of each placed run, before and after."""
    rows = {}
    for key, r in sorted(results.items()):
        if key not in before["results"] or r["run"]["placement_source"] == "gold":
            continue
        old = load(baseline / before["results"][key]["file"])
        rows[key] = {"scheme": r["scheme"], "corpus": r["run"]["corpus"]} | {
            when: {"coverage": v["coverage"], "cost_norm": v["cost_norm"], "skill": v["skill"],
                   "top_class": v["run"]["placement_stats"]["top_class"],
                   "top_class_share": v["run"]["placement_stats"]["top_class_share"],
                   "top_class_terms": v["run"]["placement_stats"]["top_class_terms"]}
            for when, v in (("before", old), ("after", r))}
    return rows


def compare_gold(before, results, skipped):
    """Coverage and scores of each gold run, against the baseline's gold run or skip."""
    rows = {}
    for key, r in sorted(results.items()):
        if r["run"]["placement_source"] != "gold":
            continue
        old = before["results"].get(key)
        rows[key] = {"scheme": r["scheme"], "corpus": r["run"]["corpus"],
                     "before": ({f: old[f] for f in ("coverage", "cost_norm", "skill")}
                                if old else {"skipped": key in before.get("skipped", {})}),
                     "after": {f: r[f] for f in ("coverage", "cost_norm", "skill")}}
    for key, info in sorted(skipped.items()):
        old = before["results"].get(key)
        rows[key] = {"scheme": info["scheme"], "corpus": info["corpus"],
                     "before": ({f: old[f] for f in ("coverage", "cost_norm", "skill")}
                                if old else {"skipped": key in before.get("skipped", {})}),
                     "after": {"skipped": True}}
    return rows


def gold_comparison_section(c):
    def cell(field):
        def get(r):
            def show(v):
                if v.get("skipped"):
                    return "skipped"
                return f"{v[field]:.4f}" if field in v else "not run"
            return f"{show(r['before'])} → {show(r['after'])}"
        return get
    rows = sorted(c["comparison"].values(), key=lambda r: (r["corpus"], r["scheme"]))
    return ["## Before and after", "",
            (f"Against the gold results in `{c['baseline']['results']}` (corpus.jsonl sha256 "
             f"`{c['baseline']['corpus']['corpus.jsonl']}`), on the same graphs and "
             "average-linkage baselines with the same scheme files."), "",
            table(rows, [("scheme", lambda r: r["scheme"]), ("corpus", lambda r: r["corpus"]),
                         ("coverage", cell("coverage")), ("cost_norm", cell("cost_norm")),
                         ("skill", cell("skill"))]),
            ""]


def render_md(c, results, notes=None):
    def pick(source, corpus):
        rows = [r for r in results.values()
                if (r["run"]["placement_source"] == "gold") == (source == "gold")
                and r["run"]["corpus"] == corpus]
        return sorted(rows, key=lambda r: r["cost_norm"])

    ref = {k: g["reference"]["ref_cost_norm"] for k, g in c["graphs"].items()}
    gold_only = "placed" not in c["placement"]
    lines = [
        f"# Scheme benchmark results, {c['date']}",
        "",
        ("Every scheme is scored as a hierarchy over a fixed similarity graph: Dasgupta cost and "
         "Moseley-Wang revenue (both normalized to [0, 1]; lower cost and higher revenue are "
         "better), skill relative to the average-linkage baseline built on the same graph "
         "(`(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the baseline), placement "
         "coverage, and structure and navigation statistics. Items without a placement attach "
         "to the root, so low coverage pushes cost toward 1."),
        "",
        f"- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `{c['corpus']['corpus.jsonl']}`.",
        (f"- Holdout: 500 Open Library editions, `holdout.jsonl` sha256 "
         f"`{c['corpus']['holdout.jsonl']}`."),
        (f"- Model: `{c['model']['model_id']}` ({c['model']['runtime']}), weights "
         f"`{c['model']['weights_repository']}` revision `{c['model']['weights_revision']}`, "
         f"`model.onnx` sha256 `{c['model']['weights_sha256']}`."),
        (f"- Graph: k = 20 mean-offset kNN cosine. Average-linkage baseline cost_norm: corpus "
         f"{ref['corpus']:.4f}, holdout {ref['holdout']:.4f}."),
        f"- Harness: `shelfmark-bench` at commit `{c['harness']['commit']}`.",
        *([] if gold_only else
          [f"- Classifier: `shelfmark` at commit `{c['classifier']['commit']}`, `{PROFILE}`."]),
        "",
        *([notes, ""] if notes else []),
        *(gold_only_lines(c, pick) if gold_only else placed_lines(c, results, pick)),
        "## Skipped",
        "",
    ]
    if c["skipped"]:
        for key, info in sorted(c["skipped"].items()):
            lines.append(f"- `{key}`: {info.get('reason') or skip_reason(info)}")
    else:
        lines.append("None.")
    if c["failures"]:
        lines += ["", "## Failures", ""] + [f"- `{k}`: {v}" for k, v in sorted(c["failures"].items())]
    lines += ["", "## Runtime", ""] + [f"- {k}: {v}" for k, v in c["runtime"].items()] + [""]
    return "\n".join(lines)


def gold_only_lines(c, pick):
    """Gold tables of a gold-only run, then its comparison with the baseline."""
    lines = [("Only gold placements were scored: the source labels, normalized to scheme codes. "
              "Items without a label attach to the root."), "", COLUMN_NOTES, "",
             "## Gold placements", "", "arXiv corpus:", "", table(pick("gold", "corpus"), COLS), ""]
    if pick("gold", "holdout"):
        lines += ["Book holdout:", "", table(pick("gold", "holdout"), COLS), ""]
    return lines + (gold_comparison_section(c) if c.get("comparison") else [])


def placed_lines(c, results, pick):
    return [
        "## Classifier-bias caveat",
        "",
        ("Gold and shelfmark-placed results are reported separately and should not be ranked "
         "against each other. Placed results come from shelfmark's own lexical classifier "
         f"(`{PROFILE}`), so they measure the scheme *as reached by that classifier*: a "
         "scheme with rich vocabulary links gets more and better placements than one with bare "
         "labels, independent of how well its hierarchy fits the corpus. Low-confidence "
         "placements (below the 0.8 target) are kept; fallback placements, where no tier "
         "produced any candidate and shelfmark files the item under the first top-level class, "
         "are counted as unplaced. Gold results use the labels the source supplied (arXiv "
         "categories, MSC, JEL; LC call numbers for the holdout) and cover only the items that "
         "carry them."),
        "",
        COLUMN_NOTES,
        "",
        *gold_section(c, pick),
        f"## shelfmark-placed ({PROFILE})",
        "",
        "arXiv corpus:",
        "",
        table(pick("placed", "corpus"), COLS),
        "",
        "Book holdout (general schemes only):",
        "",
        table(pick("placed", "holdout"), COLS),
        "",
        "Placement counts (fallback = no candidate, counted as unplaced):",
        "",
        table(sorted((r for r in results.values() if r["run"]["placement_stats"]
                      and "placed" in r["run"]["placement_stats"]),
                     key=lambda r: (r["run"]["corpus"], r["scheme"])),
              [("scheme", lambda r: r["scheme"]), ("corpus", lambda r: r["run"]["corpus"]),
               ("placed", lambda r: r["run"]["placement_stats"]["placed"]),
               ("flagged", lambda r: r["run"]["placement_stats"]["flagged"]),
               ("fallback", lambda r: r["run"]["placement_stats"]["fallback"]),
               ("failed", lambda r: r["run"]["placement_stats"]["failed"]),
               ("top class", lambda r: f"`{r['run']['placement_stats']['top_class']}`"),
               ("top share", lambda r: r["run"]["placement_stats"]["top_class_share"]),
               ("top terms", lambda r: ", ".join(
                   f"{t} ({n})" for t, n in r["run"]["placement_stats"]["top_class_terms"].items())),
               ("classify s", lambda r: f"{r['run']['placement_stats']['classify_seconds']:.1f}")]),
        "",
        capture_note(results),
        "",
        *comparison_section(c),
    ]


def gold_section(c, pick):
    if "baseline" in c:
        return ["## Gold placements", "",
                (f"Not rerun. Gold placements use the source labels, not the classifier, so the "
                 f"gold results in `{c['baseline']['results']}` stand; they were scored on the "
                 f"same graphs against the same average-linkage baselines."), ""]
    return ["## Gold placements", "", "arXiv corpus:", "", table(pick("gold", "corpus"), COLS), "",
            "Book holdout (LC call numbers placed at the deepest covering class):", "",
            table(pick("gold", "holdout"), COLS), ""]


def comparison_section(c):
    if not c.get("comparison"):
        return []
    rows = sorted(c["comparison"].values(), key=lambda r: (r["corpus"], r["scheme"]))

    def top(when):
        return lambda r: f"{r[when]['top_class_share']:.1%} `{r[when]['top_class']}`"

    def delta(field):
        return lambda r: f"{r['before'][field]:.4f} → {r['after'][field]:.4f}"
    return [f"## Before and after ({c['baseline']['placement_source'].split()[-1]} → {PROFILE})",
            "",
            (f"Against `{c['baseline']['results']}`, same graphs, scheme files and baselines; "
             "only the classifier changed. `top share` is the largest single-class share of the "
             "placed items."),
            "",
            table(rows, [("scheme", lambda r: r["scheme"]), ("corpus", lambda r: r["corpus"]),
                         ("top share before", top("before")), ("top share after", top("after")),
                         ("coverage", delta("coverage")), ("cost_norm", delta("cost_norm")),
                         ("skill", delta("skill"))]),
            ""]


def capture_note(results):
    """Name the placed runs where one class took over a fifth of the placed items."""
    hits = []
    for key, r in sorted(results.items()):
        stats = r["run"]["placement_stats"] or {}
        if stats.get("placed", 0) >= 100 and stats.get("top_class_share", 0) > 0.2:
            doc = load(ROOT / "bench/corpus/cache/schemes" / f"{r['scheme']}.json")
            label = next(c["label"] for c in doc["classes"] if c["code"] == stats["top_class"])
            terms = ", ".join(f'"{t}"' for t in stats["top_class_terms"])
            hits.append(f"`{key}` files {stats['top_class_share']:.1%} of its placed items "
                        f"under `{stats['top_class']}` ({label[:60]}) by the term(s) {terms}")
    holdout = ("All 500 holdout rows are titles only (3.5 words on average, no description), "
               "which explains the low holdout coverage.")
    if not hits:
        return ("Single-class captures: no placed run files more than 20% of its placed items "
                "under one class. " + holdout)
    return ("Single-class captures: the lexical vocab tier matches words of the text against "
            "class labels and vocabulary, so in some schemes one class absorbs a large share of "
            "the corpus. These placements are kept as the classifier made them, and they shape "
            "the placed scores above: " + "; ".join(hits) + ". " + holdout)


def skip_reason(info):
    stats = info.get("stats", {})
    return (f"no {stats.get('field', 'gold')} label in {info.get('corpus')} is a code of the "
            f"scheme ({stats.get('rows', '?')} rows checked)" + (f"; {info['note']}"
                                                                 if info.get("note") else ""))


if __name__ == "__main__":
    main()
