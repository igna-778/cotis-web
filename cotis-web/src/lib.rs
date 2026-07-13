//! Browser/WASM render backend for the [Cotis](https://docs.rs/cotis) UI framework.
//!
//! `cotis-web` implements [`CotisRendererAsync`](cotis::renders::CotisRendererAsync) for
//! HTML/CSS output. Layout output from [`cotis-layout`](https://docs.rs/cotis-layout) is
//! converted to [`cotis_defaults::render_commands`](https://docs.rs/cotis-defaults) draw
//! commands by [`cotis-pipes`](https://docs.rs/cotis-pipes), then mapped to DOM elements
//! via CSS absolute positioning and a companion JavaScript module (`renderer.js`).
//!
//! # Stack
//!
//! - [`wasm-bindgen`](https://docs.rs/wasm-bindgen) / [`web-sys`](https://docs.rs/web-sys) —
//!   Rust ↔ browser FFI
//! - [`serde_json`](https://docs.rs/serde_json) — custom-element property serialization
//! - [`renderer.js`](https://github.com/igna-778/cotis-web) — frame timing, input, DOM host
//!   management (loaded via `wasm-bindgen` `module =` attribute)
//!
//! This crate is **async-only**: use [`HTMLRenderer`](renderer::HTMLRenderer) with
//! [`AsyncRenderApp`](cotis::cotis_app::AsyncRenderApp), not the sync
//! [`CotisRenderer`](cotis::renders::CotisRenderer) trait.
//!
//! # DOM model
//!
//! Each render command id maps to one host element with id `cotis-{id}`. Drawables accumulate
//! CSS on that node via [`HTMLCanvas::ensure_command_element`](rendering::HTMLCanvas::ensure_command_element)
//! and [`HTMLCanvas::current_element_css_push`](rendering::HTMLCanvas::current_element_css_push).
//! At the end of a frame, [`HTMLCanvas::finish`](rendering::HTMLCanvas::finish) removes DOM
//! nodes that were not used.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use cotis::cotis_app::{AsyncRenderApp, CotisApp};
//! use cotis_layout::preamble::CotisLayoutManager;
//! use cotis_pipes::cotis_layout_pipes::CotisLayoutToRenderListPipeForGenerics;
//! use cotis_utils::math::Dimensions;
//! use cotis_web::renderer::HTMLRenderer;
//!
//! let renderer = HTMLRenderer::new();
//! let manager = CotisLayoutManager::new(Dimensions::new(800.0, 800.0));
//! let mut app = CotisApp::new(renderer, manager, CotisLayoutToRenderListPipeForGenerics);
//! // AsyncRenderApp::compute_frame_async(&mut app, |layout| { ... }).await;
//! ```
//!
//! See [`web-example`](../web-example/src/lib.rs) for a complete UI tree with custom HTML,
//! images, and form state reading.
//!
//! # WASM bootstrap
//!
//! Typical page load sequence (see bundled `resources/on-root/index.html`):
//!
//! 1. `await init()` — load the `.wasm` binary (wasm-bindgen glue).
//! 2. `init_wasm()` — initialize logging and a panic hook (wasm export from this crate).
//! 3. Call an app entry export:
//!    - **`main_run`** — WASM export in the app crate (e.g. [`web-example`](../web-example/src/lib.rs))
//!      that calls [`cotis_launch_async`](cotis::launch::cotis_launch_async) after `init_wasm()`.
//!      Requires the `app_launch` feature on `cotis` / `cotis-web`.
//!
//! # Build workflow
//!
//! Use the [`cotis-web-builder`](../cotis-web-builder/README.md) crate / CLI plugin to run
//! `wasm-pack`, copy `renderer.js` and `index.html`, and patch the WASM import path.
//!
//! # Modules
//!
//! | Module | Visibility | Role |
//! |--------|------------|------|
//! | [`renderer`] | public | [`HTMLRenderer`](renderer::HTMLRenderer) — main app-facing type |
//! | [`rendering`] | public | [`HTMLCanvas`](rendering::HTMLCanvas), [`HTMLDrawable`](rendering::HTMLDrawable), draw pipeline |
//! | [`images`] | public | [`URLImage`](images::URLImage) trait and [`URLImageEnum`](images::URLImageEnum) for CSS `background-image` |
//! | [`color`] | public | Cotis [`Color`](cotis_defaults::colors::Color) → CSS string helpers |
//! | [`custom_data`] | public | [`ExtraHTMLData`](custom_data::ExtraHTMLData) for custom DOM tags and styles |
//! | [`cotis_traits`] | public | Cotis context trait impls on [`HTMLRenderer`](renderer::HTMLRenderer) |
//! | `start_up` | public | `init`, `main_run` helpers for WASM app crates |
//! | `interactivity` | crate-private | Mouse/keyboard providers for the browser |
//! | `web_functions` | crate-private | Low-level JS FFI bindings |
//!
//! # Custom HTML and images
//!
//! Attach [`ExtraHTMLData`](custom_data::ExtraHTMLData) as the `extra_data` generic on
//! [`ElementConfig`](cotis_defaults::element_configs::ElementConfig) / render commands to
//! control the host element tag (`div`, `button`, …) and optional inline CSS.
//!
//! Use [`URLImageEnum`](images::URLImageEnum) as the image type parameter and implement
//! [`URLImage`](images::URLImage) (or use the provided enum) so image drawables emit
//! `background-image: url(...)`.
//!
//! Read back form state with [`HTMLRenderer::get_custom_element_html`](renderer::HTMLRenderer::get_custom_element_html) and
//! [`HTMLRenderer::get_custom_element_properties`](renderer::HTMLRenderer::get_custom_element_properties).
//!
//! Load fonts with [`HTMLRenderer::load_font`](renderer::HTMLRenderer::load_font) and a [`FontManager`](cotis_utils::font_manager::FontManager).
//!
//! # Features
//!
//! By default, only solid [`Color`](cotis_defaults::colors::Color) values are supported.
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `complex_color` | Background/fill colors become [`ColorLayer`](cotis_defaults::colors::ColorLayer) |
//! | `complex_colored_text` | Text colors become `ColorLayer` (implies `complex_color` in `cotis-defaults`) |
//!
//! Enable via Cargo, e.g. `cotis-web = { path = "...", features = ["complex_color"] }`.
//!
//! **Gradient roadmap:** when features are enabled, non-`Solid` [`ColorLayer`](cotis_defaults::colors::ColorLayer)
//! variants currently map to `"transparent"` in the Rust draw path. Full CSS gradient
//! support is planned; the legacy JSON `draw_frame` path in `renderer.js` already handles
//! gradients but is not used by [`HTMLRenderer`](renderer::HTMLRenderer).
//!
//! # Related crates (not part of this crate)
//!
//! - [`cotis-web-builder`](../cotis-web-builder) — WASM build routine
//! - [`web-example`](../web-example) — full example application

pub(crate) mod web_functions;

pub mod renderer;

pub mod rendering;

pub mod start_up;

pub mod images;

pub mod color;

pub mod custom_data;

pub mod cotis_traits;

#[allow(dead_code)]
pub(crate) mod interactivity;
