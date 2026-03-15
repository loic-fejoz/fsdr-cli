# Agent Guidance System: fsdr-cli

## Mission
`fsdr-cli` is a Rust-based CLI tool leveraging FutureSDR for digital signal processing, acting as an advanced replacement for `csdr`. The goal is to produce reliable, high-performance DSP flowgraphs.

## Critical Commands
- **Build:** `cargo build` / `cargo build --release`
- **Test:** `make test` (runs both `cargo test` and `csdr` verification checks), `cargo test`
- **Typecheck & Lint:** `cargo clippy -- -D warnings`, `cargo fmt`

## Directory Map
- `src/`: Core logic (`main.rs`, `lib.rs`) and CLI parsing (`cmd_line.pest`, `cmd_line.rs`).
- `src/blocks/`: Custom FutureSDR DSP blocks specific to this CLI.
- `src/csdr_cmd/`: Parsers and mapping logic to translate `csdr` commands to FutureSDR blocks.
- `src/grc/`: GNU Radio Companion YAML/layout support.
- `tests/`: Integration tests, data files, and benchmarking.
- `Makefile`: Heavily used for end-to-end `csdr` output comparison checks.

## Documentation Index
Please read the following documents in `agent_docs/` for targeted context before jumping into complex development:
1. **`agent_docs/architecture.md`**: For understanding FutureSDR flowgraph construction and CLI dispatching.
2. **`agent_docs/testing_guidelines.md`**: For instructions on how to replicate and test `csdr` DSP functionality.
3. **`agent_docs/conventions.md`**: For DSP-specific logic patterns and naming standards.

## Verification
**CRITICAL:** ALWAYS verify your work using the project's test suite (`cargo test` and specifically `make test` for csdr comparisons) before concluding any task. Ensure no breaking changes to expected byte streams occur unless intended.
