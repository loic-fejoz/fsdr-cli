# Coding Conventions

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
Use `Grc2FutureSdr::parameter_as_f64` and similar helpers in `src/grc/converter/mod.rs` to extract block parameters with expression evaluation support.

## Testing
Always run `cargo test` after modifying building logic. Complex commands often have dedicated tests (e.g., `tests/grc_parse.rs`).
