# AI Agent Guide (AGENTS.md)

Welcome! This document provides concise, high-leverage context for AI agents working on `fsdr-cli`.

## Mission & Project Overview
`fsdr-cli` is a Rust-based CLI tool leveraging FutureSDR for digital signal processing, acting as an advanced replacement for `csdr`. The goal is to produce reliable, high-performance DSP flowgraphs.
It translates CSDR-style command pipelines into an intermediate graph structure (GRC) that is strictly compatible with the **GNU Radio Companion (.grc)** file format.

### Dual-Backend Architecture (Futamura Projection)
The project supports two execution modes through an abstract backend:
1. **Interpreted (Runtime)**: Builds a live FutureSDR flowgraph for immediate execution.
2. **Compiled (Codegen)**: Generates optimized Rust source code where all complex parameters (filter taps, windows) are pre-computed and baked into the binary.

- **Tech Stack**: Rust (edition 2021), FutureSDR, anynow, pest, **proc-macros (fsdr-cli-macros)**.
- **Core Dependencies**: `futuresdr`, `fsdr-blocks`, `quote`, `proc-macro2`.

## Critical Commands
- **Build:** `cargo build` / `cargo build --release`
- **Test:** `make test`, `cargo test`, `cargo test -p fsdr-cli-macros`
- **Typecheck & Lint:** `cargo clippy -- -D warnings`, `cargo fmt`

## Directory Map
- `src/`: Core logic and CLI parsing.
- `src/grc/backend.rs`: The `FsdrBackend` trait and its Runtime/Codegen implementations.
- `src/grc/converter/`: Generic block converters supporting both backends.
- `fsdr-cli-macros/`: Procedural macro crate providing `#[fsdr_instantiate]`.
- `src/blocks/`: Custom FutureSDR DSP blocks specific to this CLI.
- `src/csdr_cmd/`: Parsers and mapping logic to translate `csdr` commands to GRC instances.
- `tests/`: Integration tests, data files, and benchmarking.
- `Makefile`: Heavily used for end-to-end `csdr` output comparison checks.

## Documentation Index
Please read the following documents in `agent_docs/` for targeted context before jumping into complex development:
1. **`agent_docs/architecture.md`**: For understanding FutureSDR flowgraph construction, GRC standards, and CLI dispatching.
2. **`agent_docs/conventions.md`**: For DSP-specific logic patterns, error handling, and naming standards.
3. **`agent_docs/testing_guidelines.md`**: For instructions on how to replicate and test `csdr` DSP functionality.

## Verification
**CRITICAL:** ALWAYS verify your work using the project's test suite (`cargo test` and specifically `make test` for csdr comparisons) before concluding any task. Ensure no breaking changes to expected byte streams occur unless intended.

---
*Note: This file is optimized for AI consumption. For human contributors, see [CONTRIBUTING.md](file:///home/loic/projets/fsdr-cli/CONTRIBUTING.md).*
