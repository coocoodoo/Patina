//! Keyboard and mouse state, and the mapping from keys to game actions.

use std::collections::HashSet;

use glam::Vec2;
pub use winit::keyboard::KeyCode;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// Swing the selected tool or weapon.
    Use,
    /// Talk, open, harvest, place, eat.
    Interact,
    Dodge,
    Inventory,
    Crafting,
    Menu,
    Confirm,
    Cancel,
    Up,
    Down,
    Left,
    Right,
    NextSlot,
    PrevSlot,
    Map,
}

const USE_KEYS: &[KeyCode] = &[KeyCode::KeyJ, KeyCode::KeyZ];
const INTERACT_KEYS: &[KeyCode] = &[KeyCode::KeyE, KeyCode::KeyK, KeyCode::KeyX, KeyCode::KeyF];
const DODGE_KEYS: &[KeyCode] = &[KeyCode::Space, KeyCode::ShiftLeft, KeyCode::ShiftRight];
const INVENTORY_KEYS: &[KeyCode] = &[KeyCode::Tab, KeyCode::KeyI];
const CRAFT_KEYS: &[KeyCode] = &[KeyCode::KeyC];
const MENU_KEYS: &[KeyCode] = &[KeyCode::Escape, KeyCode::KeyP];
const CONFIRM_KEYS: &[KeyCode] = &[
    KeyCode::Enter,
    KeyCode::NumpadEnter,
    KeyCode::KeyE,
    KeyCode::KeyJ,
    KeyCode::KeyZ,
    KeyCode::Space,
];
const CANCEL_KEYS: &[KeyCode] = &[
    KeyCode::Escape,
    KeyCode::Backspace,
    KeyCode::KeyX,
    KeyCode::KeyK,
];
const UP_KEYS: &[KeyCode] = &[KeyCode::KeyW, KeyCode::ArrowUp];
const DOWN_KEYS: &[KeyCode] = &[KeyCode::KeyS, KeyCode::ArrowDown];
const LEFT_KEYS: &[KeyCode] = &[KeyCode::KeyA, KeyCode::ArrowLeft];
const RIGHT_KEYS: &[KeyCode] = &[KeyCode::KeyD, KeyCode::ArrowRight];
const NEXT_KEYS: &[KeyCode] = &[KeyCode::BracketRight, KeyCode::KeyR];
const PREV_KEYS: &[KeyCode] = &[KeyCode::BracketLeft, KeyCode::KeyQ];
const MAP_KEYS: &[KeyCode] = &[KeyCode::KeyM];

fn keys(a: Action) -> &'static [KeyCode] {
    match a {
        Action::Use => USE_KEYS,
        Action::Interact => INTERACT_KEYS,
        Action::Dodge => DODGE_KEYS,
        Action::Inventory => INVENTORY_KEYS,
        Action::Crafting => CRAFT_KEYS,
        Action::Menu => MENU_KEYS,
        Action::Confirm => CONFIRM_KEYS,
        Action::Cancel => CANCEL_KEYS,
        Action::Up => UP_KEYS,
        Action::Down => DOWN_KEYS,
        Action::Left => LEFT_KEYS,
        Action::Right => RIGHT_KEYS,
        Action::NextSlot => NEXT_KEYS,
        Action::PrevSlot => PREV_KEYS,
        Action::Map => MAP_KEYS,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Left = 0,
    Right = 1,
    Middle = 2,
}

#[derive(Default)]
pub struct Input {
    down: HashSet<KeyCode>,
    pressed: HashSet<KeyCode>,
    /// Keys that auto-repeated this frame (for menu navigation).
    repeated: HashSet<KeyCode>,
    /// Mouse position in internal (low-res) pixels.
    pub mouse: Vec2,
    pub mouse_inside: bool,
    /// Set when the mouse moved this frame.
    pub mouse_moved: bool,
    buttons: [bool; 3],
    buttons_pressed: [bool; 3],
    buttons_released: [bool; 3],
    /// Wheel movement this frame in lines (positive = up).
    pub wheel: f32,
    /// Seconds since the last keyboard/mouse button activity.
    pub idle: f32,
    /// True when the last aiming input came from the mouse.
    pub mouse_aim: bool,
}

impl Input {
    pub fn key_event(&mut self, code: KeyCode, pressed: bool, repeat: bool) {
        if pressed {
            if repeat {
                self.repeated.insert(code);
            } else if self.down.insert(code) {
                self.pressed.insert(code);
            }
            if !matches!(code, KeyCode::KeyE | KeyCode::KeyF) {
                self.mouse_aim = false;
            }
            self.idle = 0.0;
        } else {
            self.down.remove(&code);
        }
    }

    pub fn button_event(&mut self, b: Button, pressed: bool) {
        let i = b as usize;
        if pressed {
            if !self.buttons[i] {
                self.buttons_pressed[i] = true;
            }
            self.mouse_aim = true;
            self.idle = 0.0;
        } else if self.buttons[i] {
            self.buttons_released[i] = true;
        }
        self.buttons[i] = pressed;
    }

    pub fn mouse_move(&mut self, p: Vec2) {
        if (p - self.mouse).length_squared() > 0.25 {
            self.mouse_moved = true;
            self.mouse_aim = true;
        }
        self.mouse = p;
        self.mouse_inside = true;
    }

    /// Forgets held keys (the window lost focus).
    pub fn release_all(&mut self) {
        self.down.clear();
        self.buttons = [false; 3];
    }

    /// Call once per frame after the game has read the input.
    pub fn end_frame(&mut self, dt: f32) {
        self.pressed.clear();
        self.repeated.clear();
        self.buttons_pressed = [false; 3];
        self.buttons_released = [false; 3];
        self.wheel = 0.0;
        self.mouse_moved = false;
        self.idle += dt;
    }

    pub fn key_down(&self, k: KeyCode) -> bool {
        self.down.contains(&k)
    }

    pub fn down(&self, a: Action) -> bool {
        let held = keys(a).iter().any(|k| self.down.contains(k));
        held || match a {
            Action::Use => self.buttons[0],
            Action::Interact => self.buttons[1],
            _ => false,
        }
    }

    pub fn pressed(&self, a: Action) -> bool {
        let hit = keys(a).iter().any(|k| self.pressed.contains(k));
        hit || match a {
            Action::Use => self.buttons_pressed[0],
            Action::Interact => self.buttons_pressed[1],
            _ => false,
        }
    }

    /// Pressed or auto-repeated (menus).
    pub fn pressed_repeat(&self, a: Action) -> bool {
        self.pressed(a) || keys(a).iter().any(|k| self.repeated.contains(k))
    }

    pub fn button_pressed(&self, b: Button) -> bool {
        self.buttons_pressed[b as usize]
    }

    /// Movement from WASD / arrows, normalised.
    pub fn move_axis(&self) -> Vec2 {
        let mut v = Vec2::ZERO;
        if self.down(Action::Left) {
            v.x -= 1.0;
        }
        if self.down(Action::Right) {
            v.x += 1.0;
        }
        if self.down(Action::Up) {
            v.y -= 1.0;
        }
        if self.down(Action::Down) {
            v.y += 1.0;
        }
        v.normalize_or_zero()
    }

    /// Number key 1..9, 0 pressed this frame, as a hotbar index 0..9.
    pub fn number_pressed(&self) -> Option<usize> {
        const DIGITS: [KeyCode; 10] = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
            KeyCode::Digit0,
        ];
        DIGITS.iter().position(|k| self.pressed.contains(k))
    }
}
