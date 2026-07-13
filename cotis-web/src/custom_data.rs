//! Custom HTML host data for render commands.
//!
//! Attach [`ExtraHTMLData`] as the `extra_data` type parameter on
//! [`ElementConfig`](cotis_defaults::element_configs::ElementConfig) and render commands
//! ([`Rectangle`](cotis_defaults::render_commands::Rectangle),
//! [`Text`](cotis_defaults::render_commands::Text), etc.). During drawing,
//! [`OptionalExtraHTMLData::extra_html_data`] is read and passed to
//! [`HTMLCanvas::ensure_command_element`](crate::rendering::HTMLCanvas::ensure_command_element),
//! which sets the DOM tag name and optional inline style on the `cotis-{id}` host element.
//!
//! For arbitrary HTML fragments inside a host, use [`HTMLElement`] with custom draw logic
//! or [`HTMLRenderer::get_custom_element_html`](crate::renderer::HTMLRenderer::get_custom_element_html).

#[allow(dead_code)]
pub(crate) trait IntoHTMLElement {
    fn html_element(&self) -> Option<HTMLElement>;
}

impl IntoHTMLElement for () {
    fn html_element(&self) -> Option<HTMLElement> {
        None
    }
}

impl IntoHTMLElement for HTMLElement {
    fn html_element(&self) -> Option<HTMLElement> {
        Some(self.clone())
    }
}

/// Raw HTML content for a custom host element's `innerHTML`.
#[derive(Debug, Clone)]
pub struct HTMLElement {
    html: String,
}

impl HTMLElement {
    /// Creates a wrapper around an HTML fragment string.
    ///
    /// # Examples
    ///
    /// ```
    /// use cotis_web::custom_data::HTMLElement;
    ///
    /// let el = HTMLElement::new("<span>Hello</span>");
    /// assert_eq!(el.html(), "<span>Hello</span>");
    /// ```
    pub fn new(html: &str) -> HTMLElement {
        Self {
            html: html.to_string(),
        }
    }

    /// Returns the stored HTML fragment.
    pub fn html(&self) -> &str {
        &self.html
    }
}

/// Types that may carry [`ExtraHTMLData`] on a render command's `extra_data` field.
pub trait OptionalExtraHTMLData {
    /// Returns host tag/style overrides, or `None` for default `div` styling.
    fn extra_html_data(&self) -> Option<ExtraHTMLData>;
}

impl OptionalExtraHTMLData for () {
    fn extra_html_data(&self) -> Option<ExtraHTMLData> {
        None
    }
}

impl OptionalExtraHTMLData for ExtraHTMLData {
    fn extra_html_data(&self) -> Option<ExtraHTMLData> {
        Some(self.clone())
    }
}

/// HTML tag name for a render command's DOM host element (`cotis-{id}`).
///
/// Used via [`ExtraHTMLData::tag`] on render command
/// [`RenderCommandInfo::extra_data`](cotis_defaults::render_commands::RenderCommandInfo::extra_data).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CustomHtmlElementTag {
    /// Standard block container (`<div>`).
    #[default]
    Div,
    /// Clickable button host (`<button>`).
    Button,
}

impl CustomHtmlElementTag {
    /// Returns the lowercase HTML tag name for this variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use cotis_web::custom_data::CustomHtmlElementTag;
    ///
    /// assert_eq!(CustomHtmlElementTag::Div.as_json_str(), "div");
    /// assert_eq!(CustomHtmlElementTag::Button.as_json_str(), "button");
    /// ```
    pub const fn as_json_str(self) -> &'static str {
        match self {
            Self::Div => "div",
            Self::Button => "button",
        }
    }
}

/// Per-command overrides for the DOM host element tag and inline CSS.
#[derive(Clone, Debug, Default)]
pub struct ExtraHTMLData {
    /// HTML tag for the host element (default [`CustomHtmlElementTag::Div`]).
    pub tag: CustomHtmlElementTag,
    /// Optional inline `style` attribute value applied before drawable CSS pushes.
    pub extra_style: Option<String>,
}
