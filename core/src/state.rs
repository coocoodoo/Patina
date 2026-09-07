//! Global application state: the node arena, windows, timers, and the host event handler.
//!
//! Everything lives behind one mutex. The rule that keeps this simple: the lock is never
//! held while calling back into the host, so host callbacks may freely call the API.

use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::anim::Motion;
use crate::node::*;
use crate::paint::ShadowCache;
use crate::props::*;
use crate::svg::SvgCache;
use crate::text::TextSystem;
use crate::theme::Theme;

pub use crate::node::Id;

/// Host callback. `text` is only valid for the duration of the call.
pub type EventFn = unsafe extern "C" fn(
    user: usize,
    node: u64,
    event: u32,
    a: i64,
    b: i64,
    text: *const u8,
    text_len: usize,
);

#[derive(Clone, Copy, Default)]
pub struct Handler {
    pub func: Option<EventFn>,
    pub user: usize,
}

#[derive(Clone, Debug)]
pub struct OutEvent {
    pub node: Id,
    pub event: u32,
    pub a: i64,
    pub b: i64,
    pub text: Option<String>,
}

impl OutEvent {
    pub fn new(node: Id, event: u32) -> OutEvent {
        OutEvent {
            node,
            event,
            a: 0,
            b: 0,
            text: None,
        }
    }

    pub fn a(mut self, a: i64) -> OutEvent {
        self.a = a;
        self
    }

    pub fn b(mut self, b: i64) -> OutEvent {
        self.b = b;
        self
    }

    pub fn text(mut self, t: impl Into<String>) -> OutEvent {
        self.text = Some(t.into());
        self
    }

    pub fn f64(mut self, v: f64) -> OutEvent {
        self.a = v.to_bits() as i64;
        self
    }
}

/// User event used to wake the winit loop from other threads.
#[derive(Clone, Copy, Debug)]
pub struct Wake;

#[derive(Clone, Debug)]
pub struct Timer {
    pub token: u64,
    pub deadline: Instant,
    pub interval: Option<Duration>,
}

struct Slot {
    generation: u32,
    node: Option<Box<Node>>,
}

/// Generational arena. Ids are `(generation << 32) | (index + 1)`.
#[derive(Default)]
pub struct Arena {
    slots: Vec<Slot>,
    free: Vec<usize>,
}

impl Arena {
    fn split(id: Id) -> Option<(usize, u32)> {
        let low = (id & 0xffff_ffff) as usize;
        if low == 0 {
            return None;
        }
        Some((low - 1, (id >> 32) as u32))
    }

    pub fn insert(&mut self, node: Node) -> Id {
        if let Some(i) = self.free.pop() {
            let slot = &mut self.slots[i];
            slot.node = Some(Box::new(node));
            return ((slot.generation as u64) << 32) | (i as u64 + 1);
        }
        self.slots.push(Slot {
            generation: 1,
            node: Some(Box::new(node)),
        });
        (1u64 << 32) | (self.slots.len() as u64)
    }

    pub fn get(&self, id: Id) -> Option<&Node> {
        let (i, g) = Self::split(id)?;
        let slot = self.slots.get(i)?;
        if slot.generation != g {
            return None;
        }
        slot.node.as_deref()
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Node> {
        let (i, g) = Self::split(id)?;
        let slot = self.slots.get_mut(i)?;
        if slot.generation != g {
            return None;
        }
        slot.node.as_deref_mut()
    }

    pub fn contains(&self, id: Id) -> bool {
        self.get(id).is_some()
    }

    pub fn remove(&mut self, id: Id) -> Option<Node> {
        let (i, g) = Self::split(id)?;
        let slot = self.slots.get_mut(i)?;
        if slot.generation != g || slot.node.is_none() {
            return None;
        }
        slot.generation = slot.generation.wrapping_add(1).max(1);
        self.free.push(i);
        slot.node.take().map(|b| *b)
    }

    pub fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct State {
    pub nodes: Arena,
    pub windows: Vec<Id>,
    pub handler: Handler,
    pub proxy: Option<winit::event_loop::EventLoopProxy<Wake>>,
    pub theme: Theme,
    pub text: TextSystem,
    pub shadows: ShadowCache,
    pub svgs: SvgCache,
    pub timers: Vec<Timer>,
    pub posted: Vec<u64>,
    /// Events produced by API calls (e.g. focus changes) that must be delivered on the UI thread.
    pub pending_events: Vec<OutEvent>,
    pub quit: bool,
    pub running: bool,
    pub last_error: String,
    pub start: Instant,
    /// Global animation speed factor (1 = normal, 0.5 = slow motion, 0 = no motion).
    pub anim_speed: f32,
    /// The animation clock in seconds: wall time scaled by `anim_speed`.
    pub clock: f64,
    clock_last: Option<Instant>,
    /// Default motion style for value, toggle and entrance tweens.
    pub motion: Motion,
}

pub static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::new()));

