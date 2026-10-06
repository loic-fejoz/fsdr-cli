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
- **Fast-Math Performance**: Crucial DSP inner loops utilize `core::intrinsics` (`fadd_fast`, `fsub_fast`, `fmul_fast`) with `#![feature(core_intrinsics)]` and `#![allow(internal_features)]` for maximum throughput, following SatDump/GNU Radio fast_math practices.

## Critical Commands
- **Build:** `cargo build` / `cargo build --release`
- **Test & CI Verification:**
  - Standard test suite: `cargo test --all-targets --workspace`
  - Full Makefile verification (csdr byte-stream comparison): `make test`
  - Strict CI Linter check: `cargo clippy --all-targets --workspace -- -D warnings`
  - Strict CI Formatter check: `cargo fmt --check`
  - **Isolated CI Environment Test (without local `.cargo/config.toml` overrides):**
    `bash -c "mv .cargo/config.toml .cargo/config.tmp && cargo clippy --all-targets --workspace -- -D warnings && cargo test --all-targets --workspace ; STATUS=\$? ; mv .cargo/config.tmp .cargo/config.toml ; exit \$STATUS"`

## Multi-Repository & Dependency Workflow
When developing `fsdr-cli` alongside sibling repositories (such as `fsdr-blocks` or `FutureSDR`):
1. **Local Overrides in `.cargo/config.toml` (git-ignored):** Use `.cargo/config.toml` for local `[patch]` directives pointing to local paths (e.g. `../fsdr-blocks`). Never place local file paths in `Cargo.toml`.
2. **Upstream Alignment:** Ensure all dependent local changes in sibling crates (`fsdr-blocks`) are committed and pushed to their remote branches before pushing `fsdr-cli`.
3. **Explicit Rev / Branch Pinning:** In `Cargo.toml`, explicitly pin git dependencies to their matching remote `rev` or `branch` (e.g. `fsdr-blocks = { git = "...", branch = "feat/perf" }`) to prevent Cargo from resolving duplicate versions of transitive dependencies on CI.
4. **Lockfile Synchronization:** Run `cargo update -p <crate>` after updating remote branches to sync `Cargo.lock`.

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
**CRITICAL:** ALWAYS verify your work using the full CI check suite (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and `make test`) before concluding any task. Ensure no breaking changes to expected byte streams occur unless intended.

---
*Note: This file is optimized for AI consumption. For human contributors, see [CONTRIBUTING.md](file:///home/loic/projets/fsdr-cli/CONTRIBUTING.md).*
