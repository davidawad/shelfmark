#!/usr/bin/env python3
"""External, non-LLM relatedness signals for the false-join validity check.

Subcommands:

  openalex  For the 10,000 arXiv papers (bench/corpus/corpus.jsonl), fetch the
            OpenAlex work behind each arXiv DOI (10.48550/arXiv.<id>) with
            its referenced_works, then every work that cites a corpus paper
            (filter cites:...) with its referenced_works. OpenAlex data is
            CC0. Requests are batched with the pipe-separated filter (50
            values each), spaced at least 1.1 s apart, carry the mailto
            parameter, and are cached on disk under
            bench/corpus/cache/openalex/, so a rerun makes no request.
            Writes bench/results/2026-10-11-falsejoin/external/openalex.jsonl.gz:
            one row per paper with its OpenAlex id, references and citing works.
  lcsh      For the 3,000 books (bench/corpus/holdout-books.jsonl), the Library
            of Congress subject headings of each record, normalised; writes
            .../external/lcsh.jsonl.gz.

Pair signals derived from these files (see falsejoin_validity.py):

  bibliographic coupling  |refs(a) ∩ refs(b)| / sqrt(|refs(a)| |refs(b)|)
  co-citation             |citers(a) ∩ citers(b)| / sqrt(|citers(a)| |citers(b)|)
  shared LCSH             IDF-weighted cosine of the two heading sets

Only the Python standard library is used.
"""
import argparse
import gzip
import hashlib
import json
import re
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CACHE = ROOT / "bench/corpus/cache/openalex"
OUT = ROOT / "bench/results/2026-10-11-falsejoin/external"
API = "https://api.openalex.org/works"
MAILTO = "https://gitlab.com/davidawad/shelfmark"
BATCH = 50
SPACING = 1.1
_last = [0.0]


def jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


def write_gz_jsonl(path, rows):
    """Reproducible gzip: sorted keys, no file name or time stamp in the header."""
    path.parent.mkdir(parents=True, exist_ok=True)
    data = "".join(json.dumps(r, sort_keys=True, ensure_ascii=False) + "\n" for r in rows).encode()
    path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0))


def get(params):
    """One cached GET of the works endpoint; at most one request per SPACING seconds."""
    params = {**params, "mailto": MAILTO}
    url = f"{API}?{urllib.parse.urlencode(params)}"
    key = hashlib.sha256(url.encode()).hexdigest()[:32]
    path = CACHE / f"{key}.json"
    if path.exists():
        return json.loads(path.read_text())
    for attempt in range(6):
        wait = _last[0] + SPACING - time.monotonic()
        if wait > 0:
            time.sleep(wait)
        _last[0] = time.monotonic()
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": f"shelfmark-bench ({MAILTO})"}),
                                        timeout=120) as r:
                body = json.load(r)
            break
        except (urllib.error.URLError, TimeoutError) as e:
            code = getattr(e, "code", None)
            if code not in (None, 429, 500, 502, 503, 504) or attempt == 5:
                raise
            time.sleep(min(60, 5 * 2 ** attempt))
    CACHE.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"url": url.replace(MAILTO, "MAILTO"), "body": body}))
    return {"url": url, "body": body}


def short(openalex_id):
    return openalex_id.rsplit("/", 1)[-1]


def arxiv_ids_of(work, wanted):
    """The corpus arXiv ids a returned work stands for: its arXiv DOI, or an arXiv location."""
    hits = set()
    doi = (work.get("doi") or "").lower()
    m = re.search(r"10\.48550/arxiv\.(.+)$", doi)
    if m and m.group(1) in wanted:
        hits.add(m.group(1))
    for loc in work.get("locations") or []:
        for u in (loc.get("landing_page_url") or "", loc.get("pdf_url") or ""):
            m = re.search(r"arxiv\.org/(?:abs|pdf)/([^\s?#]+?)(?:v\d+)?(?:\.pdf)?$", u)
            if m and m.group(1).lower() in wanted:
                hits.add(m.group(1).lower())
    return hits


