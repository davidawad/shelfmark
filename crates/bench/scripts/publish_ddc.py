#!/usr/bin/env python3
"""Publish the DDC comparison from `bench/corpus/cache/ddc-bench/` into a results directory.

Writes `results.json` (inputs and their hashes, graphs, tools, schemes,
every run with its bootstrap point and interval, and the paired cost
differences), `runs/` (harness results), `bootstrap/` (one file per
corpus), `placements/` (every placement file, gzipped when over 20 MB),
`graphs/` (the books graph and the subset graph with manifests),
`schemes/` (the Dewey notation tree and the inserted learned schemes) and
the tables of `results.md` to `<work>/tables.md`.
"""
import argparse
import gzip
import json
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from ddc_benchmark import CORPORA, CORPUS, LEARNED, PLACED, PUBLISHED, ROOT
from run_benchmark import sha256

LIMIT = 20 * 1024 * 1024
TABLE_METRICS = ["coverage", "cost_norm", "revenue_norm", "skill", "nav_log_a1", "mean_depth",
                 "effective_classes", "largest_class_share"]
ORDER = ["ddc-gold", "ddc-section", "lcc-gold", "lcc-full-gold", "fdc-ddc"] + [f"{n}-placed" for n in PLACED] + ["learned-own"]
LABEL = {"ddc-gold": "ddc (numbers)", "ddc-section": "ddc (sections)", "fdc-ddc": "fdc",
         "learned-own": "learned"}


def relative(text):
    """Paths in the published JSON are relative to the repository root."""
    return text.replace(f"{ROOT}/", "")


def copy(src, dst, gz_over=LIMIT):
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.suffix == ".json":
        dst.write_text(relative(src.read_text(encoding="utf-8")), encoding="utf-8")
        return dst
    if src.stat().st_size > gz_over:
        with src.open("rb") as f, gzip.GzipFile(dst.with_name(dst.name + ".gz"), "wb", mtime=0) as g:
            shutil.copyfileobj(f, g)
        return dst.with_name(dst.name + ".gz")
    shutil.copy(src, dst)
    return dst


def gz(src, dst):
    dst.parent.mkdir(parents=True, exist_ok=True)
    with src.open("rb") as f, gzip.GzipFile(dst, "wb", mtime=0) as g:
        shutil.copyfileobj(f, g)
    if dst.stat().st_size > LIMIT:
        dst.unlink()
        return None
    return dst


def fmt(x, ci, digits):
    return f"{x:.{digits}f} [{ci[0]:.{digits}f}, {ci[1]:.{digits}f}]"


def table(boot, runs):
    head = ("| scheme | placement | coverage | cost_norm | revenue_norm | skill | nav log a=1 "
            "| mean depth | effective classes | largest class share |")
    lines = [head, "| --- | --- |" + " ---: |" * 8]
    for key in ORDER:
        b = boot["runs"][key]
        p, ci = b["point"], b["ci"]
        name = LABEL.get(key) or key.removesuffix("-gold").removesuffix("-placed")
        source = runs[key]["run"]["placement_source"]
        source = {"gold Dewey number (notation tree)": "gold DDC", "gold LC call number": "gold LCC",
                  "gold Dewey number cut to its section": "gold DDC, 3 digits",
                  "gold Dewey number mapped to FDC": "gold DDC mapped",
                  "own insertion (nearest leaf centroid, leaf splits)": "own insertion"}.get(source, "shelfmark")
        cells = [fmt(p["coverage"], ci["coverage"], 3), fmt(p["cost_norm"], ci["cost_norm"], 4),
                 fmt(p["revenue_norm"], ci["revenue_norm"], 4), fmt(p["skill"], ci["skill"], 3),
                 fmt(p["nav_log_a1"], ci["nav_log_a1"], 2), fmt(p["mean_depth"], ci["mean_depth"], 2),
                 fmt(p["effective_classes"], ci["effective_classes"], 1),
                 fmt(p["largest_class_share"], ci["largest_class_share"], 3)]
        lines.append(f"| {name} | {source} | " + " | ".join(cells) + " |")
    return "\n".join(lines)


def paired_table(boot):
    lines = ["| a | b | cost_norm a - b | 95% CI | share of draws a lower |", "| --- | --- | ---: | ---: | ---: |"]
    for row in boot["paired"]["cost_norm"]:
        if row["a"] == "ddc-gold" or row["b"] == "ddc-gold":
            sign = 1 if row["a"] == "ddc-gold" else -1
            other = row["b"] if sign == 1 else row["a"]
            lo, hi = sorted((sign * row["ci"][0], sign * row["ci"][1]))
            lower = row["share_a_lower"] if sign == 1 else 1 - row["share_a_lower"]
            lines.append(f"| ddc-gold | {other} | {sign * row['diff']:+.4f} | [{lo:+.4f}, {hi:+.4f}] | {lower:.3f} |")
    return "\n".join(lines)


