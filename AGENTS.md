# AI Agent Guide (AGENTS.md)

Welcome! This document provides concise, high-leverage context for AI agents working on `fsdr-cli`.

## Project Overview
`fsdr-cli` is a Rust-based command-line interface for DSP tasks, built on the [FutureSDR](http://www.futuresdr.org) runtime. It translates CSDR-style commands into an intermediate graph structure (GRC) that is strictly compatible with the **GNU Radio Companion (.grc)** file format.

### GRC Compatibility
The intermediate graph must be 100% compatible with GNU Radio Companion. This means:
- Block IDs and parameter names must match GRC definitions.
- Parameter values and enumerations (e.g., window types, item types) must follow GRC standards.
- This compatibility allows `fsdr-cli` to execute `.grc` files directly using the FutureSDR runtime.

- **Tech Stack**: Rust (edition 2021), FutureSDR, anynow, pest (Grc/Command grammar).
- **Core Dependencies**: `futuresdr`, `fsdr-blocks`, `cpal` (audio).

## Critical Commands
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Linter**: `cargo clippy`
- **Format**: `cargo fmt`

## Coding Standards
- **Error Handling**: Use `anyhow` for all non-library errors. Avoid `unwrap()` and `expect()`; use `.context("...")?` instead.
- **Conventions**: [agent_docs/conventions.md](file:///home/loic/projets/fsdr-cli/agent_docs/conventions.md)

## Specialized Context
- **Architecture**: [agent_docs/architecture.md](file:///home/loic/projets/fsdr-cli/agent_docs/architecture.md)
- **Adding Commands**: See common patterns in [src/csdr_cmd/mod.rs](file:///home/loic/projets/fsdr-cli/src/csdr_cmd/mod.rs).
- **GRC Converters**: [src/grc/converter/mod.rs](file:///home/loic/projets/fsdr-cli/src/grc/converter/mod.rs) lists all available block converters.

---
*Note: This file is optimized for AI consumption. For human contributors, see [CONTRIBUTING.md](file:///home/loic/projets/fsdr-cli/CONTRIBUTING.md).*
