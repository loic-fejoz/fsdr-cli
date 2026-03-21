# Testing Guidelines

## Standard Tests
- Module-level unit tests live in `#[cfg(test)]` blocks alongside the code in `src/`.
- `cargo test` is responsible for these standard Rust checks.

## CSDR Comparison Integration Tests
The most critical part of `fsdr-cli` is perfectly replicating or replacing `csdr` pipelines.
- **The Makefile is the authority:** See `Makefile` targets like `csdr-compare-*`.
- **Methodology:** We pipe a real data file (e.g., `tests/france-culture-extract.c32`) through the original `csdr` C binary, and also through `target/release/fsdr-cli`.
- We then use `cmp -l` to ensure both binaries produce identically processed binary streams.
- **When adding a feature:** If you implement a legacy `csdr` command, you should add a corresponding `csdr-compare-[feature]` target in the `Makefile` and ensure it runs continuously.

## Mocking
- Avoid over-mocking the Runtime. 
- You can test individual blocks by creating a manual `Flowgraph`, instantiating a `futuresdr::blocks::VectorSource` with known byte slices, your block under test, and a `futuresdr::blocks::VectorSink` to capture the output and assert against expected results.

## Code Generation Testing
When adding or modifying block generation:
1. **Verification**: Run `cargo run --bin fsdr-cli -- csdr --generate test.rs [your command]` and verify `test.rs` is valid Rust.
2. **Integration Tests**: Add a test case to `tests/codegen.rs`.
3. **Optimized Baking**: Ensure that pre-computed values (like filter taps) appear as literals in the generated code, confirming that the Futamura projection is actually baking the data.
