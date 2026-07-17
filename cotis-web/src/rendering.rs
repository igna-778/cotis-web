//! DOM canvas and drawable trait for mapping Cotis render commands to HTML/CSS.
//!
//! The typical app path is [`HTMLRenderer::draw_frame`](crate::renderer::HTMLRenderer) →
//! [`draw_html_render_commands`], which creates an [`HTMLCanvas`], draws each
//! [`HTMLDrawable`] command, then calls [`HTMLCanvas::finish`].
//!
//! Default drawables for [`cotis_defaults::render_commands`]
//! live in [`drawable_defaults`].
//!
//! ## Frame lifecycle
//!
//! 1. [`HTMLCanvas::new`] → JS `beginFrame`
//! 2. For each command: [`HTMLCanvas::ensure_command_element`] + CSS pushes (or clip stack ops)
//! 3. [`HTMLCanvas::finish`] → finalize DOM nodes → JS `endFrame` removes unused `cotis-*` ids
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use cotis_web::rendering::{HTMLCanvas, HTMLDrawable};
//!
//! let mut canvas = HTMLCanvas::new();
//! canvas
//!     .ensure_command_element(1, "div", None)
//!     .current_element_css_push("position: absolute; left: 10px; top: 10px")
//!     .current_element_css_push("width: 100px; height: 50px")
//!     .current_element_css_push("background-color: red; border-radius: 5px");
//! canvas.finish();
//! ```
//!
//! ## Core API
//!
//! - [`HTMLCanvas::ensure_command_element`] — open or reuse host for a command id (`cotis-{id}`)
//! - [`HTMLCanvas::new_element`] — ad-hoc child element with generated id
//! - [`HTMLCanvas::current_element_css_push`] — append/overwrite CSS on current host
//! - [`HTMLCanvas::finish`] — commit frame and remove stale DOM nodes

pub mod drawable_defaults;

use crate::web_functions::primitives::{
    append_to_current_container, begin_frame, end_frame, get_or_create_host_element,
    scissor_stack_pop, scissor_stack_push,
};
use cotis_defaults::render_commands::CornerRadii;
use js_sys::Set;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// Per-frame DOM builder: creates/updates `cotis-{id}` elements and manages clip stacks.
///
/// One instance should be created per frame via [`HTMLCanvas::new`] and consumed with
/// [`HTMLCanvas::finish`]. Commands that share a render id reuse the same host element.
pub struct HTMLCanvas {
    // Tracks all used element IDs for cleanup
    used_element_ids: Set,

    // Current element being built
    current_element: Option<ElementBuilder>,

    /// When set, `current_element` belongs to this render command id (`cotis-{id}` in the DOM).
    active_command_element_id: Option<u128>,

    // Counter for generating unique element IDs (e.g. clip containers)
    element_counter: usize,
}

struct ElementBuilder {
    id: String,
    tag_name: String,
    extra_style: Option<String>,
    css_properties: HashMap<String, String>,
    text_content: Option<String>,
    inner_html: Option<String>,
}

impl Default for HTMLCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl HTMLCanvas {
    /// Starts a new rendering frame (calls JS `beginFrame`).
    ///
    /// # WASM
    ///
    /// Requires an initialized HTML root (`init_html_root` in `renderer.js`).
    pub fn new() -> Self {
        begin_frame();
        Self {
            used_element_ids: Set::new(&JsValue::NULL),
            current_element: None,
            active_command_element_id: None,
            element_counter: 0,
        }
    }

    /// Opens or reuses the host element for a render command. Multiple drawables that share the
    /// same `command_id` accumulate `current_element_css_push` on one DOM node (`cotis-{id}`).
    pub fn ensure_command_element(
        &mut self,
        command_id: u128,
        html_type: &str,
        extra_style: Option<&str>,
    ) {
        if self.active_command_element_id == Some(command_id) {
            return;
        }
        self.finalize_current_element();
        self.active_command_element_id = Some(command_id);
        let dom_id = format!("cotis-{}", command_id);
        self.current_element = Some(ElementBuilder {
            id: dom_id,
            tag_name: html_type.to_string(),
            extra_style: extra_style.map(String::from),
            css_properties: HashMap::new(),
            text_content: None,
            inner_html: None,
        });
    }

    /// Add CSS properties to the current element. Properties are parsed from CSS syntax.
    /// If a property already exists, it will be overwritten.
    /// Example: "background-color: red; padding: 10px"
    pub fn current_element_css_push(&mut self, value: &str) -> &mut Self {
        if let Some(builder) = &mut self.current_element {
            // Parse CSS string and add to properties map
            for declaration in value.split(';') {
                let declaration = declaration.trim();
                if declaration.is_empty() {
                    continue;
                }

                if let Some((prop, val)) = declaration.split_once(':') {
                    let prop = prop.trim().to_string();
                    let val = val.trim().to_string();
                    builder.css_properties.insert(prop, val);
                }
            }
        }
        self
    }

    /// Set the text content of the current element (internal use)
    pub fn set_text_content(&mut self, text: String) {
        if let Some(builder) = &mut self.current_element {
            builder.text_content = Some(text);
        }
    }

