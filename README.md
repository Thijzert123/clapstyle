# Clapstyle
Simple utility library to match your CLI's output to Clap's output.

Features:
- Macros for styling (supported: `print`, `println`, `eprint`, `eprintln`, `panic`)
- Style methods via a trait (for example: `"text".style_error()`)
- Nested styles with style stack
- `anyhow` error styling
- Change Clap's style via a static variable

For more information, please take a look at the docs.

## Installation
Clapstyle is available on [crates.io](https://crates.io/crates/clapstyle).
```bash
cargo add clapstyle
```
