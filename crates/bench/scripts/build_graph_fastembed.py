#!/usr/bin/env python3
"""Embed JSONL corpus and build the benchmark's exact mean-offset kNN graph."""
import argparse
import hashlib
import json
import os
import resource
import time
from pathlib import Path

import numpy as np
from fastembed import TextEmbedding
from huggingface_hub import snapshot_download

MODEL_CACHE = "/tmp/fastembed_cache"
# Pinned CPU models: fastembed model id, ONNX weights repository and revision,
# weights file, download patterns (None: the whole repository) and the prefix
# put before every text. nomic-embed-text-v1.5 expects a task prefix.
MODELS = {
    "minilm": ("sentence-transformers/all-MiniLM-L6-v2", "Qdrant/all-MiniLM-L6-v2-onnx",
               "d13954661f83248295ba75c1ed411eef3b7b936e", "model.onnx", None, ""),
    "bge-small": ("BAAI/bge-small-en-v1.5", "Qdrant/bge-small-en-v1.5-onnx-Q",
                  "aa8f8b060edb00e03bfdd08813a2949946c8ba55", "model_optimized.onnx", None, ""),
    "nomic": ("nomic-ai/nomic-embed-text-v1.5", "nomic-ai/nomic-embed-text-v1.5",
              "e9b6763023c676ca8431644204f50c2b100d9aab", "onnx/model.onnx",
              ["*.json", "onnx/model.onnx"], "clustering: "),
}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def peak_rss_gib():
    return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 2**20


def embed(model, texts, path, batch_size):
    """Embed and L2-normalize batch by batch, streaming the vectors into a memmap at `path`."""
    started = time.monotonic()
    out = None
    done = 0
    batch = []
    for vector in model.embed(texts, batch_size=batch_size):
        batch.append(vector)
        if len(batch) < batch_size and done + len(batch) < len(texts):
            continue
        block = np.asarray(batch, dtype=np.float32)
        block /= np.maximum(np.linalg.norm(block, axis=1, keepdims=True), 1e-12)
        if out is None:
            out = np.memmap(path, dtype="<f4", mode="w+", shape=(len(texts), block.shape[1]))
        out[done:done + len(block)] = block
        done += len(block)
        batch = []
        if done % 1000 < batch_size or done == len(texts):
            print(f"embedded {done}/{len(texts)} rows in {time.monotonic() - started:.1f}s, "
                  f"peak RSS {peak_rss_gib():.2f} GiB", flush=True)
    out.flush()
    return out


def main():
    p = argparse.ArgumentParser()
    p.add_argument("corpus")
    p.add_argument("graph")
    p.add_argument("manifest")
    p.add_argument("--k", type=int, default=20)
    p.add_argument("--model", choices=sorted(MODELS), default="minilm")
    p.add_argument("--threads", type=int, default=min(4, os.cpu_count() or 1))
    p.add_argument("--vectors", help="also write the L2-normalized vectors here "
                   "(little-endian f32, row-major, n x dimensions)")
    p.add_argument("--from-vectors", help="read the vectors written by an earlier --vectors run of "
                   "the same model and corpus instead of embedding again")
    p.add_argument("--vectors-only", action="store_true",
                   help="stop after writing --vectors (no graph, no manifest)")
    p.add_argument("--batch-size", type=int, default=64)
    p.add_argument("--max-length", type=int,
                   help="truncate every text to this many tokens (default: the model's own limit)")
    a = p.parse_args()
    model_id, repo, revision, weights, patterns, prefix = MODELS[a.model]
    with open(a.corpus, encoding="utf-8") as f:
        rows = [json.loads(line) for line in f if line.strip()]
    texts = [f"{prefix}{r.get('title', '')}. {r.get('abstract', r.get('abstract_text', r.get('description', '')))}"
             for r in rows]
    model_path = snapshot_download(repo, revision=revision, cache_dir=MODEL_CACHE, allow_patterns=patterns)
    if a.vectors_only and not a.vectors:
        p.error("--vectors-only needs --vectors")
    if a.from_vectors:
        vectors = np.fromfile(a.from_vectors, dtype="<f4").reshape(len(texts), -1)
    else:
        model = TextEmbedding(model_id, threads=a.threads, cache_dir=MODEL_CACHE,
                              specific_model_path=model_path)
        if a.max_length:
            model.model.tokenizer.enable_truncation(max_length=a.max_length)
        target = a.vectors or f"{a.graph}.f32.tmp"
        Path(target).parent.mkdir(parents=True, exist_ok=True)
        vectors = embed(model, texts, target, a.batch_size)
        del model
        if a.vectors_only:
            print(json.dumps({"n": len(vectors), "dimensions": int(vectors.shape[1]),
                              "max_length": a.max_length, "batch_size": a.batch_size,
                              "peak_rss_gib": round(peak_rss_gib(), 2)}))
            return
        vectors = np.array(vectors)
        if not a.vectors:
            os.unlink(target)
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
    model_file = Path(model_path) / weights
    manifest = {
        "model_id": model_id,
        "runtime": "fastembed 0.9.0 / ONNX Runtime CPU",
        "weights_repository": repo,
        "weights_revision": revision,
        "weights_sha256": sha256(model_file),
        "corpus_sha256": sha256(a.corpus),
        "n": n,
        "dimensions": int(vectors.shape[1]),
        "graph_k": a.k,
        "tau_rule": "exact mean cosine over all unordered corpus pairs",
        "tau": tau,
        "graph_sha256": sha256(a.graph),
    }
    if a.model != "minilm":
        manifest = {**manifest, "weights_file": weights, "text_prefix": prefix}
    if a.max_length:
        manifest["max_length_tokens"] = a.max_length
    Path(a.manifest).write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"n": n, "edges": len(edges), "tau": tau, **manifest,
                      "peak_rss_gib": round(peak_rss_gib(), 2)}, indent=2))


if __name__ == "__main__":
    main()
