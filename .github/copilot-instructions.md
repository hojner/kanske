# Kanske — Copilot Instructions

Kanske is a Rust rewrite of [Kanshi](https://git.sr.ht/~emersion/kanshi) — a
Wayland display configuration daemon. It reads a profile-based config file and
automatically applies output settings when specific monitor combinations are
detected via hotplug events.

## Build, Test, Lint

```bash
cargo build
cargo test
cargo test -- <test_name>        # run a single test
cargo clippy
cargo fmt
```

Tests live in `kanske-lib/src/parser/parse.rs` (inline `#[cfg(test)]` module).
Scripts in `scripts/` are Fish shell utilities for headless Wayland testing with
Sway — they are experimental and not reliable yet.

## Architecture

Three-crate workspace:

- **`kanske-lib`** — core library: config parser + Wayland protocol handling.
  All meaningful logic lives here.
- **`kanske`** — daemon binary. Connects to Wayland, runs the event loop,
  applies profiles.
- **`kanskectl`** — CLI control tool, currently an empty stub.

### Parsing pipeline

`parse_file(path)` in `config_parser.rs` orchestrates the full pipeline:

```
config file  →  Lexer::tokenizer()  →  Vec<Token>  →  Parser::parse()  →  Config (AST)
```

Both steps return `ParseResult<T>` and errors are wrapped with file context via
`.into_config_error(path)` before becoming `AppResult<T>`.

### Wayland integration

`AppState` in `lib.rs` implements `Dispatch<_, ()>` for each Wayland protocol
object. The daemon runs a blocking `event_queue.blocking_dispatch()` loop and
tracks `serial` to detect hotplug events (a serial change means the output
configuration changed).

The config path is currently hardcoded as `./test.txt` in `kanske/src/main.rs`.

## Key Conventions

### Error handling

- **No `anyhow`/`thiserror`** — all errors are manual `enum` types with
  hand-written `Display` impls.
- Two error types: `KanskeError` (top-level) wrapping `ConfigParseError`
  (parser-specific).
- Two result aliases: `AppResult<T>` and `ParseResult<T>`.
- Convert parse errors to app errors with `.into_config_error(file_string)`.

### AST types (`parser/ast.rs`)

`Config` → `Vec<ConfigItem>` → `Profile` → `Vec<OutputConfig>` →
`Vec<OutputCommand>`

Output commands: `Enable`, `Disable`, `Mode { width, height, frequency }`,
`Position { x, y }`, `Scale(f32)`, `Transform(Transform)`, `AdaptiveSync(bool)`.

### Workspace dependencies

All crates inherit `version`, `edition`, `authors`, `license` from
`[workspace.package]`. Wayland crates are declared in `[workspace.dependencies]`
and referenced with `.workspace = true`.

### `edition = "2024"`

The workspace uses `edition = "2024"` — be aware this is non-standard (Rust
editions are 2015/2018/2021). Treat it as 2021 for practical purposes.
