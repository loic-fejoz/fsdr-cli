# Project Architecture

`fsdr-cli` translates DSP command pipelines (CSDR-style) into an intermediate **GRC (GNU Radio Companion)** graph. This graph is designed to be fully compatible with the official `.grc` file format, allowing both translated CSDR commands and native GRC files to be executed via the FutureSDR runtime.
It acts as an intelligent command-line parser that constructs and executes `FutureSDR` flowgraphs dynamically based on terminal input. It heavily interoperates with `fsdr-blocks` and `FutureSDR` primitives.

## Data Flow
1. **Parsing**: `pest` grammar in `src/cmd_line.pest` parses command lines.
2. **Dispatch**: `src/csdr_cmd/mod.rs` and `src/csdr_cmd/any_cmd.rs` route commands to specific builders.
3. **Builder**: `src/grc/builder.rs` manages the `GrcBuilder` state, constructing a graph that complies with GRC standards (block IDs, parameter names, and enumerations).
4. **Conversion**: Individual block converters in `src/grc/converter/` (e.g., `blocks_file_sink.rs`) map these GRC-defined blocks and parameters to specific FutureSDR kernels.
5. **Execution:** The `Runtime` starts the flowgraph, usually connecting `stdin` to `stdout` processing.

## Key Modules
- `src/csdr_cmd/`: Builders for various `csdr` compatible commands.
- `src/grc/`: GRC parsing and the core `GrcBuilder` logic.
- `src/grc/converter/`: specialized logic for mapping GRC blocks to FutureSDR kernels.
- `src/iqengine_blockconverter.rs`: Bridge for IQEngine plugin integration.

## Important Patterns
- **Pest Parsing:** See `src/cmd_grammar.rs` and `src/cmd_line.rs` for how the rules defined in `.pest` are evaluated.
- **csdr equivalence:** `src/csdr_cmd/` is the central hub mapping legacy `csdr` commands to modern SDR block representations. Look at existing definitions for implementing a new command.

## Extension Patterns
- **New CSDR Command**: Add a rule to `cmd_line.pest`, create a new module in `src/csdr_cmd/`, and update `AnyCmd` in `src/csdr_cmd/any_cmd.rs`.
- **New GRC Block**: Implement `BlockConverter` in `src/grc/converter/` and register it in `src/grc/converter/mod.rs`.
