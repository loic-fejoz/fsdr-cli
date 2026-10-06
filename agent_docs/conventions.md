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
- **Fast-Math Intrinsics:** Critical DSP inner loops use `core::intrinsics::fadd_fast`, `fsub_fast`, and `fmul_fast` to optimize float performance (similar to SatDump / GNU Radio fast_math). `#![allow(internal_features)]` is maintained in `src/lib.rs` and `src/main.rs`.

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
- **FromStr vs TryFrom**: Always implement `std::str::FromStr` for string-to-enum conversions (`FromStr` is idiomatic in Rust and avoids `clippy::try_from_instead_of_from_str` warnings). Implement `TryFrom<&str>` by delegating to `value.parse()`.

## Multi-Repository Dependency Conventions
- **`Cargo.toml` vs `.cargo/config.toml`**: `Cargo.toml` must ONLY contain valid, public remote Git URLs (`https://github.com/...`) with explicit branch or commit revision pins (`rev = "..."`, `branch = "..."`). Local filesystem paths (`path = "../..."`) belong exclusively in local, untracked `.cargo/config.toml` files.
- **Transitive Dependency Unification**: When multiple dependencies reference common crates (e.g., `futuresdr`), ensure all entries in `Cargo.toml` specify identical Git `rev` or `branch` targets to prevent Cargo duplicate crate errors on CI.

## Parameter Extraction
Use `Grc2FutureSdr::parameter_as_f64` and similar helpers in `src/grc/converter/mod.rs` to extract block parameters with expression evaluation support.

## Testing
Always run `cargo test` after modifying building logic. Complex commands often have dedicated tests (e.g., `tests/grc_parse.rs`).
