# Project Architecture

`fsdr-cli` translates DSP command pipelines (CSDR-style) into an intermediate **GRC (GNU Radio Companion)** graph. This graph is designed to be fully compatible with the official `.grc` file format, allowing both translated CSDR commands and native GRC files to be executed via the FutureSDR runtime.

## Data Flow
1. **Parsing**: `pest` grammar in `src/cmd_grammar.pest` parses command lines.
2. **Dispatch**: `src/csdr_cmd/mod.rs` and `src/csdr_cmd/any_cmd.rs` route commands to specific builders.
3. **Builder**: `src/grc/builder.rs` manages the `GrcBuilder` state, constructing a graph that complies with GRC standards (block IDs, parameter names, and enumerations).
4. **Conversion**: Individual block converters in `src/grc/converter/` (e.g., `blocks_file_sink.rs`) map these GRC-defined blocks and parameters to specific FutureSDR kernels.

## Key Modules
- `src/csdr_cmd/`: Builders for various `csdr` compatible commands.
- `src/grc/`: GRC parsing and the core `GrcBuilder` logic.
- `src/grc/converter/`: specialized logic for mapping GRC blocks to FutureSDR kernels.
- `src/iqengine_blockconverter.rs`: Bridge for IQEngine plugin integration.

## Extension Patterns
- **New CSDR Command**: Add a rule to `cmd_grammar.pest`, create a new module in `src/csdr_cmd/`, and update `AnyCmd` in `src/csdr_cmd/any_cmd.rs`.
- **New GRC Block**: Implement `BlockConverter` in `src/grc/converter/` and register it in `src/grc/converter/mod.rs`.
