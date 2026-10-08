# CPU semantic tiers — spike and benchmark

Bead `dot-materials-classifier-v1-gis4b.4`, plan
`docs/plans/2026-10-07-001-feat-materials-classifier-and-kb-resource-plan.md` (R8, R9; the
laya comparison from the R8 scope note). A standalone Cargo workspace with no dependency on
the surrounding repo (R14). Everything here is a spike: the code is meant to inform the
`shortlist` and `rerank` tiers, not to become them.

**Remote boxes only.** Never run `scripts/bench.sh` on a personal laptop.

## What was measured

- **Task:** coarse-to-fine LCC placement. Each item is embedded and ranked against the 21
  top-level classes. Subclasses are then shortlisted inside the best 3 classes, and the top
  10 are optionally reranked.
- **Scheme:** `data/lcc-outline.json` has 21 classes and 230 subclasses. Captions come from
  the LC Classification Outline PDFs; `scope` strings are editorial.
- **Items:** `data/samples.jsonl` has 49 Project Gutenberg books covering all 21 classes,
  2–4 per class.
  - Text is title plus the Gutenberg summary (`marc520`), clipped to 1,500 characters.
  - Gold labels are Gutenberg's LoCC field, hand-checked.
- **Box:**
  - AMD EPYC Genoa (KVM), Linux x86_64, 22 GB RAM.
  - 12 vCPUs are visible, but the cgroup quota is **4 CPUs** (`cpu.max 400000 100000`), so
    the `4core:*` rows (`taskset -c 0-3`, 4 threads) are the representative numbers.
  - Built with `-C target-cpu=x86-64-v3`; see `.cargo/config.toml` and "Findings".
- **Metrics:**
  - `class@1`: top-1 class, taken from the parent of the best coarse-to-fine subclass.
  - `class@3`: coarse top-3.
  - `sub@1`: best coarse-to-fine subclass.
  - `recall@10`: gold subclass is in the 10-item shortlist, i.e. the ceiling a reranker
    could reach.
  - `load s`: model construction from the local cache, after a prefetch pass; downloads
    are excluded.
  - `index s`: embedding all 251 labels, a one-off per scheme version and memoizable (R7).
  - `RSS MB`: resident memory after everything is loaded; `peak MB` is VmHWM.
  - Latency is per item, single item (batch 1), after one warm-up item.
- **Process isolation:** every profile runs in a fresh process, so memory figures are
  per-profile.
- **Sample size:** 49 items means about ±7 percentage points of standard error. Differences
  under about 10 points are noise.

## Profile recommendation table

Representative numbers come from 4 threads on the 4-CPU quota and n = 49. The lexical
profile runs no model and is benchmarked elsewhere; it is listed so the ladder is complete.

| R8 profile | Tiers / models | Steady RSS | Peak | Load | Label index (251) | Per item p50 (p95) | class@1 | sub@1 | sub recall@10 | Recommendation |
|---|---|---|---|---|---|---|---|---|---|---|
| `lexical` | identifiers, vocabulary, rules; no model | ~0 | ~0 | — | — | — | — | — | — | **Default everywhere.** It decides first; semantic tiers run only below the target confidence (AE3). |
| `shortlist-lite` | BGE-small-en-v1.5 int8 (ONNX via fastembed, MIT, 33M) | 570 MB | 572 MB | 0.2 s | 3.5 s | **46 ms** (85) | 65% | 49% | 73% | **Recommended light shortlist.** Any machine with ≥1 GB free; English only. |
| `shortlist` | EmbeddingGemma-300M q (ONNX via fastembed) | 2.3 GB | 2.45 GB | 1.4 s | 15 s | 199 ms (301) | **73%** | **53%** | **80%** | **Most accurate measured.** Needs ≥3 GB. Multilingual. Check the Gemma Terms of Use before making it a bundled default; download on demand. |
| `shortlist` (Apache-2.0 alternative) | Qwen3-Embedding-0.6B Q8_0 (own candle, in-process quantized) | 814 MB | 1.96 GB | 1.8 s | 81 s | 1.28 s (1.65) | 71% | 47% | 76% | Use when the license must be Apache-2.0 or for 100+ languages. Accuracy matches Gemma within noise; 6× slower. Q4K: 603 MB RSS, 1.33 s, 67% / 49% / 82%. |
| `shortlist+rerank` | Qwen3-Embedding Q8_0 → Qwen3-Reranker-0.6B Q8_0, top 10 | 1.6 GB | 2.76 GB | 3.5 s | 80 s | 7.3 s (rerank 5.9 s) | 65% | 47% | (76%) | **Not recommended yet.** No sub@1 gain, −6 points class@1, 5.7× cost. Keep behind the tier interface or a feature flag and re-measure on a larger gold set. |
| `shortlist+rerank` (light) | Qwen3 Q8_0 → jina-reranker-v1-turbo-en (ONNX) | 991 MB | 1.96 GB | 2.2 s | 82 s | 1.63 s (rerank 226 ms) | 57% | 33% | (76%) | **Rejected.** Cheap, but it loses 14 points of sub@1 against the shortlist it reranks. |
| experimental | laya-decision 0.2.5, ModernBERT-large `convaiinnovations/laya`; choice ≤20 options, own shortlist above 20 | 1.68 GB | 2.64 GB | 2.1 s | — | 27.8 s (92 s) | 43% | 27% | — | **Comparison only, not a tier.** Worse and about 20× slower than Qwen3 Q8_0. |

