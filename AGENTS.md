# AI Agent Guide (AGENTS.md)

Welcome! This document provides concise, high-leverage context for AI agents working on `fsdr-cli`.

## Mission & Project Overview
`fsdr-cli` is a Rust-based CLI tool leveraging FutureSDR for digital signal processing, acting as an advanced replacement for `csdr`. The goal is to produce reliable, high-performance DSP flowgraphs.
It translates CSDR-style command pipelines into an intermediate graph structure (GRC) that is strictly compatible with the **GNU Radio Companion (.grc)** file format.

### GRC Compatibility
The intermediate graph must be 100% compatible with GNU Radio Companion. This means:
- Block IDs and parameter names must match GRC definitions.
- Parameter values and enumerations (e.g., window types, item types) must follow GRC standards.
- This compatibility allows `fsdr-cli` to execute `.grc` files directly using the FutureSDR runtime.

- **Tech Stack**: Rust (edition 2021), FutureSDR, anynow, pest (Grc/Command grammar).
- **Core Dependencies**: `futuresdr`, `fsdr-blocks`, `cpal` (audio).

## Critical Commands
- **Build:** `cargo build` / `cargo build --release`
- **Test:** `make test` (runs both `cargo test` and `csdr` verification checks), `cargo test`
- **Typecheck & Lint:** `cargo clippy -- -D warnings`, `cargo fmt`

## Directory Map
- `src/`: Core logic (`main.rs`, `lib.rs`) and CLI parsing (`cmd_line.pest`, `cmd_grammar.rs`, `cmd_line.rs`).
- `src/blocks/`: Custom FutureSDR DSP blocks specific to this CLI.
- `src/csdr_cmd/`: Parsers and mapping logic to translate `csdr` commands to FutureSDR blocks.
- `src/grc/`: GNU Radio Companion YAML/layout support and standard generation (`builder.rs`).
- `src/grc/converter/`: Specialized logic for mapping GRC blocks back into FutureSDR kernels.
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
