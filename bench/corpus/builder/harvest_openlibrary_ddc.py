#!/usr/bin/env python3
"""Fetch Dewey numbers for the book holdout and harvest a 3,000-edition DDC holdout.

Two stages, both resumable, with every HTTP response cached under
`bench/corpus/cache/openlibrary-ddc/http/`, one request at a time and at
least 1.05 s between request starts to openlibrary.org:

`holdout` re-reads each edition of `holdout.jsonl` and writes
`holdout-ddc.jsonl`: the edition's `dewey_decimal_class` (empty when the
record has none) and the record revision read.

`books` samples editions that carry both an LC classification and a Dewey
number, plus a description or subjects, stratified over the 21 LCC main
classes (`A` ... `Z`, equal quotas, 3,000 in total). For each class the
Search API is read at evenly spaced offsets of the works with an LCC in the
class and any DDC, ordered by `lcc_sort` (pages past the middle are read
from the reverse order at the mirrored offset, and a page that still fails
from the last `lcc_sort` anchor before it, as the API fails on very deep
offsets), so candidates come from across the whole class. Candidates are ranked by SHA-256 of the seed and work key.
For each work in that order the first editions page is read; an edition
qualifies when its first LC call number is in the class and one of its
Dewey values holds a number. Among qualifying editions the one with the
lowest seeded SHA-256 rank is taken, then the work record supplies the
description and subjects. Works in the 500-book holdout are skipped. Only
Open Library (CC0) fields are written.
"""
import argparse
import hashlib
import json
import pathlib
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
CACHE = ROOT / "cache" / "openlibrary-ddc"
NS = "https://openlibrary.org"
UA = "shelfmark-bench/1.0 (classification benchmark; +https://gitlab.com/davidawad/shelfmark)"
SEED = "openlibrary-ddc-holdout-v1-2026"
CLASSES = "ABCDEFGHJKLMNPQRSTUVZ"
TOTAL = 3000
OFFSETS = 20
PAGE = 30
MAX_DESCRIPTION = 4000
MAX_SUBJECTS = 30
_last = [0.0]


def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)


def search_ok(path, body):
    """A Search API answer is complete: under load the API can answer 200
    with no results (`numFound` 0) or a short page."""
    if not path.startswith("/search.json"):
        return True
    query = urllib.parse.parse_qs(path.split("?", 1)[1])
    limit, offset = int(query["limit"][0]), int(query.get("offset", ["0"])[0])
    found = (body or {}).get("numFound", 0)
    return found > 0 and len(body.get("docs", [])) == max(0, min(limit, found - offset))


def get(path, attempts=8):
    """GET `path` as JSON, from the cache when present; None for HTTP 404.

    Incomplete search answers are retried and never cached (nor used from
    the cache).
    """
    key = hashlib.sha256(path.encode()).hexdigest()
    cached = CACHE / "http" / key[:2] / f"{key}.json"
    if cached.exists():
        body = json.loads(cached.read_text(encoding="utf-8"))["body"]
        if search_ok(path, body):
            return body
    for attempt in range(attempts):
        # One request at a time, started at least 1.05 s after the previous one.
        delay = 1.05 - (time.time() - _last[0])
        if delay > 0:
            time.sleep(delay)
        _last[0] = time.time()
        req = urllib.request.Request(NS + path, headers={"User-Agent": UA})
        try:
            with urllib.request.urlopen(req, timeout=120) as response:
                body = json.loads(response.read())
            if search_ok(path, body):
                break
            wait = 5 * 2 ** attempt
            log(f"incomplete answer for {path}; retry in {wait}s")
        except urllib.error.HTTPError as e:
            if e.code == 404:
                body = None
                break
            wait = 5 * 2 ** attempt
            log(f"HTTP {e.code} for {path}; retry in {wait}s")
        except (urllib.error.URLError, TimeoutError, OSError, json.JSONDecodeError) as e:
            wait = 5 * 2 ** attempt
            log(f"{type(e).__name__} for {path}; retry in {wait}s")
        time.sleep(wait)
    else:
        raise RuntimeError(f"giving up on {path}")
    cached.parent.mkdir(parents=True, exist_ok=True)
    cached.write_text(json.dumps({"path": path, "fetched": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                                  "body": body}), encoding="utf-8")
    return body


def rank(*parts):
    return hashlib.sha256("|".join((SEED,) + parts).encode()).hexdigest()


DDC = re.compile(r"\s*\[?\s*(?:[jJ]\s*)?(\d{3})((?:[./' ]*\d+)*)")


def ddc_number(value):
    """`813/.54` -> `813.54`; `016.3092 s` -> `016.3092`; `[Fic]` -> None.

    Prime marks and slashes are Dewey segmentation marks and are dropped;
    a trailing letter (`s` for a series number, `B`) and anything after it
    is ignored, as are trailing zeros of the decimal part.
    """
    m = DDC.match(value)
    if not m:
        return None
    head, rest = m.group(1), m.group(2)
    digits = re.sub(r"[./' ]", "", rest).rstrip("0")
    return f"{head}.{digits}" if digits else head


