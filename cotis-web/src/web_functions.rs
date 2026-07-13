//! Low-level JavaScript FFI bindings to `resources/web-renderer/renderer.js`.
//!
//! The [`primitives`] module wraps wasm-bindgen imports for:
//!
//! | Category | Functions |
//! |----------|-----------|
//! | Frame timing | `wait_for_next_frame`, `get_delta_time_ms`, `window_dimensions` |
//! | DOM canvas | `begin_frame`, `end_frame`, `get_or_create_host_element`, scissor stack |
//! | Input | mouse position/buttons/wheel, keyboard state |
//! | Fonts | `load_font` |
//! | Custom elements | `get_custom_element_html`, `get_custom_element_properties` |
//!
//! [`HTMLRenderer`](crate::renderer::HTMLRenderer) uses the DOM canvas path. The legacy JSON
//! `draw_frame` export in `renderer.js` is **not** called from Rust today.

mod bindings {
    use js_sys::{Array, JsString, Promise};
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(module = "/resources/web-renderer/renderer.js")]
    extern "C" {
        pub fn get_delta_time_ms() -> JsValue;
        pub fn window_dimensions() -> Array;
        pub fn init_html_root();
        pub fn text_measuring_function(
            string: JsString,
            id: JsValue,
            size: JsValue,
            line_height: JsValue,
            letter_spacing: JsValue,
        ) -> Array;
        pub fn get_mouse_position() -> Array;
        pub fn mouse_button_down(n: usize) -> JsValue;
        pub fn get_mouse_wheel_move_v() -> JsValue;

        pub fn keyDown(keyCode: JsValue) -> JsValue; // Return all currently pressed keys

        // read (dequeue) one character
        pub fn readChar() -> JsValue;

        // check if anything is waiting
        pub fn hasInput() -> JsValue;
        pub fn getPressedKeys() -> Array; // Returns an array of all keys

        pub fn loadFont(id: JsValue, url: JsValue) -> Promise;
        pub fn get_custom_element_html(element_id: JsValue) -> JsValue;
        pub fn get_custom_element_properties(element_id: JsValue, selector: JsValue) -> JsValue;

        pub async fn waitForNextFrame();

        // Rendering functions
        pub fn beginFrame();
        pub fn endFrame(usedElementIds: JsValue);
        pub fn getOrCreateHostElement(
            id: JsValue,
            tagName: JsValue,
            extraStyle: JsValue,
            applyStyleCallback: JsValue,
        ) -> JsValue;
        pub fn appendToCurrentContainer(element: JsValue);
        pub fn scissorStackPush(container: JsValue, globalX: f64, globalY: f64);
        pub fn scissorStackPop();
    }
}

pub(crate) mod primitives {
    use crate::web_functions::*;
    use cotis_defaults::element_configs::text_config::TextConfig;
    use cotis_utils::interactivity::keyboard::KeyboardKey;
    use cotis_utils::interactivity::mouse;
    use cotis_utils::math::Dimensions;
    use js_sys::{Array, JsString, Promise};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    /// Returns frame delta time in **seconds** (converted from JS milliseconds).
    pub fn get_delta_time_ms() -> f32 {
        let delta: JsValue = bindings::get_delta_time_ms();
        let delta = delta.as_f64().unwrap_or(0.0) as f32;
        delta / 1000.0 // milliseconds → seconds
    }
    pub(crate) fn window_dimensions() -> (f32, f32) {
        let array = bindings::window_dimensions();
        let width: f32 = array.get(0).as_f64().unwrap_or(0.0) as f32;
        let height: f32 = array.get(1).as_f64().unwrap_or(0.0) as f32;
        (width, height)
    }
    pub fn text_measuring_function(s: &str, config: &TextConfig) -> Dimensions {
        let js_string: JsString = JsString::from(s);
        let array = bindings::text_measuring_function(
            js_string,
            JsValue::from(config.font_id),
            JsValue::from(config.font_size),
            JsValue::from(config.line_height),
            JsValue::from(config.letter_spacing),
        );

        let width: f32 = array.get(0).as_f64().unwrap_or(0.0) as f32;
        let height: f32 = array.get(1).as_f64().unwrap_or(0.0) as f32;

        Dimensions { width, height }
    }
    pub fn get_mouse_position() -> (f32, f32) {
        let array = bindings::get_mouse_position();
        let x: f32 = array.get(0).as_f64().unwrap_or(0.0) as f32;
        let y: f32 = array.get(1).as_f64().unwrap_or(0.0) as f32;
        (x, y)
    }

