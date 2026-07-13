//! Application-facing renderer type for browser/WASM targets.
//!
//! [`HTMLRenderer`] is the main entry point: wire it into [`CotisApp`](cotis::cotis_app::CotisApp)
//! with a layout manager and render-list pipe, then drive frames through
//! [`AsyncRenderApp::compute_frame_async`](cotis::cotis_app::AsyncRenderApp::compute_frame_async).
//!
//! Each frame:
//!
//! 1. [`CotisRendererAsync::draw_frame`] awaits the JS `waitForNextFrame` helper.
//! 2. Render commands (types implementing [`HTMLDrawable`]) are
//!    drawn via [`draw_html_render_commands`].
//!
//! Cotis context traits ([`CotisWindowContext`](cotis_utils::traits::CotisWindowContext),
//! [`CotisFrameContext`](cotis_utils::traits::CotisFrameContext), text measuring, etc.) are
//! implemented in [`crate::cotis_traits`].
//!
//! Custom DOM hosts and form reading: [`crate::custom_data`] and
//! [`HTMLRenderer::get_custom_element_html`] / [`HTMLRenderer::get_custom_element_properties`].
//!
//! Image URLs: [`crate::images`].

use crate::rendering::{HTMLDrawable, draw_html_render_commands};
use crate::web_functions::primitives::{load_font, wait_for_next_frame};
use cotis::renders::CotisRendererAsync;
use cotis_defaults::element_configs::text_config::TextConfig;
use cotis_utils::font_manager::FontManager;
use cotis_utils::math::Dimensions;

/// Reserved type alias for a text-measuring callback (currently unused).
pub type MeasureFunType = fn(&str, &TextConfig) -> Dimensions;

/// Reserved type alias for a synchronous loop callback (currently unused).
pub type LoopRoutine = Box<dyn FnMut() + Send>;

/// Async HTML/CSS renderer for Cotis applications running in the browser.
///
/// Implements [`CotisRendererAsync`] and Cotis context traits (see [`crate::cotis_traits`]).
/// Holds the page origin (`window.location.origin`) for resolving relative asset URLs.
///
/// # WASM
///
/// Requires a browser environment with a global `window` object. Construct after
/// `init_wasm()` and before starting the async frame loop.
///
/// # Example
///
/// ```rust,ignore
/// use cotis_web::renderer::HTMLRenderer;
///
/// let mut renderer = HTMLRenderer::new();
/// // Wire into CotisApp, then compute_frame_async in a loop.
/// ```
pub struct HTMLRenderer {
    _url: String,
}

impl<HTMLDrawableType: HTMLDrawable> CotisRendererAsync<HTMLDrawableType> for HTMLRenderer {
    /// Draw one frame: wait for the next animation frame, then update the DOM.
    ///
    /// Calls the JS `waitForNextFrame` helper, then runs each command through
    /// [`HTMLCanvas`](crate::rendering::HTMLCanvas) and
    /// [`draw_html_render_commands`].
    /// Unused `cotis-{id}` elements from previous frames are removed in
    /// [`HTMLCanvas::finish`](crate::rendering::HTMLCanvas::finish).
    ///
    /// # WASM
    ///
    /// Requires an initialized DOM root (see bundled `renderer.js` / `init_html_root`).
    async fn draw_frame(&mut self, render_commands: impl Iterator<Item = HTMLDrawableType>) {
        wait_for_next_frame().await;
        draw_html_render_commands(render_commands);
    }
}

impl Default for HTMLRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HTMLRenderer {
    /// Creates a renderer bound to the current page origin.
    ///
    /// Reads `window.location.origin` (e.g. `"https://example.com:8080"`) for asset URL
    /// resolution.
    ///
    /// # Panics
    ///
    /// Panics if there is no global `window` or if `location.origin()` is unavailable.
    pub fn new() -> Self {
        use web_sys::window;
        let window = window().expect("no global `window` exists");
        let location = window.location();

        let origin = location.origin().expect("could not get origin");
        Self { _url: origin }
    }

    /// Registers a font URL with the browser and assigns it an id in `manager`.
    ///
    /// Pushes `url` into `manager`, then calls the JS `loadFont` helper so CSS
    /// `font-family: font{id}` (used by text drawables) resolves correctly.
    ///
    /// # WASM
    ///
    /// Font loading is asynchronous in the browser; allow a frame or two before relying on
    /// measured text dimensions.
    pub fn load_font(&mut self, manager: &FontManager<String>, url: &str) {
        let id = manager.push_font(url.to_string());
        load_font(url, id);
    }

    /// Returns the `innerHTML` of the custom host element for the given render command id.
    ///
    /// The element id in the DOM is `cotis-{element_id}`. Returns `None` if no such element
    /// exists in the current document.
    ///
    /// # WASM
    ///
    /// Queries the live DOM; call after the frame that created or updated the element.
    pub fn get_custom_element_html(&self, element_id: u64) -> Option<String> {
        crate::web_functions::primitives::get_custom_element_html(element_id)
    }

    /// Returns JSON describing properties of a custom host element or a descendant.
    ///
    /// Use `selector` to target a child, e.g. `Some(r#"input[name="username"]"#)` to read an
    /// input's `value`. With `None`, the JS helper auto-picks the first form-like element or
    /// the host itself.
    ///
    /// Parse the returned string with [`serde_json`] to access fields like `value`,
    /// `placeholder`, `checked`, etc.
    ///
    /// # WASM
    ///
    /// Queries the live DOM; call after the frame that created or updated the element.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use cotis_web::renderer::HTMLRenderer;
    /// let renderer = HTMLRenderer::new();
    /// if let Some(json) = renderer.get_custom_element_properties(42, Some("input")) {
    ///     let props: serde_json::Value = serde_json::from_str(&json).unwrap();
    ///     let value = props["value"].as_str();
    /// }
    /// ```
    pub fn get_custom_element_properties(
        &self,
        element_id: u64,
        selector: Option<&str>,
    ) -> Option<String> {
        crate::web_functions::primitives::get_custom_element_properties(element_id, selector)
    }
}