LCC = re.compile(r"\s*([A-Z]{1,3})\s*\d")


def lcc_main_class(value):
    """Main class letter of an LC call number; None for MLCS/MLCM shelf numbers."""
    if value.strip().upper().startswith("MLC"):
        return None
    m = LCC.match(value.upper())
    return m.group(1)[0] if m else None


def text_value(value):
    if isinstance(value, dict):
        value = value.get("value", "")
    return " ".join(str(value or "").split())


def holdout_stage(args):
    rows = [json.loads(line) for line in (ROOT / "holdout.jsonl").open(encoding="utf-8")]
    out = []
    for i, row in enumerate(rows, 1):
        edition = row["id"].split(":", 1)[1]
        record = get(f"/books/{edition}.json") or {}
        values = record.get("dewey_decimal_class") or []
        out.append({"id": row["id"], "dewey_decimal_class": values,
                    "source": {"kind": "openlibrary", "url": f"{NS}/books/{edition}",
                               "revision": record.get("revision"), "metadata_license": "CC0 1.0",
                               "label_source": "dewey_decimal_class"}})
        if i % 50 == 0:
            log(f"holdout: {i}/{len(rows)}; with a Dewey value: {sum(bool(r['dewey_decimal_class']) for r in out)}")
    path = ROOT / "holdout-ddc.jsonl"
    path.write_text("".join(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n" for r in out),
                    encoding="utf-8")
    log(f"wrote {path}: {len(out)} rows, {sum(any(map(ddc_number, r['dewey_decimal_class'])) for r in out)} with a Dewey number")


def candidates(letter):
    """Work keys spread over one LCC main class, ranked by seeded SHA-256."""
    q = f"lcc:{letter}* AND ddc:*"
    base = {"q": q, "sort": "lcc_sort", "fields": "key"}
    first = get("/search.json?" + urllib.parse.urlencode(dict(base, limit=1, offset=0)))
    found = first["numFound"]
    keys, anchored = [], []
    for i in range(OFFSETS):
        offset = max(0, min(found - PAGE, (2 * i + 1) * found // (2 * OFFSETS) - PAGE // 2))
        query = dict(base, limit=PAGE, offset=offset)
        if 2 * i >= OFFSETS:
            # Very deep offsets fail on the Search API (HTTP 500), so the back
            # half of the class is read from the reverse order at the mirrored
            # offset: the same positions in the lcc_sort order.
            query = dict(base, sort="lcc_sort desc", limit=PAGE, offset=found - offset - PAGE)
        # Pages near the middle of the largest classes (P) are deep from both
        # ends; one that fails 3 times is read from a nearby anchor, and a
        # marker makes a rerun read it from the anchor again.
        marker = CACHE / "anchored" / f"{letter}-{offset}"
        try:
            if marker.exists():
                raise RuntimeError("read from the anchor before")
            page = get("/search.json?" + urllib.parse.urlencode(query), attempts=3)
        except RuntimeError:
            log(f"class {letter}: reading the page at offset {offset} from an anchor")
            page = anchored_page(q, letter, offset)
            marker.parent.mkdir(parents=True, exist_ok=True)
            marker.touch()
            anchored.append(offset)
        keys += [d["key"].rsplit("/", 1)[-1] for d in page.get("docs", [])]
    keys = list(dict.fromkeys(keys))
    return found, sorted(keys, key=lambda w: rank(letter, w)), anchored


def anchors(letter):
    """LC numbers in `lcc_sort` order: `P1`, `P1000` ... `PZ9000`, every 1,000 class numbers."""
    return [f"{letter}{s}{n}" for s in [""] + list("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
            for n in [1] + list(range(1000, 10000, 1000))]


def anchored_page(q, letter, offset):
    """The `lcc_sort` page at `offset`, read as a shallow page after the
    last anchor at or before it. An anchor's position is the number of works
    sorted at or before it (`lcc_sort:[A1 TO anchor]`); the API normalizes
    range bounds as LC numbers and rejects `*` or `0` as a lower bound.
    """
    def position(anchor):
        found = get("/search.json?" + urllib.parse.urlencode(
            {"q": f"{q} AND lcc_sort:[A1 TO {anchor}]", "fields": "key", "limit": 1}))
        return found["numFound"]

    grid = anchors(letter)
    lo, hi = 0, len(grid) - 1
    if position(grid[0]) > offset:
        raise RuntimeError(f"class {letter}: offset {offset} is before the first anchor")
    while lo < hi:
        mid = (lo + hi + 1) // 2
        if position(grid[mid]) <= offset:
            lo = mid
        else:
            hi = mid - 1
    anchor = grid[lo]
    return get("/search.json?" + urllib.parse.urlencode(
        {"q": f"{q} AND lcc_sort:[{anchor} TO *]", "sort": "lcc_sort", "fields": "key",
         "limit": PAGE, "offset": offset - position(anchor)}))


def pick_edition(work, letter):
    page = get(f"/works/{work}/editions.json?limit=100") or {}
    best = None
    for edition in page.get("entries", []):
        lcc = [v for v in edition.get("lc_classifications") or [] if lcc_main_class(v)]
        ddc = edition.get("dewey_decimal_class") or []
        if not lcc or lcc_main_class(lcc[0]) != letter or not any(map(ddc_number, ddc)):
            continue
        key = edition.get("key", "").rsplit("/", 1)[-1]
        if key and (best is None or rank(key) < rank(best["key"].rsplit("/", 1)[-1])):
            best = edition
    return best


def book_row(work, edition, record):
    edition_key = edition["key"].rsplit("/", 1)[-1]
    description = text_value(edition.get("description")) or text_value(record.get("description"))
    subjects = []
    for s in (edition.get("subjects") or []) + (record.get("subjects") or []):
        s = text_value(s).rstrip(".")
        if s and s not in subjects:
            subjects.append(s)
    subjects = subjects[:MAX_SUBJECTS]
    description = description[:MAX_DESCRIPTION]
    if not description and not subjects:
        return None
    parts = [description] if description else []
    if subjects:
        parts.append("Subjects: " + "; ".join(subjects) + ".")
    return {
        "id": f"openlibrary:{edition_key}",
        "title": text_value(edition.get("title")) or text_value(record.get("title")),
        "abstract_text": " ".join(parts),
        "ol_description": description,
        "subjects": subjects,
        "authors": [],
        "lc_classifications": edition.get("lc_classifications") or [],
        "dewey_decimal_class": edition.get("dewey_decimal_class") or [],
        "license": "CC0 1.0",
        "source": {"kind": "openlibrary", "url": f"{NS}/books/{edition_key}", "work": work,
                   "edition_revision": edition.get("revision"), "work_revision": record.get("revision"),
                   "metadata_license": "CC0 1.0",
                   "label_source": "lc_classifications+dewey_decimal_class",
                   "text_source": "description (edition, else work) + subjects (edition, then work)"}}


def books_stage(args):
    skip = {json.loads(line)["source"]["work"] for line in (ROOT / "holdout.jsonl").open(encoding="utf-8")}
    quotas = {c: TOTAL // len(CLASSES) + (i < TOTAL % len(CLASSES)) for i, c in enumerate(CLASSES)}
    state_path = CACHE / "books-state.json"
    rows, pools = [], {}
    for letter in CLASSES:
        found, pool, anchored = candidates(letter)
        taken = 0
        tried = 0
        for work in pool:
            if taken == quotas[letter]:
                break
            if work in skip:
                continue
            tried += 1
            edition = pick_edition(work, letter)
            if edition is None:
                continue
            row = book_row(work, edition, get(f"/works/{work}.json") or {})
            if row is None:
                continue
            skip.add(work)
            row["source"]["stratum"] = letter
            rows.append(row)
            taken += 1
        pools[letter] = {"works_in_class": found, "pages_from_anchor_at": anchored, "candidates": len(pool), "tried": tried,
                         "taken": taken, "quota": quotas[letter]}
        log(f"class {letter}: {taken}/{quotas[letter]} from {tried} tried of {len(pool)} candidates")
        state_path.write_text(json.dumps(pools, indent=2) + "\n", encoding="utf-8")
    if len(rows) < TOTAL:
        log(f"only {len(rows)} rows; short classes: "
            + ", ".join(c for c, p in pools.items() if p["taken"] < p["quota"]))
    rows.sort(key=lambda r: r["id"])
    out = ROOT / "holdout-books.jsonl"
    out.write_text("".join(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n" for r in rows),
                   encoding="utf-8")
    (ROOT / "holdout-books-ids.txt").write_text("".join(r["id"] + "\n" for r in rows), encoding="utf-8")
    (ROOT / "holdout-books.sampling.json").write_text(json.dumps(
        {"seed": SEED, "classes": CLASSES, "total": TOTAL, "offsets_per_class": OFFSETS,
         "page_size": PAGE, "search": "lcc:<class>* AND ddc:*, sort=lcc_sort, fields=key",
         "per_class": pools}, indent=2) + "\n", encoding="utf-8")
    log(f"wrote {out}: {len(rows)} rows")


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("stage", choices=["holdout", "books", "all"])
    a = p.parse_args()
    if a.stage in ("holdout", "all"):
        holdout_stage(a)
    if a.stage in ("books", "all"):
        books_stage(a)


if __name__ == "__main__":
    sys.exit(main())
