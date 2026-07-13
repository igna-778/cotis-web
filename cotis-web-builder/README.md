# cotis-web-builder

Web/WASM build routine for Cotis.

This crate can be used in two ways:

- **As a cotis-cli routine plugin** (dynamic library loaded by `cotis-cli`)
- **Standalone** (run its binary directly)

See the [repository README](../README.md) for prerequisites and ecosystem links.

## Use with `cotis-cli` (plugin)

Install from a local checkout:

```bash
cotis-cli install path:/path/to/cotis-web/cotis-web-builder
```

Show routine help:

```bash
cotis-cli help-routine cotis-web-builder
```

Run (example output folder):

```bash
cotis-cli run cotis-web-builder -- --output ./target/cotis_web_out

# with feature forwarding to wasm-pack/cargo:
cotis-cli run cotis-web-builder -- --features complex_colored_text,complex_color
```

Run from your WASM app crate directory (where `Cargo.toml` defines the cdylib).

## Standalone usage

Build and run directly from your WASM app crate directory:

```bash
cargo run -p cotis-web-builder-cli -- --output ./target/cotis_web_out

# with features:
cargo run -p cotis-web-builder-cli -- --features complex_colored_text,complex_color
```

Help:

```bash
cargo run -p cotis-web-builder-cli -- --help
```

## Notes / requirements

- Requires `wasm-pack` available on PATH for building.
- On first run it will `git clone` `cotis-web` into the OS cache directory to fetch resources.
