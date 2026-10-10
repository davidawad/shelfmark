#!/usr/bin/env python3
"""Prepare gold rows, shelfmark batch requests, and placement JSONL."""
import argparse
import hashlib
import json
import re
from pathlib import Path


def read_jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


def write_jsonl(path, rows):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text("".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows), encoding="utf-8")


def classify_request(row, scheme):
    text = row.get("abstract", row.get("abstract_text", row.get("description", "")))
    return {"verb": "classify", "args": {"title": row.get("title", ""), "text": text,
            "scheme": scheme, "profile": "profile:lexical@1"}}


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


def requests(args):
    rows = read_jsonl(args.corpus)
    batch = [classify_request(row, args.scheme) for row in rows]
    write_jsonl(args.output, batch)
    print(json.dumps({"requests": len(batch), "scheme": args.scheme, "output": args.output}))


def extract(args):
    corpus = read_jsonl(args.corpus)
    results = read_jsonl(args.batch)
    result_by_key = {result.get("input"): result for result in results}
    rows = []
    failures = 0
    flagged = 0
    for item in corpus:
        key = request_key(classify_request(item, args.scheme))
        result = result_by_key.get(key, {})
        placement = result.get("data", {}).get("placement")
        if result.get("ok") is not True or not placement:
            failures += 1
            classes = []
        else:
            class_id = placement["class"]
            classes = [class_id.rsplit(":", 1)[-1]]
            flagged += placement.get("status") == "flagged"
        rows.append({"id": item["id"], "classes": classes})
    write_jsonl(args.output, rows)
    print(json.dumps({"rows": len(rows), "failed": failures, "flagged": flagged, "output": args.output}))


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
    req = sub.add_parser("requests")
    req.add_argument("--corpus", required=True)
    req.add_argument("--scheme", required=True)
    req.add_argument("--output", required=True)
    req.set_defaults(run=requests)
    out = sub.add_parser("extract")
    out.add_argument("--corpus", required=True)
    out.add_argument("--batch", required=True)
    out.add_argument("--scheme", required=True)
    out.add_argument("--output", required=True)
    out.set_defaults(run=extract)
    args = p.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