def tool_hash(path):
    return {"path": Path(path).name, "sha256": sha256(path)} if path else None


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    p.add_argument("--out", default=str(ROOT / "bench/results/2026-10-10-ddc"))
    p.add_argument("--harness", required=True)
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--learned", required=True)
    a = p.parse_args()
    work, out = Path(a.cache) / "ddc-bench", Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    for name, c in CORPORA.items():
        if "subset_of" in c:
            c.setdefault("rows", work / "subsets" / f"{name}.jsonl")
    commit = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "HEAD"], capture_output=True,
                            text=True, check=True).stdout.strip()
    result = {"date": "2026-10-10", "corpora": {}, "graphs": {}, "schemes": {}, "results": {},
              "tools": {"commit_before_run": commit, "harness": tool_hash(a.harness),
                        "shelfmark": tool_hash(a.shelfmark), "learned_scheme": tool_hash(a.learned),
                        "profile": "profile:lexical@2"}}
    tables = []
    for corpus, c in CORPORA.items():
        parent = c.get("subset_of", corpus)
        rows = Path(c["rows"]).read_text(encoding="utf-8").splitlines()
        result["corpora"][corpus] = {"rows": len(rows), "sha256": sha256(c["rows"]),
                                     "file": str(Path(c["rows"]).relative_to(ROOT)) if parent == corpus else None,
                                     "subset_of": c.get("subset_of")}
        manifest = json.loads((work / "graphs" / f"{corpus}-k20.manifest.json").read_text())
        graph = work / "graphs" / f"{corpus}-k20.json"
        if corpus == "holdout":
            manifest["published"] = str((PUBLISHED / "graphs/holdout-k20.json.gz").relative_to(ROOT))
        else:
            stored = gz(graph, out / "graphs" / f"{corpus}-k20.json.gz")
            manifest["published"] = f"graphs/{stored.name}" if stored else "not published (over 20 MB gzipped)"
            (out / "graphs" / f"{corpus}-k20.manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        if "subset_of" in c:
            shutil.copy(work / "subsets" / f"{corpus}-ids.txt", out / "graphs" / f"{corpus}-ids.txt")
        ref = json.loads((work / "runs" / f"{corpus}-reference.json").read_text())
        result["graphs"][corpus] = dict(manifest, reference=ref)
        boot = json.loads((work / "runs" / f"{corpus}-bootstrap.json").read_text())
        copy(work / "runs" / f"{corpus}-bootstrap.json", out / "bootstrap" / f"{corpus}.json")
        runs = {}
        for key in ORDER:
            src = work / "runs" / f"{corpus}-{key}.json"
            runs[key] = json.loads(src.read_text())
            copy(src, out / "runs" / corpus / f"{key}.json")
        if parent == corpus:
            for f in sorted((work / "placements" / corpus).glob("*")):
                copy(f, out / "placements" / corpus / f.name)
        result["results"][corpus] = {
            "reference_cost_norm": ref["ref_cost_norm"],
            "bootstrap": {k: boot[k] for k in ("method", "resamples", "seed", "confidence", "reference")},
            "runs": {key: {"scheme_id": runs[key]["run"]["scheme_id"],
                           "placement_source": runs[key]["run"]["placement_source"],
                           "placement_stats": runs[key]["run"]["placement_stats"],
                           "point": boot["runs"][key]["point"], "ci": boot["runs"][key]["ci"],
                           "tree_sha256": runs[key]["hashes"]["tree"]}
                     for key in ORDER},
            "paired_cost_norm_vs_ddc": [r for r in boot["paired"]["cost_norm"] if "ddc-gold" in (r["a"], r["b"])],
        }
        n = result["corpora"][corpus]["rows"]
        tables.append(f"## {corpus} (n = {n})\n\n{table(boot, runs)}\n\n### Paired cost_norm differences "
                      f"against gold DDC\n\n{paired_table(boot)}\n")
    for name in ["ddc", "lcc", "lcc-full", "fdc", "bk", "ilc2", "unesco-thesaurus", "learned",
                 "learned-holdout", "learned-books"]:
        f = work / "schemes" / f"{name}.json"
        doc = json.loads(f.read_text())
        entry = {"scheme_id": f"scheme:{doc['name']}@{doc['version']}", "file_sha256": sha256(f),
                 "classes": len(doc["classes"])}
        if name in ("ddc", "learned-books"):
            entry["published"] = str(copy(f, out / "schemes" / f"{name}.json").relative_to(out))
        elif name == "learned-holdout":
            committed = ROOT / "bench/results/2026-10-10-learned/learned-holdout.json"
            same = json.loads(committed.read_text())["classes"] == doc["classes"]
            # The committed copy predates the corpus re-pin in the source note.
            entry["same_classes_as"] = str(committed.relative_to(ROOT)) if same else None
        result["schemes"][name] = entry
    sampling = CORPUS / "holdout-books.sampling.json"
    result["corpora"]["books"]["sampling"] = json.loads(sampling.read_text())
    result["corpora"]["holdout"]["ddc_labels"] = {"file": "bench/corpus/holdout-ddc.jsonl",
                                                  "sha256": sha256(CORPUS / "holdout-ddc.jsonl")}
    result["learned_inputs"] = {"centroids_sha256": sha256(LEARNED / "data/centroids.json.gz"),
                                "holdout_vectors_sha256": sha256(LEARNED / "data/holdout-vectors.json.gz")}
    (out / "results.json").write_text(relative(json.dumps(result, indent=2)) + "\n", encoding="utf-8")
    (work / "tables.md").write_text("\n".join(tables), encoding="utf-8")
    print(f"wrote {out}/results.json and {work}/tables.md")


if __name__ == "__main__":
    main()
