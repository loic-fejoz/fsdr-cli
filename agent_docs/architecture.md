# Project Architecture

`fsdr-cli` translates DSP command pipelines (CSDR-style) into an intermediate **GRC (GNU Radio Companion)** graph. This graph is designed to be fully compatible with the official `.grc` file format, allowing both translated CSDR commands and native GRC files to be executed via the FutureSDR runtime.
It acts as an intelligent command-line parser that constructs and executes `FutureSDR` flowgraphs dynamically based on terminal input. It heavily interoperates with `fsdr-blocks` and `FutureSDR` primitives.

## Data Flow
1. **Parsing**: `pest` grammar in `src/cmd_line.pest` parses command lines.
2. **Dispatch**: `src/csdr_cmd/mod.rs` and `src/csdr_cmd/any_cmd.rs` route commands to specific builders.
3. **Builder**: `src/grc/builder.rs` manages the `GrcBuilder` state, constructing a graph that complies with GRC standards (block IDs, parameter names, and enumerations).
4. **Backend Selection**: Based on CLI flags (`--generate`), either `RuntimeBackend` or `CodegenBackend` is selected.
5. **Conversion**: Individual block converters in `src/grc/converter/` (e.g., `fir_filter_xx.rs`) use the selected backend to instantiate blocks.
6. **Execution/Generation**:
   - **Runtime**: The `Flowgraph` is executed by the FutureSDR runtime.
   - **Codegen**: Optimized Rust source code is emitted to a file.

## Dual-Backend Architecture (Futamura Projection)
The core of `fsdr-cli`'s efficiency is the separation of **semantic resolution** from **physical instantiation**. 
Block converters perform all complex mathematical operations (like calculating filter taps for a given transition bandwidth) once. The resulting data is then passed to the `FsdrBackend` trait.

### `#[fsdr_instantiate]` Macro
To ensure "One Source of Truth", we use the `#[fsdr_instantiate]` procedural macro. This macro takes a standard Rust block instantiation function and automatically generates:
- A compiled version for the `RuntimeBackend`.
- A `TokenStream` emitting version for the `CodegenBackend`.

## Key Modules
- `src/grc/backend.rs`: The abstraction layer for flowgraph construction.
- `fsdr-cli-macros/`: The engine behind the Futamura projection.
- `src/grc/converter/`: Generic converters that work across both backends.

## Extension Patterns
- **New CSDR Command**: Add a rule to `cmd_line.pest`, create a new module in `src/csdr_cmd/`, and update `AnyCmd` in `src/csdr_cmd/mod.rs`.
- **New GRC Block**: 
  1. Add a method to `FsdrBackend` in `src/grc/backend.rs`.
  2. Implement the instantiation logic once using `#[fsdr_instantiate]`.
  3. Implement `BlockConverter<B>` in `src/grc/converter/` using the new backend method.
