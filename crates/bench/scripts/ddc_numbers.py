#!/usr/bin/env python3
"""Dewey numbers without the Dewey schedules.

`scheme` builds a scheme file whose classes are only the class numbers seen
in Open Library `dewey_decimal_class` values, with the hierarchy implied by
the notation: main class (one digit, `8` for 800-899), division (two
digits, `82`), section (three digits, `823`), then one level per further
decimal digit (`823.9`, `823.91`, `823.912`). Labels are the notation
itself (`8xx`, `82x`, `823`, `823.912`); no caption from any edition of the
Dewey Decimal Classification is used.

`gold` writes one placement per row: the first Dewey value that holds a
number, at that number's own class, or with `--level 3` at its section.

`fdc` writes the same numbers mapped into Free Decimal Correspondence: the
deepest FDC class whose number is a leading part of the Dewey number, or
whose range contains it. FDC follows Dewey numbering, so where its numbers
coincide this is the class a Dewey-classed library would shelve the book in.
"""
import argparse
import json
import re
from decimal import Decimal
from pathlib import Path

DDC = re.compile(r"\s*\[?\s*(?:[jJ]\s*)?(\d{3})((?:[./' ]*\d+)*)")


def ddc_number(value):
    """`813/.54` -> `813.54`; `016.3092 s` -> `016.3092`; `[Fic]`, `B`, `92` -> None.

    Prime marks and slashes are segmentation marks and are dropped, as is
    anything after the number (`s` for a series number) and trailing zeros
    of the decimal part. Must match harvest_openlibrary_ddc.ddc_number.
    """
    m = DDC.match(value)
    if not m:
        return None
    digits = re.sub(r"[./' ]", "", m.group(2)).rstrip("0")
    return f"{m.group(1)}.{digits}" if digits else m.group(1)


def first_number(row):
    for value in row.get("dewey_decimal_class") or []:
        number = ddc_number(value)
        if number:
            return number
    return None


def chain(number):
    """Codes from the main class down to `number`: 8, 82, 823, 823.9, 823.91."""
    head, _, decimals = number.partition(".")
    codes = [head[0], head[:2], head]
    codes += [f"{head}.{decimals[:i]}" for i in range(1, len(decimals) + 1)]
    return codes


def level(code):
    """1 main class, 2 division, 3 section, 4+ one per decimal digit."""
    return len(code.replace(".", ""))


def label(code):
    return code + "x" * (3 - len(code)) if len(code) < 3 else code


def read_jsonl(path):
    return [json.loads(line) for line in Path(path).open(encoding="utf-8") if line.strip()]


def write_jsonl(path, rows):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in rows), encoding="utf-8")


def ddc_rows(args):
    """Rows with `dewey_decimal_class`, joined by id from --labels when given."""
    rows = read_jsonl(args.corpus)
    if getattr(args, "labels", None):
        labels = {r["id"]: r for r in read_jsonl(args.labels)}
        for row in rows:
            row["dewey_decimal_class"] = labels.get(row["id"], {}).get("dewey_decimal_class", [])
    return rows


def scheme(args):
    codes = set()
    for corpus, labels in zip(args.corpus, args.labels + [None] * (len(args.corpus) - len(args.labels))):
        for row in ddc_rows(argparse.Namespace(corpus=corpus, labels=labels)):
            for value in row.get("dewey_decimal_class") or []:
                number = ddc_number(value)
                if number:
                    codes.update(chain(number))
    ordered = sorted(codes, key=lambda c: (c.replace(".", ""), len(c)))
    classes = []
    for code in ordered:
        cls = {"code": code, "label": label(code)}
        parent = code[:-1].rstrip(".") or None
        if parent:
            cls["parent"] = parent
        classes.append(cls)
    doc = {
        "name": "ddc",
        "version": "numbers-2026-10-10",
        "title": "Dewey Decimal Classification numbers (notation only, no schedule captions)",
        "source": {
            "description": "Class numbers from Open Library dewey_decimal_class values (CC0 1.0); "
                           "hierarchy implied by the decimal notation; labels are the notation itself. "
                           "No caption or other text of the Dewey Decimal Classification schedules is included. "
                           "Dewey and Dewey Decimal Classification are registered trademarks of OCLC, Inc.; "
                           "this file is not affiliated with or endorsed by OCLC.",
            "url": "https://openlibrary.org/developers/dumps",
            "license": "CC0 1.0 (Open Library catalog data); numbers only",
        },
        "code_grammar": "[0-9]{1,3}(\\.[0-9]+)?",
        "classes": classes,
    }
    Path(args.out).write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    levels = {}
    for c in classes:
        depth = level(c["code"])
        levels[depth] = levels.get(depth, 0) + 1
    print(json.dumps({"classes": len(classes), "per_level": levels, "output": args.out}))