def openalex(_a):
    ids = [r["id"] for r in jsonl(ROOT / "bench/corpus/corpus.jsonl")]
    wanted = {i.lower(): i for i in ids}
    works = {}
    for start in range(0, len(ids), BATCH):
        chunk = ids[start:start + BATCH]
        f = "doi:" + "|".join(f"10.48550/arxiv.{i.lower()}" for i in chunk)
        body = get({"filter": f, "per-page": "200",
                    "select": "id,doi,locations,referenced_works,cited_by_count,publication_year"})["body"]
        for w in body["results"]:
            for aid in arxiv_ids_of(w, set(c.lower() for c in chunk)):
                prev = works.get(wanted[aid])
                # Several records for one paper: keep the one with most references.
                if prev is None or len(w["referenced_works"]) > len(prev["referenced_works"]):
                    works[wanted[aid]] = w
        if start // BATCH % 20 == 0:
            print(f"works: {start + len(chunk)}/{len(ids)} queried, {len(works)} found", flush=True)
    by_oa = {short(w["id"]): aid for aid, w in works.items()}
    cited = sorted(k for k, aid in by_oa.items() if works[aid]["cited_by_count"] > 0)
    citers = {aid: set() for aid in works}
    for start in range(0, len(cited), BATCH):
        chunk = cited[start:start + BATCH]
        members = set(chunk)
        cursor = "*"
        while cursor:
            body = get({"filter": "cites:" + "|".join(chunk), "per-page": "200", "cursor": cursor,
                        "select": "id,referenced_works"})["body"]
            for w in body["results"]:
                for ref in w["referenced_works"]:
                    if short(ref) in members:
                        citers[by_oa[short(ref)]].add(short(w["id"]))
            cursor = body["meta"].get("next_cursor") if body["results"] else None
        if start // BATCH % 10 == 0:
            print(f"citing works: {start + len(chunk)}/{len(cited)} cited papers queried", flush=True)
    rows = []
    for aid in ids:
        w = works.get(aid)
        rows.append({"id": aid, "openalex": short(w["id"]) if w else None,
                     "doi": w.get("doi") if w else None,
                     "publication_year": w.get("publication_year") if w else None,
                     "cited_by_count": w.get("cited_by_count") if w else None,
                     "refs": sorted(short(r) for r in w["referenced_works"]) if w else [],
                     "citers": sorted(citers.get(aid, ()))})
    write_gz_jsonl(OUT / "openalex.jsonl.gz", rows)
    print(json.dumps({"papers": len(ids), "found": len(works),
                      "with_refs": sum(bool(r["refs"]) for r in rows),
                      "cited": sum(bool(r["citers"]) for r in rows)}))


def headings(row):
    """Normalised LCSH strings of a book record: Open Library `subjects`, keeping
    headings written in LCSH form (not all-capitals BISAC-style terms)."""
    out = set()
    for s in row.get("subjects") or []:
        s = re.sub(r"\s+", " ", s).strip().rstrip(".")
        if not s or s.upper() == s:
            continue
        s = re.sub(r"\s*--\s*", " -- ", s)
        out.add(s.casefold())
    return sorted(out)


def lcsh(_a):
    rows = [{"id": r["id"], "headings": headings(r)} for r in jsonl(ROOT / "bench/corpus/holdout-books.jsonl")]
    write_gz_jsonl(OUT / "lcsh.jsonl.gz", rows)
    print(json.dumps({"books": len(rows), "with_headings": sum(bool(r["headings"]) for r in rows),
                      "distinct_headings": len({h for r in rows for h in r["headings"]})}))


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("step", choices=["openalex", "lcsh"])
    a = p.parse_args()
    {"openalex": openalex, "lcsh": lcsh}[a.step](a)


if __name__ == "__main__":
    main()
