//! Handheld consoles that run Linux with no desktop, like the Anbernic RG351P on ArkOS,
//! AmberELEC or ROCKNIX: the screen, the controls and any keyboard through SDL2, which every
//! such system ships (it's what their own menus and ports run on). SDL is loaded when the
//! game starts, so the game still needs nothing installed to run anywhere else.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::time::{Duration, Instant};

use libloading::Library;

use crate::audio::Audio;
use crate::game::{Game, Io};
use crate::input::{Input, KeyCode};
use crate::pad::{PadButton as P, PadState};
use crate::render::Renderer;

const INIT_VIDEO: u32 = 0x20;
const INIT_JOYSTICK: u32 = 0x200;
const INIT_GAMECONTROLLER: u32 = 0x2000;
const INIT_EVENTS: u32 = 0x4000;
const WINDOW_FULLSCREEN: u32 = 0x1;
const WINDOW_SHOWN: u32 = 0x4;
const WINDOWPOS_UNDEFINED: c_int = 0x1FFF_0000;
const RENDERER_ACCELERATED: u32 = 0x2;
const RENDERER_PRESENTVSYNC: u32 = 0x4;
/// 32-bit pixels holding 0x00RRGGBB, as the frame is blitted.
const PIXELFORMAT_RGB888: u32 = 0x1616_1804;
const TEXTUREACCESS_STREAMING: c_int = 1;

const QUIT: u32 = 0x100;
const KEYDOWN: u32 = 0x300;
const KEYUP: u32 = 0x301;
const JOYDEVICEADDED: u32 = 0x605;
const JOYDEVICEREMOVED: u32 = 0x606;
const CONTROLLERDEVICEADDED: u32 = 0x653;
const CONTROLLERDEVICEREMOVED: u32 = 0x654;

const FRAME: Duration = Duration::from_micros(16_667);

#[repr(C)]
#[derive(Default)]
struct DisplayMode {
    format: u32,
    w: c_int,
    h: c_int,
    refresh_rate: c_int,
    driverdata: usize,
}

#[repr(C)]
struct Rect {
    x: c_int,
    y: c_int,
    w: c_int,
    h: c_int,
}

type Ptr = *mut c_void;

/// The SDL functions the game uses, with their documented C signatures.
struct Sdl {
    _lib: Library,
    init: unsafe extern "C" fn(u32) -> c_int,
    quit: unsafe extern "C" fn(),
    get_error: unsafe extern "C" fn() -> *const c_char,
    set_hint: unsafe extern "C" fn(*const c_char, *const c_char) -> c_int,
    get_current_display_mode: unsafe extern "C" fn(c_int, *mut DisplayMode) -> c_int,
    create_window: unsafe extern "C" fn(*const c_char, c_int, c_int, c_int, c_int, u32) -> Ptr,
    destroy_window: unsafe extern "C" fn(Ptr),
    show_cursor: unsafe extern "C" fn(c_int) -> c_int,
    create_renderer: unsafe extern "C" fn(Ptr, c_int, u32) -> Ptr,
    destroy_renderer: unsafe extern "C" fn(Ptr),
    get_renderer_output_size: unsafe extern "C" fn(Ptr, *mut c_int, *mut c_int) -> c_int,
    create_texture: unsafe extern "C" fn(Ptr, u32, c_int, c_int, c_int) -> Ptr,
    destroy_texture: unsafe extern "C" fn(Ptr),
    update_texture: unsafe extern "C" fn(Ptr, *const Rect, *const c_void, c_int) -> c_int,
    set_render_draw_color: unsafe extern "C" fn(Ptr, u8, u8, u8, u8) -> c_int,
    render_clear: unsafe extern "C" fn(Ptr) -> c_int,
    render_copy: unsafe extern "C" fn(Ptr, Ptr, *const Rect, *const Rect) -> c_int,
    render_present: unsafe extern "C" fn(Ptr),
    poll_event: unsafe extern "C" fn(*mut c_void) -> c_int,
    num_joysticks: unsafe extern "C" fn() -> c_int,
    is_game_controller: unsafe extern "C" fn(c_int) -> c_int,
    gc_open: unsafe extern "C" fn(c_int) -> Ptr,
    gc_close: unsafe extern "C" fn(Ptr),
    gc_button: unsafe extern "C" fn(Ptr, c_int) -> u8,
    gc_axis: unsafe extern "C" fn(Ptr, c_int) -> i16,
    joy_open: unsafe extern "C" fn(c_int) -> Ptr,
    joy_close: unsafe extern "C" fn(Ptr),
    joy_button: unsafe extern "C" fn(Ptr, c_int) -> u8,
    joy_axis: unsafe extern "C" fn(Ptr, c_int) -> i16,
    joy_hat: unsafe extern "C" fn(Ptr, c_int) -> u8,
    joy_num_axes: unsafe extern "C" fn(Ptr) -> c_int,
    joy_num_hats: unsafe extern "C" fn(Ptr) -> c_int,
}

