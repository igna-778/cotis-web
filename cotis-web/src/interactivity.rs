//! Browser input providers for Cotis interactivity traits.
//!
//! # Mouse
//!
//! [`MouseProvider`](cotis_utils::interactivity::mouse::MouseProvider) is implemented on
//! [`HTMLRenderer`](crate::renderer::HTMLRenderer). Use the renderer instance in your app
//! when querying mouse position, buttons, and wheel delta.
//!
//! # Keyboard
//!
//! [`SimpleKeyboardProvider`](cotis_utils::interactivity::keyboard::SimpleKeyboardProvider) is
//! implemented on [`HTMLInteractivity`], a zero-sized handle type. Keyboard state is tracked
//! in a global [`HTMLKeyboardProvider`] updated each frame with delta time from
//! [`CotisFrameContext::get_delta_time`](cotis_utils::traits::CotisFrameContext::get_delta_time).

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use cotis_utils::interactivity::keyboard::{KEYBOARD_LIST, KeyboardKey, SimpleKeyboardProvider};
use cotis_utils::interactivity::mouse;
use cotis_utils::interactivity::mouse::MouseProvider;
use cotis_utils::math::Vector2;

use crate::renderer::HTMLRenderer;
use crate::web_functions::primitives::{
    get_mouse_position, get_mouse_wheel_move_v, get_next_char, mouse_button_down,
};

/// Zero-sized type implementing [`SimpleKeyboardProvider`] via global keyboard state.
pub struct HTMLInteractivity;

impl MouseProvider for HTMLRenderer {
    fn get_mouse_position(&self) -> Vector2 {
        let array = get_mouse_position();
        Vector2 {
            x: array.0,
            y: array.1,
        }
    }

    fn mouse_button_down(&self, n: mouse::MouseButton) -> bool {
        mouse_button_down(n)
    }

    fn get_mouse_wheel_move_v(&self) -> Vector2 {
        let array = get_mouse_wheel_move_v();
        Vector2 {
            x: array.0,
            y: array.1,
        }
    }
}

pub(crate) struct HTMLKeyboardProvider {
    keyboard_prev_state: Vec<KeyboardKey>,
    keyboard_once_state: Vec<KeyboardKey>,
    keyboard_once_release: Vec<KeyboardKey>,
    cooldown: HashMap<KeyboardKey, f32>,
}

impl HTMLKeyboardProvider {
    const COOLDOWN: f32 = 0.1;
    const INITIAL_COOLDOWN: f32 = 5.0;

    pub(crate) fn update(&mut self, delta_time: f32) {
        self.keyboard_once_state.clear();
        self.keyboard_once_release.clear();
        for key in KEYBOARD_LIST {
            if self.is_key_down(key) {
                if !self.keyboard_prev_state.contains(&key) {
                    self.keyboard_prev_state.push(key);
                    self.keyboard_once_state.push(key);
                    self.cooldown.insert(key, Self::INITIAL_COOLDOWN);
                }
            } else {
                if self.keyboard_prev_state.contains(&key)
                    && !self.keyboard_once_release.contains(&key)
                {
                    self.keyboard_once_release.push(key);
                }
                self.keyboard_prev_state.retain(|e| *e != key);
                self.cooldown.retain(|e, _| *e != key);
            }
        }
        for timer in self.cooldown.values_mut() {
            if *timer > 0.0 {
                *timer = (*timer - delta_time).max(0.0);
            } else {
                *timer = Self::COOLDOWN;
            }
        }
    }

    fn get_next_char(&mut self) -> Option<char> {
        get_next_char()
    }

    fn is_key_pressed(&self, key: KeyboardKey) -> bool {
        self.keyboard_once_state.contains(&key)
    }

    fn is_key_pressed_repeat(&self, key: KeyboardKey) -> bool {
        if !self.keyboard_prev_state.contains(&key) {
            return false;
        }
        if let Some(timer) = self.cooldown.get(&key) {
            timer <= &0.0
        } else {
            false
        }
    }

    fn is_key_down(&self, key: KeyboardKey) -> bool {
        self.keyboard_prev_state.contains(&key)
    }

    fn is_key_released(&self, key: KeyboardKey) -> bool {
        self.keyboard_once_release.contains(&key)
    }

    fn is_key_up(&self, key: KeyboardKey) -> bool {
        !self.is_key_down(key)
    }

    fn get_key_pressed(&mut self) -> Option<KeyboardKey> {
        self.keyboard_once_state.first().cloned()
    }

    fn get_char_pressed(&mut self) -> Option<char> {
        self.get_next_char()
    }
}

static KEYBOARD_PROVIDER: LazyLock<Mutex<HTMLKeyboardProvider>> = LazyLock::new(|| {
    Mutex::new(HTMLKeyboardProvider {
        keyboard_prev_state: Vec::new(),
        keyboard_once_state: Vec::new(),
        keyboard_once_release: Vec::new(),
        cooldown: HashMap::new(),
    })
});

impl MouseProvider for HTMLInteractivity {
    fn get_mouse_position(&self) -> Vector2 {
        let array = get_mouse_position();
        Vector2 {
            x: array.0,
            y: array.1,
        }
    }

    fn mouse_button_down(&self, n: mouse::MouseButton) -> bool {
        mouse_button_down(n)
    }

    fn get_mouse_wheel_move_v(&self) -> Vector2 {
        let array = get_mouse_wheel_move_v();
        Vector2 {
            x: array.0,
            y: array.1,
        }
    }
}

impl SimpleKeyboardProvider for HTMLInteractivity {
    fn get_pressed_keys(&self) -> indexmap::set::IndexSet<KeyboardKey> {
        KEYBOARD_PROVIDER
            .lock()
            .map(|provider| provider.keyboard_prev_state.iter().cloned().collect())
            .unwrap_or_default()
    }
}
