#!/usr/bin/env python3
"""Embed JSONL corpus and build the benchmark's exact mean-offset kNN graph."""
import argparse
import hashlib
import json
import os
import time
from pathlib import Path

import numpy as np
from fastembed import TextEmbedding
from huggingface_hub import snapshot_download

MODEL_REPO = "Qdrant/all-MiniLM-L6-v2-onnx"
MODEL_REVISION = "d13954661f83248295ba75c1ed411eef3b7b936e"
MODEL_CACHE = "/tmp/fastembed_cache"


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("corpus")
    p.add_argument("graph")
    p.add_argument("manifest")
    p.add_argument("--k", type=int, default=20)
    p.add_argument("--vectors", help="also write the L2-normalized vectors here "
                   "(little-endian f32, row-major, n x dimensions)")
    a = p.parse_args()
    with open(a.corpus, encoding="utf-8") as f:
        rows = [json.loads(line) for line in f if line.strip()]
    texts = [f"{r.get('title', '')}. {r.get('abstract', r.get('abstract_text', r.get('description', '')))}" for r in rows]
    model_path = snapshot_download(MODEL_REPO, revision=MODEL_REVISION, cache_dir=MODEL_CACHE)
    model = TextEmbedding("sentence-transformers/all-MiniLM-L6-v2", threads=min(4, os.cpu_count() or 1),
                          cache_dir=MODEL_CACHE, specific_model_path=model_path)
    started = time.monotonic()
    embedded = []
    for i, vector in enumerate(model.embed(texts, batch_size=64), start=1):
        embedded.append(vector)
        if i % 1000 == 0 or i == len(texts):
            print(f"embedded {i}/{len(texts)} rows in {time.monotonic() - started:.1f}s", flush=True)
    vectors = np.asarray(embedded, dtype=np.float32)
    vectors /= np.maximum(np.linalg.norm(vectors, axis=1, keepdims=True), 1e-12)
    if a.vectors:
        Path(a.vectors).parent.mkdir(parents=True, exist_ok=True)
        vectors.astype("<f4").tofile(a.vectors)
    n = len(vectors)
    # Exact all-pairs mean cosine, accumulated in bounded blocks.
    cosine_sum = 0.0
    pair_count = 0
    block = 128
    for start in range(0, n, block):
        sims = vectors[start:start + block] @ vectors.T
        for offset, row in enumerate(sims):
            i = start + offset
            cosine_sum += float(row[i + 1:].sum(dtype=np.float64))
            pair_count += n - i - 1
    tau = cosine_sum / pair_count if pair_count else 0.0
    neighbors = [[] for _ in range(n)]
    for start in range(0, n, block):
        sims = vectors[start:start + block] @ vectors.T
        for offset, row in enumerate(sims):
            i = start + offset
            row[i] = -np.inf
            # Partition first, then apply the Rust implementation's stable
            # similarity-descending/index-ascending order to only k entries.
            candidates = np.argpartition(row, n - a.k)[n - a.k:]
            cutoff = row[candidates].min()
            greater = np.flatnonzero(row > cutoff)
            equal = np.flatnonzero(row == cutoff)
            selected = np.concatenate((greater, equal[:a.k - len(greater)]))
            order = sorted(selected.tolist(), key=lambda j: (-float(row[j]), j))
            neighbors[i] = [(int(j), max(0.0, float(row[j]) - tau)) for j in order]
    edges = {}
    for i, picks in enumerate(neighbors):
        for j, weight in picks:
            if weight <= 0:
                continue
            key = (min(i, j), max(i, j))
            edges[key] = max(edges.get(key, 0.0), weight)
    graph = {"n": n, "k": a.k, "tau": tau,
             "edges": [{"left": i, "right": j, "weight": w}
                       for (i, j), w in sorted(edges.items())]}
    Path(a.graph).parent.mkdir(parents=True, exist_ok=True)
    Path(a.graph).write_text(json.dumps(graph, separators=(",", ":")), encoding="utf-8")
    model_file = Path(model_path) / "model.onnx"
    manifest = {
        "model_id": "sentence-transformers/all-MiniLM-L6-v2",
        "runtime": "fastembed 0.9.0 / ONNX Runtime CPU",
        "weights_repository": MODEL_REPO,
        "weights_revision": MODEL_REVISION,
        "weights_sha256": sha256(model_file),
        "corpus_sha256": sha256(a.corpus),
        "n": n,
        "dimensions": int(vectors.shape[1]),
        "graph_k": a.k,
        "tau_rule": "exact mean cosine over all unordered corpus pairs",
        "tau": tau,
        "graph_sha256": sha256(a.graph),
    }
    Path(a.manifest).write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"n": n, "edges": len(edges), "tau": tau, **manifest}, indent=2))


if __name__ == "__main__":
    main()
