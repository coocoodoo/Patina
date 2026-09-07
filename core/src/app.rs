//! The winit application: window creation, event translation, frame pacing and presentation.
//!
//! Host callbacks are always delivered from this thread, and never while the state lock is held.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{Key as WKey, NamedKey};
use winit::window::{CursorIcon, Icon, Theme as WinitTheme, Window, WindowId, WindowLevel};

use crate::input::{self, Key, Out};
use crate::layout;
use crate::node::*;
use crate::props::*;
use crate::render;
use crate::state::{self, Id, OutEvent, State, Wake};

const FRAME: Duration = Duration::from_millis(16);
const BLINK: f64 = 0.53;

struct Surf {
    surface: Surface<OwnedDisplayHandle, Arc<Window>>,
    window: Arc<Window>,
    node: Id,
    pixmap: Pixmap,
    last_frame: Instant,
    last_frame_secs: f64,
    animating: bool,
    cursor: CursorKind,
}

#[derive(Clone)]
struct WinSpec {
    title: String,
    width: f32,
    height: f32,
    min_width: f32,
    min_height: f32,
    resizable: bool,
    decorations: bool,
    visible: bool,
    maximized: bool,
    always_on_top: bool,
    icon: Option<Arc<WindowIcon>>,
}

pub struct App {
    context: Option<Context<OwnedDisplayHandle>>,
    surfaces: HashMap<WindowId, Surf>,
    by_node: HashMap<Id, WindowId>,
    auto_quit: Option<Instant>,
    started: bool,
}

/// Deliver events to the host. Must be called without holding the state lock.
pub fn dispatch(events: Vec<OutEvent>) {
    if events.is_empty() {
        return;
    }
    let h = state::lock().handler;
    let func = match h.func {
        Some(f) => f,
        None => return,
    };
    for e in events {
        let (ptr, len) = match &e.text {
            Some(t) => (t.as_ptr(), t.len()),
            None => (std::ptr::null(), 0),
        };
        unsafe { func(h.user, e.node, e.event, e.a, e.b, ptr, len) };
    }
}

