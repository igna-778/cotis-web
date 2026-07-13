# cotis-web

Browser/WASM render backend for the [Cotis](https://github.com/igna-778/cotis) UI framework. `cotis-web` implements [`CotisRendererAsync`](https://docs.rs/cotis/latest/cotis/renders/trait.CotisRendererAsync.html) for HTML/CSS output. Layout output from [`cotis-layout`](https://github.com/igna-778/cotis-layout) is converted to render commands by [`cotis-pipes`](https://github.com/igna-778/cotis-pipes), then mapped to DOM elements via CSS absolute positioning and a companion JavaScript module (`renderer.js`).

> **Status:** Early `0.1.0-alpha` release. APIs are still evolving.

## Workspace crates

| Crate | Role |
|-------|------|
| [`cotis-web`](https://docs.rs/cotis-web) | WASM renderer backend — `HTMLRenderer`, DOM draw pipeline, browser interactivity |
| [`cotis-web-builder`](https://docs.rs/cotis-web-builder) | `wasm-pack` build routine and [`cotis-cli`](https://github.com/igna-778/cotis-cli) plugin |
| `web-example` | Full example WASM application (not published) |

## Prerequisites

- [Rust](https://rustup.rs/) **stable** toolchain (see `rust-toolchain.toml`)
- `wasm32-unknown-unknown` target:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) on PATH (for building WASM output)

## Installation

Add the renderer crate to your `Cargo.toml`:

```toml
cotis-web = "0.1.0-alpha"
```

A full browser application also needs layout, pipes, and a WASM app crate. See [Quick start](#quick-start).

## Quick start

Build and serve the bundled example from the repository:

```bash
git clone https://github.com/igna-778/cotis-web
cd cotis-web/web-example
cargo run -p cotis-web-builder-cli -- --output ../target/cotis_web
```

Then serve the output directory (contains `index.html`, `pkg/`, and `web-renderer/renderer.js`) with any static file server.

### With cotis-cli

Install the build routine as a plugin:

```bash
cotis-cli install path:/path/to/cotis-web/cotis-web-builder
cotis-cli run cotis-web-builder -- --output ./target/cotis_web
```

Run from your WASM app crate directory (where `Cargo.toml` defines the cdylib).

### Typical app wiring

```rust
use cotis::cotis_app::{AsyncRenderApp, CotisApp};
use cotis_layout::preamble::CotisLayoutManager;
use cotis_pipes::cotis_layout_pipes::CotisLayoutToRenderListPipeForGenerics;
use cotis_utils::math::Dimensions;
use cotis_web::renderer::HTMLRenderer;

let renderer = HTMLRenderer::new();
let manager = CotisLayoutManager::new(Dimensions::new(800.0, 800.0));
let mut app = CotisApp::new(renderer, manager, CotisLayoutToRenderListPipeForGenerics);
// AsyncRenderApp::compute_frame_async(&mut app, |layout| { ... }).await;
```

See [`web-example/src/lib.rs`](web-example/src/lib.rs) for a complete UI tree with custom HTML, images, and form state.

## Features

### `cotis-web`

- `app_launch` — WASM export helpers via `cotis` launch hooks (enable on `cotis` as well)
- `complex_color` — layered and gradient color support (forwards to `cotis-defaults`)
- `complex_colored_text` — extends `complex_color` to text color fields

## Ecosystem

Cotis is split across several repositories. This repo provides the browser renderer and build tooling.

| Repository | Role |
|------------|------|
| [`cotis`](https://github.com/igna-778/cotis) | Core traits and `CotisApp` orchestration |
| [**cotis-web**](https://github.com/igna-778/cotis-web) (this repo) | Browser/WASM renderer and build routine |
| [`cotis-layout`](https://github.com/igna-778/cotis-layout) | Flexbox-style layout engine |
| [`cotis-pipes`](https://github.com/igna-778/cotis-pipes) | Layout output to render commands |
| [`cotis-cli`](https://github.com/igna-778/cotis-cli) | Plugin host for installable build and run routines |
| [`cotis-wgpu`](https://github.com/igna-778/cotis-wgpu) | Desktop renderer (wgpu + winit + glyphon) |
| [`cotis-raylib`](https://github.com/igna-778/cotis-raylib) | Desktop renderer (raylib) |

## Documentation

- API reference: `cargo doc --open -p cotis-web --target wasm32-unknown-unknown`
- Published docs: [cotis-web](https://docs.rs/cotis-web), [cotis-web-builder](https://docs.rs/cotis-web-builder)
- Contributing: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security: [SECURITY.md](SECURITY.md)
- License: [MIT](LICENSE)