**Machine guidance for R9's "which profiles can this machine run":**
- Under 1 GB free: `lexical` only.
- 1–3 GB: add `shortlist-lite`.
- 3 GB or more: add `shortlist` (Gemma, or Qwen3 Q8_0 for Apache-2.0).
- Gate on **peak** memory, not steady RSS; load-time transients reach 2–2.5× the steady
  figure.
- Take the thread count from the cgroup-aware CPU budget (`std::thread::available_parallelism`),
  not the visible core count. See finding 5.

Full per-profile table (generated by `semspike report results/*.json`): `results/table.md`.
Unprefixed rows used 12 threads on the 4-CPU quota and are throttled. They are kept because
they show the oversubscription penalty.


## Findings

1. **Embed labels as queries and items as passages.** Qwen3-Embedding is trained on
   instructed short queries against long passages. Given an item summary as the query and
   terse captions as passages, it collapsed onto hub classes (A General Works, CB History
   of civilization): sub@1 20%, class@1 51%. Flipping the direction (labels get the
   instruction; `--flip`, now the default in `bench.sh`) lifted Qwen3 to 47% / 71% /
   recall 76%. It also helped EmbeddingGemma (class@1 69% → 73%) and BGE-small (59% → 65%).
   The tier should treat the scheme side as the query side.
2. **Mean-centering label vectors**, a training-free hubness fix, only helped the
   unflipped Qwen3 (sub@1 20% → 29%). It hurt BGE-small and Gemma and is unnecessary once
   flipped. It is kept as `--center` but unused.
3. **Reranking did not pay on this task.**
   - The shortlist ceiling is recall@10 = 76%, so a perfect reranker could reach 76%
     sub@1.
   - Qwen3-Reranker reached 47% at f32, Q8_0 and Q4K alike, the same as the shortlist,
     while lowering class@1.
   - The general-purpose ONNX cross-encoders were worse: jina-turbo 27–33%, bge-reranker-base
     16%.
   - Likely causes: one-line captions plus scope strings are poor "documents" for a
     relevance model, and 49 items cannot resolve small gains.
   - Retry with richer label texts (scope notes, example titles) and a larger gold set
     before building a rerank tier.
4. **CPU engineering that changed the numbers.** These are all things the real tier must
   get right:
   - **AVX2 must be enabled at compile time for candle's quantized kernels.** Without it
     they fall back to scalar code: about 90 s against 8.4 s per 240-word Q8_0 query.
     `.cargo/config.toml` pins `target-cpu=x86-64-v3` (Haswell or newer, 2013+) on x86_64.
     aarch64 gets NEON by default.
   - **Quantized prefill.** candle's GGML CPU matmul synchronises its thread pool once per
     input row, so Q8_0 ran 5× slower than f32 on long inputs. The fix dequantizes one
     layer's weights on the fly and uses f32 gemm when an input has 16 or more rows
     (`SEMSPIKE_DEQUANT_ROWS`). Q8_0 went from 8.4 s to 2.0 s per query, and parity
     against f32 improved (cos 0.9996 or better). Resident weights stay quantized.
   - **Batching.** `broadcast_matmul` against a transposed weight copied the weight per
     batch row: 50 s for 16 labels, against 0.85 s after flattening to 2-D.
   - **BF16 is not a CPU option in candle 0.11.** It fails with `unsupported dtype BF16 for
     op matmul`, so `fe-qwen3:bf16` was dropped. Use f32 or GGML quantization.