/// Run the event loop on the current thread until the last window closes or `quit` is called.
pub fn run() -> Result<(), String> {
    let event_loop = EventLoop::<Wake>::with_user_event()
        .build()
        .map_err(|e| format!("failed to create event loop: {e}"))?;
    let proxy = event_loop.create_proxy();
    {
        let mut s = state::lock();
        if s.running {
            return Err("patina_run called while the event loop is already running".into());
        }
        s.proxy = Some(proxy);
        s.running = true;
        s.quit = false;
    }
    event_loop.set_control_flow(ControlFlow::Wait);
    let auto_quit = std::env::var("PATINA_AUTO_QUIT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|ms| Instant::now() + Duration::from_millis(ms));
    let mut app = App {
        context: None,
        surfaces: HashMap::new(),
        by_node: HashMap::new(),
        auto_quit,
        started: false,
    };
    let result = event_loop.run_app(&mut app);
    {
        let mut s = state::lock();
        s.running = false;
        s.proxy = None;
        for id in s.windows.clone() {
            if let Some(wd) = s.window_data_mut(id) {
                wd.winit = None;
                wd.closed = true;
            }
        }
    }
    result.map_err(|e| format!("event loop error: {e}"))
}

fn spec_of(w: &WindowData) -> WinSpec {
    WinSpec {
        title: w.title.clone(),
        width: w.width,
        height: w.height,
        min_width: w.min_width,
        min_height: w.min_height,
        resizable: w.resizable,
        decorations: w.decorations,
        visible: w.visible,
        maximized: w.maximized,
        always_on_top: w.always_on_top,
        icon: w.icon.clone(),
    }
}

fn make_icon(icon: &WindowIcon) -> Option<Icon> {
    Icon::from_rgba(icon.rgba.clone(), icon.width, icon.height).ok()
}

fn map_key(event: &KeyEvent, shortcut: bool) -> Option<Key> {
    match &event.logical_key {
        WKey::Named(n) => match n {
            NamedKey::Backspace => Some(Key::Backspace),
            NamedKey::Delete => Some(Key::Delete),
            NamedKey::ArrowLeft => Some(Key::Left),
            NamedKey::ArrowRight => Some(Key::Right),
            NamedKey::ArrowUp => Some(Key::Up),
            NamedKey::ArrowDown => Some(Key::Down),
            NamedKey::Home => Some(Key::Home),
            NamedKey::End => Some(Key::End),
            NamedKey::Enter => Some(Key::Enter),
            NamedKey::Tab => Some(Key::Tab),
            NamedKey::Escape => Some(Key::Escape),
            NamedKey::Space => Some(Key::Char(" ".into())),
            _ => None,
        },
        WKey::Character(c) => {
            if shortcut {
                Some(Key::Char(c.to_string()))
            } else {
                match &event.text {
                    Some(t) => Some(Key::Char(t.to_string())),
                    None => Some(Key::Char(c.to_string())),
                }
            }
        }
        _ => None,
    }
}

fn cursor_icon(c: CursorKind) -> CursorIcon {
    match c {
        CursorKind::Default => CursorIcon::Default,
        CursorKind::Pointer => CursorIcon::Pointer,
        CursorKind::Text => CursorIcon::Text,
    }
}

impl App {
    fn create_pending_windows(&mut self, el: &ActiveEventLoop) {
        let pending: Vec<(Id, WinSpec)> = {
            let s = state::lock();
            s.windows
                .iter()
                .filter_map(|&id| {
                    let w = s.window_data(id)?;
                    if w.winit.is_none() && !w.closed {
                        Some((id, spec_of(w)))
                    } else {
                        None
                    }
                })
                .collect()
        };
        for (id, spec) in pending {
            let mut attrs = Window::default_attributes()
                .with_title(spec.title.clone())
                .with_inner_size(LogicalSize::new(
                    spec.width.max(1.0) as f64,
                    spec.height.max(1.0) as f64,
                ))
                .with_resizable(spec.resizable)
                .with_decorations(spec.decorations)
                .with_visible(spec.visible)
                .with_maximized(spec.maximized)
                .with_window_level(if spec.always_on_top {
                    WindowLevel::AlwaysOnTop
                } else {
                    WindowLevel::Normal
                });
            if let Some(icon) = spec.icon.as_deref().and_then(make_icon) {
                attrs = attrs.with_window_icon(Some(icon));
            }
            if spec.min_width > 0.0 || spec.min_height > 0.0 {
                attrs = attrs.with_min_inner_size(LogicalSize::new(
                    spec.min_width.max(1.0) as f64,
                    spec.min_height.max(1.0) as f64,
                ));
            }
            let window = match el.create_window(attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    let msg = format!("failed to create window: {e}");
                    eprintln!("patina: {msg}");
                    let mut s = state::lock();
                    s.set_error(msg);
                    if let Some(wd) = s.window_data_mut(id) {
                        wd.closed = true;
                    }
                    continue;
                }
            };
            if self.context.is_none() {
                match Context::new(el.owned_display_handle()) {
                    Ok(c) => self.context = Some(c),
                    Err(e) => {
                        let msg = format!("failed to create drawing context: {e}");
                        eprintln!("patina: {msg}");
                        state::lock().set_error(msg);
                        continue;
                    }
                }
            }
            let surface =
                match Surface::new(self.context.as_ref().expect("context"), window.clone()) {
                    Ok(s) => s,
                    Err(e) => {
                        let msg = format!("failed to create drawing surface: {e}");
                        eprintln!("patina: {msg}");
                        state::lock().set_error(msg);
                        continue;
                    }
                };
            let wid = window.id();
            let scale = window.scale_factor() as f32;
            let size = window.inner_size();
            let theme = window.theme();
            {
                let mut s = state::lock();
                if let Some(t) = theme {
                    s.theme.system_dark = t == WinitTheme::Dark;
                }
                if let Some(wd) = s.window_data_mut(id) {
                    wd.winit = Some(window.clone());
                    wd.scale = scale;
                    wd.phys_w = size.width;
                    wd.phys_h = size.height;
                    if size.width > 0 && size.height > 0 {
                        wd.width = size.width as f32 / scale;
                        wd.height = size.height as f32 / scale;
                    }
                    wd.layout_dirty = true;
                    wd.needs_redraw = true;
                }
            }
            self.surfaces.insert(
                wid,
                Surf {
                    surface,
                    window: window.clone(),
                    node: id,
                    pixmap: Pixmap::new(1, 1).expect("pixmap"),
                    last_frame: Instant::now(),
                    last_frame_secs: 0.0,
                    animating: false,
                    cursor: CursorKind::Default,
                },
            );
            self.by_node.insert(id, wid);
            window.request_redraw();
        }
    }

    fn close_window(&mut self, id: Id) {
        if let Some(wid) = self.by_node.remove(&id) {
            self.surfaces.remove(&wid);
        }
        let mut s = state::lock();
        if let Some(wd) = s.window_data_mut(id) {
            wd.winit = None;
            wd.closed = true;
            wd.close_requested = false;
            wd.hovered = None;
            wd.pressed = None;
            wd.focused = None;
            wd.drag = Drag::None;
        }
    }

    /// Apply changes made through the API since the last iteration.
    fn process_wake(&mut self, el: &ActiveEventLoop) {
        self.create_pending_windows(el);
        let mut to_close = Vec::new();
        let mut redraws = Vec::new();
        let (posted, pending, quit) = {
            let mut s = state::lock();
            let quit = s.quit;
            let posted = std::mem::take(&mut s.posted);
            let pending = std::mem::take(&mut s.pending_events);
            for id in self.by_node.keys() {
                if !s.nodes.contains(*id) {
                    to_close.push(*id);
                }
            }
            let windows = s.windows.clone();
            for id in windows {
                let wd = match s.window_data_mut(id) {
                    Some(w) => w,
                    None => continue,
                };
                if wd.props_dirty {
                    wd.props_dirty = false;
                    if let Some(w) = &wd.winit {
                        if wd.icon_dirty {
                            wd.icon_dirty = false;
                            w.set_window_icon(wd.icon.as_deref().and_then(make_icon));
                        }
                        w.set_title(&wd.title);
                        w.set_resizable(wd.resizable);
                        w.set_decorations(wd.decorations);
                        w.set_window_level(if wd.always_on_top {
                            WindowLevel::AlwaysOnTop
                        } else {
                            WindowLevel::Normal
                        });
                        w.set_maximized(wd.maximized);
                        w.set_visible(wd.visible);
                        if wd.min_width > 0.0 || wd.min_height > 0.0 {
                            w.set_min_inner_size(Some(LogicalSize::new(
                                wd.min_width.max(1.0) as f64,
                                wd.min_height.max(1.0) as f64,
                            )));
                        } else {
                            w.set_min_inner_size(None::<LogicalSize<f64>>);
                        }
                        let cur = w.inner_size();
                        let want_w = (wd.width * wd.scale).round() as u32;
                        let want_h = (wd.height * wd.scale).round() as u32;
                        if want_w > 0 && want_h > 0 && (want_w != cur.width || want_h != cur.height)
                        {
                            let _ = w.request_inner_size(LogicalSize::new(
                                wd.width as f64,
                                wd.height as f64,
                            ));
                        }
                    }
                }
                if wd.close_requested {
                    to_close.push(id);
                }
                if wd.needs_redraw && wd.winit.is_some() {
                    redraws.push(id);
                }
            }
            (posted, pending, quit)
        };
        for id in to_close {
            self.close_window(id);
        }
        for id in redraws {
            if let Some(surf) = self.by_node.get(&id).and_then(|wid| self.surfaces.get(wid)) {
                surf.window.request_redraw();
            }
        }
        if !pending.is_empty() {
            dispatch(pending);
        }
        if !posted.is_empty() {
            dispatch(
                posted
                    .into_iter()
                    .map(|t| OutEvent::new(0, EV_POSTED).a(t as i64))
                    .collect(),
            );
        }
        if quit {
            el.exit();
        }
    }

    fn after_input(&mut self, wid: WindowId, out: Out) {
        let node = match self.surfaces.get(&wid) {
            Some(s) => s.node,
            None => return,
        };
        let icon = state::lock()
            .window_data(node)
            .map(|w| w.cursor_icon)
            .unwrap_or_default();
        if let Some(surf) = self.surfaces.get_mut(&wid) {
            if surf.cursor != icon {
                surf.cursor = icon;
                surf.window.set_cursor(cursor_icon(icon));
            }
            if out.redraw {
                surf.window.request_redraw();
            }
        }
        dispatch(out.events);
    }

    fn render(&mut self, wid: WindowId) {
        let surf = match self.surfaces.get_mut(&wid) {
            Some(s) => s,
            None => return,
        };
        let size = surf.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        let now = Instant::now();
        let dt = (now - surf.last_frame).as_secs_f32().min(0.1);
        surf.last_frame = now;
        let node = surf.node;
        let animating;
        {
            let mut s = state::lock();
            surf.last_frame_secs = s.now();
            let scale = match s.window_data(node) {
                Some(w) => w.scale,
                None => return,
            };
            s.tick();
            let speed = s.anim_speed;
            if speed <= 0.0 {
                s.theme.settle();
            }
            let clock = s.clock;
            let motion = s.motion;
            animating = render::advance_animations(
                &mut s.nodes,
                node,
                dt * speed.max(0.0),
                clock,
                speed <= 0.0,
                motion,
            ) || s.theme.transitioning(clock);
            let dirty = s
                .window_data(node)
                .map(|w| w.layout_dirty || w.phys_w != size.width || w.phys_h != size.height)
                .unwrap_or(false);
            if dirty {
                {
                    let State { nodes, text, .. } = &mut *s;
                    if !text.ensure_loaded() {
                        if let Some(e) = text.error() {
                            eprintln!("patina: {e}");
                        }
                    }
                    layout::layout_window(
                        nodes,
                        text,
                        node,
                        size.width as f32,
                        size.height as f32,
                        scale,
                    );
                }
                if let Some(wd) = s.window_data_mut(node) {
                    wd.layout_dirty = false;
                    wd.phys_w = size.width;
                    wd.phys_h = size.height;
                }
            }
            if surf.pixmap.width() != size.width || surf.pixmap.height() != size.height {
                surf.pixmap = match Pixmap::new(size.width, size.height) {
                    Some(p) => p,
                    None => return,
                };
            }
            render::render_window(&mut s, node, surf.pixmap.as_mut(), scale);
            if let Some(wd) = s.window_data_mut(node) {
                wd.needs_redraw = false;
                wd.animating = animating;
            }
        }
        surf.animating = animating;
        if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            if surf.surface.resize(w, h).is_ok() {
                if let Ok(mut buf) = surf.surface.buffer_mut() {
                    for (dst, px) in buf.iter_mut().zip(surf.pixmap.pixels()) {
                        *dst = ((px.red() as u32) << 16)
                            | ((px.green() as u32) << 8)
                            | (px.blue() as u32);
                    }
                    surf.window.pre_present_notify();
                    let _ = buf.present();
                }
            }
        }
    }
}

