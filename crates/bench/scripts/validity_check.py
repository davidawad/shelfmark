#!/usr/bin/env python3
"""Librarian-side validity check of the Dasgupta-cost metric (docs/validity/).

Subcommands, all deterministic for fixed inputs:

  sample  Build each run's induced tree from the regenerated placements (as
          the harness does), check its Dasgupta cost_norm against the harness
          result, and draw a stratified, seeded sample of item pairs per run:
          A  high-similarity graph edges that the scheme splits (LCA at the
             root or at depth 1),
          B  zero-weight pairs (cosine at or below the graph's tau) that the
             scheme joins deep (LCA depth >= the run's deep threshold),
          C  uniform random pairs of placed items.
          Writes docs/validity/sample.jsonl (keeping any labels already there
          for the same pair ids) and, with --check, only verifies that the
          committed pair ids and metric fields are reproduced.
  blind   Write the blinded judging batches (stratum, similarity and cost
          hidden; pairs shuffled; work order randomized) and the private
          uid -> pair key, for the first pass and a seeded 25% second pass.
  merge   Fold the judge label files back into sample.jsonl.
  report  Agreement between judge labels and the metric's cost contribution,
          with bootstrap intervals; writes docs/validity/results.md and
          results.json.
  human   Write the human-librarian packet (docs/validity/human/): a seeded
          subset of the sample, HUMAN_PER_CELL pairs per run and stratum,
          blinded and shuffled as for the judge, as packet.md and a blank
          labels.csv. Needs only the eight scheme files (committed ones are
          used when the cache has none); each is checked against
          sample-meta.json. The uid is a hash of the pair id, so no key file
          is written.
  human-report  Agreement of the filled labels.csv with the metric and with
          the LLM judge's pass-1 labels; writes docs/validity/human/results.md
          and results.json.

Inputs are the ignored cache of run_benchmark.py / learned_benchmark.py and
the embedding vectors of build_graph_fastembed.py --vectors (see
docs/validity/README.md). Only the Python standard library is used.
"""
import argparse
import gzip
import hashlib
import json
import math
import random
import re
from array import array
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / "docs/validity"
CORPUS = ROOT / "bench/corpus/corpus.jsonl"
SEED = 20261010
PER_STRATUM = 40
STRATA = ["A", "B", "C"]
REPEAT_SHARE = 0.25
BATCH = 40
BOOT = 2000
ABSTRACT_CHARS = 900
LABEL_CHARS = 90
# run key -> (scheme name, scheme file, placement source). Placement source is
# a placements.jsonl (cache-relative) or ("gold", corpus field) for gold runs,
# which place each item at its first source label as the harness does.
RUNS = {
    "lcc-full-placed": ("lcc-full", "schemes/lcc-full.json", "placed/corpus-lcc-full/placements.jsonl"),
    "fdc-placed": ("fdc", "schemes/fdc.json", "placed/corpus-fdc/placements.jsonl"),
    "ilc2-placed": ("ilc2", "schemes/ilc2.json", "placed/corpus-ilc2/placements.jsonl"),
    "unesco-thesaurus-placed": ("unesco-thesaurus", "schemes/unesco-thesaurus.json",
                                "placed/corpus-unesco-thesaurus/placements.jsonl"),
    "openalex-topics-placed": ("openalex-topics", "schemes/openalex-topics.json",
                               "placed/corpus-openalex-topics/placements.jsonl"),
    "arxiv-gold": ("arxiv", "schemes/arxiv.json", ("gold", "gold/corpus-arxiv.jsonl", "categories")),
    "msc-gold": ("msc", "schemes/msc.json", ("gold", "gold/corpus-msc.jsonl", "msc_classes")),
    "learned-placed": ("learned", "learned-bench/learned.json", "placed/corpus-learned/placements.jsonl"),
}
HARNESS = {k: f"runs/{k}.json" for k in RUNS}
HARNESS["learned-placed"] = "learned-bench/runs/learned-placed.json"
METRIC_FIELDS = ["lca_depth", "lca_mass", "deep_threshold", "w_sparse", "cos", "w_dense",
                 "contribution", "contribution_dense", "mean_edge_contribution",
                 "metric_together", "metric_flag", "metric_flag_dense"]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def jsonl(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


def write_jsonl(path, rows):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text("".join(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n" for r in rows),
                          encoding="utf-8")


def rng_for(*parts):
    return random.Random(int(hashlib.sha256(":".join(map(str, (SEED,) + parts)).encode()).hexdigest()[:16], 16))


def pair_id(run, a, b):
    a, b = sorted([a, b])
    return f"{run}:{a}|{b}"


class Run:
    """One run's induced tree, in the harness's terms: the virtual root has
    depth 0, scheme top classes depth 1; unplaced items hang off the root."""

    def __init__(self, cache, key, ids):
        self.key = key
        name, scheme_file, source = RUNS[key]
        self.scheme_path = cache / scheme_file
        doc = json.loads(self.scheme_path.read_text(encoding="utf-8"))
        self.title = doc.get("title", name)
        self.classes = {c["code"]: c for c in doc["classes"]}
        index = {i: n for n, i in enumerate(ids)}
        self.code = [None] * len(ids)
        if isinstance(source, tuple):
            _, gold_file, field = source
            self.source_hash = sha256(cache / gold_file)
            for row in jsonl(cache / gold_file):
                labels = row.get(field) or []
                if labels:
                    self.code[index[row["id"]]] = labels[0]
        else:
            self.source_hash = sha256(cache / source)
            for row in jsonl(cache / source):
                classes = row["classes"]
                if not classes:
                    continue
                if len(classes) != 1:
                    raise SystemExit(f"{key}: fractional placement for {row['id']} is not supported")
                c = classes[0]
                self.code[index[row["id"]]] = c if isinstance(c, str) else c["code"]
        self.chain = {}
        for code in {c for c in self.code if c}:
            self.chain[code] = self.ancestors(code)
        self.n = len(ids)
        self.mass = {}
        for code in self.code:
            if code:
                for node in self.chain[code]:
                    self.mass[node] = self.mass.get(node, 0) + 1
        self.placed = [i for i, c in enumerate(self.code) if c]
        depths = sorted(len(self.chain[self.code[i]]) for i in self.placed)
        # Deep threshold: the depth of the median placed item's class.
        self.deep = max(2, depths[len(depths) // 2])

    def ancestors(self, code):
        out = []
        while code:
            if code not in self.classes:
                raise SystemExit(f"{self.key}: unknown class {code}")
            out.append(code)
            code = self.classes[code].get("parent")
        return tuple(reversed(out))

    def lca(self, i, j):
        """(depth, code, mass) of the LCA of items i and j."""
        a, b = self.code[i], self.code[j]
        if not a or not b:
            return 0, None, self.n
        ca, cb = self.chain[a], self.chain[b]
        d = 0
        while d < min(len(ca), len(cb)) and ca[d] == cb[d]:
            d += 1
        return (d, ca[d - 1], self.mass[ca[d - 1]]) if d else (0, None, self.n)

    def deep_joined(self, i, j):
        """The scheme joins i and j deep: their LCA is at depth >= 2 and at or
        below the run's deep threshold, or is the class of either item."""
        d = self.lca(i, j)[0]
        a, b = len(self.chain[self.code[i]]), len(self.chain[self.code[j]])
        return d >= 2 and d >= min(self.deep, a, b)

    def path(self, i):
        return [f"{c} — {self.classes[c].get('label', '')[:LABEL_CHARS]}" for c in self.chain[self.code[i]]]


class Inputs:
    def __init__(self, cache):
        self.cache = Path(cache)
        self.rows = jsonl(CORPUS)
        self.ids = [r["id"] for r in self.rows]
        graph_path = self.cache / "graphs/corpus-k20.json"
        manifest = json.loads((self.cache / "graphs/corpus-k20.manifest.json").read_text())
        self.graph_sha = sha256(graph_path)
        if self.graph_sha != manifest["graph_sha256"]:
            raise SystemExit(f"{graph_path}: sha256 does not match its manifest")
        graph = json.loads(graph_path.read_text())
        self.tau = graph["tau"]
        self.edges = {(e["left"], e["right"]): e["weight"] for e in graph["edges"]}
        weights = sorted(self.edges.values())
        self.median_w = weights[len(weights) // 2]
        vec_path = self.cache / "embed/corpus.f32"
        self.vec_sha = sha256(vec_path)
        self.vec = array("f")
        with open(vec_path, "rb") as f:
            self.vec.frombytes(f.read())
        self.dim = len(self.vec) // len(self.ids)

    def w(self, i, j):
        return self.edges.get((min(i, j), max(i, j)), 0.0)

    def cos(self, i, j):
        v, d = self.vec, self.dim
        return math.fsum(v[i * d + k] * v[j * d + k] for k in range(d))


def metric(inp, run, i, j, mean_edge):
    depth, code, mass = run.lca(i, j)
    w = inp.w(i, j)
    cos = inp.cos(i, j)
    wd = max(0.0, cos - inp.tau)
    return {
        "lca_depth": depth, "lca_code": code, "lca_mass": mass, "deep_threshold": run.deep,
        "deep_joined": run.deep_joined(i, j),
        "w_sparse": round(w, 6), "cos": round(cos, 4), "w_dense": round(wd, 4),
        "contribution": round(w * mass / run.n, 8), "contribution_dense": round(wd * mass / run.n, 8),
        "mean_edge_contribution": round(mean_edge, 8),
        "metric_together": w > 0,
        "metric_flag": w * mass / run.n > mean_edge,
        "metric_flag_dense": wd * mass / run.n > mean_edge,
    }


def draw(inp, run):
    """40 pairs per stratum for one run, keyed by pair id."""
    placed = set(run.placed)
    rng = rng_for(run.key, "sample")
    out = {}
    # A: graph edges at or above the median edge weight, split at depth <= 1.
    pool = sorted((i, j) for (i, j), w in inp.edges.items()
                  if w >= inp.median_w and i in placed and j in placed and run.lca(i, j)[0] <= 1)
    a = rng.sample(pool, min(PER_STRATUM, len(pool)))
    # B: uniform over deep-joined pairs, conditioned on cos <= tau. Pairs
    # meeting at depth >= 2 are drawn uniformly (group by depth-2 ancestor,
    # weight C(m, 2)) and kept when deep and dissimilar. Small populations
    # are enumerated exhaustively instead.
    groups = {}
    for i in run.placed:
        chain = run.chain[run.code[i]]
        if len(chain) >= 2:
            groups.setdefault(chain[1], []).append(i)
    groups = sorted((k, v) for k, v in groups.items() if len(v) > 1)
    weights = [len(v) * (len(v) - 1) // 2 for _, v in groups]
    b, tries = [], 0
    if sum(weights) <= 300000:
        population = [(i, j) for _, v in groups for x, i in enumerate(v) for j in v[x + 1:]
                      if run.deep_joined(i, j) and inp.cos(i, j) <= inp.tau]
        tries = sum(weights)
        b = rng.sample(sorted(population), min(PER_STRATUM, len(population)))
    else:
        seen = set()
        while len(b) < PER_STRATUM and tries < 500000:
            tries += 1
            members = rng.choices(groups, weights)[0][1]
            i, j = sorted(rng.sample(members, 2))
            if (i, j) not in seen and run.deep_joined(i, j) and inp.cos(i, j) <= inp.tau:
                seen.add((i, j))
                b.append((i, j))
    # C: uniform random pairs of placed items.
    c, seen = [], set()
    while len(c) < PER_STRATUM:
        i, j = sorted(rng.sample(run.placed, 2))
        if (i, j) not in seen:
            seen.add((i, j))
            c.append((i, j))
    for stratum, pairs in zip(STRATA, (a, b, c)):
        for i, j in pairs:
            out[pair_id(run.key, inp.ids[i], inp.ids[j])] = (stratum, i, j)
    return out, {"A_pool": len(pool), "B_found": len(b), "B_tries": tries}


def sample(a):
    inp = Inputs(a.cache)
    previous = {r["pair_id"]: r for r in jsonl(OUT / "sample.jsonl")} if (OUT / "sample.jsonl").exists() else {}
    rows, runs_meta = [], {}
    for key in RUNS:
        run = Run(inp.cache, key, inp.ids)
        harness = json.loads((inp.cache / HARNESS[key]).read_text())
        # Recompute the run's sparse Dasgupta cost and check it against the harness.
        cost = weight = 0.0
        for (i, j), w in inp.edges.items():
            cost += w * run.lca(i, j)[2]
            weight += w
        cost_norm = cost / (run.n * weight)
        if abs(cost_norm - harness["cost_norm"]) > 1e-9:
            raise SystemExit(f"{key}: cost_norm {cost_norm} != harness {harness['cost_norm']}")
        mean_edge = cost / run.n / len(inp.edges)
        pairs, info = draw(inp, run)
        runs_meta[key] = {"scheme": RUNS[key][0], "scheme_title": run.title,
                          "scheme_sha256": sha256(run.scheme_path), "placements_sha256": run.source_hash,
                          "harness_tree_sha256": harness["hashes"]["tree"], "cost_norm": cost_norm,
                          "coverage": len(run.placed) / run.n, "deep_threshold": run.deep,
                          "mean_edge_contribution": mean_edge, **info}
        print(f"{key}: cost_norm {cost_norm:.6f} (harness ok), deep>={run.deep}, {info}", flush=True)
        for pid, (stratum, i, j) in pairs.items():
            x, y = (i, j) if inp.ids[i] < inp.ids[j] else (j, i)
            row = {"pair_id": pid, "run": key, "scheme": RUNS[key][0], "stratum": stratum,
                   "id_a": inp.ids[x], "id_b": inp.ids[y],
                   "class_a": run.code[x], "class_b": run.code[y]}
            row.update(metric(inp, run, x, y, mean_edge))
            if pid in previous and "labels" in previous[pid]:
                row["labels"] = previous[pid]["labels"]
            rows.append(row)
    meta = {"seed": SEED, "per_stratum": PER_STRATUM, "corpus_sha256": sha256(CORPUS),
            "graph_sha256": inp.graph_sha, "vectors_sha256": inp.vec_sha, "tau": inp.tau,
            "median_edge_weight": inp.median_w, "runs": runs_meta}
    if a.check:
        old = jsonl(OUT / "sample.jsonl")
        strip = [{k: r[k] for k in ["pair_id", "stratum", "class_a", "class_b", "lca_depth", "lca_mass",
                                     "w_sparse", "contribution", "metric_flag", "metric_together"]}
                 for r in old]
        new = [{k: r[k] for k in strip[0]} for r in rows]
        if strip != new:
            diff = sum(x != y for x, y in zip(strip, new)) + abs(len(strip) - len(new))
            raise SystemExit(f"sample.jsonl differs from the regenerated sample in {diff} rows")
        print(f"sample.jsonl reproduced: {len(rows)} pairs")
        return
    write_jsonl(OUT / "sample.jsonl", rows)
    (OUT / "sample-meta.json").write_text(json.dumps(meta, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {len(rows)} pairs")


def blind(a):
    rows = jsonl(OUT / "sample.jsonl")
    meta = json.loads((OUT / "sample-meta.json").read_text())
    corpus = {r["id"]: r for r in jsonl(CORPUS)}
    ids = list(corpus)
    index = {i: n for n, i in enumerate(ids)}
    cache = Path(a.cache)
    runs = {}
    out = Path(a.out)
    key = []
    repeat = {r["pair_id"] for r in rng_for("repeat").sample(rows, round(REPEAT_SHARE * len(rows)))}
    for p in (1, 2):
        chosen = [r for r in rows if p == 1 or r["pair_id"] in repeat]
        rng = rng_for("blind", p)
        rng.shuffle(chosen)
        items = []
        for r in chosen:
            if r["run"] not in runs:
                runs[r["run"]] = Run(cache, r["run"], ids)
            run = runs[r["run"]]
            uid = hashlib.sha256(f"{SEED}:{p}:{r['pair_id']}".encode()).hexdigest()[:10]
            ia, ib = index[r["id_a"]], index[r["id_b"]]
            works = [(r["id_a"], ia), (r["id_b"], ib)]
            if rng.random() < 0.5:
                works.reverse()
            item = {"uid": uid, "scheme": meta["runs"][r["run"]]["scheme_title"]}
            for n, (wid, idx) in enumerate(works, start=1):
                row = corpus[wid]
                text = re.sub(r"\s+", " ", row.get("abstract_text") or row.get("description") or "").strip()
                if len(text) > ABSTRACT_CHARS:
                    text = text[:ABSTRACT_CHARS].rsplit(" ", 1)[0] + " …"
                item[f"work_{n}"] = {"title": re.sub(r"\s+", " ", row["title"]).strip(), "abstract": text}
                item[f"class_path_{n}"] = run.path(idx)
            items.append(item)
            key.append({"uid": uid, "pass": p, "pair_id": r["pair_id"], "work_1": works[0][0]})
        for b in range(0, len(items), BATCH):
            write_jsonl(out / f"pass{p}-batch{b // BATCH + 1:02d}.jsonl", items[b:b + BATCH])
    write_jsonl(cache / "validity/key.jsonl", key)
    print(f"pass 1: {len(rows)} pairs, pass 2: {len(repeat)} pairs, batches in {out}")


LABELS = {"relation": {"same-topic", "same-field", "different-field"},
          "together": {"together", "apart"},
          "scheme_decision": {"correct", "incorrect"},
          "class_1_ok": {"yes", "no"}, "class_2_ok": {"yes", "no"}}


def merge(a):
    key = {k["uid"]: k for k in jsonl(Path(a.cache) / "validity/key.jsonl")}
    labels = {}
    for path in sorted(Path(a.labels).glob("*.jsonl")):
        for lab in jsonl(path):
            k = key[lab["uid"]]
            for field, allowed in LABELS.items():
                if lab.get(field) not in allowed:
                    raise SystemExit(f"{path}: {lab['uid']} has invalid {field}={lab.get(field)!r}")
            # Works were shown in random order; store the class checks per item.
            first, second = ("a", "b") if k["pair_id"].split(":", 1)[1].startswith(k["work_1"] + "|") \
                else ("b", "a")
            out = {f: lab[f] for f in ["relation", "together", "scheme_decision", "reason"] if f in lab}
            out[f"class_{first}_ok"], out[f"class_{second}_ok"] = lab["class_1_ok"], lab["class_2_ok"]
            labels.setdefault(k["pair_id"], {})[f"pass{k['pass']}"] = out
    missing = [k for k in key.values() if f"pass{k['pass']}" not in labels.get(k["pair_id"], {})]
    if missing:
        raise SystemExit(f"{len(missing)} blinded pairs have no label, e.g. {missing[0]}")
    rows = jsonl(OUT / "sample.jsonl")
    for r in rows:
        r["labels"] = labels[r["pair_id"]]
    write_jsonl(OUT / "sample.jsonl", rows)
    print(f"merged labels for {len(labels)} pairs")


def kappa(x, y):
    n = len(x)
    if not n:
        return None
    cats = sorted(set(x) | set(y))
    po = sum(a == b for a, b in zip(x, y)) / n
    pe = sum((x.count(c) / n) * (y.count(c) / n) for c in cats)
    return None if pe >= 1 else (po - pe) / (1 - pe)


def auc(scores, positive):
    pos = [s for s, p in zip(scores, positive) if p]
    neg = [s for s, p in zip(scores, positive) if not p]
    if not pos or not neg:
        return None
    wins = sum((p > q) + 0.5 * (p == q) for p in pos for q in neg)
    return wins / (len(pos) * len(neg))


def spearman(x, y):
    def ranks(v):
        order = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0] * len(v)
        i = 0
        while i < len(v):
            j = i
            while j + 1 < len(v) and v[order[j + 1]] == v[order[i]]:
                j += 1
            for k in range(i, j + 1):
                r[order[k]] = (i + j) / 2
            i = j + 1
        return r
    rx, ry = ranks(x), ranks(y)
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = math.sqrt(sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry))
    return num / den if den else None


def stats(rows):
    """Agreement of the judge's scheme verdict with the metric's flag, and of
    the judge's together call with the metric's (graph edge) together call."""
    judge_bad = [r["labels"]["pass1"]["scheme_decision"] == "incorrect" for r in rows]
    flag = [r["metric_flag"] for r in rows]
    flag_d = [r["metric_flag_dense"] for r in rows]
    jt = [r["labels"]["pass1"]["together"] == "together" for r in rows]
    mt = [r["metric_together"] for r in rows]
    joined = [r["lca_depth"] >= 2 for r in rows]
    edge_jt = [t for t, r in zip(jt, rows) if r["metric_together"]]
    n = len(rows)
    return {
        "n": n,
        "judge_incorrect": sum(judge_bad) / n,
        "metric_flagged": sum(flag) / n,
        "agree": sum(a == b for a, b in zip(judge_bad, flag)) / n,
        "kappa": kappa(judge_bad, flag),
        "agree_dense": sum(a == b for a, b in zip(judge_bad, flag_d)) / n,
        "kappa_dense": kappa(judge_bad, flag_d),
        "auc": auc([r["contribution"] for r in rows], judge_bad),
        "auc_dense": auc([r["contribution_dense"] for r in rows], judge_bad),
        "together_agree": sum(a == b for a, b in zip(jt, mt)) / n,
        "together_kappa": kappa(jt, mt),
        "judge_together": sum(jt) / n,
        "scheme_vs_judge_together_kappa": kappa(jt, joined),
        # Judge-incorrect pairs by direction: split at depth <= 1 or joined below it.
        "false_split": sum(b and r["lca_depth"] <= 1 for b, r in zip(judge_bad, rows)) / n,
        "false_join": sum(b and r["lca_depth"] >= 2 for b, r in zip(judge_bad, rows)) / n,
        "edge_n": len(edge_jt),
        "edge_judged_together": sum(edge_jt) / len(edge_jt) if edge_jt else None,
    }


def boot(rows, fields, reps=BOOT, tag=""):
    """Percentile 95% intervals, resampling pairs within each stratum."""
    by = {}
    for r in rows:
        by.setdefault(r["stratum"], []).append(r)
    rng = rng_for("boot", tag)
    draws = {f: [] for f in fields}
    for _ in range(reps):
        sample_rows = [rng.choice(v) for s in sorted(by) for v in [by[s]] for _ in v]
        st = stats(sample_rows)
        for f in fields:
            if st[f] is not None:
                draws[f].append(st[f])
    out = {}
    for f, v in draws.items():
        v.sort()
        out[f] = [v[int(0.025 * len(v))], v[min(len(v) - 1, int(0.975 * len(v)))]] if v else None
    return out


def fmt(v, d=2):
    return "–" if v is None else f"{v:.{d}f}"


def ci(v, c, d=2):
    return f"{fmt(v, d)} [{fmt(c[0], d)}, {fmt(c[1], d)}]" if c else fmt(v, d)


def report(a):
    rows = jsonl(OUT / "sample.jsonl")
    meta = json.loads((OUT / "sample-meta.json").read_text())
    if any("pass1" not in r.get("labels", {}) for r in rows):
        raise SystemExit("sample.jsonl has unlabelled pairs; run merge first")
    res = {"runs": {}, "strata": {}, "consistency": {}}
    for key in RUNS:
        rr = [r for r in rows if r["run"] == key]
        st = stats(rr)
        st["ci"] = boot(rr, ["agree", "kappa", "auc", "together_kappa"], tag=key)
        st["by_stratum"] = {s: stats([r for r in rr if r["stratum"] == s]) for s in STRATA}
        res["runs"][key] = st
    allst = stats(rows)
    allst["ci"] = boot(rows, ["agree", "kappa", "auc", "together_kappa"], tag="all")
    res["all"] = allst
    res["strata"] = {s: stats([r for r in rows if r["stratum"] == s]) for s in STRATA}
    rep = [r for r in rows if "pass2" in r["labels"]]
    for f in ["scheme_decision", "together", "relation"]:
        x = [r["labels"]["pass1"][f] for r in rep]
        y = [r["labels"]["pass2"][f] for r in rep]
        res["consistency"][f] = {"n": len(rep), "agree": sum(p == q for p, q in zip(x, y)) / len(rep),
                                 "kappa": kappa(x, y)}
    rng = rng_for("boot", "consistency")
    kb = []
    for _ in range(BOOT):
        s = [rng.choice(rep) for _ in rep]
        k = kappa([r["labels"]["pass1"]["scheme_decision"] for r in s],
                  [r["labels"]["pass2"]["scheme_decision"] for r in s])
        if k is not None:
            kb.append(k)
    kb.sort()
    res["consistency"]["scheme_decision"]["ci"] = [kb[int(0.025 * len(kb))], kb[min(len(kb) - 1, int(0.975 * len(kb)))]]
    # Placement errors: how often the judge rejects an assigned class.
    for key in RUNS:
        rr = [r for r in rows if r["run"] == key]
        bad = sum((r["labels"]["pass1"]["class_a_ok"] == "no") + (r["labels"]["pass1"]["class_b_ok"] == "no")
                  for r in rr)
        res["runs"][key]["class_rejected"] = bad / (2 * len(rr))

    L = ["# Validity check results", "",
         ("Generated by `crates/bench/scripts/validity_check.py report` from `sample.jsonl`. "
         "Method and caveats: `README.md`. Brackets are 95% percentile bootstrap intervals "
         f"({BOOT} replicates, pairs resampled within strata)."), "",
         "## Judge verdict on the scheme's decision against the metric's flag", "",
         ("The metric flags a pair when its cost contribution `w·|leaves(LCA)|/n` exceeds the run's "
         "mean contribution per graph edge. Positive class: judge says the decision is incorrect / "
         "metric flags it."), "",
         ("| run | n | judge: incorrect | metric: flagged | agreement | Cohen κ | AUC (contribution) | "
         "dense κ | class rejected |"),
         "|---|---|---|---|---|---|---|---|---|"]
    for key, st in list(res["runs"].items()) + [("all runs", res["all"])]:
        c = st["ci"]
        L.append(f"| {key} | {st['n']} | {fmt(st['judge_incorrect'])} | {fmt(st['metric_flagged'])} | "
                 f"{ci(st['agree'], c['agree'])} | {ci(st['kappa'], c['kappa'])} | {ci(st['auc'], c['auc'])} | "
                 f"{fmt(st['kappa_dense'])} | {fmt(st.get('class_rejected'))} |")
    L += ["", "## Per stratum", "",
          ("A: similar pairs split at depth ≤ 1. B: zero-weight pairs joined deep. C: random placed pairs. "
          "Cells: judge-incorrect share / metric-flagged share / agreement / κ "
          "(κ is undefined, –, when either side is constant within the stratum)."), "",
          "| run | A | B | C |", "|---|---|---|---|"]
    for key, st in list(res["runs"].items()) + [("all runs", {"by_stratum": res["strata"]})]:
        cells = [f"{fmt(s['judge_incorrect'])} / {fmt(s['metric_flagged'])} / {fmt(s['agree'])} / {fmt(s['kappa'])}"
                 for s in (st["by_stratum"][x] for x in STRATA)]
        L.append(f"| {key} | " + " | ".join(cells) + " |")
    L += ["", "## Together/apart: judge against the metric's graph", "",
          ("Metric together = the pair is an edge of the k = 20 graph (the only pairs the cost rewards "
          "joining). Scheme together = LCA depth ≥ 2."), "",
          "| run | judge: together | agreement (metric) | κ (metric) | κ (scheme) |", "|---|---|---|---|---|"]
    for key, st in list(res["runs"].items()) + [("all runs", res["all"])]:
        L.append(f"| {key} | {fmt(st['judge_together'])} | {fmt(st['together_agree'])} | "
                 f"{ci(st['together_kappa'], st['ci']['together_kappa'])} | "
                 f"{fmt(st['scheme_vs_judge_together_kappa'])} |")
    L += ["", "## Where the judge disagrees with the scheme", "",
          ("Shares of all sampled pairs of the run. False split: judged incorrect and split at depth ≤ 1. "
          "False join: judged incorrect and joined at depth ≥ 2. Edges judged together: share of the "
          "sampled graph edges (w > 0) that the judge would put together."), "",
          "| run | false split | false join | A judged together | sampled edges | edges judged together |",
          "|---|---|---|---|---|---|"]
    for key, st in list(res["runs"].items()) + [("all runs", res["all"])]:
        a_st = st["by_stratum"]["A"] if "by_stratum" in st else res["strata"]["A"]
        L.append(f"| {key} | {fmt(st['false_split'])} | {fmt(st['false_join'])} | {fmt(a_st['judge_together'])} | "
                 f"{st['edge_n']} | {fmt(st['edge_judged_together'])} |")
    keys = list(RUNS)
    cost = [meta["runs"][k]["cost_norm"] for k in keys]
    c_bad = [res["runs"][k]["by_stratum"]["C"]["judge_incorrect"] for k in keys]
    all_bad = [res["runs"][k]["judge_incorrect"] for k in keys]
    res["cross_run"] = {"spearman_cost_vs_C_incorrect": spearman(cost, c_bad),
                        "spearman_cost_vs_all_incorrect": spearman(cost, all_bad)}
    L += ["", "## Across schemes: cost_norm against the judge", "",
          ("Does the scheme ranking by Dasgupta cost_norm (lower is better) match the ranking by the judge's "
          "error rate on random placed pairs (stratum C)? Eight runs only; read as descriptive."), "",
          "| run | cost_norm | judge-incorrect, C | judge-incorrect, all strata |", "|---|---|---|---|"]
    for k, c, b, ab in sorted(zip(keys, cost, c_bad, all_bad), key=lambda t: t[1]):
        L.append(f"| {k} | {fmt(c, 4)} | {fmt(b)} | {fmt(ab)} |")
    L += ["", (f"Spearman ρ(cost_norm, judge-incorrect on C) = {fmt(res['cross_run']['spearman_cost_vs_C_incorrect'])}; "
          f"on all strata = {fmt(res['cross_run']['spearman_cost_vs_all_incorrect'])}.")]
    (OUT / "results.json").write_text(json.dumps(res, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    cons = res["consistency"]
    L += ["", "## Judge self-consistency (second blind pass)", "",
          "| label | n | agreement | Cohen κ |", "|---|---|---|---|"]
    for f, c in cons.items():
        L.append(f"| {f} | {c['n']} | {fmt(c['agree'])} | {ci(c['kappa'], c.get('ci'))} |")
    L += ["", "## Sample", "",
          "| run | coverage | cost_norm | deep threshold | A pool | B found / tries |", "|---|---|---|---|---|---|"]
    for key, m in meta["runs"].items():
        L.append(f"| {key} | {fmt(m['coverage'])} | {fmt(m['cost_norm'], 4)} | {m['deep_threshold']} | "
                 f"{m['A_pool']} | {m['B_found']} / {m['B_tries']} |")
    (OUT / "results.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    print("\n".join(L))


# Human-librarian packet. Committed copies of the scheme files, used when the
# cache has none; ilc2 and msc are fetch-only and must be built
# (crates/bench/scripts/build_schemes.sh ilc2 msc).
HUMAN_PER_CELL = 5
HUMAN = OUT / "human"
COMMITTED_SCHEMES = {
    "lcc-full-placed": "schemes/lcc-full/lcc-full.json.gz",
    "fdc-placed": "schemes/fdc/fdc.json",
    "unesco-thesaurus-placed": "schemes/unesco-thesaurus/unesco-thesaurus.json",
    "openalex-topics-placed": "schemes/openalex-topics/openalex-topics.json",
    "arxiv-gold": "schemes/arxiv/arxiv.json",
    "learned-placed": "schemes/learned/learned.json",
}
HUMAN_FIELDS = ["relation", "together", "scheme_decision", "class_1_ok", "class_2_ok"]


def human_uid(pid):
    return hashlib.sha256(f"{SEED}:human:{pid}".encode()).hexdigest()[:10]


def human_works(r):
    """The two works in packet order (work 1, work 2), seeded per pair."""
    works = [(r["id_a"], r["class_a"], "a"), (r["id_b"], r["class_b"], "b")]
    if int(hashlib.sha256(f"{SEED}:human-order:{r['pair_id']}".encode()).hexdigest()[:8], 16) & 1:
        works.reverse()
    return works


def human_subset(rows):
    """HUMAN_PER_CELL pairs per (run, stratum), seeded, in sample order."""
    out = []
    for key in RUNS:
        for st in STRATA:
            cell = sorted((r for r in rows if r["run"] == key and r["stratum"] == st), key=lambda r: r["pair_id"])
            out += rng_for("human", key, st).sample(cell, min(HUMAN_PER_CELL, len(cell)))
    return out


def scheme_doc(cache, key, want):
    paths = [cache / RUNS[key][1]]
    if key in COMMITTED_SCHEMES:
        paths.append(ROOT / COMMITTED_SCHEMES[key])
    for path in paths:
        if not path.exists():
            continue
        raw = path.read_bytes()
        if path.suffix == ".gz":
            raw = gzip.decompress(raw)
        if hashlib.sha256(raw).hexdigest() == want:
            return json.loads(raw)
    raise SystemExit(f"{key}: no scheme file with sha256 {want[:12]}… in {[str(p) for p in paths]}")


def class_path(classes, code):
    out = []
    while code:
        c = classes[code]
        out.append(f"{code} — {c.get('label', '')[:LABEL_CHARS]}")
        code = c.get("parent")
    return list(reversed(out))


def human(a):
    rows = jsonl(OUT / "sample.jsonl")
    meta = json.loads((OUT / "sample-meta.json").read_text())
    corpus = {r["id"]: r for r in jsonl(CORPUS)}
    cache = Path(a.cache)
    docs = {k: scheme_doc(cache, k, meta["runs"][k]["scheme_sha256"]) for k in RUNS}
    classes = {k: {c["code"]: c for c in d["classes"]} for k, d in docs.items()}
    chosen = human_subset(rows)
    rng = rng_for("human", "order")
    rng.shuffle(chosen)
    L = ["# Human-librarian validity packet", "",
         f"{len(chosen)} pairs. Label each pair in `labels.csv` (one row per uid) by the rubric in",
         "`../rubric.md`, section \"Labels\": `relation` (same-topic, same-field, different-field),",
         "`together` (together, apart), `scheme_decision` (correct, incorrect), `class_1_ok` and",
         "`class_2_ok` (yes, no), and an optional one-line `reason`. Judge from the titles,",
         "abstracts and class paths below only. Do not open `../sample.jsonl`, `../results.md`",
         "or any benchmark result before you finish: they hold the metric's and the LLM judge's",
         "view of the same pairs.", "",
         "Annotator, date and any rubric questions go in `annotator.md`.", ""]
    for n, r in enumerate(chosen, start=1):
        L += [f"## {n}. uid `{human_uid(r['pair_id'])}`", "",
              f"Scheme: {docs[r['run']].get('title', RUNS[r['run']][0])}", ""]
        for m, (wid, code, _) in enumerate(human_works(r), start=1):
            row = corpus[wid]
            text = re.sub(r"\s+", " ", row.get("abstract_text") or row.get("description") or "").strip()
            if len(text) > ABSTRACT_CHARS:
                text = text[:ABSTRACT_CHARS].rsplit(" ", 1)[0] + " …"
            title = re.sub(r"\s+", " ", row["title"]).strip()
            L += [f"**Work {m}.** {title}", "", f"> {text}", "",
                  "Class path: " + " › ".join(class_path(classes[r["run"]], code)), ""]
    HUMAN.mkdir(parents=True, exist_ok=True)
    (HUMAN / "packet.md").write_text("\n".join(L), encoding="utf-8")
    csv_rows = ["uid," + ",".join(HUMAN_FIELDS) + ",reason"]
    csv_rows += [human_uid(r["pair_id"]) + "," * (len(HUMAN_FIELDS) + 1) for r in chosen]
    (HUMAN / "labels.csv").write_text("\n".join(csv_rows) + "\n", encoding="utf-8")
    print(f"wrote {len(chosen)} pairs to {HUMAN}")


def human_report(a):
    import csv
    rows = jsonl(OUT / "sample.jsonl")
    chosen = {human_uid(r["pair_id"]): r for r in human_subset(rows)}
    with open(HUMAN / "labels.csv", encoding="utf-8") as f:
        labels = list(csv.DictReader(f))
    filled = [lab for lab in labels if any(lab.get(k) for k in HUMAN_FIELDS)]
    for lab in filled:
        if lab["uid"] not in chosen:
            raise SystemExit(f"labels.csv: unknown uid {lab['uid']}")
        for f in HUMAN_FIELDS:
            if lab.get(f) not in LABELS[f]:
                raise SystemExit(f"labels.csv: {lab['uid']} has invalid {f}={lab.get(f)!r}")
    if not filled:
        raise SystemExit("labels.csv has no labels yet")
    pairs = [(chosen[lab["uid"]], lab) for lab in filled]
    res = {"n": len(pairs), "of": len(chosen)}
    human_bad = [lab["scheme_decision"] == "incorrect" for _, lab in pairs]
    llm_bad = [r["labels"]["pass1"]["scheme_decision"] == "incorrect" for r, _ in pairs]
    flag = [r["metric_flag"] for r, _ in pairs]
    res["human_vs_metric"] = {"agree": sum(x == y for x, y in zip(human_bad, flag)) / len(pairs),
                              "kappa": kappa(human_bad, flag)}
    res["human_vs_llm"] = {}
    for f in ["scheme_decision", "together", "relation"]:
        x = [lab[f] for _, lab in pairs]
        y = [r["labels"]["pass1"][f] for r, _ in pairs]
        res["human_vs_llm"][f] = {"agree": sum(p == q for p, q in zip(x, y)) / len(pairs), "kappa": kappa(x, y)}
    # class_1_ok / class_2_ok refer to packet order; map them back to a / b.
    agree_cls = []
    for r, lab in pairs:
        for m, (_, _, side) in enumerate(human_works(r), start=1):
            agree_cls.append((lab[f"class_{m}_ok"], r["labels"]["pass1"][f"class_{side}_ok"]))
    res["human_vs_llm"]["class_ok"] = {"agree": sum(p == q for p, q in agree_cls) / len(agree_cls),
                                       "kappa": kappa([p for p, _ in agree_cls], [q for _, q in agree_cls])}
    res["llm_vs_metric_same_pairs"] = {"agree": sum(x == y for x, y in zip(llm_bad, flag)) / len(pairs),
                                       "kappa": kappa(llm_bad, flag)}
    (HUMAN / "results.json").write_text(json.dumps(res, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    L = ["# Human-librarian labels: agreement", "", f"{res['n']} of {res['of']} pairs labelled.", "",
         "| comparison | agreement | Cohen κ |", "|---|---|---|",
         f"| human scheme_decision vs metric flag | {fmt(res['human_vs_metric']['agree'])} | "
         f"{fmt(res['human_vs_metric']['kappa'])} |",
         f"| LLM judge vs metric flag, same pairs | {fmt(res['llm_vs_metric_same_pairs']['agree'])} | "
         f"{fmt(res['llm_vs_metric_same_pairs']['kappa'])} |"]
    for f, c in res["human_vs_llm"].items():
        L.append(f"| human vs LLM judge: {f} | {fmt(c['agree'])} | {fmt(c['kappa'])} |")
    (HUMAN / "results.md").write_text("\n".join(L) + "\n", encoding="utf-8")
    print("\n".join(L))


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--cache", default=str(ROOT / "bench/corpus/cache"))
    sub = p.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("sample")
    s.add_argument("--check", action="store_true", help="verify the committed sample instead of writing it")
    b = sub.add_parser("blind")
    b.add_argument("--out", required=True)
    m = sub.add_parser("merge")
    m.add_argument("--labels", required=True)
    sub.add_parser("report")
    sub.add_parser("human")
    sub.add_parser("human-report")
    a = p.parse_args()
    {"sample": sample, "blind": blind, "merge": merge, "report": report,
     "human": human, "human-report": human_report}[a.cmd](a)


if __name__ == "__main__":
    main()
