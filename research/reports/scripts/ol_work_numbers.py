#!/usr/bin/env python3
"""Work-level Dewey and LC numbers from Open Library, for report 9.

For every work of two book samples, one Open Library search request
(`q=key:"/works/<id>"`, fields `key,ddc,lcc,edition_count`) reads the
Dewey numbers (`ddc`) and LC call numbers (`lcc`, sort form) that Open
Library aggregates over all editions of the work:

- `holdout`: the works of the 500-edition book holdout
  (`bench/corpus/holdout.jsonl`, `source.work`);
- `books50`: the 50 well-known books of the Open Library tier measurement,
  read with `git show <ref>:crates/tier-openlibrary/eval/books-50.json`
  (that branch is not merged; the default ref is `origin/dot-4xu6b.6`).

Requests go one at a time, at least 1.05 s apart, with the benchmark's
User-Agent, and every answer is cached under
`bench/corpus/cache/ol-work-numbers/` (git-ignored), so a rerun replays
from the cache. Open Library data is CC0
(https://openlibrary.org/developers/licensing). Output: one JSONL file per
sample in `bench/results/2026-10-10-proposal/availability/`.

    python3 -I research/reports/scripts/ol_work_numbers.py
"""
import argparse
import hashlib
import json
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CACHE = ROOT / "bench/corpus/cache/ol-work-numbers"
OUT = ROOT / "bench/results/2026-10-10-proposal/availability"
NS = "https://openlibrary.org"
UA = "shelfmark-bench/1.0 (classification benchmark; +https://gitlab.com/davidawad/shelfmark)"
_last = [0.0]


def get(path, attempts=6):
    key = hashlib.sha256(path.encode()).hexdigest()
    cached = CACHE / f"{key}.json"
    if cached.exists():
        return json.loads(cached.read_text(encoding="utf-8"))
    for attempt in range(attempts):
        delay = 1.05 - (time.time() - _last[0])
        if delay > 0:
            time.sleep(delay)
        _last[0] = time.time()
        req = urllib.request.Request(NS + path, headers={"User-Agent": UA})
        try:
            with urllib.request.urlopen(req, timeout=60) as response:
                body = json.loads(response.read())
            if "docs" in body:
                CACHE.mkdir(parents=True, exist_ok=True)
                cached.write_text(json.dumps(body), encoding="utf-8")
                return body
        except (urllib.error.URLError, TimeoutError, OSError, json.JSONDecodeError) as e:
            print(f"retry {attempt + 1} for {path}: {e}", flush=True)
        time.sleep(5 * 2 ** attempt)
    raise SystemExit(f"no answer for {path}")


def work_numbers(work):
    q = urllib.parse.urlencode({"q": f'key:"/works/{work}"', "fields": "key,ddc,lcc,edition_count",
                                "limit": 1})
    docs = get(f"/search.json?{q}")["docs"]
    doc = docs[0] if docs else {}
    return {"work": work, "found": bool(docs), "edition_count": doc.get("edition_count", 0),
            "ddc": sorted(doc.get("ddc", [])), "lcc": sorted(doc.get("lcc", []))}


def samples(ref):
    holdout = []
    with open(ROOT / "bench/corpus/holdout.jsonl", encoding="utf-8") as f:
        for line in f:
            row = json.loads(line)
            holdout.append({"id": row["id"], "work": row["source"]["work"]})
    books = json.loads(subprocess.run(
        ["git", "show", f"{ref}:crates/tier-openlibrary/eval/books-50.json"],
        cwd=ROOT, check=True, capture_output=True, text=True).stdout)["books"]
    books50 = [{"id": b["title"], "work": b["record"].rsplit("/", 1)[1]} for b in books]
    return {"holdout": holdout, "books50": books50}


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--ref", default="origin/dot-4xu6b.6")
    a = p.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    for name, rows in samples(a.ref).items():
        out = []
        for i, row in enumerate(rows, 1):
            out.append({"id": row["id"], **work_numbers(row["work"])})
            if i % 50 == 0:
                print(f"{name}: {i}/{len(rows)}", flush=True)
        path = OUT / f"{name}-works.jsonl"
        path.write_text("".join(json.dumps(r, sort_keys=True) + "\n" for r in out), encoding="utf-8")
        print(json.dumps({"sample": name, "rows": len(out), "found": sum(r["found"] for r in out),
                          "with_ddc": sum(bool(r["ddc"]) for r in out),
                          "with_lcc": sum(bool(r["lcc"]) for r in out), "output": str(path)}))


if __name__ == "__main__":
    main()
