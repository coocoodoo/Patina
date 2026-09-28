//! The window: winit events in, softbuffer pixels out, paced to 60 frames a second.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{
    ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Icon, Window, WindowId};

use crate::audio::Audio;
use crate::game::{Game, Io};
use crate::input::{Button, Input};
use crate::render::Renderer;

const FRAME: Duration = Duration::from_micros(16_667);

/// Integer pixel scale for a window size: the internal image is ~225-270 pixels tall.
pub fn pixel_scale(w: u32, h: u32) -> usize {
    let by_h = (h / 225).max(1);
    let by_w = (w / 400).max(1);
    by_h.min(by_w) as usize
}

struct Surf {
    window: Arc<Window>,
    surface: Surface<OwnedDisplayHandle, Arc<Window>>,
}

pub struct App {
    context: Option<Context<OwnedDisplayHandle>>,
    surf: Option<Surf>,
    game: Game,
    renderer: Renderer,
    input: Input,
    audio: Audio,
    scale: usize,
    last: Instant,
    next: Instant,
    fullscreen: bool,
    shot_requested: bool,
}

impl App {
    pub fn new(game: Game, audio: Audio) -> App {
        let fullscreen = game.settings.fullscreen;
        audio.set_volume(game.settings.music, game.settings.sfx);
        App {
            context: None,
            surf: None,
            game,
            renderer: Renderer::new(427, 240),
            input: Input::default(),
            audio,
            scale: 3,
            last: Instant::now(),
            next: Instant::now(),
            fullscreen,
            shot_requested: false,
        }
    }

    fn set_fullscreen(&mut self, on: bool) {
        self.fullscreen = on;
        self.game.settings.fullscreen = on;
        self.game.settings.dirty = true;
        if let Some(s) = &self.surf {
            s.window.set_fullscreen(if on {
                Some(Fullscreen::Borderless(None))
            } else {
                None
            });
        }
    }

    fn frame(&mut self, el: &ActiveEventLoop) {
        let Some(s) = &mut self.surf else { return };
        let size = s.window.inner_size();
        let (pw, ph) = (size.width.max(1), size.height.max(1));
        self.scale = pixel_scale(pw, ph);
        let iw = (pw as usize).div_ceil(self.scale);
        let ih = (ph as usize).div_ceil(self.scale);
        self.renderer.resize(iw, ih);

        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        let mut io = Io {
            dt,
            input: &self.input,
            audio: &self.audio,
            view: (iw, ih),
            quit: false,
            toggle_fullscreen: false,
        };
        self.game.update(&mut io);
        let (quit, toggle) = (io.quit, io.toggle_fullscreen);
        self.input.end_frame(dt);
        if quit {
            self.game.shutdown();
            el.exit();
            return;
        }
        if toggle {
            let on = !self.fullscreen;
            self.set_fullscreen(on);
        }
        self.game.draw(&mut self.renderer, &self.input);
        if self.shot_requested {
            self.shot_requested = false;
            if let Some(dir) = crate::game::save::dir() {
                let dir = dir.join("screenshots");
                let _ = std::fs::create_dir_all(&dir);
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let path = dir.join(format!("hollowbloom-{stamp}.png"));
                if crate::shot::save_png(&path, &self.renderer.fb, self.scale.max(2)).is_ok() {
                    eprintln!("screenshot saved to {}", path.display());
                }
            }
        }
        let Some(s) = &mut self.surf else { return };
        if let (Some(w), Some(h)) = (NonZeroU32::new(pw), NonZeroU32::new(ph)) {
            if s.surface.resize(w, h).is_ok() {
                if let Ok(mut buf) = s.surface.buffer_mut() {
                    self.renderer.fb.blit_scaled(
                        &mut buf,
                        pw as usize,
                        ph as usize,
                        self.scale,
                        &self.renderer.sh.rgb,
                    );
                    s.window.pre_present_notify();
                    let _ = buf.present();
                }
            }
        }
    }
}

