# Coding Conventions

## Naming
- Use standard Rust conventions (`snake_case` functions/modules, `CamelCase` types/structs).
- Maintain parity with `csdr` nomenclature when writing CLI command matchers (e.g., `shift_addition_cc`, `convert_u8_f`) for user familiarity.
- `_cmd` suffixes are typical for modules wrapping CLI logic (e.g., `csdr_cmd`).

## Logic Patterns
- **Types:** Rely heavily on `num_complex::Complex32` representing IQ streams instead of custom structures where possible, as FutureSDR standardizes on this.
- **Parsing:** Use standard `pest_derive` structures. Do not manually write string tokenizers; extend `cmd_line.pest`.
- **No Manual Formatting:** Do not configure or argue over spaces/tabs/braces. We use what `cargo fmt` defines.
- **Linting:** We enforce `cargo clippy`. Fix all warnings before submitting changes.

## Error Handling
We use `anyhow` for robust, contextual error reporting.
- **Don't**: Use `unwrap()` or `expect()`.
- **Do**: Use `.context("contextual message")?` on `Option` or `Result`.
- **Bailing**: Use `bail!("error message")` for explicit failures.

## GRC Compatibility & Naming
The intermediate graph is the source of truth for block definitions.
- **Naming**: All block IDs and parameter keys MUST match their GNU Radio Companion counterparts.
- **Enumerations**: Parameter values (like `window.WIN_HAMMING` or `byte`) must be identical to what GRC expects.
- **GrcItemType**: Use the `GrcItemType` enum in `src/grc/builder.rs` for handling data types, using `as_grc()` to get the string representation expected by GRC (e.g., "byte" instead of "u8").
- **TryInto**: Conversion from strings to `GrcItemType` must use `try_into()?` for safe propagation.

## Parameter Extraction
Use `parameter_as_f64` and similar standalone helpers in `src/grc/converter/mod.rs` to extract block parameters with expression evaluation support.

## Backend Abstraction
All block conversion logic must be generic over `B: FsdrBackend`.
- **Do not** use `Flowgraph` directly in block converters.
- **Do not** use `BlockId` directly in return types; use `B::BlockRef`.
- Return `Box<dyn ConnectorAdapter<B::BlockRef>>`.

## Futamura Projection (Code Generation)
When adding new blocks, you must ensure they work for both runtime execution and source code generation.
- **Single Source of Truth**: Use the `#[fsdr_instantiate]` macro in `src/grc/backend.rs`.
- **ToTokens Wrappers**: If a parameter is a complex type (like `Vec<f32>` or `Complex32`), use one of the `Codegen...` wrappers (e.g., `CodegenTaps`) to ensure it can be baked into the generated source code.
- **Explicit Types**: In `#[fsdr_instantiate]` functions, always use explicit type annotations for `Apply` or `Sink` closures to aid the compiler in the generated code.

## Testing
Always run `cargo test` after modifying building logic. 
- Use `cargo test -p fsdr-cli-macros` for changes to the macro system.
- Use `tests/codegen.rs` to verify that new blocks generate valid, compilable Rust code.