impl ApplicationHandler<Wake> for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.create_pending_windows(el);
        if !self.started {
            self.started = true;
            dispatch(vec![OutEvent::new(0, EV_STARTED)]);
        }
    }

    fn user_event(&mut self, el: &ActiveEventLoop, _event: Wake) {
        self.process_wake(el);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, wid: WindowId, event: WindowEvent) {
        let node = match self.surfaces.get(&wid) {
            Some(s) => s.node,
            None => return,
        };
        match event {
            WindowEvent::CloseRequested => {
                let intercept = state::lock()
                    .window_data(node)
                    .map(|w| w.intercept_close)
                    .unwrap_or(false);
                dispatch(vec![OutEvent::new(node, EV_WINDOW_CLOSE)]);
                if !intercept {
                    self.close_window(node);
                    if self.surfaces.is_empty() {
                        el.exit();
                    }
                }
            }
            WindowEvent::Resized(size) => {
                let mut events = Vec::new();
                {
                    let mut s = state::lock();
                    if let Some(wd) = s.window_data_mut(node) {
                        wd.phys_w = size.width;
                        wd.phys_h = size.height;
                        if size.width > 0 && size.height > 0 {
                            wd.width = size.width as f32 / wd.scale;
                            wd.height = size.height as f32 / wd.scale;
                        }
                        wd.layout_dirty = true;
                        wd.needs_redraw = true;
                        events.push(
                            OutEvent::new(node, EV_WINDOW_RESIZED)
                                .a(wd.width.round() as i64)
                                .b(wd.height.round() as i64),
                        );
                    }
                }
                if let Some(surf) = self.surfaces.get(&wid) {
                    surf.window.request_redraw();
                }
                dispatch(events);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let mut s = state::lock();
                if let Some(wd) = s.window_data_mut(node) {
                    wd.scale = scale_factor as f32;
                    wd.layout_dirty = true;
                    wd.needs_redraw = true;
                }
                drop(s);
                if let Some(surf) = self.surfaces.get(&wid) {
                    surf.window.request_redraw();
                }
            }
            WindowEvent::ThemeChanged(theme) => {
                {
                    let mut s = state::lock();
                    let clock = s.clock;
                    s.theme.set_system_dark(theme == WinitTheme::Dark, clock);
                    if !s.started() {
                        s.theme.settle();
                    }
                }
                for surf in self.surfaces.values() {
                    surf.window.request_redraw();
                }
            }
            WindowEvent::Focused(focused) => {
                if !focused {
                    let out = {
                        let mut s = state::lock();
                        let mut o = input::cursor_left(&mut s, node);
                        let closed = input::close_menus(&mut s, node);
                        if !closed.is_empty() {
                            o.events.extend(closed);
                            o.redraw = true;
                        }
                        o
                    };
                    self.after_input(wid, out);
                }
                dispatch(vec![OutEvent::new(node, EV_WINDOW_FOCUS).a(focused as i64)]);
            }
            WindowEvent::RedrawRequested => self.render(wid),
            WindowEvent::CursorMoved { position, .. } => {
                let out = {
                    let mut s = state::lock();
                    input::cursor_moved(&mut s, node, position.x as f32, position.y as f32)
                };
                self.after_input(wid, out);
            }
            WindowEvent::CursorLeft { .. } => {
                let out = {
                    let mut s = state::lock();
                    input::cursor_left(&mut s, node)
                };
                self.after_input(wid, out);
            }
            WindowEvent::MouseInput {
                state: st, button, ..
            } => {
                let b = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    _ => 3,
                };
                let out = {
                    let mut s = state::lock();
                    input::mouse_button(&mut s, node, st == ElementState::Pressed, b)
                };
                self.after_input(wid, out);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let out = {
                    let mut s = state::lock();
                    let scale = s.window_data(node).map(|w| w.scale).unwrap_or(1.0);
                    let (dx, dy) = match delta {
                        MouseScrollDelta::LineDelta(x, y) => (x * 40.0 * scale, y * 40.0 * scale),
                        MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                    };
                    input::wheel(&mut s, node, dx, dy)
                };
                self.after_input(wid, out);
            }
            WindowEvent::ModifiersChanged(m) => {
                let st = m.state();
                let mut s = state::lock();
                if let Some(wd) = s.window_data_mut(node) {
                    wd.modifiers = Modifiers {
                        shift: st.shift_key(),
                        ctrl: st.control_key(),
                        alt: st.alt_key(),
                        meta: st.super_key(),
                    };
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                let out = {
                    let mut s = state::lock();
                    let mods = s.window_data(node).map(|w| w.modifiers).unwrap_or_default();
                    match map_key(&event, mods.ctrl || mods.meta) {
                        Some(key) => input::key_down(&mut s, node, key, mods),
                        None => Out::default(),
                    }
                };
                self.after_input(wid, out);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.process_wake(el);
        let now = Instant::now();
        let fired = state::lock().due_timers(now);
        if !fired.is_empty() {
            dispatch(
                fired
                    .into_iter()
                    .map(|t| OutEvent::new(0, EV_TIMER).a(t as i64))
                    .collect(),
            );
            self.process_wake(el);
        }
        if let Some(d) = self.auto_quit {
            if now >= d {
                el.exit();
                return;
            }
        }
        if self.started && self.surfaces.is_empty() {
            // No live windows: keep running only while a window is still waiting to be created.
            let open = {
                let s = state::lock();
                s.windows
                    .iter()
                    .any(|w| s.window_data(*w).map(|d| !d.closed).unwrap_or(false))
            };
            if !open {
                el.exit();
                return;
            }
        }

        let tooltip_windows = {
            let mut s = state::lock();
            let secs = s.now();
            input::tooltips_due(&mut s, secs)
        };
        for id in tooltip_windows {
            if let Some(surf) = self.by_node.get(&id).and_then(|w| self.surfaces.get(w)) {
                surf.window.request_redraw();
            }
        }
        let mut next: Option<Instant> = {
            let s = state::lock();
            let mut next = s.next_timer_deadline();
            if let Some(d) = input::next_tooltip_deadline(&s) {
                let due = s.start + Duration::from_secs_f64(d.max(0.0));
                next = Some(next.map_or(due, |n| n.min(due)));
            }
            next
        };
        if let Some(d) = self.auto_quit {
            next = Some(next.map_or(d, |n| n.min(d)));
        }
        let mut redraw: Vec<WindowId> = Vec::new();
        for (wid, surf) in &self.surfaces {
            if surf.animating {
                let due = surf.last_frame + FRAME;
                if due <= now {
                    redraw.push(*wid);
                } else {
                    next = Some(next.map_or(due, |n| n.min(due)));
                }
                continue;
            }
            let blink = {
                let s = state::lock();
                let epoch = s
                    .window_data(surf.node)
                    .and_then(|w| w.focused)
                    .and_then(|f| s.nodes.get(f))
                    .filter(|n| n.kind == Kind::TextInput && !n.disabled)
                    .map(|n| n.input.blink_epoch);
                epoch.map(|e| (e, s.now()))
            };
            if let Some((epoch, now_secs)) = blink {
                let elapsed = (now_secs - epoch).max(0.0);
                let last_toggle = epoch + (elapsed / BLINK).floor() * BLINK;
                if surf.last_frame_secs < last_toggle {
                    redraw.push(*wid);
                } else {
                    let wait = (last_toggle + BLINK - now_secs).max(0.005);
                    let due = now + Duration::from_secs_f64(wait);
                    next = Some(next.map_or(due, |n| n.min(due)));
                }
            }
        }
        for wid in redraw {
            if let Some(surf) = self.surfaces.get(&wid) {
                surf.window.request_redraw();
            }
        }
        el.set_control_flow(match next {
            Some(t) => ControlFlow::WaitUntil(t),
            None => ControlFlow::Wait,
        });
    }
}