def gold(args):
    rows = ddc_rows(args)
    out = []
    for row in rows:
        number = first_number(row)
        if number and args.level:
            number = chain(number)[min(args.level, level(number)) - 1]
        out.append({"id": row["id"], "classes": [number] if number else []})
    write_jsonl(args.output, out)
    placed = sum(bool(r["classes"]) for r in out)
    print(json.dumps({"rows": len(out), "rows_with_ddc_number": placed, "output": args.output}))


def fdc_index(path):
    doc = json.loads(Path(path).read_text(encoding="utf-8"))
    parents = {c["code"]: c.get("parent") for c in doc["classes"]}

    def depth(code):
        d = 0
        while code:
            d, code = d + 1, parents[code]
        return d

    return doc["classes"], {c["code"]: depth(c["code"]) for c in doc["classes"]}


def within(number, bound, upper):
    """`number` truncated to `bound`'s precision compared with `bound`."""
    head, _, dec = number.partition(".")
    places = len(bound.partition(".")[2])
    value = Decimal(head + ("." + dec[:places] if places and dec else ""))
    return value <= Decimal(bound) if upper else value >= Decimal(bound)


def fdc_class(number, classes, depths):
    """Deepest FDC class equal to a leading part of `number` or containing it."""
    prefixes = set(chain(number)[2:])
    best = None
    for c in classes:
        code, rng = c["code"], c.get("range")
        hit = code in prefixes
        if not hit and rng:
            hit = within(number, rng["start"], False) and within(number, rng["end"], True)
        if hit and (best is None or (depths[code], len(code)) > (depths[best], len(best))):
            best = code
    return best


def fdc(args):
    classes, depths = fdc_index(args.fdc)
    rows = ddc_rows(args)
    out, levels = [], {}
    for row in rows:
        number = first_number(row)
        code = fdc_class(number, classes, depths) if number else None
        out.append({"id": row["id"], "classes": [code] if code else []})
        if code:
            exact = "exact" if code == number else ("prefix" if code in chain(number) else "range")
            levels[exact] = levels.get(exact, 0) + 1
    write_jsonl(args.output, out)
    print(json.dumps({"rows": len(out), "rows_with_fdc_class": sum(bool(r["classes"]) for r in out),
                      "match": levels, "output": args.output}))


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="command", required=True)
    s = sub.add_parser("scheme")
    s.add_argument("--corpus", action="append", required=True)
    s.add_argument("--labels", action="append", default=[],
                   help="JSONL with id and dewey_decimal_class, joined to the matching --corpus")
    s.add_argument("--out", required=True)
    s.set_defaults(run=scheme)
    for name, fn in (("gold", gold), ("fdc", fdc)):
        q = sub.add_parser(name)
        q.add_argument("--corpus", required=True)
        q.add_argument("--labels")
        q.add_argument("--output", required=True)
        if name == "fdc":
            q.add_argument("--fdc", required=True)
        else:
            q.add_argument("--level", type=int, help="truncate to this level (3: the section)")
        q.set_defaults(run=fn)
    a = p.parse_args()
    a.run(a)


if __name__ == "__main__":
    main()