    pub fn mouse_button_down(n: mouse::MouseButton) -> bool {
        let true_n = match n {
            mouse::MouseButton::MouseButtonLeft => 0,
            mouse::MouseButton::MouseButtonRight => 2,
            mouse::MouseButton::MouseButtonMiddle => 1,
            _ => return false,
        };
        let res = bindings::mouse_button_down(true_n);
        res.as_bool().unwrap_or(false)
    }
    pub fn get_mouse_wheel_move_v() -> (f32, f32) {
        let value: JsValue = bindings::get_mouse_wheel_move_v();
        let x: f32 = 0.0f32;
        let y: f32 = value.as_f64().unwrap_or(0.0) as f32;
        (x, y)
    }

    #[allow(dead_code)]
    pub fn key_down(key_code: &KeyboardKey) -> bool {
        let key_code = match key_code {
            KeyboardKey::KeyNull => "Null",
            KeyboardKey::KeyApostrophe => "Quote",
            KeyboardKey::KeyComma => "Comma",
            KeyboardKey::KeyMinus => "Minus",
            KeyboardKey::KeyPeriod => "Period",
            KeyboardKey::KeySlash => "Slash",

            KeyboardKey::KeyZero => "Digit0",
            KeyboardKey::KeyOne => "Digit1",
            KeyboardKey::KeyTwo => "Digit2",
            KeyboardKey::KeyThree => "Digit3",
            KeyboardKey::KeyFour => "Digit4",
            KeyboardKey::KeyFive => "Digit5",
            KeyboardKey::KeySix => "Digit6",
            KeyboardKey::KeySeven => "Digit7",
            KeyboardKey::KeyEight => "Digit8",
            KeyboardKey::KeyNine => "Digit9",

            KeyboardKey::KeySemicolon => "Semicolon",
            KeyboardKey::KeyEqual => "Equal",

            KeyboardKey::KeyA => "KeyA",
            KeyboardKey::KeyB => "KeyB",
            KeyboardKey::KeyC => "KeyC",
            KeyboardKey::KeyD => "KeyD",
            KeyboardKey::KeyE => "KeyE",
            KeyboardKey::KeyF => "KeyF",
            KeyboardKey::KeyG => "KeyG",
            KeyboardKey::KeyH => "KeyH",
            KeyboardKey::KeyI => "KeyI",
            KeyboardKey::KeyJ => "KeyJ",
            KeyboardKey::KeyK => "KeyK",
            KeyboardKey::KeyL => "KeyL",
            KeyboardKey::KeyM => "KeyM",
            KeyboardKey::KeyN => "KeyN",
            KeyboardKey::KeyO => "KeyO",
            KeyboardKey::KeyP => "KeyP",
            KeyboardKey::KeyQ => "KeyQ",
            KeyboardKey::KeyR => "KeyR",
            KeyboardKey::KeyS => "KeyS",
            KeyboardKey::KeyT => "KeyT",
            KeyboardKey::KeyU => "KeyU",
            KeyboardKey::KeyV => "KeyV",
            KeyboardKey::KeyW => "KeyW",
            KeyboardKey::KeyX => "KeyX",
            KeyboardKey::KeyY => "KeyY",
            KeyboardKey::KeyZ => "KeyZ",

            KeyboardKey::KeyLeftBracket => "BracketLeft",
            KeyboardKey::KeyBackslash => "Backslash",
            KeyboardKey::KeyRightBracket => "BracketRight",
            KeyboardKey::KeyGrave => "Backquote",

            KeyboardKey::KeySpace => "Space",
            KeyboardKey::KeyEscape => "Escape",
            KeyboardKey::KeyEnter => "Enter",
            KeyboardKey::KeyTab => "Tab",
            KeyboardKey::KeyBackspace => "Backspace",

            KeyboardKey::KeyInsert => "Insert",
            KeyboardKey::KeyDelete => "Delete",
            KeyboardKey::KeyRight => "ArrowRight",
            KeyboardKey::KeyLeft => "ArrowLeft",
            KeyboardKey::KeyDown => "ArrowDown",
            KeyboardKey::KeyUp => "ArrowUp",
            KeyboardKey::KeyPageUp => "PageUp",
            KeyboardKey::KeyPageDown => "PageDown",
            KeyboardKey::KeyHome => "Home",
            KeyboardKey::KeyEnd => "End",

            KeyboardKey::KeyCapsLock => "CapsLock",
            KeyboardKey::KeyScrollLock => "ScrollLock",
            KeyboardKey::KeyNumLock => "NumLock",
            KeyboardKey::KeyPrintScreen => "PrintScreen",
            KeyboardKey::KeyPause => "Pause",

            KeyboardKey::KeyF1 => "F1",
            KeyboardKey::KeyF2 => "F2",
            KeyboardKey::KeyF3 => "F3",
            KeyboardKey::KeyF4 => "F4",
            KeyboardKey::KeyF5 => "F5",
            KeyboardKey::KeyF6 => "F6",
            KeyboardKey::KeyF7 => "F7",
            KeyboardKey::KeyF8 => "F8",
            KeyboardKey::KeyF9 => "F9",
            KeyboardKey::KeyF10 => "F10",
            KeyboardKey::KeyF11 => "F11",
            KeyboardKey::KeyF12 => "F12",

            KeyboardKey::KeyLeftShift => "ShiftLeft",
            KeyboardKey::KeyLeftControl => "ControlLeft",
            KeyboardKey::KeyLeftAlt => "AltLeft",
            KeyboardKey::KeyLeftSuper => "MetaLeft",
            KeyboardKey::KeyRightShift => "ShiftRight",
            KeyboardKey::KeyRightControl => "ControlRight",
            KeyboardKey::KeyRightAlt => "AltRight",
            KeyboardKey::KeyRightSuper => "MetaRight",

            KeyboardKey::KeyKbMenu => "ContextMenu",

            KeyboardKey::KeyKp0 => "Numpad0",
            KeyboardKey::KeyKp1 => "Numpad1",
            KeyboardKey::KeyKp2 => "Numpad2",
            KeyboardKey::KeyKp3 => "Numpad3",
            KeyboardKey::KeyKp4 => "Numpad4",
            KeyboardKey::KeyKp5 => "Numpad5",
            KeyboardKey::KeyKp6 => "Numpad6",
            KeyboardKey::KeyKp7 => "Numpad7",
            KeyboardKey::KeyKp8 => "Numpad8",
            KeyboardKey::KeyKp9 => "Numpad9",

            KeyboardKey::KeyKpDecimal => "NumpadDecimal",
            KeyboardKey::KeyKpDivide => "NumpadDivide",
            KeyboardKey::KeyKpMultiply => "NumpadMultiply",
            KeyboardKey::KeyKpSubtract => "NumpadSubtract",
            KeyboardKey::KeyKpAdd => "NumpadAdd",
            KeyboardKey::KeyKpEnter => "NumpadEnter",
            KeyboardKey::KeyKpEqual => "NumpadEqual",

            KeyboardKey::KeyBack => "BrowserBack",
            KeyboardKey::KeyMenu => "BrowserMenu",
            KeyboardKey::KeyVolumeUp => "AudioVolumeUp",
            KeyboardKey::KeyVolumeDown => "AudioVolumeDown",
        };
        let value: JsValue = bindings::keyDown(JsValue::from_str(key_code));
        value.as_bool().unwrap()
    }

