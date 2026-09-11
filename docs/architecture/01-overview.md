# Lumin Architecture Overview

Lumin is a local-first learning companion built with Tauri. The frontend is React running on Bun and Vite. The backend is Rust. All inference and classroom analytics stay on the device unless a student explicitly joins a teacher session. Network traffic is limited to metadata-sized `AnalysisEvent` messages over the local network.

## Component Architecture

```mermaid
flowchart TB
    subgraph Frontend["React UI (Bun + Vite)"]
        A[Student View]
        B[Teacher View]
        C[Settings / Model Manager]
    end

    subgraph Tauri["Tauri IPC Layer"]
        D[Tauri Commands]
        E[Event Channel]
    end

    subgraph Rust["Rust Backend"]
        F[lumin_core]
        G[inference]
        H[network]
        I[storage]
    end

    subgraph Platform["Platform Services"]
        J[ort / ONNX Runtime]
        K[HTTP / mDNS]
        L[App Data Directory]
    end

    A --> D
    B --> D
    C --> D
    D --> F
    D --> G
    D --> H
    D --> I
    G --> J
    H --> K
    I --> L
    E --> A
    E --> B
```

### Frontend

The React frontend renders two primary modes from a single app.

- **Student View**: question cards, answer input, three-stage hints, retry flow.
- **Teacher View**: session setup, live aggregate dashboard, next-lesson draft editor, history.
- **Settings / Model Manager**: download and switch between the Gemma variants (Gemma 3 1B, Gemma 3n E2B, Gemma 4 E2B / E4B), select execution provider, check disk space.

Frontend code calls Rust through `invoke("command_name")` and listens to events such as `session-discovered`, `analysis-event-ack`, or `download-progress`.

### Tauri IPC Layer

Tauri exposes typed commands and events between JavaScript and Rust.

- Commands are defined in `src-tauri/src/lib.rs`.
- Capabilities live in `src-tauri/capabilities/default.json`.
- Long-running work uses Tauri events instead of blocking command returns.

### Rust Backend

The backend is split into modules by responsibility.

| Module | Responsibility |
| --- | --- |
| `lumin_core` | Question bank, answer analysis, misconception labels, class summaries, lesson-plan drafts |
| `inference` | `ort` session management, tokenizer, model catalog, Gemma 3 / 3n / 4 generation, CPU / GPU execution providers |
| `network` | Teacher HTTP server, student HTTP client, mDNS service discovery, join-code validation |
| `storage` | App data directory access, offline queue, session history, model cache |

## Classroom Session Data Flow

```mermaid
sequenceDiagram
    participant T as Teacher Device
    participant S1 as Student Device 1
    participant S2 as Student Device 2

    T->>T: Start classroom session (teacher mode)
    T->>T: Start HTTP server on local address
    T->>Network: Advertise via mDNS with session ID

    S1->>Network: Discover teacher via mDNS
    S2->>Network: Discover teacher via mDNS

    S1->>T: POST /join with 4-digit code
    T-->>S1: Return anonymous participant token
    S2->>T: POST /join with 4-digit code
    T-->>S2: Return anonymous participant token

    loop Per question
        S1->>S1: Analyze answer locally
        S1->>T: POST /event (AnalysisEvent without answer text)
        T-->>S1: Ack event ID
        S2->>S2: Analyze answer locally
        S2->>T: POST /event (AnalysisEvent without answer text)
        T-->>S2: Ack event ID
        T->>T: Update aggregate dashboard
    end

    T->>T: Generate next-lesson draft from ClassSummary
    T->>T: Save anonymous summary to local history
```

Teacher mode starts an HTTP server on the local network. Students discover it through mDNS and connect with a 4-digit join code. After validation, each student receives an anonymous token. Students analyze their answers on-device and send only `AnalysisEvent` metadata to the teacher. The teacher dashboard aggregates events and produces a 10-minute next-lesson draft from the `ClassSummary`.

## Privacy Boundary

The privacy model is enforced by design. Answer text never leaves the student device.

### Stays on the student device

- The full answer text entered by the student
- Intermediate answer state
- Hints shown to the student
- Raw inference outputs used for personalization

### Travels to the teacher device

- Anonymous session token
- Question ID and concept tag
- Misconception label
- First-try correctness, hint count, retry result

### Stays on the teacher device

- Aggregate dashboards
- Edited lesson-plan drafts
- Anonymous session history (up to 100 items)

If the network drops, student `AnalysisEvent` objects are queued locally. They are re-sent after reconnection and removed only when the teacher acknowledges the event ID. Teacher history is stored in the app data directory and can be deleted from the UI.

## Model and Inference Stack

Inference uses Rust `ort` (ONNX Runtime). `src-tauri/src/inference/catalog.rs` is
the single source of truth for the installable models; download specs, on-disk
layout and the inference wiring are all derived from it.

| Variant | Source repo | Architecture |
| --- | --- | --- |
| `1b-int4`, `1b-int8` | `onnx-community/gemma-3-1b-it-ONNX` | decoder-only |
| `3n-e2b-int4` | `onnx-community/gemma-3n-E2B-it-ONNX` | embed-chained |
| `4-e2b-int4` | `onnx-community/gemma-4-E2B-it-ONNX` | embed-chained |
| `4-e4b-int4` | `onnx-community/gemma-4-E4B-it-ONNX` | embed-chained |

- Runtime: `ort` with CPU by default, execution providers switched via Cargo features
- Execution providers: CoreML on Apple Silicon / iOS, NNAPI / XNNPACK on Android, CUDA / DirectML on desktop where available
- Tokenizer: SentencePiece-based `tokenizer.json`
- Model storage: app data directory, every file verified by SHA256 after download
- Active variant: persisted to `active_model.json` in the model root; switching it drops the loaded session so the next generation loads the new graphs

**Decoder-only** models take `input_ids` directly. **Embed-chained** models
(Gemma 3n and Gemma 4) run an `embed_tokens` graph first and feed its
`inputs_embeds` and `per_layer_inputs` into the decoder, along with
`position_ids` and a scalar `num_logits_to_keep`.

KV-cache geometry is read from the decoder graph's declared inputs rather than
hardcoded: Gemma 4 hides its shared-KV layers (E2B exposes 15 cache entries for
35 layers, E4B 24 for 42) and gives sliding-window layers twice the head dim of
full-attention layers.

Gemma 3n and Gemma 4 are installed into per-variant subdirectories
(`models/gemma-4-e2b-int4/` and so on). Their external-data files all share
generic upstream names such as `decoder_model_merged_q4.onnx_data`, and the
`.onnx` graph references that data by the literal filename recorded at export
time — so the names cannot be changed, and a shared directory would have one
variant overwrite another.

The inference module does not call external services. Generation happens entirely on-device.

## Current Scaffold State

At this stage the repository contains a working Tauri + React scaffold.

- `src-tauri/src/lib.rs` exposes `greet` and `get_system_info` commands only.
- `src/App.tsx` shows a scaffold UI to confirm the IPC pipeline.
- `inference`, `network`, and `lumin_core` modules are not yet implemented.
- The `docs/architecture/` folder is the first documentation being added.

The architecture above describes the target design that the scaffold will grow into.
