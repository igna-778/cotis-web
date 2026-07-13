//! Cotis context trait implementations for [`HTMLRenderer`].
//!
//! These traits connect the renderer to the Cotis app loop:
//!
//! | Trait | Provides |
//! |-------|----------|
//! | [`RendererTextMeasuringProvider`] | Browser-based text measurement via `renderer.js` |
//! | [`CotisWindowContext`] | Viewport size in pixels |
//! | [`CotisFrameContext`] | Frame delta time in **seconds** |
//! | [`cotis_utils::traits::CotisRenderContext`] | Provided automatically via blanket impl when window + frame traits are implemented |
//! | [`RenderCompatibleWith`] | No-op layout init hook |
//!
//! Mouse input is implemented on [`HTMLRenderer`] directly (see the `interactivity` module).
//! Keyboard state is available through `HTMLInteractivity` in the `interactivity` module
//! ([`SimpleKeyboardProvider`](cotis_utils::interactivity::keyboard::SimpleKeyboardProvider)).

use crate::renderer::HTMLRenderer;
use crate::web_functions::primitives::{
    get_delta_time_ms, text_measuring_function, window_dimensions,
};
use cotis::renders::RenderCompatibleWith;
use cotis_defaults::element_configs::text_config::TextConfig;
use cotis_utils::math::Dimensions;
use cotis_utils::text::RendererTextMeasuringProvider;
use cotis_utils::traits::{CotisFrameContext, CotisWindowContext};

impl RendererTextMeasuringProvider<TextConfig> for HTMLRenderer {
    /// Returns a measurer that uses off-screen DOM layout in `renderer.js`.
    ///
    /// Respects `TextConfig` font id, size, line height, and letter spacing.
    fn provide_measurer(&mut self) -> Box<dyn Fn(&str, &TextConfig) -> Dimensions> {
        Box::new(text_measuring_function)
    }
}

impl CotisWindowContext for HTMLRenderer {
    /// Returns the browser viewport size `(width, height)` in CSS pixels.
    fn window_dimensions(&self) -> Dimensions {
        let array = window_dimensions();
        Dimensions {
            width: array.0,
            height: array.1,
        }
    }
}

impl CotisFrameContext for HTMLRenderer {
    /// Returns elapsed time since the previous frame, in **seconds**.
    ///
    /// Sourced from JS `get_delta_time_ms()` (milliseconds) and converted to seconds.
    fn get_delta_time(&self) -> f32 {
        get_delta_time_ms()
    }
}

impl<M> RenderCompatibleWith<M> for HTMLRenderer {
    /// No-op: the HTML renderer does not require layout-manager initialization.
    fn init(&mut self, _layout_manager: &mut M) {}
}
