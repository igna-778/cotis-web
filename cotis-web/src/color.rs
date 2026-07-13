//! CSS color conversion helpers for HTML drawables.
//!
//! These functions turn Cotis color types into CSS color strings used by
//! [`drawable_defaults`](crate::rendering::drawable_defaults) (`background-color`, `color`, etc.).
//!
//! # Default vs feature-gated API
//!
//! Without features, all helpers take [`Color`].
//!
//! With `complex_color`, fill/background helpers take [`ColorLayer`](cotis_defaults::colors::ColorLayer).
//! With `complex_colored_text`, [`text_color_css`] takes `ColorLayer`.
//!
//! # Gradients
//!
//! When features are enabled, only [`ColorLayer::Solid`](cotis_defaults::colors::ColorLayer::Solid)
//! produces a real CSS color. Other layer variants (linear, radial, layered) currently return
//! `"transparent"`. Full CSS gradient mapping is on the roadmap.

use cotis_defaults::colors::Color;
#[cfg(any(feature = "complex_color", feature = "complex_colored_text"))]
use cotis_defaults::colors::ColorLayer;

/// CSS `background-color` (or fill) for a [`Rectangle`](cotis_defaults::render_commands::Rectangle).
#[cfg(not(feature = "complex_color"))]
pub fn rectangle_fill_css(color: &Color) -> String {
    color_css(color)
}

#[cfg(feature = "complex_color")]
pub fn rectangle_fill_css(color: &ColorLayer) -> String {
    match color {
        ColorLayer::Solid(c) => color_css(c),
        _ => "transparent".to_string(),
    }
}

/// CSS `color` for a [`Text`](cotis_defaults::render_commands::Text) drawable.
#[cfg(not(feature = "complex_colored_text"))]
pub fn text_color_css(color: &Color) -> String {
    color_css(color)
}

#[cfg(feature = "complex_colored_text")]
pub fn text_color_css(color: &ColorLayer) -> String {
    match color {
        ColorLayer::Solid(c) => color_css(c),
        _ => "transparent".to_string(),
    }
}

/// CSS background color behind an [`Image`](cotis_defaults::render_commands::Image) drawable.
#[cfg(not(feature = "complex_color"))]
pub fn image_background_css(color: &Color) -> String {
    color_css(color)
}

#[cfg(feature = "complex_color")]
pub fn image_background_css(color: &ColorLayer) -> String {
    match color {
        ColorLayer::Solid(c) => color_css(c),
        _ => "transparent".to_string(),
    }
}

/// CSS background color for custom HTML host elements.
#[cfg(not(feature = "complex_color"))]
pub fn custom_background_css(color: &Color) -> String {
    color_css(color)
}

#[cfg(feature = "complex_color")]
pub fn custom_background_css(color: &ColorLayer) -> String {
    match color {
        ColorLayer::Solid(c) => color_css(c),
        _ => "transparent".to_string(),
    }
}

/// Normalizes alpha to the 0.0–1.0 range expected by CSS `rgba()`.
///
/// Values greater than `1.0` are treated as 0–255 byte alpha and divided by 255.
/// Values in `0.0..=1.0` are clamped to that range.
///
/// # Examples
///
/// ```
/// use cotis_web::color::css_alpha;
///
/// assert!((css_alpha(0.5) - 0.5).abs() < f32::EPSILON);
/// assert!((css_alpha(128.0) - 128.0 / 255.0).abs() < 0.001);
/// assert!((css_alpha(300.0) - 1.0).abs() < f32::EPSILON);
/// ```
pub fn css_alpha(a: f32) -> f32 {
    if a > 1.0 {
        (a / 255.0).clamp(0.0, 1.0)
    } else {
        a.clamp(0.0, 1.0)
    }
}

/// Formats a Cotis [`Color`] as a CSS `rgba(r, g, b, a)` string.
///
/// RGB components are passed through as-is (typically 0–255). Alpha is normalized via
/// [`css_alpha`].
///
/// # Examples
///
/// ```
/// use cotis_defaults::colors::Color;
/// use cotis_web::color::color_css;
///
/// let c = Color::rgba(255.0, 0.0, 128.0, 255.0);
/// assert_eq!(color_css(&c), "rgba(255, 0, 128, 1)");
/// ```
pub fn color_css(c: &Color) -> String {
    format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, css_alpha(c.a))
}