5. **Threads: size to the cgroup quota.** Qwen3 Q8_0 per-item p50 by thread count
   (`results/threads/`):

   | Threads | p50 per item |
   |---|---|
   | 2 | 1.61 s |
   | 4 | 1.27 s |
   | 6 | 1.59 s |
   | 8 | 1.58 s |
   | 12 | 2.64 s |

   The 4-thread Qwen3 rerank profile ran in 7.3 s per item, against 14 s with 12 threads.
6. **Memory.**
   - Quantizing from the BF16 safetensors at load gives low steady RSS: Q8_0 814 MB, Q4K
     603 MB, f32 2.15 GB.
   - Peak stays about 1.75–1.96 GB from load transients, and Q4K quantization adds 8 s to
     load.
   - Shipping pre-quantized GGUF files would cut both. This is the main open item if the
     Qwen3 profile is chosen.
   - fastembed and ORT profiles sit near their peak (the arena allocator).
7. **Throughput and memoization.** Qwen3-0.6B on 4 CPUs runs at roughly 100–250 tokens/s.
   Embedding the 251 LCC labels costs about 80 s, once per (scheme version, model,
   precision), and must be memoized per R7. Per-item cost is about 1.3 s for a 1,500-character
   summary.
8. **Backend parity** (`semspike parity`, `/work/artifacts/semantic/parity.json` on the
   bench box):
   - This crate's candle Qwen3 f32 matches fastembed's independent Qwen3 implementation
     (cos 1.0000).
   - Q8_0 cos is 0.9996 or better; Q4K is 0.94–0.97.
   - The reranker's shared-prefix KV cache matches full-prompt scoring to 1e-5.
   - `results/qwen-f32.json` and `results/qwen-fe-f32.json` give identical accuracy.
9. **Cross-platform status.** Only Linux x86_64 was measured.
   - candle is pure Rust; on macOS and Windows the same code builds with the same rustflags
     (Metal and Accelerate are optional features).
   - fastembed downloads prebuilt ONNX Runtime binaries for all three operating systems.
   - Apple Silicon numbers still need a remote macOS box. Never run this on David's laptop.
10. **Data caveats.**
    - E and F share the caption "History of the Americas"; their scope strings tell them
      apart.
    - Gutenberg LoCC labels are coarse, and some summaries are generic.
    - With n = 49, ±7 points is noise, so Gemma against Qwen3 Q8_0 against Q4K is a tie.
    - The gap between them and BGE-small (about 8 points class@1) is suggestive. The gap to
      MiniLM, Arctic-XS and laya is real.


## Layout and reproduction

```
data/lcc-outline.json        two-level LCC outline (21 classes, 230 subclasses)
data/samples.jsonl           49 labelled public sample texts
crates/semspike/             benchmark binary (candle + fastembed/ORT)
  src/qwen3.rs               Qwen3 decoder: dense f32 or in-process GGML quantization,
                             last-token embedding, prefix-cached yes/no reranking
  src/embedders.rs           fastembed ONNX models, fastembed-candle Qwen3, own Qwen3
  src/rerankers.rs           Qwen3-Reranker (own), fastembed ONNX cross-encoders
  src/cascade.rs             coarse-to-fine shortlist, rerank, metrics
crates/laya-compare/         laya-decision comparison (feature `laya`, off by default)
scripts/bench.sh             runs every profile in a fresh process, writes results/
results/*.json               per-profile summary + per-item predictions and timings
results/table.md             generated table
```

```bash
cd materials-classifier-work/semantic
cargo build --release -p semspike
target/release/semspike parity                     # backend cross-checks (see Findings)
scripts/bench.sh                                   # all profiles; or a regex filter
cargo run --release -p laya-compare --features laya -- --data data --out results/laya.json
cargo test -p semspike
```

Weights are cached outside the repo: `~/.cache/huggingface` and
`$FASTEMBED_CACHE_DIR` (default `~/.cache/fastembed`).
