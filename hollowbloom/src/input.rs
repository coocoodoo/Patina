//! Keyboard, mouse and gamepad state, and the mapping from keys and buttons to game
//! actions. The gamepad layout is the Steam Deck's (the same as an Xbox pad's).

use std::collections::HashSet;

use glam::Vec2;
pub use winit::keyboard::KeyCode;

use crate::pad::{PAD_BUTTONS, PadButton as P, PadState};

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
    /// The quest journal.
    Quests,
    /// Cast the first or second readied spell.
    Spell1,
    Spell2,
    /// Turn the furniture you're holding (at home).
    Turn,
    /// A second choice on whatever's under the cursor in a menu (a right click): split a
    /// stack, wear a piece, sell just one. Controllers only; the mouse has its right button.
    Alt,
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
const QUEST_KEYS: &[KeyCode] = &[KeyCode::KeyL];
const SPELL1_KEYS: &[KeyCode] = &[KeyCode::KeyQ];
const SPELL2_KEYS: &[KeyCode] = &[KeyCode::KeyR];
const TURN_KEYS: &[KeyCode] = &[KeyCode::KeyT];

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
        Action::Quests => QUEST_KEYS,
        Action::Spell1 => SPELL1_KEYS,
        Action::Spell2 => SPELL2_KEYS,
        Action::Turn => TURN_KEYS,
        Action::Alt => &[],
    }
}

/// The Steam Deck layout: the left stick (or the D-pad) walks and the right stick aims;
/// A talks and interacts, X swings, B rolls, Y opens the bag, L1/R1 step along the hotbar,
/// L2/R2 cast your two spells, View opens the crafting book and Menu pauses; clicking the
/// left stick shows the map and the right one the quest journal. In menus A picks, B goes
/// back, X is a right click and L1/R1 flip tabs.
fn pad_buttons(a: Action) -> &'static [P] {
    match a {
        Action::Use | Action::Alt => &[P::X],
        Action::Interact | Action::Confirm => &[P::A],
        Action::Dodge | Action::Cancel | Action::Turn => &[P::B],
        Action::Inventory => &[P::Y],
        Action::Crafting => &[P::View],
        Action::Menu => &[P::Menu],
        Action::Up => &[P::Up],
        Action::Down => &[P::Down],
        Action::Left => &[P::Left],
        Action::Right => &[P::Right],
        Action::NextSlot => &[P::R1],
        Action::PrevSlot => &[P::L1],
        Action::Map => &[P::L3],
        Action::Quests => &[P::R3],
        Action::Spell1 => &[P::L2],
        Action::Spell2 => &[P::R2],
    }
}

/// The Steam Deck button to show for an action in on-screen hints.
pub fn pad_name(a: Action) -> &'static str {
    match a {
        Action::Use | Action::Alt => "X",
        Action::Interact | Action::Confirm => "A",
        Action::Dodge | Action::Cancel | Action::Turn => "B",
        Action::Inventory => "Y",
        Action::Crafting => "View",
        Action::Menu => "Menu",
        Action::Up | Action::Down | Action::Left | Action::Right => "D-pad",
        Action::NextSlot => "R1",
        Action::PrevSlot => "L1",
        Action::Map => "L3",
        Action::Quests => "R3",
        Action::Spell1 => "L2",
        Action::Spell2 => "R2",
    }
}

/// The left stick pushed this far counts as the D-pad in menus (and lets go below the
/// second figure).
const STICK_ON: f32 = 0.55;
const STICK_OFF: f32 = 0.35;
/// How far the sticks must move before they count at all.
const MOVE_DEAD: f32 = 0.2;
const AIM_DEAD: f32 = 0.35;
/// Held menu directions repeat after this long, then this often.
const REPEAT_DELAY: f32 = 0.38;
const REPEAT_EVERY: f32 = 0.085;

/// A stick's position with the deadzone cut out and the rest stretched back to 0..1.
fn live(v: Vec2, dead: f32) -> Vec2 {
    let l = v.length();
    if l <= dead {
        Vec2::ZERO
    } else {
        v / l * ((l - dead) / (1.0 - dead)).min(1.0)
    }
}

