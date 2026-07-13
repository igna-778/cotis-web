//! Image URL types for web rendering.
//!
//! Image drawables ([`cotis_defaults::render_commands::Image`]) require an associated type that
//! implements [`URLImage`]. The renderer uses [`URLImage::url`] to set CSS
//! `background-image: url("...")`.
//!
//! [`URLImageEnum`] is the usual choice for [`ElementConfig`](cotis_defaults::element_configs::ElementConfig)
//! image type parameters in WASM apps.
//!
//! # Path conventions
//!
//! URLs are passed to the browser as-is. Use paths relative to the page origin (e.g.
//! `"assets/logo.png"`) or absolute URLs. The renderer does not prepend
//! [`HTMLRenderer::new`](crate::renderer::HTMLRenderer::new)'s stored origin automatically today.
//!
//! # API status
//!
//! The image URL API is **incomplete** (`// TODO Finish this` in source). Current behavior
//! delegates to `get_path()` on [`PNGImage`],
//! [`JPEGImage`], and
//! [`SVGImage`]. Blob URLs, data URLs, and async
//! loading are not yet handled.

use cotis_defaults::generic_image::{JPEGImage, PNGImage, SVGImage};

/// Types that can provide a URL string for CSS `background-image`.
pub trait URLImage {
    /// Returns the URL or path used in `background-image: url(...)`.
    fn url(&self) -> &str;
}

impl URLImage for PNGImage {
    fn url(&self) -> &str {
        self.get_path()
    }
}

impl URLImage for JPEGImage {
    fn url(&self) -> &str {
        self.get_path()
    }
}

impl URLImage for SVGImage {
    fn url(&self) -> &str {
        self.get_path()
    }
}

/// Sum type over the standard Cotis image formats for web apps.
pub enum URLImageEnum {
    /// PNG image path or URL.
    PNG(PNGImage),
    /// JPEG image path or URL.
    JPEG(JPEGImage),
    /// SVG image path or URL.
    SVG(SVGImage),
}

impl URLImage for URLImageEnum {
    fn url(&self) -> &str {
        match self {
            URLImageEnum::PNG(s) => s.url(),
            URLImageEnum::JPEG(s) => s.url(),
            URLImageEnum::SVG(s) => s.url(),
        }
    }
}
