# Coding Conventions

## Naming
- Use standard Rust conventions (`snake_case` functions/modules, `CamelCase` types/structs).
- Maintain parity with `csdr` nomenclature when writing CLI command matchers (e.g., `shift_addition_cc`, `convert_u8_f`) for user familiarity.
- `_cmd` suffixes are typical for modules wrapping CLI logic (e.g., `csdr_cmd`).

## Logic Patterns
- **Types:** Rely heavily on `num_complex::Complex32` representing IQ streams instead of custom structures where possible, as FutureSDR standardizes on this.
- **Parsing:** Use standard `pest_derive` structures. Do not manually write string tokenizers; extend `cmd_line.pest`.
- **Error Handling:** Propagate errors upwards using `Result`/`anyhow` rather than unwrapping dynamically allocated graphs.
- **No Manual Formatting:** Do not configure or argue over spaces/tabs/braces. We use what `cargo fmt` defines.
- **Linting:** We enforce `cargo clippy`. Fix all warnings before submitting changes.