impl Sdl {
    fn load() -> Option<Sdl> {
        // SAFETY: loading the system's SDL2 and resolving documented symbols with their
        // documented C signatures.
        unsafe {
            let lib = ["libSDL2-2.0.so.0", "libSDL2-2.0.so", "libSDL2.so"]
                .iter()
                .find_map(|name| Library::new(name).ok())?;
            macro_rules! sym {
                ($name:literal) => {
                    *lib.get(concat!($name, "\0").as_bytes()).ok()?
                };
            }
            Some(Sdl {
                init: sym!("SDL_Init"),
                quit: sym!("SDL_Quit"),
                get_error: sym!("SDL_GetError"),
                set_hint: sym!("SDL_SetHint"),
                get_current_display_mode: sym!("SDL_GetCurrentDisplayMode"),
                create_window: sym!("SDL_CreateWindow"),
                destroy_window: sym!("SDL_DestroyWindow"),
                show_cursor: sym!("SDL_ShowCursor"),
                create_renderer: sym!("SDL_CreateRenderer"),
                destroy_renderer: sym!("SDL_DestroyRenderer"),
                get_renderer_output_size: sym!("SDL_GetRendererOutputSize"),
                create_texture: sym!("SDL_CreateTexture"),
                destroy_texture: sym!("SDL_DestroyTexture"),
                update_texture: sym!("SDL_UpdateTexture"),
                set_render_draw_color: sym!("SDL_SetRenderDrawColor"),
                render_clear: sym!("SDL_RenderClear"),
                render_copy: sym!("SDL_RenderCopy"),
                render_present: sym!("SDL_RenderPresent"),
                poll_event: sym!("SDL_PollEvent"),
                num_joysticks: sym!("SDL_NumJoysticks"),
                is_game_controller: sym!("SDL_IsGameController"),
                gc_open: sym!("SDL_GameControllerOpen"),
                gc_close: sym!("SDL_GameControllerClose"),
                gc_button: sym!("SDL_GameControllerGetButton"),
                gc_axis: sym!("SDL_GameControllerGetAxis"),
                joy_open: sym!("SDL_JoystickOpen"),
                joy_close: sym!("SDL_JoystickClose"),
                joy_button: sym!("SDL_JoystickGetButton"),
                joy_axis: sym!("SDL_JoystickGetAxis"),
                joy_hat: sym!("SDL_JoystickGetHat"),
                joy_num_axes: sym!("SDL_JoystickNumAxes"),
                joy_num_hats: sym!("SDL_JoystickNumHats"),
                _lib: lib,
            })
        }
    }

    fn error(&self) -> String {
        // SAFETY: SDL_GetError returns a valid, NUL-terminated string.
        unsafe { CStr::from_ptr((self.get_error)()) }
            .to_string_lossy()
            .into_owned()
    }
}

/// Whether SDL2 is there to use (a handheld's system always has it).
pub fn available() -> bool {
    Sdl::load().is_some()
}

/// A controller SDL knows the layout of, or a joystick it doesn't (read in the usual order).
enum Pad {
    Mapped(Ptr),
    Raw(Ptr),
}

/// Every controller plugged in.
struct Pads {
    open: Vec<Pad>,
    /// Swap the face buttons, for a system that maps them by position rather than label.
    swap: bool,
}

