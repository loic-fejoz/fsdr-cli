# Refactoring Proposal for fsdr-cli

This document outlines proposed refactorings for the `fsdr-cli` codebase based on an initial analysis. The goals are to reduce boilerplate, improve type safety, and centralize common logic.

## 1. Unified Command Parsing and Dispatch
Current command parsing is split between the `pest` grammar in `cmd_line.pest` and manual logic in `src/csdr_cmd/mod.rs` and `src/csdr_cmd/any_cmd.rs`.

### Current State
- `pest` identifies top-level commands and their parameters.
- `AnyCmd::parse` manually matches on rules and calls `build_*` methods.
- `build_*` methods manually extract parameters from `pest` pairs.

### Proposed Change
- Move more parameter extraction logic into `pest` if possible, or use a more declarative approach.
- Consider using a trait or a macro for command building to avoid the large match block in `AnyCmd::parse`.
- Unify the handling of CSDR commands and iqengine commands where patterns overlap.

## 2. Reduce Boilerplate in Block Converters
Each `BlockConverter` manually extracts parameters and sets up the FutureSDR block.

### Current State
- `parameter_as_f64` is called repeatedly.
- Type mapping (e.g., `"ccc"` -> `Complex32`) is done via string matching in each converter.
- Window type matching is duplicated across multiple filter converters.

### Proposed Change
- **Core Principle**: All refactorings must preserve **GNU Radio Companion (GRC)** compatibility. Block IDs, parameter names, and enumerations (like `byte`, `float64`, or `window.WIN_HAMMING`) must strictly follow GRC standards.
- Create a `ConverterContext` helper to streamline parameter extraction with better error handling:
  ```rust
  let decim = ctx.get_param_as_usize("decim")?;
  let window = ctx.get_window("win")?;
  ```
- Use an `ItemType` enum to handle `"ccc"`, `"ff"`, etc., and centralize the mapping to Rust types.
- Move shared window and tap generation logic into a common module (e.g., `src/grc/converter/filter_util.rs`).

## 3. Error Handling and Safety
The codebase makes significant use of `expect()`, `unwrap()`, and `todo!()`.

### Current Status: Completed
- [x] Replaced `expect()` with `context()` and `bail!()` from the `anyhow` crate.
- [x] Updated `GrcItemType` to use `TryFrom` for safe error propagation.
- [x] Eliminated common `todo!()` occurrences in converters with clear error messages.
- [x] Implemented more robust validation in `iqengine` plugin modules.

## 5. Metadata and GrcBuilder Refinement
`GrcBuilder` manages the intermediate state when parsing CSDR commands.

### Proposed Change
- Simplify the block linking logic.
- Ensure that metadata (like sample rate) is consistently propagated throughout the builder.
- Improve the `ensure_sink()` logic to handle cases where a sink is already present or explicitly defined.

## 6. Project Structure
- Consider moving some of the manual conversion logic from `src/csdr_cmd/agc_cmd.rs` into the corresponding converter in `src/grc/converter/`.
- Ensure a clean separation between the GRC-focused conversion and the CLI command parsing.
