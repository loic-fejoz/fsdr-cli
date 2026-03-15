# Architecture Map

## Core Design
`fsdr-cli` acts as an intelligent command-line parser that constructs and executes `FutureSDR` flowgraphs dynamically based on terminal input. It heavily interoperates with `fsdr-blocks` and `FutureSDR` primitives.

## Data Flow
1. **CLI Parsing:** `pest` grammar (`src/cmd_line.pest`) parses the complex pipeline string (e.g., `csdr convert_u8_f | csdr ...`) into tokens.
2. **Dispatching:** The AST is evaluated (e.g., `src/csdr_cmd/`), mapping strings to actual block instantiations.
3. **Graph Assembly:** A `futuresdr::runtime::Flowgraph` is dynamically generated, adding blocks and connecting them in sequence.
4. **Execution:** The `Runtime` starts the flowgraph, usually connecting `stdin` to `stdout` processing.

## Important Patterns
- **Pest Parsing:** See `src/cmd_line.rs` for how the rules defined in `.pest` are evaluated.
- **csdr equivalence:** `src/csdr_cmd/` is the central hub mapping legacy `csdr` commands to modern SDR block representations. Look at existing definitions for implementing a new command.