/// Lock the global state, recovering from poisoning (a caught panic must not brick the UI).
pub fn lock() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> State {
        State {
            nodes: Arena::default(),
            windows: Vec::new(),
            handler: Handler::default(),
            proxy: None,
            theme: Theme::default(),
            text: TextSystem::new(),
            shadows: ShadowCache::new(),
            svgs: SvgCache::new(),
            timers: Vec::new(),
            posted: Vec::new(),
            pending_events: Vec::new(),
            quit: false,
            running: false,
            last_error: String::new(),
            start: Instant::now(),
            anim_speed: 1.0,
            clock: 0.0,
            clock_last: None,
            motion: Motion::Ease,
        }
    }

    /// Seconds since the state was created.
    pub fn now(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Advance the animation clock (call once per frame; extra calls in the same instant are
    /// harmless).
    /// Whether the animation clock has started (the first frame has been rendered). Theme
    /// changes before that apply without a cross-fade.
    pub fn started(&self) -> bool {
        self.clock_last.is_some()
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = match self.clock_last {
            Some(last) => (now - last).as_secs_f64().min(0.1),
            None => 0.0,
        };
        self.clock_last = Some(now);
        self.clock += dt * self.anim_speed.max(0.0) as f64;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.last_error = msg.into();
    }

    /// Wake the event loop so it can pick up pending work. Safe from any thread.
    pub fn wake(&self) {
        if let Some(p) = &self.proxy {
            let _ = p.send_event(Wake);
        }
    }

    // ---- tree ------------------------------------------------------------

    pub fn new_node(&mut self, kind: Kind) -> Id {
        self.nodes.insert(Node::new(kind))
    }

    pub fn new_window(&mut self) -> Id {
        let id = self.nodes.insert(Node::new(Kind::Window));
        self.windows.push(id);
        self.wake();
        id
    }

    pub fn window_of(&self, mut id: Id) -> Option<Id> {
        for _ in 0..10_000 {
            let n = self.nodes.get(id)?;
            if n.kind == Kind::Window {
                return Some(id);
            }
            id = n.parent?;
        }
        None
    }

    pub fn window_data(&self, win: Id) -> Option<&WindowData> {
        self.nodes.get(win)?.win()
    }

    pub fn window_data_mut(&mut self, win: Id) -> Option<&mut WindowData> {
        self.nodes.get_mut(win)?.win_mut()
    }

    /// Something that affects geometry changed under `id`.
    pub fn mark_layout_dirty(&mut self, id: Id) {
        if let Some(w) = self.window_of(id) {
            if let Some(wd) = self.window_data_mut(w) {
                wd.layout_dirty = true;
                wd.needs_redraw = true;
            }
            self.wake();
        }
    }

    /// Something visual (but not geometric) changed under `id`.
    pub fn mark_redraw(&mut self, id: Id) {
        if let Some(w) = self.window_of(id) {
            if let Some(wd) = self.window_data_mut(w) {
                wd.needs_redraw = true;
            }
            self.wake();
        }
    }

    pub fn is_ancestor(&self, maybe_ancestor: Id, mut node: Id) -> bool {
        for _ in 0..10_000 {
            if node == maybe_ancestor {
                return true;
            }
            match self.nodes.get(node).and_then(|n| n.parent) {
                Some(p) => node = p,
                None => return false,
            }
        }
        false
    }

    /// Remove `child` from its parent without freeing it.
    pub fn detach(&mut self, child: Id) {
        let parent = match self.nodes.get(child).and_then(|n| n.parent) {
            Some(p) => p,
            None => return,
        };
        self.mark_layout_dirty(parent);
        if let Some(p) = self.nodes.get_mut(parent) {
            p.children.retain(|c| *c != child);
            if let Some(wd) = p.win_mut() {
                if wd.content == Some(child) {
                    wd.content = None;
                }
                wd.overlays.retain(|o| *o != child);
                if wd.menu_focus == Some(child) {
                    wd.menu_focus = None;
                }
            }
        }
        if let Some(c) = self.nodes.get_mut(child) {
            c.parent = None;
        }
    }

    pub fn insert(&mut self, parent: Id, child: Id, index: Option<usize>) -> bool {
        if parent == child || !self.nodes.contains(parent) || !self.nodes.contains(child) {
            return false;
        }
        if self.is_ancestor(child, parent) {
            return false;
        }
        if self
            .nodes
            .get(child)
            .map(|n| n.kind == Kind::Window)
            .unwrap_or(true)
        {
            return false;
        }
        self.detach(child);
        let is_window = self
            .nodes
            .get(parent)
            .map(|n| n.kind == Kind::Window)
            .unwrap_or(false);
        if is_window {
            if let Some(old) = self.window_data(parent).and_then(|w| w.content) {
                self.detach(old);
            }
        }
        if let Some(p) = self.nodes.get_mut(parent) {
            match index {
                Some(i) if i < p.children.len() => p.children.insert(i, child),
                _ => p.children.push(child),
            }
            if let Some(wd) = p.win_mut() {
                wd.content = Some(child);
            }
        }
        if let Some(c) = self.nodes.get_mut(child) {
            c.parent = Some(parent);
            c.entered = false;
        }
        self.mark_layout_dirty(parent);
        true
    }

    /// Detach and free a node together with its subtree.
    pub fn free_subtree(&mut self, id: Id) {
        self.detach(id);
        let mut stack = vec![id];
        while let Some(n) = stack.pop() {
            if let Some(node) = self.nodes.remove(n) {
                stack.extend(node.children.iter().copied());
                if node.kind == Kind::Window {
                    self.windows.retain(|w| *w != n);
                }
            }
        }
        self.wake();
    }

    /// Focusable nodes of a window in tree order.
    pub fn focusables(&self, win: Id) -> Vec<Id> {
        let mut out = Vec::new();
        let mut stack: Vec<Id> = self
            .window_data(win)
            .and_then(|w| w.content)
            .into_iter()
            .collect();
        // Depth-first, preserving document order.
        while let Some(n) = stack.pop() {
            if let Some(node) = self.nodes.get(n) {
                if !node.visible {
                    continue;
                }
                if node.kind.is_interactive() && !node.disabled {
                    out.push(n);
                }
                for c in node.children.iter().rev() {
                    stack.push(*c);
                }
            }
        }
        out
    }

    /// Move keyboard focus, producing blur/focus events.
    pub fn set_focus(&mut self, win: Id, target: Option<Id>, visible: bool) -> Vec<OutEvent> {
        let mut events = Vec::new();
        let old = match self.window_data(win) {
            Some(w) => w.focused,
            None => return events,
        };
        if old == target {
            if let Some(wd) = self.window_data_mut(win) {
                wd.focus_visible = visible;
            }
            return events;
        }
        if let Some(o) = old {
            if let Some(n) = self.nodes.get_mut(o) {
                n.anim.focus.set(0.0);
                n.input.anchor = None;
                events.push(OutEvent::new(o, EV_BLUR));
            }
        }
        let now = self.now();
        if let Some(t) = target {
            if let Some(n) = self.nodes.get_mut(t) {
                n.anim.focus.set(1.0);
                n.input.blink_epoch = now;
                events.push(OutEvent::new(t, EV_FOCUS));
            }
        }
        if let Some(wd) = self.window_data_mut(win) {
            wd.focused = target;
            wd.focus_visible = visible;
            wd.needs_redraw = true;
        }
        events
    }

    // ---- timers / posting ---------------------------------------------------

    pub fn timer_start(&mut self, token: u64, ms: u64, repeat: bool) {
        self.timers.retain(|t| t.token != token);
        let d = Duration::from_millis(ms);
        self.timers.push(Timer {
            token,
            deadline: Instant::now() + d,
            interval: if repeat {
                Some(d.max(Duration::from_millis(1)))
            } else {
                None
            },
        });
        self.wake();
    }

    pub fn timer_cancel(&mut self, token: u64) {
        self.timers.retain(|t| t.token != token);
    }

    pub fn post(&mut self, token: u64) {
        self.posted.push(token);
        self.wake();
    }

    pub fn next_timer_deadline(&self) -> Option<Instant> {
        self.timers.iter().map(|t| t.deadline).min()
    }

    /// Pop timers that are due, re-arming repeating ones.
    pub fn due_timers(&mut self, now: Instant) -> Vec<u64> {
        let mut fired = Vec::new();
        let mut i = 0;
        while i < self.timers.len() {
            if self.timers[i].deadline <= now {
                let t = &mut self.timers[i];
                fired.push(t.token);
                match t.interval {
                    Some(iv) => {
                        t.deadline += iv;
                        if t.deadline <= now {
                            t.deadline = now + iv;
                        }
                        i += 1;
                    }
                    None => {
                        self.timers.remove(i);
                    }
                }
            } else {
                i += 1;
            }
        }
        fired
    }
}
