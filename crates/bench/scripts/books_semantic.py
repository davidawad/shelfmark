#!/usr/bin/env python3
"""Classifier agreement with librarians on the 3,000 books, per profile.

Places `bench/corpus/holdout-books.jsonl` into `fdc`, `lcc` (outline) and the
bundled `lcc@2024` (outline + LCSH) with each profile given, exactly as the
`-placed` runs of `bench/results/2026-10-10-ddc/` and
`bench/results/2026-10-11-book-stability/` were made: one isolated store per
scheme and profile with an empty rules file, requests through
`shelfmark batch`, low-confidence placements kept, fallbacks unplaced.

Then it compares each placement with the librarian's number at depth 1, 2
and 3 (`fdc-ddc` for FDC, `lcc-gold` for both LCC runs, from
`2026-10-10-ddc/placements/books/`), with Wilson 95% intervals, as
`research/reports/scripts/kb_proposal_tables.py` `t_classifier` does.

The lexical profile must reproduce the committed lexical placements byte
for byte (`--check-lexical`); that checks the store setup before the
semantic numbers are read.

    python3 crates/bench/scripts/books_semantic.py --shelfmark $BIN/shelfmark \
        --profile profile:lexical@2 --profile profile:shortlist@2 \
        --out bench/results/2026-10-11-books-semantic --check-lexical

Needs a shelfmark built with `--features semantic-shortlist` for the
shortlist profile (it downloads BGE-small-en-v1.5 on first use).

`--tables-only` skips placement and recomputes `results.json` and
`table.md` from the placements already under `--out`. For every profile
after the first it also scores the books that profile places and the first
profile leaves unplaced, and counts the books both place in the same class.
"""
import argparse
import gzip
import hashlib
import json
import math
import shutil
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PREP = [sys.executable, str(HERE / "prepare_placements.py")]
BOOKS = ROOT / "bench/corpus/holdout-books.jsonl"
DDC = ROOT / "bench/results/2026-10-10-ddc/placements/books"
STABILITY = ROOT / "bench/results/2026-10-11-book-stability/placements/books"
FDC = ROOT / "schemes/fdc/fdc.json"
LCC = ROOT / "data/schemes/lcc/lcc.json"
# run -> (scheme file compiled into the store or None for the bundled scheme,
#         scheme id, gold placements, parents file, committed lexical placements)
RUNS = {
    "fdc-placed": (FDC, None, DDC / "fdc-ddc.jsonl", FDC, DDC / "fdc-placed.jsonl"),
    "lcc-placed": (LCC, None, DDC / "lcc-gold.jsonl", LCC, DDC / "lcc-placed.jsonl"),
    "lcc-lcsh-placed": (None, "scheme:lcc@2024", DDC / "lcc-gold.jsonl", LCC,
                        STABILITY / "lcc-lcsh-placed.jsonl"),
}


def run(cmd):
    p = subprocess.run([str(c) for c in cmd], capture_output=True, text=True)
    if p.returncode:
        raise SystemExit(f"{' '.join(map(str, cmd))}\n{p.stdout[-2000:]}\n{p.stderr[-2000:]}")
    return p.stdout


def last_json(text):
    return json.loads([line for line in text.splitlines() if line.strip()][-1])


def jsonl(path):
    path = Path(path)
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def wilson(k, n, z=1.959964):
    phat = k / n
    centre = (phat + z * z / (2 * n)) / (1 + z * z / n)
    half = z * math.sqrt(phat * (1 - phat) / n + z * z / (4 * n * n)) / (1 + z * z / n)
    return centre - half, centre + half


def ancestor_at(code, parents, depth):
    chain = []
    while code:
        chain.append(code)
        code = parents.get(code)
    chain = chain[::-1]
    return chain[min(depth, len(chain)) - 1]


def committed(path):
    """The committed lexical placements; book-stability ones may be gzipped."""
    for p in (path, path.with_name(path.name + ".gz")):
        if p.exists():
            return p
    raise SystemExit(f"missing {path}")


def place(a, key, profile, work):
    scheme_file, scheme_id, *_ = RUNS[key]
    tag = profile.split(":")[1].split("@")[0]
    d = work / tag / key
    out = d / "placements.jsonl"
    stats_path = d / "stats.json"
    if stats_path.exists():
        return out, json.loads(stats_path.read_text())
    store = d / "store"
    store.mkdir(parents=True, exist_ok=True)
    (store / "rules.json").write_text('{"rules":[]}\n', encoding="utf-8")
    base = [a.shelfmark, "--json", "--store", store]
    if scheme_file:
        preview = json.loads(run(base + ["scheme", "compile", scheme_file]))
        run(base + ["--commit", "--expected-revision", preview["data"]["revision"], "scheme", "compile",
                    scheme_file])
        doc = json.loads(Path(scheme_file).read_text(encoding="utf-8"))
        scheme_id = f"scheme:{doc['name']}@{doc['version']}"
    requests, batch = d / "requests.jsonl", d / "batch.jsonl"
    run(PREP + ["requests", "--corpus", BOOKS, "--scheme", scheme_id, "--profile", profile,
                "--output", requests])
    started = time.monotonic()
    run(base + ["batch", "--requests", requests, "--out", batch] + (["--resume"] if batch.exists() else []))
    seconds = round(time.monotonic() - started, 1)
    stats = last_json(run(PREP + ["extract", "--corpus", BOOKS, "--batch", batch, "--scheme", scheme_id,
                                  "--profile", profile, "--output", out]))
    stats.update(classify_seconds=seconds, profile=profile, scheme=scheme_id, store_rules="empty")
    stats_path.write_text(json.dumps(stats, indent=2) + "\n", encoding="utf-8")
    print(f"{tag} {key}: placed {stats.get('placed')} in {seconds} s", flush=True)
    return out, stats