/// The key to show for an action in on-screen hints.
pub fn key_name(a: Action) -> &'static str {
    match a {
        Action::Use => "J",
        Action::Interact | Action::Confirm => "E",
        Action::Dodge => "Space",
        Action::Inventory => "Tab",
        Action::Crafting => "C",
        Action::Menu | Action::Cancel => "Esc",
        Action::Up => "W",
        Action::Down => "S",
        Action::Left => "A",
        Action::Right => "D",
        Action::NextSlot => "]",
        Action::PrevSlot => "[",
        Action::Map => "M",
        Action::Quests => "L",
        Action::Spell1 => "Q",
        Action::Spell2 => "R",
        Action::Turn => "T",
        Action::Alt => "Right click",
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
    /// The gamepad: its state this frame, buttons (including the left stick's four
    /// directions and the triggers) down now, gone down this frame and auto-repeating, and
    /// how long each has been held.
    pub pad: PadState,
    pad_down: u32,
    pad_pressed: u32,
    pad_repeated: u32,
    pad_held: [f32; PAD_BUTTONS + 4],
    /// True while the controller is what's being played with (hints show its buttons).
    pub pad_active: bool,
}

/// Bits past the pad's own buttons: the left stick pushed up, down, left and right.
const STICK_UP: u32 = 1 << PAD_BUTTONS;
const STICK_DOWN: u32 = 1 << (PAD_BUTTONS + 1);
const STICK_LEFT: u32 = 1 << (PAD_BUTTONS + 2);
const STICK_RIGHT: u32 = 1 << (PAD_BUTTONS + 3);