    #[allow(dead_code)]
    pub fn get_next_char() -> Option<char> {
        let has_input: JsValue = bindings::hasInput();
        match has_input.as_bool() {
            Some(false) => {
                return None;
            }
            None => {
                return None;
            }
            Some(true) => {}
        }
        let new_char: JsValue = bindings::readChar();
        match new_char.as_string() {
            None => None,
            Some(c) => Some(c.chars().next()?),
        }
    }
    #[allow(dead_code)]
    pub fn get_pressed_keys() -> Vec<String> {
        let values: Array = bindings::getPressedKeys();
        let mut res: Vec<String> = vec![];
        for value in values {
            let value: JsValue = value;
            res.push(value.as_string().unwrap());
        }
        res
    }

    pub fn load_font(url: &str, id: usize) {
        let _: Promise =
            bindings::loadFont(JsValue::from_str(&id.to_string()), JsValue::from_str(url));
    }

    pub async fn wait_for_next_frame() {
        bindings::waitForNextFrame().await;
    }

    pub fn get_custom_element_html(element_id: u64) -> Option<String> {
        bindings::get_custom_element_html(JsValue::from_f64(element_id as f64)).as_string()
    }

    pub fn get_custom_element_properties(
        element_id: u64,
        selector: Option<&str>,
    ) -> Option<String> {
        let selector = selector.map(JsValue::from_str).unwrap_or(JsValue::NULL);
        bindings::get_custom_element_properties(JsValue::from_f64(element_id as f64), selector)
            .as_string()
    }

    pub fn begin_frame() {
        bindings::beginFrame();
    }

    pub fn end_frame(used_element_ids: &js_sys::Set) {
        bindings::endFrame(used_element_ids.into());
    }

    pub fn get_or_create_host_element(
        id: &str,
        tag_name: &str,
        extra_style: Option<&str>,
        apply_style_fn: &js_sys::Function,
    ) -> web_sys::Element {
        let extra_style_js = extra_style.map(JsValue::from_str).unwrap_or(JsValue::NULL);

        bindings::getOrCreateHostElement(
            JsValue::from_str(id),
            JsValue::from_str(tag_name),
            extra_style_js,
            apply_style_fn.into(),
        )
        .unchecked_into()
    }

    pub fn append_to_current_container(element: &web_sys::Element) {
        bindings::appendToCurrentContainer(element.into());
    }

    pub fn scissor_stack_push(container: &web_sys::Element, global_x: f64, global_y: f64) {
        bindings::scissorStackPush(container.into(), global_x, global_y);
    }

    pub fn scissor_stack_pop() {
        bindings::scissorStackPop();
    }
}