impl Pads {
    fn scan(&mut self, sdl: &Sdl) {
        self.close(sdl);
        // SAFETY: plain SDL calls on indexes it just counted; each opened handle is kept
        // until it's closed.
        unsafe {
            for i in 0..(sdl.num_joysticks)() {
                if (sdl.is_game_controller)(i) != 0 {
                    let gc = (sdl.gc_open)(i);
                    if !gc.is_null() {
                        self.open.push(Pad::Mapped(gc));
                    }
                } else {
                    let joy = (sdl.joy_open)(i);
                    if !joy.is_null() {
                        self.open.push(Pad::Raw(joy));
                    }
                }
            }
        }
    }

    fn close(&mut self, sdl: &Sdl) {
        for p in self.open.drain(..) {
            // SAFETY: each handle was opened by `scan` and is closed once.
            unsafe {
                match p {
                    Pad::Mapped(gc) => (sdl.gc_close)(gc),
                    Pad::Raw(joy) => (sdl.joy_close)(joy),
                }
            }
        }
    }

    /// Everything the controllers are doing, as one pad.
    fn state(&self, sdl: &Sdl) -> PadState {
        let mut s = PadState::default();
        let axis = |v: i16| (v as f32 / 32767.0).clamp(-1.0, 1.0);
        for p in &self.open {
            s.connected = true;
            let mut one = PadState::default();
            // SAFETY: reading the state of handles `scan` opened.
            unsafe {
                match *p {
                    Pad::Mapped(gc) => {
                        // SDL's buttons: A B X Y, Back, Guide, Start, the sticks, the
                        // shoulders, then the D-pad.
                        let map = [
                            (0, P::A),
                            (1, P::B),
                            (2, P::X),
                            (3, P::Y),
                            (4, P::View),
                            (6, P::Menu),
                            (7, P::L3),
                            (8, P::R3),
                            (9, P::L1),
                            (10, P::R1),
                            (11, P::Up),
                            (12, P::Down),
                            (13, P::Left),
                            (14, P::Right),
                        ];
                        for (b, to) in map {
                            if (sdl.gc_button)(gc, b) != 0 {
                                one.buttons |= to.bit();
                            }
                        }
                        let a = |i| axis((sdl.gc_axis)(gc, i));
                        one.left = glam::Vec2::new(a(0), a(1));
                        one.right = glam::Vec2::new(a(2), a(3));
                        one.lt = a(4).max(0.0);
                        one.rt = a(5).max(0.0);
                    }
                    Pad::Raw(joy) => {
                        // No layout known: the order most handhelds' own pads use.
                        let map = [
                            (0, P::A),
                            (1, P::B),
                            (2, P::X),
                            (3, P::Y),
                            (4, P::L1),
                            (5, P::R1),
                            (6, P::View),
                            (7, P::Menu),
                            (8, P::L3),
                            (9, P::R3),
                            (10, P::L2),
                            (11, P::R2),
                        ];
                        for (b, to) in map {
                            if (sdl.joy_button)(joy, b) != 0 {
                                one.buttons |= to.bit();
                            }
                        }
                        let axes = (sdl.joy_num_axes)(joy);
                        let a = |i| {
                            if i < axes {
                                axis((sdl.joy_axis)(joy, i))
                            } else {
                                0.0
                            }
                        };
                        one.left = glam::Vec2::new(a(0), a(1));
                        one.right = glam::Vec2::new(a(2), a(3));
                        if (sdl.joy_num_hats)(joy) > 0 {
                            let hat = (sdl.joy_hat)(joy, 0);
                            for (bit, to) in
                                [(1u8, P::Up), (2, P::Right), (4, P::Down), (8, P::Left)]
                            {
                                if hat & bit != 0 {
                                    one.buttons |= to.bit();
                                }
                            }
                        }
                        // Digital triggers.
                        if one.held(P::L2) {
                            one.lt = 1.0;
                        }
                        if one.held(P::R2) {
                            one.rt = 1.0;
                        }
                    }
                }
            }
            if self.swap {
                one.buttons = swap_faces(one.buttons);
            }
            s.buttons |= one.buttons;
            if one.left.length_squared() > s.left.length_squared() {
                s.left = one.left;
            }
            if one.right.length_squared() > s.right.length_squared() {
                s.right = one.right;
            }
            s.lt = s.lt.max(one.lt);
            s.rt = s.rt.max(one.rt);
        }
        s
    }
}

