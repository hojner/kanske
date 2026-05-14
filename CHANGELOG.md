# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-12

### Added

- Full config parser pipeline: lexer, AST, profile parsing with `include`
  (glob, tilde expansion, recursive up to depth 10) and `exec` directives
- Wayland output manager integration via `wlr-output-management` protocol
- Profile composer: merges global output defaults into profiles
- Profile matcher: head count, named output, and wildcard matching
- Profile applier: mode, position, scale, transform, adaptive sync,
  enable/disable
- calloop-driven event loop multiplexing Wayland events and Unix signals
- SIGINT/SIGTERM clean shutdown with RAII PID file cleanup
- SIGHUP config reload with immediate re-apply
- PID file at `$XDG_RUNTIME_DIR/kanske.pid`
- `kanskectl` CLI with `list` (connected outputs) and `reload` subcommands
- Config file safety: `notify-send` warning on bad file permissions
- Default config creation on first run
- Proper line/column numbers in all parse error messages

[Unreleased]: https://github.com/hojner/kanske/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hojner/kanske/releases/tag/v0.1.0
