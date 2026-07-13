//! Default [`HTMLDrawable`] implementations for
//! [`cotis_defaults::render_commands`] types.
//!
//! These impls map layout output to CSS on shared DOM host elements (`cotis-{id}`).
//! Multiple drawables with the same command id accumulate styles on one node via
//! [`HTMLCanvas::ensure_command_element`](crate::rendering::HTMLCanvas::ensure_command_element).
//!
//! # Supported commands
//!
//! | Command | DOM host | Key CSS |
//! |---------|----------|---------|
//! | [`Rectangle`] | `cotis-{id}`, tag from [`ExtraHTMLData`](crate::custom_data::ExtraHTMLData) | `position:absolute`, `background-color`, `border-radius` |
//! | [`Border`] | same | per-side `border-*: Npx solid` |
//! | [`Text`] | same | `font-family: font{id}`, `color`, flex, `white-space: pre-wrap` |
//! | [`Image`] | same | `background-image: url(...)`, `background-size: 100% 100%` |
//! | [`ClipStart`] / [`ClipEnd`] | clip container stack | `overflow: hidden`, rounded clip |
//! | [`RenderTList`] | (recursive) | dispatches `Type` / `List` variants |

use crate::color::*;
use crate::custom_data::OptionalExtraHTMLData;
use crate::images::URLImage;
use crate::rendering::{HTMLCanvas, HTMLDrawable};
use cotis_defaults::render_commands::render_t_list::{IsRenderList, RenderTList};
use cotis_defaults::render_commands::*;

impl<T: HTMLDrawable, L: IsRenderList + HTMLDrawable> HTMLDrawable for RenderTList<T, L> {
    fn draw(self, canvas: &mut HTMLCanvas) {
        match self {
            RenderTList::Type(t) => {
                t.draw(canvas);
            }
            RenderTList::List(l) => {
                l.draw(canvas);
            }
        }
    }
}

impl<T: HTMLDrawable> HTMLDrawable for RenderTList<T, ()> {
    fn draw(self, canvas: &mut HTMLCanvas) {
        match self {
            RenderTList::Type(t) => t.draw(canvas),
            RenderTList::List(()) => {}
        }
    }
}

impl<'a, ExtraData: OptionalExtraHTMLData> HTMLDrawable for Rectangle<'a, ExtraData> {
    fn draw(self, canvas: &mut HTMLCanvas) {
        let extra_data = match self.info.extra_data {
            None => Default::default(),
            Some(extra) => extra.as_ref().extra_html_data().unwrap_or_default(),
        };
        canvas.ensure_command_element(
            self.info.id,
            extra_data.tag.as_json_str(),
            extra_data.extra_style.as_deref(),
        );
        let bg = rectangle_fill_css(&self.color);
        canvas
            .current_element_css_push(&format!(
                "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px",
                self.info.bounding_box.x,
                self.info.bounding_box.y,
                self.info.bounding_box.width,
                self.info.bounding_box.height
            ))
            .current_element_css_push(&format!("background-color: {}", bg))
            .current_element_css_push(&format!(
                "border-radius: {}px {}px {}px {}px",
                self.corner_radii.top_left,
                self.corner_radii.top_right,
                self.corner_radii.bottom_right,
                self.corner_radii.bottom_left
            ))
            .current_element_css_push("box-sizing: border-box; pointer-events: auto");
    }
}

impl<'a, ExtraData: OptionalExtraHTMLData> HTMLDrawable for Border<'a, ExtraData> {
    fn draw(self, canvas: &mut HTMLCanvas) {
        let extra_data = match self.info.extra_data {
            None => Default::default(),
            Some(extra) => extra.as_ref().extra_html_data().unwrap_or_default(),
        };
        canvas.ensure_command_element(
            self.info.id,
            extra_data.tag.as_json_str(),
            extra_data.extra_style.as_deref(),
        );
        let border_color = color_css(&self.color);
        let bb = self.info.bounding_box;
        canvas
            .current_element_css_push(&format!(
                "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px",
                bb.x, bb.y, bb.width, bb.height
            ))
            .current_element_css_push(&format!(
                "border-top: {}px solid {}",
                self.width.top, border_color
            ))
            .current_element_css_push(&format!(
                "border-right: {}px solid {}",
                self.width.right, border_color
            ))
            .current_element_css_push(&format!(
                "border-bottom: {}px solid {}",
                self.width.bottom, border_color
            ))
            .current_element_css_push(&format!(
                "border-left: {}px solid {}",
                self.width.left, border_color
            ))
            .current_element_css_push(&format!(
                "border-radius: {}px {}px {}px {}px",
                self.corner_radii.top_left,
                self.corner_radii.top_right,
                self.corner_radii.bottom_right,
                self.corner_radii.bottom_left
            ))
            .current_element_css_push("box-sizing: border-box; pointer-events: auto");
    }
}