    /// Set `innerHTML` for the current element (e.g. custom HTML hosts)
    pub fn set_inner_html(&mut self, html: String) {
        if let Some(builder) = &mut self.current_element {
            builder.inner_html = Some(html);
        }
    }

    /// Create a new child element. This finalizes the current element and starts building a new one.
    ///
    /// # Arguments
    /// * `prefix` - A prefix for the element ID (for debugging/identification)
    /// * `html_type` - The HTML tag name (e.g., "div", "button", "span")
    pub fn new_element(&mut self, prefix: &str, html_type: &str) -> &mut Self {
        // Finalize the current element if one exists
        self.finalize_current_element();

        // Start building a new element
        self.element_counter += 1;
        let id = format!("{}_{}", prefix, self.element_counter);

        self.current_element = Some(ElementBuilder {
            id,
            tag_name: html_type.to_string(),
            extra_style: None,
            css_properties: HashMap::new(),
            text_content: None,
            inner_html: None,
        });

        self
    }

    /// Finalize the current element by creating/updating the DOM element and appending it
    fn finalize_current_element(&mut self) {
        if let Some(builder) = self.current_element.take() {
            self.active_command_element_id = None;
            // Collect CSS properties
            let css_map = builder.css_properties.clone();
            let text_content = builder.text_content.clone();
            let inner_html = builder.inner_html.clone();
            let id_str = builder.id.clone();

            // Create closure to apply styles
            let apply_style_fn = Closure::wrap(Box::new(move |el: web_sys::Element| {
                if let Some(html) = &inner_html {
                    if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                        html_el.set_inner_html(html);
                    }
                } else if let Some(text) = &text_content {
                    el.set_text_content(Some(text));
                }

                // Apply CSS properties
                if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                    let style = html_el.style();
                    for (prop, value) in &css_map {
                        let _ = style.set_property(prop, value);
                    }
                }
            }) as Box<dyn FnMut(web_sys::Element)>);

            // Get or create the element
            let el = get_or_create_host_element(
                &builder.id,
                &builder.tag_name,
                builder.extra_style.as_deref(),
                apply_style_fn.as_ref().unchecked_ref(),
            );

            // Append to current container (handles scissor stack)
            append_to_current_container(&el);

            // Track this element ID for cleanup
            self.used_element_ids.add(&JsValue::from_str(&id_str));

            // Clean up the closure
            apply_style_fn.forget();
        }
    }

    /// Commits the frame and removes DOM nodes not referenced in `used_element_ids`.
    ///
    /// Consumes `self`. Must be called once per frame after all drawables have run.
    pub fn finish(mut self) {
        // Finalize any remaining element
        self.finalize_current_element();

        // End frame and cleanup unused elements
        end_frame(&self.used_element_ids);
    }

    /// Pushes a clip region ([`ClipStart`](cotis_defaults::render_commands::ClipStart)).
    ///
    /// Creates an overflow-hidden container and pushes it onto the JS scissor stack so
    /// subsequent elements are clipped. Used by [`drawable_defaults`](drawable_defaults).
    pub(crate) fn push_scissor(
        &mut self,
        command_id: u128,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        corner_radii: CornerRadii,
    ) {
        // Finalize current element first
        self.finalize_current_element();

        let id = format!("cotis-{}", command_id);

        let container = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        container.set_id(&id);

        let style = container.dyn_ref::<web_sys::HtmlElement>().unwrap().style();
        let _ = style.set_property("position", "absolute");
        let _ = style.set_property("left", &format!("{}px", x));
        let _ = style.set_property("top", &format!("{}px", y));
        let _ = style.set_property("width", &format!("{}px", width));
        let _ = style.set_property("height", &format!("{}px", height));
        let _ = style.set_property("overflow", "hidden");
        let _ = style.set_property(
            "border-radius",
            &format!(
                "{}px {}px {}px {}px",
                corner_radii.top_left,
                corner_radii.top_right,
                corner_radii.bottom_right,
                corner_radii.bottom_left
            ),
        );
        let _ = style.set_property("box-sizing", "border-box");

        append_to_current_container(&container);
        scissor_stack_push(&container, x as f64, y as f64);
        self.used_element_ids.add(&JsValue::from_str(&id));
    }

    /// Pops the top clip region ([`ClipEnd`](cotis_defaults::render_commands::ClipEnd)).
    pub(crate) fn pop_scissor(&mut self) {
        // Finalize current element first
        self.finalize_current_element();

        scissor_stack_pop();
    }
}

/// Types that can draw themselves onto an [`HTMLCanvas`] (typically `cotis-defaults` commands).
pub trait HTMLDrawable {
    /// Renders this command onto `canvas`, usually via `ensure_command_element` and CSS pushes.
    fn draw(self, canvas: &mut HTMLCanvas);
}

/// Renders an iterator of drawables as one DOM frame.
///
/// Creates an [`HTMLCanvas`], draws each item (e.g. a [`RenderTList`](cotis_defaults::render_commands::render_t_list::RenderTList)
/// from layout output), then calls [`HTMLCanvas::finish`].
pub fn draw_html_render_commands(commands: impl Iterator<Item = impl HTMLDrawable>) {
    let mut canvas = HTMLCanvas::new();
    for cmd in commands {
        cmd.draw(&mut canvas);
    }
    canvas.finish();
}