/// The pad bits that count as an action: its buttons, and for directions the stick too.
fn pad_bits(a: Action) -> u32 {
    let mut bits = pad_buttons(a).iter().fold(0u32, |m, b| m | b.bit() as u32);
    bits |= match a {
        Action::Up => STICK_UP,
        Action::Down => STICK_DOWN,
        Action::Left => STICK_LEFT,
        Action::Right => STICK_RIGHT,
        _ => 0,
    };
    bits
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
            if !repeat {
                self.pad_active = false;
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
            self.pad_active = false;
            self.idle = 0.0;
        } else if self.buttons[i] {
            self.buttons_released[i] = true;
        }
        self.buttons[i] = pressed;
    }

    pub fn mouse_move(&mut self, p: Vec2) {
        // Where the pointer is when it comes into the window isn't a move to aim with: a
        // Deck's pointer sits parked in the middle of the screen.
        if self.mouse_inside && (p - self.mouse).length_squared() > 0.25 {
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

    /// Takes in the gamepad's state for this frame: which buttons went down, which menu
    /// directions repeat, and whether the controller is now the thing being played with.
    pub fn pad_event(&mut self, s: PadState, dt: f32) {
        let mut down = s.buttons as u32;
        // Triggers are buttons here.
        let was = |bit: u32| self.pad_down & bit != 0;
        let l2 = P::L2.bit() as u32;
        let r2 = P::R2.bit() as u32;
        if s.lt > if was(l2) { STICK_OFF } else { STICK_ON } {
            down |= l2;
        }
        if s.rt > if was(r2) { STICK_OFF } else { STICK_ON } {
            down |= r2;
        }
        // The left stick doubles as the D-pad in menus.
        let dirs = [
            (STICK_UP, -s.left.y),
            (STICK_DOWN, s.left.y),
            (STICK_LEFT, -s.left.x),
            (STICK_RIGHT, s.left.x),
        ];
        for (bit, v) in dirs {
            if v > if was(bit) { STICK_OFF } else { STICK_ON } {
                down |= bit;
            }
        }
        self.pad_pressed = down & !self.pad_down;
        self.pad_repeated = 0;
        for (k, held) in self.pad_held.iter_mut().enumerate() {
            let bit = 1u32 << k;
            if down & bit == 0 {
                *held = 0.0;
                continue;
            }
            let before = *held;
            *held += dt;
            if before >= REPEAT_DELAY || *held >= REPEAT_DELAY {
                let t0 = ((before - REPEAT_DELAY) / REPEAT_EVERY).floor();
                let t1 = ((*held - REPEAT_DELAY) / REPEAT_EVERY).floor();
                if t1 > t0 || (before < REPEAT_DELAY && *held >= REPEAT_DELAY) {
                    self.pad_repeated |= bit;
                }
            }
        }
        self.pad_down = down;
        let moved = s.left.length() > STICK_ON || s.right.length() > STICK_ON;
        if self.pad_pressed != 0 || moved {
            // The controller takes over: aim with it, not wherever the pointer was left.
            self.pad_active = true;
            self.mouse_aim = false;
            self.idle = 0.0;
        }
        if live(s.right, AIM_DEAD) != Vec2::ZERO {
            self.mouse_aim = false;
        }
        self.pad = s;
    }

    /// Call once per frame after the game has read the input.
    pub fn end_frame(&mut self, dt: f32) {
        self.pressed.clear();
        self.repeated.clear();
        self.buttons_pressed = [false; 3];
        self.buttons_released = [false; 3];
        self.wheel = 0.0;
        self.mouse_moved = false;
        self.pad_pressed = 0;
        self.pad_repeated = 0;
        self.idle += dt;
    }

    pub fn key_down(&self, k: KeyCode) -> bool {
        self.down.contains(&k)
    }

    /// A raw key that went down this frame.
    pub fn key_pressed(&self, k: KeyCode) -> bool {
        self.pressed.contains(&k)
    }

    pub fn down(&self, a: Action) -> bool {
        let held =
            keys(a).iter().any(|k| self.down.contains(k)) || self.pad_down & pad_bits(a) != 0;
        held || match a {
            Action::Use => self.buttons[0],
            Action::Interact => self.buttons[1],
            _ => false,
        }
    }

    pub fn pressed(&self, a: Action) -> bool {
        let hit =
            keys(a).iter().any(|k| self.pressed.contains(k)) || self.pad_pressed & pad_bits(a) != 0;
        hit || match a {
            Action::Use => self.buttons_pressed[0],
            Action::Interact => self.buttons_pressed[1],
            _ => false,
        }
    }

    /// Pressed or auto-repeated (menus).
    pub fn pressed_repeat(&self, a: Action) -> bool {
        self.pressed(a)
            || keys(a).iter().any(|k| self.repeated.contains(k))
            || self.pad_repeated & pad_bits(a) != 0
    }

    /// Where the right stick points, when it's pushed: the way to face.
    pub fn aim_axis(&self) -> Option<Vec2> {
        let v = live(self.pad.right, AIM_DEAD);
        (v != Vec2::ZERO).then(|| v.normalize())
    }

    pub fn button_pressed(&self, b: Button) -> bool {
        self.buttons_pressed[b as usize]
    }

    /// Movement from WASD, the arrows or the D-pad (normalised), or from the left stick,
    /// which walks slower when pushed gently.
    pub fn move_axis(&self) -> Vec2 {
        let stick = live(self.pad.left, MOVE_DEAD);
        if stick != Vec2::ZERO {
            return stick;
        }
        let held = |a: Action| {
            keys(a).iter().any(|k| self.down.contains(k))
                || pad_buttons(a).iter().any(|b| self.pad.held(*b))
        };
        let mut v = Vec2::ZERO;
        if held(Action::Left) {
            v.x -= 1.0;
        }
        if held(Action::Right) {
            v.x += 1.0;
        }
        if held(Action::Up) {
            v.y -= 1.0;
        }
        if held(Action::Down) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pad(buttons: &[P], left: Vec2, lt: f32) -> PadState {
        PadState {
            connected: true,
            buttons: buttons.iter().fold(0, |m, b| m | b.bit()),
            left,
            lt,
            ..Default::default()
        }
    }

    #[test]
    fn the_steam_deck_layout_drives_the_game() {
        let mut i = Input::default();
        i.pad_event(pad(&[P::A], Vec2::ZERO, 0.0), 1.0 / 60.0);
        assert!(i.pressed(Action::Interact) && i.pressed(Action::Confirm));
        assert!(i.pad_active && !i.pressed(Action::Use));
        i.end_frame(1.0 / 60.0);
        i.pad_event(pad(&[P::A], Vec2::ZERO, 0.0), 1.0 / 60.0);
        assert!(i.down(Action::Interact) && !i.pressed(Action::Interact));
        // X swings, B rolls, Y opens the bag, L1/R1 step along the hotbar.
        for (b, a) in [
            (P::X, Action::Use),
            (P::B, Action::Dodge),
            (P::Y, Action::Inventory),
            (P::R1, Action::NextSlot),
            (P::L1, Action::PrevSlot),
            (P::View, Action::Crafting),
            (P::Menu, Action::Menu),
            (P::L3, Action::Map),
            (P::R3, Action::Quests),
        ] {
            i.pad_event(pad(&[], Vec2::ZERO, 0.0), 1.0 / 60.0);
            i.end_frame(1.0 / 60.0);
            i.pad_event(pad(&[b], Vec2::ZERO, 0.0), 1.0 / 60.0);
            assert!(i.pressed(a), "{b:?} -> {a:?}");
            i.end_frame(1.0 / 60.0);
        }
        // Pressing a key hands the hints back to the keyboard.
        i.key_event(KeyCode::KeyE, true, false);
        assert!(!i.pad_active);
    }

    #[test]
    fn the_sticks_walk_aim_and_scroll_menus() {
        let mut i = Input::default();
        // A nudge inside the deadzone does nothing; a gentle push walks slowly.
        i.pad_event(pad(&[], Vec2::new(0.1, 0.0), 0.0), 1.0 / 60.0);
        assert_eq!(i.move_axis(), Vec2::ZERO);
        i.pad_event(pad(&[], Vec2::new(0.5, 0.0), 0.0), 1.0 / 60.0);
        let v = i.move_axis();
        assert!(v.x > 0.2 && v.x < 0.8, "{v:?}");
        // All the way down scrolls a menu once, then repeats while held.
        i.end_frame(1.0 / 60.0);
        i.pad_event(pad(&[], Vec2::new(0.0, 1.0), 0.0), 1.0 / 60.0);
        assert!(i.pressed(Action::Down));
        let mut repeats = 0;
        for _ in 0..60 {
            i.end_frame(1.0 / 60.0);
            i.pad_event(pad(&[], Vec2::new(0.0, 1.0), 0.0), 1.0 / 60.0);
            assert!(!i.pressed(Action::Down));
            repeats += usize::from(i.pressed_repeat(Action::Down));
        }
        assert!((5..=9).contains(&repeats), "{repeats} repeats in a second");
        // The right stick aims.
        let s = PadState {
            right: Vec2::new(-1.0, 0.0),
            ..pad(&[], Vec2::ZERO, 0.0)
        };
        i.pad_event(s, 1.0 / 60.0);
        assert_eq!(i.aim_axis(), Some(Vec2::new(-1.0, 0.0)));
        // Triggers cast, with a little give before they let go.
        i.pad_event(pad(&[], Vec2::ZERO, 0.7), 1.0 / 60.0);
        assert!(i.pressed(Action::Spell1));
        i.end_frame(1.0 / 60.0);
        i.pad_event(pad(&[], Vec2::ZERO, 0.45), 1.0 / 60.0);
        assert!(i.down(Action::Spell1) && !i.pressed(Action::Spell1));
        i.pad_event(pad(&[], Vec2::ZERO, 0.1), 1.0 / 60.0);
        assert!(!i.down(Action::Spell1));
    }

    #[test]
    fn a_parked_pointer_never_steers_the_controller() {
        let mut i = Input::default();
        // The window opens under the pointer: that's no move to aim with.
        i.mouse_move(Vec2::new(213.0, 133.0));
        assert!(i.mouse_inside && !i.mouse_aim && !i.mouse_moved);
        // Moving it is.
        i.mouse_move(Vec2::new(220.0, 140.0));
        assert!(i.mouse_aim);
        // Then a button on the controller takes aiming back.
        i.pad_event(pad(&[P::X], Vec2::ZERO, 0.0), 1.0 / 60.0);
        assert!(i.pad_active && !i.mouse_aim);
    }
}