impl<'a, ExtraData: OptionalExtraHTMLData> HTMLDrawable for ClipStart<'a, ExtraData> {
    fn draw(self, canvas: &mut HTMLCanvas) {
        let bb = self.info.bounding_box;
        canvas.push_scissor(
            self.info.id,
            bb.x,
            bb.y,
            bb.width,
            bb.height,
            self.corner_radii,
        );
    }
}

impl HTMLDrawable for ClipEnd {
    fn draw(self, canvas: &mut HTMLCanvas) {
        canvas.pop_scissor();
    }
}

impl<'a, ExtraData: OptionalExtraHTMLData> HTMLDrawable for Text<'a, ExtraData> {
    fn draw(self, canvas: &mut HTMLCanvas) {
        let extra_data = match self.info.extra_data {
            None => Default::default(),
            Some(extra) => extra.as_ref().extra_html_data().unwrap_or_default(),
        };
        canvas.ensure_command_element(
            self.info.id,
            extra_data.tag.as_json_str(),
            extra_data.extra_style.as_deref(),
        );
        let text_color = text_color_css(&self.color);
        let font_size = self.font_size * 1.0;
        let line_height = if self.line_height > 0.0 {
            self.line_height
        } else {
            1.2
        };
        let bb = self.info.bounding_box;
        canvas.set_text_content(self.text.as_ref().to_string());
        canvas
            .current_element_css_push(&format!(
                "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px",
                bb.x, bb.y, bb.width, bb.height
            ))
            .current_element_css_push(&format!(
                "font-family: font{}; font-size: {}px; line-height: {}; letter-spacing: {}px",
                self.font_id, font_size, line_height, self.letter_spacing
            ))
            .current_element_css_push(&format!("color: {}", text_color))
            .current_element_css_push("display: flex; align-items: flex-start")
            .current_element_css_push(
                "white-space: pre-wrap; word-break: break-word; overflow-wrap: anywhere",
            )
            .current_element_css_push("overflow: hidden; box-sizing: border-box")
            .current_element_css_push("pointer-events: auto; user-select: text");
    }
}

impl<'a, ImageData: URLImage, ExtraData: OptionalExtraHTMLData> HTMLDrawable
    for Image<'a, ImageData, ExtraData>
{
    fn draw(self, canvas: &mut HTMLCanvas) {
        let extra_data = match self.info.extra_data {
            None => Default::default(),
            Some(extra) => extra.as_ref().extra_html_data().unwrap_or_default(),
        };
        canvas.ensure_command_element(
            self.info.id,
            extra_data.tag.as_json_str(),
            extra_data.extra_style.as_deref(),
        );
        let url = self.data.as_ref().url();
        let bg = image_background_css(&self.background_color);
        let bb = self.info.bounding_box;
        canvas
            .current_element_css_push(&format!(
                "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px",
                bb.x, bb.y, bb.width, bb.height
            ))
            .current_element_css_push(&format!("background-color: {}", bg))
            .current_element_css_push(&format!("background-image: url(\"{}\")", url))
            .current_element_css_push("background-size: 100% 100%; background-repeat: no-repeat")
            .current_element_css_push("background-position: center; display: block")
            .current_element_css_push(&format!(
                "border-radius: {}px {}px {}px {}px",
                self.corner_radii.top_left,
                self.corner_radii.top_right,
                self.corner_radii.bottom_right,
                self.corner_radii.bottom_left
            ))
            .current_element_css_push("pointer-events: auto");
    }
}
