#!/usr/bin/env python3
"""Publish benchmark runs: per-run JSON, combined results.json and results.md."""
import argparse
import gzip
import json
import shutil
import subprocess
from pathlib import Path

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
# Why a gold source was empty, checked by hand for this corpus.
GOLD_NOTES = {
    "acm-ccs": "the corpus acm_ccs field (433 values on 132 papers) holds MSC-style codes and "
               "Primary/Secondary markers copied from arXiv's ACM-class field; none is an ACM CCS "
               "2012 concept path, and no CCS 1998 to 2012 crosswalk is used",
}


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


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--out", required=True)
    p.add_argument("--harness", required=True, help="harness binary used for scoring")
    p.add_argument("--runtime", required=True, help="JSON file of wall-clock stage timings")
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

    # Graphs under 20 MB are published gzipped next to their manifests.
    graphs = {}
    (out / "graphs").mkdir(exist_ok=True)
    for corpus in ("corpus", "holdout"):
        src = cache / "graphs" / f"{corpus}-k20.json"
        manifest = load(cache / "graphs" / f"{corpus}-k20.manifest.json")
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
        if info["scheme"] in GOLD_NOTES:
            info["note"] = GOLD_NOTES[info["scheme"]]
        skipped[path.name.removesuffix(".skipped.json")] = info
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
        "placement": {"gold": "source labels, normalized to scheme codes (see REPRODUCE.md)",
                      "placed": "shelfmark profile:lexical@1, one isolated store per scheme with "
                                "an empty rules file; fallback placements (no candidate) are "
                                "counted as unplaced"},
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
    (out / "results.json").write_text(json.dumps(combined, indent=2) + "\n", encoding="utf-8")
    (out / "results.md").write_text(render_md(combined, results), encoding="utf-8")
    print(json.dumps({"results": len(results), "skipped": sorted(skipped),
                      "failures": sorted(failures)}))


def render_md(c, results):
    def pick(source, corpus):
        rows = [r for r in results.values()
                if (r["run"]["placement_source"] == "gold") == (source == "gold")
                and r["run"]["corpus"] == corpus]
        return sorted(rows, key=lambda r: r["cost_norm"])

    ref = {k: g["reference"]["ref_cost_norm"] for k, g in c["graphs"].items()}
    lines = [
        f"# Scheme benchmark results, {c['date']}",
        "",
        "Every scheme is scored as a hierarchy over a fixed similarity graph: Dasgupta cost and "
        "Moseley-Wang revenue (both normalized to [0, 1]; lower cost and higher revenue are "
        "better), skill relative to the average-linkage baseline built on the same graph "
        "(`(1 - cost_norm) / (1 - ref_cost_norm)`; 1.0 matches the baseline), placement "
        "coverage, and structure and navigation statistics. Items without a placement attach "
        "to the root, so low coverage pushes cost toward 1.",
        "",
        f"- Corpus: 10,000 arXiv papers, `corpus.jsonl` sha256 `{c['corpus']['corpus.jsonl']}`.",
        f"- Holdout: 500 Open Library editions, `holdout.jsonl` sha256 "
        f"`{c['corpus']['holdout.jsonl']}`.",
        f"- Model: `{c['model']['model_id']}` ({c['model']['runtime']}), weights "
        f"`{c['model']['weights_repository']}` revision `{c['model']['weights_revision']}`, "
        f"`model.onnx` sha256 `{c['model']['weights_sha256']}`.",
        f"- Graph: k = 20 mean-offset kNN cosine. Average-linkage baseline cost_norm: corpus "
        f"{ref['corpus']:.4f}, holdout {ref['holdout']:.4f}.",
        f"- Harness: `shelfmark-bench` at commit `{c['harness']['commit']}`.",
        "",
        "## Classifier-bias caveat",
        "",
        "Gold and shelfmark-placed results are reported separately and should not be ranked "
        "against each other. Placed results come from shelfmark's own lexical classifier "
        "(`profile:lexical@1`), so they measure the scheme *as reached by that classifier*: a "
        "scheme with rich vocabulary links gets more and better placements than one with bare "
        "labels, independent of how well its hierarchy fits the corpus. Low-confidence "
        "placements (below the 0.8 target) are kept; fallback placements, where no tier "
        "produced any candidate and shelfmark files the item under the first top-level class, "
        "are counted as unplaced. Gold results use the labels the source supplied (arXiv "
        "categories, MSC, JEL; LC call numbers for the holdout) and cover only the items that "
        "carry them.",
        "",
        "Column notes: `classes used` is the effective number of classes "
        "(exp of placement entropy); `nav log a=1` is the expected menu cost "
        "`sum(1 + log2 fanout)` from root to item; `nav scan` adds a linear menu scan and a "
        "scan of the final class shelf.",
        "",
        "## Gold placements",
        "",
        "arXiv corpus:",
        "",
        table(pick("gold", "corpus"), COLS),
        "",
        "Book holdout (LC call numbers placed at the deepest covering class):",
        "",
        table(pick("gold", "holdout"), COLS),
        "",
        "## shelfmark-placed (profile:lexical@1)",
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
    if not hits:
        return ""
    return ("Single-class captures: the lexical vocab tier matches common words against class "
            "labels and vocabulary, so in some schemes one class absorbs a large share of the "
            "corpus. These placements are kept as the classifier made them, and they shape the "
            "placed scores above: " + "; ".join(hits) + ". All 500 holdout rows are titles "
            "only (3.5 words on average, no description), which explains the low holdout "
            "coverage.")


def skip_reason(info):
    stats = info.get("stats", {})
    return (f"no {stats.get('field', 'gold')} label in {info.get('corpus')} is a code of the "
            f"scheme ({stats.get('rows', '?')} rows checked)" + (f"; {info['note']}"
                                                                 if info.get("note") else ""))


if __name__ == "__main__":
    main()
