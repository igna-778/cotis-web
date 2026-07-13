# Contributing to cotis-web

Thank you for your interest in contributing to cotis-web. This repository contains the **browser/WASM renderer** (`cotis-web`), the **build routine** (`cotis-web-builder`), and an example application (`web-example`). The core UI framework and other render backends live in sibling repositories — see the [README](README.md#ecosystem).

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) **stable** toolchain (see `rust-toolchain.toml`)
- `rustfmt` and `clippy` components:

  ```bash
  rustup component add rustfmt clippy
  ```

- `wasm32-unknown-unknown` target:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) on PATH (for full WASM builds)

### Build and test

From the repository root:

```bash
# Format (apply changes)
cargo fmt --all

# Lint (native builder crate)
cargo clippy -p cotis-web-builder --all-targets -- -D warnings

# Test (native builder crate)
cargo test -p cotis-web-builder

# Check WASM crates
cargo check -p cotis-web -p web-example --target wasm32-unknown-unknown --all-features

# Clippy (WASM library)
cargo clippy -p cotis-web --target wasm32-unknown-unknown -- -D warnings

# Documentation
cargo doc -p cotis-web --no-deps --target wasm32-unknown-unknown --all-features
```

These are the same checks run in [CI](.github/workflows/ci.yml). Please make sure they pass locally before opening a pull request.

### Runnable example

Build the example WASM app:

```bash
cd web-example
cargo run -p cotis-web-builder-cli -- --output ../target/cotis_web
```

Serve `target/cotis_web` with a static file server to run the demo in a browser.

## Making changes

### Scope

- Keep pull requests focused. Prefer several small PRs over one large change.
- Match existing code style and naming in the crate you are editing.
- Avoid drive-by refactors unrelated to the issue you are solving.

### Code style

- Run `cargo fmt --all` before committing. Project settings are in `rustfmt.toml` (Rust 2024 edition, 100-column width).
- Address all Clippy warnings (`-D warnings` in CI).
- Prefer clear, self-documenting code. Add comments only for non-obvious design decisions.

### Tests

- Add or update tests when fixing a bug or changing behavior.
- Unit tests live next to the code they cover (`#[cfg(test)]` modules).
- The builder crate has unit tests for HTML patching; run with `cargo test -p cotis-web-builder`.

### Documentation

- Update rustdoc for public APIs you add or change.
- Update `README.md` when user-facing behavior, features, or ecosystem wiring changes.
- Add an entry to `CHANGELOG.md` under **Unreleased** for notable changes.

## Pull request process

1. Fork the repository and create a branch from `main`.
2. Make your changes and ensure CI checks pass locally.
3. Open a pull request against `main` with:
   - A clear summary of **what** changed and **why**
   - Links to related issues, if any
   - Notes on breaking changes or new feature flags
4. Maintainers will review and may request changes. Once approved, your PR will be merged.

### Commit messages

Write concise commit messages in the imperative mood, consistent with existing history:

- `feat: add gradient CSS mapping for ColorLayer`
- `fix: correct wasm import path patching on Windows`
- `docs: clarify cotis-cli install path for builder`
- `refactor: simplify resource copy in builder`

## Versioning and releases

Publishable workspace crates share the same version:

| Crate | Current version |
|-------|-----------------|
| `cotis-web` | 0.1.0-alpha |
| `cotis-web-builder` | 0.1.0-alpha |

We follow [Semantic Versioning](https://semver.org/). Pre-release versions (`-alpha`, `-beta`, etc.) may include breaking API changes — check `CHANGELOG.md` before upgrading.

Release tagging and publishing are handled by maintainers. Contributors do not need to bump versions unless asked.

## Where to report issues

| Topic | Where |
|-------|-------|
| WASM renderer, DOM draw pipeline, browser interactivity | [cotis-web issues](https://github.com/igna-778/cotis-web/issues) |
| Build routine, wasm-pack orchestration, cotis-cli plugin | [cotis-web issues](https://github.com/igna-778/cotis-web/issues) |
| Core traits, defaults, utils, macros | [cotis issues](https://github.com/igna-778/cotis/issues) |
| Layout engine | [cotis-layout](https://github.com/igna-778/cotis-layout) |
| Layout-to-render pipes | [cotis-pipes](https://github.com/igna-778/cotis-pipes) |
| CLI host, install, cache | [cotis-cli](https://github.com/igna-778/cotis-cli) |
| Security vulnerabilities | See [SECURITY.md](SECURITY.md) |

## License

By contributing, you agree that your contributions will be licensed under the same [MIT License](LICENSE) that covers this project.