fn icon() -> Option<Icon> {
    // The sprout, drawn in palette colours.
    const ART: [&str; 16] = [
        "................",
        "......gg........",
        ".....glg...gg...",
        ".....glg..glg...",
        "......gg.glg....",
        ".......ggg......",
        "........g.......",
        "....ccccgccc....",
        "...cooooooooc...",
        "..cooyoooooooc..",
        "..cooooooooooc..",
        "..cooooooooorc..",
        "...cooooooorc...",
        "....crrrrrrc....",
        ".....cccccc.....",
        "................",
    ];
    let col = |c: char| -> Option<u32> {
        Some(match c {
            'g' => 0x1ebc73,
            'l' => 0x91db69,
            'c' => 0x3e3546,
            'o' => 0xfbb954,
            'y' => 0xfbff86,
            'r' => 0xcd683d,
            _ => return None,
        })
    };
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32 {
        for x in 0..32 {
            let ch = ART[y / 2].as_bytes()[x / 2] as char;
            match col(ch) {
                Some(c) => rgba.extend([(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]),
                None => rgba.extend([0, 0, 0, 0]),
            }
        }
    }
    Icon::from_rgba(rgba, 32, 32).ok()
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.surf.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes()
            .with_title("Hollowbloom")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(LogicalSize::new(400.0, 225.0))
            .with_window_icon(icon());
        if self.fullscreen {
            attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        let window = match el.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("hollowbloom: could not create a window: {e}");
                el.exit();
                return;
            }
        };
        if self.context.is_none() {
            match Context::new(el.owned_display_handle()) {
                Ok(c) => self.context = Some(c),
                Err(e) => {
                    eprintln!("hollowbloom: could not create a drawing context: {e}");
                    el.exit();
                    return;
                }
            }
        }
        let surface = match Surface::new(self.context.as_ref().unwrap(), window.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("hollowbloom: could not create a drawing surface: {e}");
                el.exit();
                return;
            }
        };
        window.set_cursor_visible(false);
        window.request_redraw();
        self.surf = Some(Surf { window, surface });
        self.last = Instant::now();
        self.next = Instant::now();
    }

    fn new_events(&mut self, _el: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(s) = &self.surf {
                s.window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.game.shutdown();
                el.exit();
            }
            WindowEvent::Focused(false) => self.input.release_all(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                if pressed && !repeat {
                    let alt = self.input.key_down(KeyCode::AltLeft)
                        || self.input.key_down(KeyCode::AltRight);
                    if code == KeyCode::F11 || (alt && code == KeyCode::Enter) {
                        let on = !self.fullscreen;
                        self.set_fullscreen(on);
                        return;
                    }
                    if code == KeyCode::F12 {
                        self.shot_requested = true;
                        return;
                    }
                }
                self.input.key_event(code, pressed, repeat);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let b = match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Right => Button::Right,
                    MouseButton::Middle => Button::Middle,
                    _ => return,
                };
                self.input.button_event(b, state == ElementState::Pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let s = self.scale.max(1) as f32;
                self.input.mouse_move(glam::Vec2::new(
                    position.x as f32 / s,
                    position.y as f32 / s,
                ));
            }
            WindowEvent::CursorLeft { .. } => self.input.mouse_inside = false,
            WindowEvent::MouseWheel { delta, .. } => {
                self.input.wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 40.0) as f32,
                };
            }
            WindowEvent::RedrawRequested => {
                self.frame(el);
                self.next = Instant::now() + FRAME;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        let now = Instant::now();
        if now >= self.next {
            if let Some(s) = &self.surf {
                s.window.request_redraw();
            }
            self.next = now + FRAME;
        }
        el.set_control_flow(ControlFlow::WaitUntil(self.next));
    }
}

pub fn run(game: Game, audio: Audio) -> Result<(), String> {
    crate::audio::precise_timers();
    let el = EventLoop::new().map_err(|e| format!("could not start the event loop: {e}"))?;
    el.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(game, audio);
    el.run_app(&mut app).map_err(|e| e.to_string())
}