def first_classes(path):
    return {r["id"]: (r["classes"][0] if isinstance(r["classes"][0], str) else r["classes"][0]["code"])
            for r in jsonl(path) if r["classes"]}


def agreement(key, placements, only=None):
    _, _, gold_path, parents_path, _ = RUNS[key]
    parents = {c["code"]: c.get("parent") for c in json.loads(parents_path.read_text())["classes"]}
    gold = {r["id"]: r["classes"][0] for r in jsonl(gold_path) if r["classes"]}
    placed = first_classes(placements)
    both = [i for i in placed if i in gold and (only is None or i in only)]
    rows = []
    for depth in (1, 2, 3):
        hits = sum(ancestor_at(placed[i], parents, depth) == ancestor_at(gold[i], parents, depth) for i in both)
        lo, hi = wilson(hits, len(both))
        rows.append({"depth": depth, "hits": hits, "n": len(both), "agreement": hits / len(both),
                     "ci": [lo, hi]})
    return {"placed": len(placed), "books": len(gold), "coverage": len(placed) / sum(1 for _ in jsonl(BOOKS)),
            "depths": rows}


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--shelfmark", required=True)
    p.add_argument("--profile", action="append", required=True)
    p.add_argument("--work", default=str(ROOT / "bench/corpus/cache/books-semantic"))
    p.add_argument("--out", required=True)
    p.add_argument("--tables-only", action="store_true",
                   help="recompute the tables from the placements already under --out")
    p.add_argument("--check-lexical", action="store_true",
                   help="fail unless profile:lexical@2 reproduces the committed lexical placements")
    a = p.parse_args()
    work, out = Path(a.work), Path(a.out)
    res = {"books_sha256": sha256(BOOKS), "profiles": {}}
    first = a.profile[0].split(":")[1].split("@")[0]
    for profile in a.profile:
        tag = profile.split(":")[1].split("@")[0]
        res["profiles"][profile] = {}
        for key in RUNS:
            dest = out / "placements" / tag / f"{key}.jsonl"
            if a.tables_only:
                stats = json.loads(dest.with_suffix(".stats.json").read_text())
            else:
                placements, stats = place(a, key, profile, work)
                if a.check_lexical and profile == "profile:lexical@2":
                    ref = committed(RUNS[key][4])
                    if jsonl(placements) != jsonl(ref):
                        raise SystemExit(f"{key}: lexical placements differ from {ref.relative_to(ROOT)}")
                    print(f"{key}: lexical placements equal {ref.relative_to(ROOT)}", flush=True)
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(placements, dest)
                dest.with_suffix(".stats.json").write_text(json.dumps(stats, indent=2) + "\n", encoding="utf-8")
            r = agreement(key, dest)
            r["placements_sha256"] = sha256(dest)
            r["classify_seconds"] = stats.get("classify_seconds")
            r["decided_without_flag"] = stats.get("placed", 0) - stats.get("flagged", 0)
            r["deciding_tiers"] = stats.get("deciding_tiers")
            if tag != first:
                base = first_classes(out / "placements" / first / f"{key}.jsonl")
                mine = first_classes(dest)
                r["added_over_" + first] = agreement(key, dest, only={i for i in mine if i not in base})
                r["same_class_as_" + first] = [sum(mine.get(i) == c for i, c in base.items()), len(base)]
            res["profiles"][profile][key] = r
    out.mkdir(parents=True, exist_ok=True)
    (out / "results.json").write_text(json.dumps(res, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    L = ["| profile | run | depth | placed / books | decided without flag | same class as librarian | "
         "agreement | 95% Wilson interval |", "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for profile, runs in res["profiles"].items():
        for key, r in runs.items():
            for d in r["depths"]:
                L.append(f"| `{profile}` | `{key}` | {d['depth']} | {r['placed']}/{r['books']} | "
                         f"{r['decided_without_flag']} | {d['hits']}/{d['n']} | {d['agreement']:.3f} | "
                         f"[{d['ci'][0]:.3f}, {d['ci'][1]:.3f}] |")
    added = [(profile, key, d) for profile, runs in res["profiles"].items() for key, r in runs.items()
             for d in (r.get("added_over_" + first) or {"depths": []})["depths"]]
    if added:
        L += ["", f"Books placed by a later profile that `{first}` leaves unplaced:", "",
              "| profile | run | depth | books | same class as librarian | agreement | 95% Wilson interval |",
              "| --- | --- | ---: | ---: | ---: | ---: | ---: |"]
        L += [f"| `{profile}` | `{key}` | {d['depth']} | {d['n']} | {d['hits']}/{d['n']} | {d['agreement']:.3f} | "
              f"[{d['ci'][0]:.3f}, {d['ci'][1]:.3f}] |" for profile, key, d in added]
    (out / "table.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    print("\n".join(L))


if __name__ == "__main__":
    main()