/// A and B, and X and Y, the other way round.
fn swap_faces(b: u16) -> u16 {
    let mut out = b & !(P::A.bit() | P::B.bit() | P::X.bit() | P::Y.bit());
    for (from, to) in [(P::A, P::B), (P::B, P::A), (P::X, P::Y), (P::Y, P::X)] {
        if b & from.bit() != 0 {
            out |= to.bit();
        }
    }
    out
}

/// The keys the game knows, by SDL's scancode (the USB usage number).
fn key(scancode: i32) -> Option<KeyCode> {
    use KeyCode::*;
    const LETTERS: [KeyCode; 26] = [
        KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO,
        KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    ];
    const DIGITS: [KeyCode; 10] = [
        Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Digit0,
    ];
    Some(match scancode {
        4..=29 => LETTERS[(scancode - 4) as usize],
        30..=39 => DIGITS[(scancode - 30) as usize],
        40 => Enter,
        41 => Escape,
        42 => Backspace,
        43 => Tab,
        44 => Space,
        47 => BracketLeft,
        48 => BracketRight,
        68 => F11,
        69 => F12,
        79 => ArrowRight,
        80 => ArrowLeft,
        81 => ArrowDown,
        82 => ArrowUp,
        88 => NumpadEnter,
        225 => ShiftLeft,
        229 => ShiftRight,
        _ => return None,
    })
}

/// How long Select and Start are held together to save and quit (the way out of a port on
/// these handhelds).
const QUIT_HOLD: f32 = 1.0;

