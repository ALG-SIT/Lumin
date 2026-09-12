# ONNX accelerator verification — 2026-09-12

## Decision and upstream research

Keep the existing ONNX graphs, tokenizer, KV-cache and Rust generation loop. Use ONNX Runtime's native WebGPU execution-provider plugin by default. No second inference backend or model format was added.

[Official native WebGPU documentation](https://onnxruntime.ai/docs/execution-providers/WebGPU-ExecutionProvider.html) describes Dawn dispatch to Metal, Direct3D 12 and Vulkan and direct native plugin loading. This is not browser inference. [CoreML operator support](https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html) does not cover this model's quantized MatMulNBits path. Locally, Gemma 4's correct zero-length initial cache also failed CoreML with `runtime shape {1,2,0,512} has zero elements`. The older padded-cache workaround produced incorrect/repeated text and a native crash. The previous CPU mitigation is now superseded by WebGPU; explicitly choosing CoreML for Gemma 4 returns an explanatory error.

## Implementation

- Default build dynamically loads a bundled ONNX Runtime 1.30.0 and WebGPU EP 0.3.0. Build preparation downloads pinned official wheels as archives, verifies SHA-256 and copies native libraries and licenses only. Python is a build dependency, not an application dependency.
- Registers the plugin once and selects a WebGPU GPU device. Missing library, unknown provider, unavailable GPU and EP registration errors fail explicitly. CPU is only an explicit diagnostic selection.
- Prompt prefill is bounded to 128-token chunks in the shared ONNX generation loop; every token enters the KV cache in order before any response token is sampled. The unchunked E2B all-correct lesson fixture reproducibly stalled in Metal queue submission (both after five requests and in isolation). Chunking completed the full 11-case E2B suite in 66.88 seconds. This is an observed workaround; the upstream shader/driver root cause has not been established.
- Dynamic KV sizes require graph capture off. Dawn chooses the native backend of the discovered GPU.
- Optional existing CUDA/TensorRT/DirectML/CoreML/NNAPI/XNNPACK settings still use the same ORT session factory. They require matching Cargo features and runtime binaries. DirectML disables memory patterns and parallel execution per [official requirements](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html).
- Platform packaging: macOS ARM64 (14+), Linux x86_64 (glibc 2.28+), Windows x86_64/ARM64. Unsupported host packages fail preparation; mobile and Intel Mac binaries are not provided by this lock. Windows/Linux and optional provider hardware have not been executed locally.

## Apple Silicon evidence

M4 Max / 64 GB; existing Gemma 4 E4B INT4 ONNX graph. Actual ORT profiling, not merely successful EP registration:

| Decoder operator | WebGPU events | CPU events |
| --- | ---: | ---: |
| MatMulNBits | 26,878 | 0 |
| GroupQueryAttention | 1,780 | 0 |
| RotaryEmbedding | 5,874 | 0 |
| MatMul | 3,916 | 0 |

Embedding GatherBlockQuantized also executes on WebGPU. CPU nodes handle shape/mask operations. This is intentional mixed graph placement, not whole-model CPU fallback. Profiling durations are host-side timings and are not interpreted as GPU execution-time percentages. Compact evidence: [operator placement](gemma4/onnx-webgpu-placement.json).

The initial E4B two-request run returned the exact question, 60% correct rate, and 4 cases / 40% slope-intercept confusion, followed by the expected `m0` classification. Warm classification took 4.662 seconds / 43 tokens; first request includes model loading. These are diagnostic measurements, not release performance guarantees.

Reproduce with installed model roots (synthetic classroom fixtures only):

```sh
bun run prepare:runtime
LUMIN_AI_SMOKE_MODEL_DIR=/path/to/models \
LUMIN_AI_EVAL_OUTPUT=/tmp/gemma4-eval.json \
cargo test --manifest-path src-tauri/Cargo.toml --lib \
  classroom_model_evaluation -- --ignored --nocapture
```

Set `LUMIN_ORT_PROFILE_DIR` to retain per-node profiling; files can exceed 100 MB. Normal application execution requires neither these variables nor a Python environment. For signed distribution, the native libraries need the application's normal signing and notarization validation.

## Final validation

- Rust: 207 unit tests passed; 10 real-model tests opt-in/ignored in the normal suite. Clippy passed with warnings denied. CUDA/TensorRT/DirectML/NNAPI/XNNPACK feature combination passed `cargo check` on this host (compilation, not hardware validation).
- Frontend: 28 tests passed; Biome, TypeScript/Vite build passed.
- Tauri debug `.app` built with both native libraries in `Contents/Resources/runtime`. Native 1024×720 screenshots confirmed the teacher dashboard and chat input/suggestions fit. Earlier interactive scroll checks are recorded in [UI audit](2026-09-12-ui-ai-audit.md).
- E2B final [raw prompts, generations and delivered results](gemma4/onnx-webgpu-e2b.json): 11 sequential cases completed; classifications matched the synthetic fixtures; all three lesson outputs passed schema validation. One stage-3 hint was replaced by the existing answer-leak guard. Stage-1/2 E2B hints remain less specific than E4B; this is not a broad educational-quality benchmark.
- E4B final chunked run: all 11 sequential cases completed in 134.17 seconds, without truncation or repair retries. Classifications matched fixtures, all lesson schemas passed, all four generated hints passed the guards. [Raw prompts and delivered results](gemma4/onnx-webgpu-e4b.json).
