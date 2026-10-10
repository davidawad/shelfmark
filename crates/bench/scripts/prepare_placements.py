#!/usr/bin/env python3
"""Prepare gold rows, shelfmark batch requests, and placement JSONL."""
import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

# The classification profile every placement request names.
PROFILE = "profile:lexical@2"


def read_jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


def write_jsonl(path, rows):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text("".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows), encoding="utf-8")


def classify_request(row, scheme, profile=PROFILE):
    text = row.get("abstract", row.get("abstract_text", row.get("description", "")))
    return {"verb": "classify", "args": {"title": row.get("title", ""), "text": text,
            "scheme": scheme, "profile": profile}}


def request_key(request):
    canonical = json.dumps(request, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    return "request:s256:" + hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def normalize_msc(args):
    valid = {c["code"] for c in json.loads(Path(args.scheme).read_text(encoding="utf-8"))["classes"]}
    rows = read_jsonl(args.corpus)
    rows_with_codes = 0
    for row in rows:
        values = row.pop("msc", row.get("msc_classes", []))
        codes = []
        for value in values:
            for code in re.findall(r"(?<![A-Za-z0-9])([0-9]{2}[A-Za-z][0-9]{2})(?![0-9])", value):
                code = code.upper()
                if code in valid and code not in codes:
                    codes.append(code)
        row["msc_classes"] = codes
        rows_with_codes += bool(codes)
    write_jsonl(args.output, rows)
    print(json.dumps({"rows": len(rows), "rows_with_valid_msc": rows_with_codes, "output": args.output}))


def normalize_lcc(args):
    valid = {c["code"] for c in json.loads(Path(args.scheme).read_text(encoding="utf-8"))["classes"]}
    rows = read_jsonl(args.corpus)
    placed = 0
    for row in rows:
        codes = []
        for value in row.get("lc_classifications", []):
            match = re.match(r"\s*([A-Z]{1,3})", value.upper())
            if match and match.group(1) in valid and match.group(1) not in codes:
                codes.append(match.group(1))
        row["lc_classifications"] = codes
        placed += bool(codes)
    write_jsonl(args.output, rows)
    print(json.dumps({"rows": len(rows), "rows_with_lcc_prefix": placed, "output": args.output}))


def lcc_candidates(value):
    """Codes to try for one LC call number, most specific first.

    Drops the edition year and anything after the first space, then tries
    the full call number, its class number without cutters, and its main
    class letters.
    """
    call = value.strip().upper().split(" ")[0]
    match = re.match(r"([A-Z]{1,3})(\d+(?:\.\d+)?)?", call)
    # MLCS/MLCM are LC microform shelf numbers, not classes.
    if not match or call.startswith("MLC"):
        return []
    return list(dict.fromkeys([call, match.group(0), match.group(1)]))


def check_codes(shelfmark, store, scheme_id, codes):
    """Deepest class covering each code (or None), from one CLI batch of code checks."""
    requests = [{"verb": "scheme-browse", "args": {"code": code, "scheme": scheme_id}}
                for code in codes]
    text = "".join(json.dumps(r) + "\n" for r in requests)
    proc = subprocess.run([shelfmark, "--store", store, "batch", "--requests", "-"],
                          input=text, capture_output=True, text=True, check=False)
    by_key = {}
    for line in proc.stdout.splitlines():
        line = json.loads(line)
        if "input" in line:
            by_key[line["input"]] = line
    resolved = {}
    for code, request in zip(codes, requests):
        result = by_key[request_key(request)]
        check = result.get("error", {}).get("details", {}).get("check", {})
        if result.get("ok"):
            resolved[code] = code
        elif check.get("status") in ("exact", "within"):
            resolved[code] = check["class"].rsplit(":", 1)[-1]
        else:
            resolved[code] = None
    return resolved


def normalize_lcc_deep(args):
    """Place each LC call number at the deepest covering class of the compiled scheme."""
    rows = read_jsonl(args.corpus)
    wanted = sorted({c for row in rows for v in row.get("lc_classifications", [])
                     for c in lcc_candidates(v)})
    resolved = check_codes(args.shelfmark, args.store, args.scheme_id, wanted)
    placed = 0
    for row in rows:
        codes = []
        for value in row.get("lc_classifications", []):
            code = next(filter(None, (resolved[c] for c in lcc_candidates(value))), None)
            if code and code not in codes:
                codes.append(code)
        # The harness reads `lc_classifications` for a scheme named lcc and
        # `gold[<name>]` for any other scheme name.
        if args.gold_key:
            row.setdefault("gold", {})[args.gold_key] = codes
        else:
            row["lc_classifications"] = codes
        placed += bool(codes)
    write_jsonl(args.output, rows)
    print(json.dumps({"rows": len(rows), "rows_with_lcc_class": placed, "output": args.output}))


def normalize_codes(args):
    """Keep only labels in `field` that are exact codes of the scheme."""
    valid = {c["code"] for c in json.loads(Path(args.scheme).read_text(encoding="utf-8"))["classes"]}
    rows = read_jsonl(args.corpus)
    placed = 0
    for row in rows:
        codes = []
        for value in row.get(args.field) or []:
            if value in valid and value not in codes:
                codes.append(value)
        row[args.field] = codes
        placed += bool(codes)
    write_jsonl(args.output, rows)
    print(json.dumps({"rows": len(rows), "field": args.field, "rows_with_valid_codes": placed,
                      "output": args.output}))


def requests(args):
    rows = read_jsonl(args.corpus)
    batch = [classify_request(row, args.scheme, args.profile) for row in rows]
    write_jsonl(args.output, batch)
    print(json.dumps({"requests": len(batch), "scheme": args.scheme, "output": args.output}))


def extract(args):
    corpus = read_jsonl(args.corpus)
    results = read_jsonl(args.batch)
    result_by_key = {result.get("input"): result for result in results}
    rows = []
    failures = 0
    flagged = 0
    fallback = 0
    tiers = {}
    per_class = {}
    terms = {}
    for item in corpus:
        key = request_key(classify_request(item, args.scheme, args.profile))
        result = result_by_key.get(key, {})
        placement = result.get("data", {}).get("placement")
        if result.get("ok") is not True or not placement:
            failures += 1
            classes = []
        elif not placement.get("trace", {}).get("candidates"):
            # No tier produced a candidate, so shelfmark fell back to the
            # first top-level class; that is not a classification.
            fallback += 1
            classes = []
        else:
            class_id = placement["class"]
            classes = [class_id.rsplit(":", 1)[-1]]
            flagged += placement.get("status") == "flagged"
            trace = placement["trace"]
            tier = trace["deciding_tier"] or max(trace["candidates"][0]["per_tier"],
                                                 key=lambda t: t["score"])["tier"]
            tiers[tier] = tiers.get(tier, 0) + 1
            per_class[classes[0]] = per_class.get(classes[0], 0) + 1
            for candidate in trace["candidates"]:
                if candidate["class"] == class_id:
                    for evidence in candidate.get("evidence", []):
                        if "term" in evidence:
                            counts = terms.setdefault(classes[0], {})
                            counts[evidence["term"]] = counts.get(evidence["term"], 0) + 1
        rows.append({"id": item["id"], "classes": classes})
    write_jsonl(args.output, rows)
    placed = len(rows) - failures - fallback
    # The most used class and the terms that matched it expose a classifier
    # that files many items by one common word.
    top = max(per_class, key=lambda c: (per_class[c], c)) if per_class else None
    top_terms = sorted(terms.get(top, {}).items(), key=lambda kv: (-kv[1], kv[0]))[:5]
    print(json.dumps({"rows": len(rows), "failed": failures, "fallback": fallback,
                      "placed": placed, "flagged": flagged, "deciding_tiers": tiers,
                      "classes_used": len(per_class), "top_class": top,
                      "top_class_share": per_class[top] / placed if top else 0.0,
                      "top_class_terms": dict(top_terms), "output": args.output}))


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="command", required=True)
    msc = sub.add_parser("normalize-msc")
    msc.add_argument("--corpus", required=True)
    msc.add_argument("--scheme", required=True)
    msc.add_argument("--output", required=True)
    msc.set_defaults(run=normalize_msc)
    lcc = sub.add_parser("normalize-lcc")
    lcc.add_argument("--corpus", required=True)
    lcc.add_argument("--scheme", required=True)
    lcc.add_argument("--output", required=True)
    lcc.set_defaults(run=normalize_lcc)
    deep = sub.add_parser("normalize-lcc-deep")
    deep.add_argument("--corpus", required=True)
    deep.add_argument("--shelfmark", required=True)
    deep.add_argument("--store", required=True, help="store with the LCC scheme compiled")
    deep.add_argument("--scheme-id", default="scheme:lcc@2024")
    deep.add_argument("--gold-key", help="write codes to gold[KEY] instead of lc_classifications")
    deep.add_argument("--output", required=True)
    deep.set_defaults(run=normalize_lcc_deep)
    codes = sub.add_parser("normalize-codes")
    codes.add_argument("--corpus", required=True)
    codes.add_argument("--scheme", required=True)
    codes.add_argument("--field", required=True)
    codes.add_argument("--output", required=True)
    codes.set_defaults(run=normalize_codes)
    req = sub.add_parser("requests")
    req.add_argument("--corpus", required=True)
    req.add_argument("--scheme", required=True)
    req.add_argument("--profile", default=PROFILE)
    req.add_argument("--output", required=True)
    req.set_defaults(run=requests)
    out = sub.add_parser("extract")
    out.add_argument("--corpus", required=True)
    out.add_argument("--batch", required=True)
    out.add_argument("--scheme", required=True)
    out.add_argument("--profile", default=PROFILE)
    out.add_argument("--output", required=True)
    out.set_defaults(run=extract)
    args = p.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