/// Plays the game fullscreen through SDL until it's quit.
pub fn run(mut game: Game, audio: Audio) -> Result<(), String> {
    let sdl = Sdl::load().ok_or("SDL2 isn't installed")?;
    crate::audio::precise_timers();
    crate::app::catch_close_signals();
    // SAFETY: SDL calls in the order SDL documents, from this one thread, on handles
    // checked for null before use and released before returning.
    unsafe {
        (sdl.set_hint)(c"SDL_RENDER_SCALE_QUALITY".as_ptr(), c"0".as_ptr());
        if (sdl.init)(INIT_VIDEO | INIT_JOYSTICK | INIT_GAMECONTROLLER | INIT_EVENTS) != 0 {
            return Err(format!("SDL couldn't start: {}", sdl.error()));
        }
        let mut mode = DisplayMode::default();
        if (sdl.get_current_display_mode)(0, &mut mode) != 0 || mode.w <= 0 || mode.h <= 0 {
            mode.w = 480;
            mode.h = 320;
        }
        let window = (sdl.create_window)(
            c"Hollowbloom".as_ptr(),
            WINDOWPOS_UNDEFINED,
            WINDOWPOS_UNDEFINED,
            mode.w,
            mode.h,
            WINDOW_FULLSCREEN | WINDOW_SHOWN,
        );
        if window.is_null() {
            let e = sdl.error();
            (sdl.quit)();
            return Err(format!("SDL couldn't open the screen: {e}"));
        }
        (sdl.show_cursor)(0);
        let mut vsync = true;
        let mut renderer =
            (sdl.create_renderer)(window, -1, RENDERER_ACCELERATED | RENDERER_PRESENTVSYNC);
        if renderer.is_null() {
            vsync = false;
            renderer = (sdl.create_renderer)(window, -1, 0);
        }
        if renderer.is_null() {
            let e = sdl.error();
            (sdl.destroy_window)(window);
            (sdl.quit)();
            return Err(format!("SDL couldn't draw to the screen: {e}"));
        }
        let (mut sw, mut sh) = (mode.w, mode.h);
        (sdl.get_renderer_output_size)(renderer, &mut sw, &mut sh);
        let (sw, sh) = (sw.max(1) as u32, sh.max(1) as u32);
        let scale = crate::app::pixel_scale(sw, sh);
        let iw = (sw as usize).div_ceil(scale);
        let ih = (sh as usize).div_ceil(scale);
        let texture = (sdl.create_texture)(
            renderer,
            PIXELFORMAT_RGB888,
            TEXTUREACCESS_STREAMING,
            iw as c_int,
            ih as c_int,
        );
        if texture.is_null() {
            let e = sdl.error();
            (sdl.destroy_renderer)(renderer);
            (sdl.destroy_window)(window);
            (sdl.quit)();
            return Err(format!("SDL couldn't make a picture to draw into: {e}"));
        }
        // The picture, scaled up whole and centred (any spare pixels fall off the edges).
        let dst = Rect {
            x: (sw as c_int - (iw * scale) as c_int) / 2,
            y: (sh as c_int - (ih * scale) as c_int) / 2,
            w: (iw * scale) as c_int,
            h: (ih * scale) as c_int,
        };

        audio.set_volume(game.settings.music, game.settings.sfx);
        // Its controls are shown from the start, before any are pressed.
        let mut input = Input::default();
        input.pad_active = true;
        let mut pads = Pads {
            open: Vec::new(),
            swap: std::env::var_os("HOLLOWBLOOM_SWAP_AB").is_some(),
        };
        pads.scan(&sdl);
        let mut r = Renderer::new(iw, ih);
        let mut pixels = vec![0u32; iw * ih];
        let mut event = [0u64; 8];
        let mut last = Instant::now();
        let mut next = Instant::now();
        let mut quit_hold = 0.0;
        loop {
            let mut quit = crate::app::close_asked();
            while (sdl.poll_event)(event.as_mut_ptr().cast()) != 0 {
                let bytes: &[u8; 64] = &*(event.as_ptr().cast());
                let ty = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                match ty {
                    QUIT => quit = true,
                    KEYDOWN | KEYUP => {
                        let code = i32::from_ne_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
                        if let Some(k) = key(code) {
                            input.key_event(k, ty == KEYDOWN, bytes[13] != 0);
                        }
                    }
                    JOYDEVICEADDED
                    | JOYDEVICEREMOVED
                    | CONTROLLERDEVICEADDED
                    | CONTROLLERDEVICEREMOVED => pads.scan(&sdl),
                    _ => {}
                }
            }
            let now = Instant::now();
            let dt = (now - last).as_secs_f32().min(0.1);
            last = now;
            let pad = pads.state(&sdl);
            // Select and Start held together: save and quit.
            quit_hold = if pad.held(P::View) && pad.held(P::Menu) {
                quit_hold + dt
            } else {
                0.0
            };
            input.pad_event(pad, dt);
            let mut io = Io {
                dt,
                input: &input,
                audio: &audio,
                view: (iw, ih),
                quit: false,
                toggle_fullscreen: false,
            };
            game.update(&mut io);
            quit |= io.quit || quit_hold >= QUIT_HOLD;
            input.end_frame(dt);
            if quit {
                game.shutdown();
                break;
            }
            game.draw(&mut r, &input);
            r.fb.blit_scaled(&mut pixels, iw, ih, 1, &r.sh.rgb);
            (sdl.update_texture)(
                texture,
                std::ptr::null(),
                pixels.as_ptr().cast(),
                (iw * 4) as c_int,
            );
            (sdl.set_render_draw_color)(renderer, 0, 0, 0, 255);
            (sdl.render_clear)(renderer);
            (sdl.render_copy)(renderer, texture, std::ptr::null(), &dst);
            (sdl.render_present)(renderer);
            // Without vsync to pace it, wait out the rest of the frame.
            if !vsync {
                next += FRAME;
                let now = Instant::now();
                if next > now {
                    std::thread::sleep(next - now);
                } else {
                    next = now;
                }
            }
        }
        pads.close(&sdl);
        (sdl.destroy_texture)(texture);
        (sdl.destroy_renderer)(renderer);
        (sdl.destroy_window)(window);
        (sdl.quit)();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_come_through_by_scancode() {
        assert_eq!(key(4), Some(KeyCode::KeyA));
        assert_eq!(key(29), Some(KeyCode::KeyZ));
        assert_eq!(key(30), Some(KeyCode::Digit1));
        assert_eq!(key(39), Some(KeyCode::Digit0));
        assert_eq!(key(41), Some(KeyCode::Escape));
        assert_eq!(key(82), Some(KeyCode::ArrowUp));
        assert_eq!(key(1), None);
    }

    #[test]
    fn swapping_faces_leaves_the_rest_alone() {
        let b = P::A.bit() | P::Y.bit() | P::L1.bit();
        assert_eq!(swap_faces(b), P::B.bit() | P::X.bit() | P::L1.bit());
        assert_eq!(swap_faces(swap_faces(b)), b);
    }
}
