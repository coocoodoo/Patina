//! Pointer and keyboard handling. Functions mutate state and return events for the host
//! plus whether a redraw is needed. Nothing here calls back into the host.
//!
//! Besides the content tree, a window can show overlays: popup menus (topmost first for hit
//! testing, closed by clicks outside them or Escape) and a tooltip that appears after the
//! pointer rests on a widget.

use crate::anim::Tween;
use crate::geom::Rect;
use crate::layout::metrics::*;
use crate::layout::{font_face, font_px};
use crate::node::*;
use crate::props::*;
use crate::render::{finish_close, input_scroll_x, real_to_display, scrollbar_thumb_rect};
use crate::state::{Arena, Id, OutEvent, State};
use crate::text::TextSystem;

/// Seconds the pointer must rest on a widget before its tooltip appears.
pub const TOOLTIP_DELAY: f64 = 0.55;

#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Enter,
    Tab,
    Escape,
    Char(String),
}

#[derive(Default, Debug)]
pub struct Out {
    pub events: Vec<OutEvent>,
    pub redraw: bool,
}

pub fn hit_test(nodes: &Arena, id: Id, x: f32, y: f32) -> Option<Id> {
    let n = nodes.get(id)?;
    if !n.visible || !n.rect.contains(x, y) {
        return None;
    }
    for c in n.children.iter().rev() {
        if let Some(h) = hit_test(nodes, *c, x, y) {
            return Some(h);
        }
    }
    Some(id)
}

/// The topmost node under the pointer: open menus first (last opened on top), then the
/// content.
pub fn hit_window(state: &State, win: Id, x: f32, y: f32) -> Option<Id> {
    let w = state.window_data(win)?;
    for o in w.overlays.iter().rev() {
        if state.nodes.get(*o).map(|n| n.closing).unwrap_or(true) {
            continue;
        }
        if let Some(h) = hit_test(&state.nodes, *o, x, y) {
            return Some(h);
        }
    }
    w.content.and_then(|c| hit_test(&state.nodes, c, x, y))
}

fn interactive_ancestor(nodes: &Arena, mut id: Id) -> Option<Id> {
    loop {
        let n = nodes.get(id)?;
        if n.kind.is_interactive() || n.clickable {
            return if n.disabled { None } else { Some(id) };
        }
        id = n.parent?;
    }
}

fn scroll_ancestor(nodes: &Arena, mut id: Id) -> Option<Id> {
    loop {
        let n = nodes.get(id)?;
        if n.kind == Kind::Scroll && n.scroll.content_h > n.rect.h + 0.5 {
            return Some(id);
        }
        id = n.parent?;
    }
}

fn menu_ancestor(nodes: &Arena, mut id: Id) -> Option<Id> {
    loop {
        let n = nodes.get(id)?;
        if n.kind == Kind::Menu {
            return Some(id);
        }
        id = n.parent?;
    }
}

fn tooltip_owner(nodes: &Arena, mut id: Id) -> Option<Id> {
    loop {
        let n = nodes.get(id)?;
        if !n.tooltip.is_empty() {
            return Some(id);
        }
        id = n.parent?;
    }
}

fn context_menu_ancestor(nodes: &Arena, mut id: Id) -> Option<Id> {
    loop {
        let n = nodes.get(id)?;
        if let Some(m) = n.context_menu {
            return Some(m);
        }
        id = n.parent?;
    }
}

fn translate_subtree(nodes: &mut Arena, id: Id, dx: f32, dy: f32) {
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        if let Some(node) = nodes.get_mut(n) {
            node.rect.x += dx;
            node.rect.y += dy;
            stack.extend(node.children.iter().copied());
        }
    }
}

/// Set a scroll container's offset (physical px), moving its children accordingly.
pub fn set_scroll_offset(nodes: &mut Arena, id: Id, offset: f32) -> bool {
    let (children, old, new) = match nodes.get_mut(id) {
        Some(n) if n.kind == Kind::Scroll => {
            let max = (n.scroll.content_h - n.rect.h).max(0.0);
            let new = offset.clamp(0.0, max);
            let old = n.scroll.offset;
            n.scroll.offset = new;
            (n.children.clone(), old, new)
        }
        _ => return false,
    };
    if (old - new).abs() < 0.001 {
        return false;
    }
    for c in children {
        translate_subtree(nodes, c, 0.0, old - new);
    }
    true
}

/// Update scrollbar visibility for all scroll containers under `root` given the cursor.
fn update_scroll_hover(nodes: &mut Arena, root: Option<Id>, pos: Option<(f32, f32)>) -> bool {
    let mut changed = false;
    let mut stack: Vec<Id> = root.into_iter().collect();
    while let Some(id) = stack.pop() {
        if let Some(n) = nodes.get_mut(id) {
            if n.kind == Kind::Scroll {
                let inside = pos.map(|(x, y)| n.rect.contains(x, y)).unwrap_or(false);
                let scrollable = n.scroll.content_h > n.rect.h + 0.5;
                let target = if (inside && scrollable) || n.scroll.bar_grab.is_some() {
                    1.0
                } else {
                    0.0
                };
                if n.scroll.bar.target != target {
                    n.scroll.bar.set(target);
                    changed = true;
                }
            }
            stack.extend(n.children.iter().copied());
        }
    }
    changed
}

fn cursor_for(nodes: &Arena, target: Option<Id>) -> CursorKind {
    let Some(n) = target.and_then(|t| nodes.get(t)) else {
        return CursorKind::Default;
    };
    match n.kind {
        Kind::Button | Kind::Checkbox | Kind::Switch | Kind::Slider | Kind::MenuItem => {
            CursorKind::Pointer
        }
        Kind::TextInput => CursorKind::Text,
        _ if n.clickable => CursorKind::Pointer,
        _ => CursorKind::Default,
    }
}

// ---- tooltips ----------------------------------------------------------------------------

/// Start fading out a shown tooltip (or drop a pending one). Returns true when a redraw is
/// needed.
pub fn hide_tooltip(state: &mut State, win: Id) -> bool {
    let Some(w) = state.window_data_mut(win) else {
        return false;
    };
    let Some(tip) = &mut w.tooltip else {
        return false;
    };
    if tip.shown && !tip.closing {
        tip.closing = true;
        tip.anim.set(0.0);
        return true;
    }
    if !tip.shown {
        w.tooltip = None;
    }
    false
}

/// Track which widget's tooltip is pending or shown as the pointer moves over `hit`.
fn update_tooltip(state: &mut State, win: Id, hit: Option<Id>) -> bool {
    let owner = hit.and_then(|h| tooltip_owner(&state.nodes, h));
    let text = owner
        .and_then(|o| state.nodes.get(o))
        .map(|n| n.tooltip.clone())
        .unwrap_or_default();
    let now = state.now();
    let Some(w) = state.window_data_mut(win) else {
        return false;
    };
    // Tooltips wait for the button to be released.
    let owner = if w.pressed.is_some() || w.drag != Drag::None {
        None
    } else {
        owner
    };
    if let Some(tip) = &w.tooltip {
        if Some(tip.node) == owner && !tip.closing {
            return false;
        }
    }
    let mut redraw = false;
    let mut was_shown = false;
    if let Some(tip) = &mut w.tooltip {
        if tip.shown {
            was_shown = true;
            if !tip.closing {
                tip.closing = true;
                tip.anim.set(0.0);
                redraw = true;
            }
        }
    }
    if w.tooltip.as_ref().is_some_and(|t| !t.shown) {
        w.tooltip = None;
    }
    if let Some(o) = owner {
        // Moving straight from one tooltip to the next skips the delay.
        let since = if was_shown { now - TOOLTIP_DELAY } else { now };
        w.tooltip = Some(TooltipState {
            node: o,
            text,
            since,
            shown: false,
            anim: Tween::default(),
            closing: false,
        });
    }
    redraw
}

/// Show tooltips whose delay has elapsed. Returns the windows that need a redraw.
pub fn tooltips_due(state: &mut State, now: f64) -> Vec<Id> {
    let mut out = Vec::new();
    for win in state.windows.clone() {
        if let Some(w) = state.window_data_mut(win) {
            if let Some(tip) = &mut w.tooltip {
                if !tip.shown && !tip.closing && now - tip.since >= TOOLTIP_DELAY {
                    tip.shown = true;
                    tip.anim.snap(0.0);
                    tip.anim.set(1.0);
                    w.needs_redraw = true;
                    out.push(win);
                }
            }
        }
    }
    out
}

/// The earliest moment (in `State::now` seconds) a pending tooltip should appear.
pub fn next_tooltip_deadline(state: &State) -> Option<f64> {
    state
        .windows
        .iter()
        .filter_map(|w| state.window_data(*w))
        .filter_map(|w| w.tooltip.as_ref())
        .filter(|t| !t.shown && !t.closing)
        .map(|t| t.since + TOOLTIP_DELAY)
        .reduce(f64::min)
}

// ---- menus -------------------------------------------------------------------------------

/// Menus of a window that are open and not fading out, bottom to top.
pub fn open_menus(state: &State, win: Id) -> Vec<Id> {
    state
        .window_data(win)
        .map(|w| {
            w.overlays
                .iter()
                .copied()
                .filter(|m| {
                    state
                        .nodes
                        .get(*m)
                        .is_some_and(|n| n.kind == Kind::Menu && !n.closing)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn menu_is_open(state: &State, menu: Id) -> bool {
    state
        .nodes
        .get(menu)
        .is_some_and(|n| n.kind == Kind::Menu && n.parent.is_some() && !n.closing)
}

/// Open a menu in the window of `anchor`: below the anchor widget, or at the logical point
/// `(x, y)` when the anchor is the window itself.
pub fn open_menu(state: &mut State, menu: Id, anchor: Id, x: f32, y: f32) -> Result<(), String> {
    if state.nodes.get(menu).map(|n| n.kind) != Some(Kind::Menu) {
        return Err("not a menu".into());
    }
    let win = state
        .window_of(anchor)
        .ok_or_else(|| "the anchor is not in a window".to_string())?;
    if state.nodes.get(menu).and_then(|n| n.parent).is_some() {
        finish_close(&mut state.nodes, menu);
    }
    if let Some(n) = state.nodes.get_mut(menu) {
        n.parent = Some(win);
        n.anchor = if anchor == win { None } else { Some(anchor) };
        n.anchor_pos = (x, y);
        n.closing = false;
        n.entered = false;
        n.rect = Rect::default();
        if n.visual.enter == 0 {
            n.visual.enter = 6;
        }
    }
    hide_tooltip(state, win);
    if let Some(w) = state.window_data_mut(win) {
        w.overlays.push(menu);
        w.menu_focus = None;
        w.layout_dirty = true;
        w.needs_redraw = true;
    }
    state.wake();
    Ok(())
}

/// Start closing a menu (it fades out over a few frames). Emits `EV_MENU_CLOSED`.
pub fn close_menu(state: &mut State, menu: Id) -> Vec<OutEvent> {
    let mut events = Vec::new();
    let win = match state.nodes.get_mut(menu) {
        Some(n) if n.kind == Kind::Menu && n.parent.is_some() && !n.closing => {
            n.closing = true;
            n.anim.enter.set(0.0);
            n.parent
        }
        _ => return events,
    };
    events.push(OutEvent::new(menu, EV_MENU_CLOSED));
    if let Some(w) = win {
        let hovered = state.window_data(w).and_then(|d| d.hovered);
        if let Some(h) = hovered {
            if menu_ancestor(&state.nodes, h) == Some(menu) {
                if let Some(hn) = state.nodes.get_mut(h) {
                    hn.anim.hover.set(0.0);
                    hn.anim.press.set(0.0);
                }
                events.push(OutEvent::new(h, EV_HOVER_LEAVE));
                if let Some(d) = state.window_data_mut(w) {
                    d.hovered = None;
                }
            }
        }
        let pressed_inside = state
            .window_data(w)
            .and_then(|d| d.pressed)
            .is_some_and(|p| menu_ancestor(&state.nodes, p) == Some(menu));
        if let Some(d) = state.window_data_mut(w) {
            d.menu_focus = None;
            d.needs_redraw = true;
            if pressed_inside {
                d.pressed = None;
            }
        }
    }
    state.wake();
    events
}

/// Close every open menu of a window.
pub fn close_menus(state: &mut State, win: Id) -> Vec<OutEvent> {
    let mut events = Vec::new();
    for m in open_menus(state, win) {
        events.extend(close_menu(state, m));
    }
    events
}

fn menu_items(nodes: &Arena, menu: Id) -> Vec<Id> {
    nodes
        .get(menu)
        .map(|n| {
            n.children
                .iter()
                .copied()
                .filter(|c| {
                    nodes
                        .get(*c)
                        .is_some_and(|i| i.kind == Kind::MenuItem && i.visible && !i.disabled)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Keyboard navigation while a menu is open.
fn menu_key(state: &mut State, win: Id, menu: Id, key: Key, out: &mut Out) {
    let items = menu_items(&state.nodes, menu);
    let focus = state.window_data(win).and_then(|w| w.menu_focus);
    let idx = focus.and_then(|f| items.iter().position(|i| *i == f));
    let activate = matches!(&key, Key::Enter) || matches!(&key, Key::Char(c) if c == " ");
    if activate {
        if let Some(item) = focus.filter(|f| items.contains(f)) {
            if let Some(n) = state.nodes.get_mut(item) {
                n.anim.press.snap(1.0);
                n.anim.press.set(0.0);
            }
            out.events.push(OutEvent::new(item, EV_CLICK));
            out.events.extend(close_menu(state, menu));
        }
        out.redraw = true;
        return;
    }
    match key {
        Key::Escape | Key::Tab => {
            out.events.extend(close_menu(state, menu));
        }
        Key::Down | Key::Up | Key::Home | Key::End => {
            if items.is_empty() {
                return;
            }
            let n = items.len();
            let next = match key {
                Key::Down => idx.map(|i| (i + 1) % n).unwrap_or(0),
                Key::Up => idx.map(|i| (i + n - 1) % n).unwrap_or(n - 1),
                Key::Home => 0,
                _ => n - 1,
            };
            if let Some(w) = state.window_data_mut(win) {
                w.menu_focus = Some(items[next]);
            }
        }
        _ => {}
    }
    out.redraw = true;
}

// ---- pointer -----------------------------------------------------------------------------

pub fn cursor_moved(state: &mut State, win: Id, x: f32, y: f32) -> Out {
    let mut out = Out::default();
    let (content, drag, pressed, old_hover) = match state.window_data_mut(win) {
        Some(w) => {
            w.cursor = Some((x, y));
            (w.content, w.drag, w.pressed, w.hovered)
        }
        None => return out,
    };
    match drag {
        Drag::Slider(id) => {
            if let Some(ev) = slider_set_from_x(state, win, id, x) {
                out.events.push(ev);
            }
            out.redraw = true;
            return out;
        }
        Drag::ScrollBar(id) => {
            scrollbar_drag(state, win, id, y);
            out.redraw = true;
            return out;
        }
        Drag::Select(id) => {
            let byte = caret_from_x(state, win, id, x);
            if let Some(n) = state.nodes.get_mut(id) {
                if n.input.anchor.is_none() {
                    n.input.anchor = Some(n.input.cursor);
                }
                n.input.cursor = byte;
            }
            update_input_scroll(state, win, id);
            out.redraw = true;
            return out;
        }
        Drag::None => {}
    }

    let hit = hit_window(state, win, x, y);
    let target = hit.and_then(|h| interactive_ancestor(&state.nodes, h));
    if target != old_hover {
        if let Some(o) = old_hover {
            if let Some(n) = state.nodes.get_mut(o) {
                n.anim.hover.set(0.0);
            }
            out.events.push(OutEvent::new(o, EV_HOVER_LEAVE));
        }
        if let Some(t) = target {
            if let Some(n) = state.nodes.get_mut(t) {
                n.anim.hover.set(1.0);
            }
            out.events.push(OutEvent::new(t, EV_HOVER_ENTER));
        }
        if let Some(w) = state.window_data_mut(win) {
            w.hovered = target;
        }
        out.redraw = true;
    }
    // The pointer takes over from keyboard highlighting inside menus.
    if let Some(t) = target {
        let is_item = state.nodes.get(t).is_some_and(|n| n.kind == Kind::MenuItem);
        if is_item {
            if let Some(w) = state.window_data_mut(win) {
                if w.menu_focus.is_some() && w.menu_focus != Some(t) {
                    w.menu_focus = None;
                    out.redraw = true;
                }
            }
        }
    }
    // Press feedback follows the pointer: pressing then leaving lifts the button.
    if let Some(p) = pressed {
        if let Some(n) = state.nodes.get_mut(p) {
            let want = if target == Some(p) { 1.0 } else { 0.0 };
            if n.anim.press.target != want {
                n.anim.press.set(want);
                out.redraw = true;
            }
        }
    }
    if update_scroll_hover(&mut state.nodes, content, Some((x, y))) {
        out.redraw = true;
    }
    if update_tooltip(state, win, hit) {
        out.redraw = true;
    }
    let icon = cursor_for(&state.nodes, target);
    if let Some(w) = state.window_data_mut(win) {
        w.cursor_icon = icon;
    }
    out
}

pub fn cursor_left(state: &mut State, win: Id) -> Out {
    let mut out = Out::default();
    let (content, old_hover, drag) = match state.window_data_mut(win) {
        Some(w) => {
            w.cursor = None;
            (w.content, w.hovered, w.drag)
        }
        None => return out,
    };
    if hide_tooltip(state, win) {
        out.redraw = true;
    }
    if drag != Drag::None {
        return out;
    }
    if let Some(o) = old_hover {
        if let Some(n) = state.nodes.get_mut(o) {
            n.anim.hover.set(0.0);
            n.anim.press.set(0.0);
        }
        out.events.push(OutEvent::new(o, EV_HOVER_LEAVE));
        out.redraw = true;
    }
    if let Some(w) = state.window_data_mut(win) {
        w.hovered = None;
        w.cursor_icon = CursorKind::Default;
    }
    if update_scroll_hover(&mut state.nodes, content, None) {
        out.redraw = true;
    }
    out
}

/// `button`: 0 left, 1 right, 2 middle.
pub fn mouse_button(state: &mut State, win: Id, pressed: bool, button: u8) -> Out {
    let mut out = Out::default();
    let (content, cursor, scale) = match state.window_data(win) {
        Some(w) => (w.content, w.cursor, w.scale),
        None => return out,
    };
    let (x, y) = match cursor {
        Some(c) => c,
        None => return out,
    };
    if pressed && hide_tooltip(state, win) {
        out.redraw = true;
    }
    let hit = hit_window(state, win, x, y);

    if button == 1 {
        // Right click: open the nearest context menu.
        if pressed {
            if let Some(menu) = hit.and_then(|h| context_menu_ancestor(&state.nodes, h)) {
                out.events.extend(close_menus(state, win));
                if open_menu(state, menu, win, x / scale, y / scale).is_ok() {
                    out.redraw = true;
                }
            }
        }
        return out;
    }
    if button != 0 {
        return out;
    }

    if pressed {
        // A click outside every open menu closes them and goes no further.
        let open = open_menus(state, win);
        if !open.is_empty() {
            let inside = hit
                .and_then(|h| menu_ancestor(&state.nodes, h))
                .is_some_and(|m| open.contains(&m));
            if !inside {
                out.events.extend(close_menus(state, win));
                out.redraw = true;
                return out;
            }
        }

        // Scrollbar thumb takes priority over everything under it.
        let mut probe = hit;
        while let Some(pid) = probe {
            let n = match state.nodes.get(pid) {
                Some(n) => n,
                None => break,
            };
            if n.kind == Kind::Scroll {
                if let Some(thumb) = scrollbar_thumb_rect(n, scale) {
                    if thumb.contains(x, y) {
                        if let Some(nm) = state.nodes.get_mut(pid) {
                            nm.scroll.bar_grab = Some(y - thumb.y);
                            nm.scroll.bar.set(1.0);
                        }
                        if let Some(w) = state.window_data_mut(win) {
                            w.drag = Drag::ScrollBar(pid);
                        }
                        out.redraw = true;
                        return out;
                    }
                }
            }
            probe = n.parent;
        }

        let target = hit.and_then(|h| interactive_ancestor(&state.nodes, h));
        let kind = target.and_then(|t| state.nodes.get(t)).map(|n| n.kind);
        if kind != Some(Kind::MenuItem) {
            out.events.extend(state.set_focus(win, target, false));
        }
        if let Some(w) = state.window_data_mut(win) {
            w.pressed = target;
            w.drag = Drag::None;
        }
        if let Some(t) = target {
            let clock = state.clock;
            if let Some(n) = state.nodes.get_mut(t) {
                n.anim.press.set(1.0);
                if n.visual.ripple {
                    n.ripple = Some(Ripple { x, y, start: clock });
                }
            }
            match kind {
                Some(Kind::Slider) => {
                    if let Some(w) = state.window_data_mut(win) {
                        w.drag = Drag::Slider(t);
                    }
                    if let Some(ev) = slider_set_from_x(state, win, t, x) {
                        out.events.push(ev);
                    }
                }
                Some(Kind::TextInput) => {
                    let byte = caret_from_x(state, win, t, x);
                    let now = state.now();
                    if let Some(n) = state.nodes.get_mut(t) {
                        n.input.cursor = byte;
                        n.input.anchor = Some(byte);
                        n.input.blink_epoch = now;
                    }
                    if let Some(w) = state.window_data_mut(win) {
                        w.drag = Drag::Select(t);
                    }
                }
                _ => {}
            }
        }
        out.redraw = true;
    } else {
        let (was_pressed, drag) = match state.window_data_mut(win) {
            Some(w) => {
                let p = w.pressed.take();
                let d = w.drag;
                w.drag = Drag::None;
                (p, d)
            }
            None => return out,
        };
        match drag {
            Drag::ScrollBar(id) => {
                if let Some(n) = state.nodes.get_mut(id) {
                    n.scroll.bar_grab = None;
                }
                update_scroll_hover(&mut state.nodes, content, Some((x, y)));
            }
            Drag::Select(id) => {
                if let Some(n) = state.nodes.get_mut(id) {
                    if n.input.anchor == Some(n.input.cursor) {
                        n.input.anchor = None;
                    }
                }
            }
            _ => {}
        }
        if let Some(p) = was_pressed {
            let kind = state.nodes.get(p).map(|n| n.kind);
            if let Some(n) = state.nodes.get_mut(p) {
                n.anim.press.set(0.0);
            }
            let still_over = hit.and_then(|h| interactive_ancestor(&state.nodes, h)) == Some(p);
            if still_over {
                let clock = state.clock;
                if let Some(n) = state.nodes.get_mut(p) {
                    if n.visual.press_effect != 0 {
                        n.press_wobble = Some(clock);
                    }
                }
                match kind {
                    Some(Kind::Button | Kind::Box | Kind::Scroll) => {
                        out.events.push(OutEvent::new(p, EV_CLICK))
                    }
                    Some(Kind::MenuItem) => {
                        out.events.push(OutEvent::new(p, EV_CLICK));
                        if let Some(m) = menu_ancestor(&state.nodes, p) {
                            out.events.extend(close_menu(state, m));
                        }
                    }
                    Some(Kind::Checkbox | Kind::Switch) => {
                        if let Some(ev) = toggle(state, p) {
                            out.events.push(ev);
                        }
                    }
                    _ => {}
                }
            }
        }
        out.redraw = true;
    }
    out
}

fn toggle(state: &mut State, id: Id) -> Option<OutEvent> {
    let n = state.nodes.get_mut(id)?;
    n.control.checked = !n.control.checked;
    let checked = n.control.checked;
    n.anim.check.set(if checked { 1.0 } else { 0.0 });
    Some(OutEvent::new(id, EV_TOGGLED).a(checked as i64))
}

pub fn wheel(state: &mut State, win: Id, _dx: f32, dy: f32) -> Out {
    let mut out = Out::default();
    let cursor = match state.window_data(win) {
        Some(w) => w.cursor,
        None => return out,
    };
    let (x, y) = match cursor {
        Some(c) => c,
        None => return out,
    };
    if hide_tooltip(state, win) {
        out.redraw = true;
    }
    let hit = match hit_window(state, win, x, y) {
        Some(h) => h,
        None => return out,
    };
    let sid = match scroll_ancestor(&state.nodes, hit) {
        Some(s) => s,
        None => return out,
    };
    let current = state.nodes.get(sid).map(|n| n.scroll.offset).unwrap_or(0.0);
    if set_scroll_offset(&mut state.nodes, sid, current - dy) {
        if let Some(n) = state.nodes.get_mut(sid) {
            n.scroll.bar.set(1.0);
        }
        out.redraw = true;
    }
    out
}

fn slider_set_from_x(state: &mut State, win: Id, id: Id, x: f32) -> Option<OutEvent> {
    let scale = state.window_data(win).map(|w| w.scale).unwrap_or(1.0);
    let n = state.nodes.get_mut(id)?;
    let th = THUMB_R * scale;
    let x0 = n.rect.x + th;
    let w = (n.rect.w - 2.0 * th).max(1.0);
    let frac = ((x - x0) / w).clamp(0.0, 1.0) as f64;
    let (min, max, step) = (n.control.min, n.control.max, n.control.step);
    let mut v = min + frac * (max - min);
    if step > 0.0 {
        v = min + ((v - min) / step).round() * step;
    }
    v = v.clamp(min.min(max), max.max(min));
    if (v - n.control.value).abs() < f64::EPSILON {
        return None;
    }
    n.control.value = v;
    // Dragging tracks the pointer directly; only programmatic changes ease in.
    let range = max - min;
    let frac = if range.abs() > 0.0 {
        ((v - min) / range) as f32
    } else {
        0.0
    };
    n.anim.value.snap(frac.clamp(0.0, 1.0));
    Some(OutEvent::new(id, EV_VALUE_CHANGED).f64(v))
}

fn scrollbar_drag(state: &mut State, win: Id, id: Id, y: f32) {
    let scale = state.window_data(win).map(|w| w.scale).unwrap_or(1.0);
    let (grab, rect, content_h) = match state.nodes.get(id) {
        Some(n) => (n.scroll.bar_grab, n.rect, n.scroll.content_h),
        None => return,
    };
    let grab = match grab {
        Some(g) => g,
        None => return,
    };
    let thumb = match state
        .nodes
        .get(id)
        .and_then(|n| scrollbar_thumb_rect(n, scale))
    {
        Some(t) => t,
        None => return,
    };
    let pad = SCROLLBAR_PAD * scale;
    let track_h = (rect.h - 2.0 * pad).max(1.0);
    let span = (track_h - thumb.h).max(1.0);
    let frac = ((y - grab - (rect.y + pad)) / span).clamp(0.0, 1.0);
    let new_off = frac * (content_h - rect.h).max(0.0);
    set_scroll_offset(&mut state.nodes, id, new_off);
}

fn display_to_real(n: &Node, byte: usize) -> usize {
    if n.kind == Kind::TextInput && n.control.secure {
        let chars = byte / '\u{2022}'.len_utf8();
        n.text
            .text
            .char_indices()
            .nth(chars)
            .map(|(b, _)| b)
            .unwrap_or(n.text.text.len())
    } else {
        byte.min(n.text.text.len())
    }
}

fn caret_from_x(state: &mut State, win: Id, id: Id, x: f32) -> usize {
    let scale = state.window_data(win).map(|w| w.scale).unwrap_or(1.0);
    let State { nodes, text, .. } = state;
    let n = match nodes.get(id) {
        Some(n) => n,
        None => return 0,
    };
    if !text.ready() {
        return n.text.text.len();
    }
    let px = font_px(n, scale);
    let face = font_face(n);
    let display = n.display_text();
    let layout = text.layout(&display, face, px, f32::INFINITY, false);
    let inner_x = n.rect.x + INPUT_PAD_X * scale;
    let inner_w = (n.rect.w - 2.0 * INPUT_PAD_X * scale).max(1.0);
    let caret_x = layout.caret(real_to_display(n, n.input.cursor)).1;
    let sx = input_scroll_x(&layout, caret_x, inner_w, n.input.scroll_x);
    let local = x - inner_x + sx;
    let dbyte = layout.hit(0, local);
    display_to_real(n, dbyte)
}

fn update_input_scroll(state: &mut State, win: Id, id: Id) {
    let scale = state.window_data(win).map(|w| w.scale).unwrap_or(1.0);
    let State { nodes, text, .. } = state;
    if !text.ready() {
        return;
    }
    let n = match nodes.get_mut(id) {
        Some(n) => n,
        None => return,
    };
    let px = font_px(n, scale);
    let face = font_face(n);
    let display = n.display_text();
    let layout = text.layout(&display, face, px, f32::INFINITY, false);
    let inner_w = (n.rect.w - 2.0 * INPUT_PAD_X * scale).max(1.0);
    let caret_x = layout.caret(real_to_display(n, n.input.cursor)).1;
    n.input.scroll_x = input_scroll_x(&layout, caret_x, inner_w, n.input.scroll_x);
}

fn prev_boundary(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j > 0 {
        j -= 1;
        if s.is_char_boundary(j) {
            return j;
        }
    }
    0
}

fn next_boundary(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j < s.len() {
        j += 1;
        if s.is_char_boundary(j) {
            return j;
        }
    }
    s.len()
}

fn word_left(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j > 0 && s[..j].ends_with(char::is_whitespace) {
        j = prev_boundary(s, j);
    }
    while j > 0 && !s[..j].ends_with(char::is_whitespace) {
        j = prev_boundary(s, j);
    }
    j
}

fn word_right(s: &str, i: usize) -> usize {
    let mut j = i.min(s.len());
    while j < s.len() && !s[j..].starts_with(char::is_whitespace) {
        j = next_boundary(s, j);
    }
    while j < s.len() && s[j..].starts_with(char::is_whitespace) {
        j = next_boundary(s, j);
    }
    j
}

pub fn key_down(state: &mut State, win: Id, key: Key, mods: Modifiers) -> Out {
    let mut out = Out::default();
    if hide_tooltip(state, win) {
        out.redraw = true;
    }
    if let Some(menu) = open_menus(state, win).last().copied() {
        menu_key(state, win, menu, key, &mut out);
        return out;
    }
    let focused = match state.window_data(win) {
        Some(w) => w.focused,
        None => return out,
    };
    if key == Key::Tab {
        let list = state.focusables(win);
        if list.is_empty() {
            return out;
        }
        let idx = focused.and_then(|f| list.iter().position(|x| *x == f));
        let next = match idx {
            Some(i) => {
                if mods.shift {
                    list[(i + list.len() - 1) % list.len()]
                } else {
                    list[(i + 1) % list.len()]
                }
            }
            None => {
                if mods.shift {
                    *list.last().unwrap()
                } else {
                    list[0]
                }
            }
        };
        out.events.extend(state.set_focus(win, Some(next), true));
        out.redraw = true;
        return out;
    }
    let fid = match focused {
        Some(f) => f,
        None => return out,
    };
    let kind = match state.nodes.get(fid).map(|n| n.kind) {
        Some(k) => k,
        None => return out,
    };
    match kind {
        Kind::Button => {
            if key == Key::Enter || key == Key::Char(" ".into()) {
                let clock = state.clock;
                if let Some(n) = state.nodes.get_mut(fid) {
                    n.anim.press.snap(1.0);
                    n.anim.press.set(0.0);
                    if n.visual.press_effect != 0 {
                        n.press_wobble = Some(clock);
                    }
                }
                out.events.push(OutEvent::new(fid, EV_CLICK));
                out.redraw = true;
            }
        }
        Kind::Checkbox | Kind::Switch => {
            if key == Key::Enter || key == Key::Char(" ".into()) {
                if let Some(ev) = toggle(state, fid) {
                    out.events.push(ev);
                }
                out.redraw = true;
            }
        }
        Kind::Slider => {
            if let Some(n) = state.nodes.get_mut(fid) {
                let (min, max) = (n.control.min, n.control.max);
                let step = if n.control.step > 0.0 {
                    n.control.step
                } else {
                    (max - min) / 100.0
                };
                let old = n.control.value;
                let mut v = old;
                match key {
                    Key::Left | Key::Down => v -= step,
                    Key::Right | Key::Up => v += step,
                    Key::Home => v = min,
                    Key::End => v = max,
                    _ => {}
                }
                v = v.clamp(min.min(max), max.max(min));
                if (v - old).abs() > f64::EPSILON {
                    n.control.value = v;
                    let range = max - min;
                    if range.abs() > 0.0 {
                        n.anim
                            .value
                            .set((((v - min) / range) as f32).clamp(0.0, 1.0));
                    }
                    out.events.push(OutEvent::new(fid, EV_VALUE_CHANGED).f64(v));
                    out.redraw = true;
                }
            }
        }
        Kind::TextInput => {
            if key == Key::Escape {
                if let Some(n) = state.nodes.get_mut(fid) {
                    n.input.anchor = None;
                }
                out.redraw = true;
            } else {
                out.events.extend(text_key(state, fid, key, mods));
                update_input_scroll(state, win, fid);
                out.redraw = true;
            }
        }
        _ => {}
    }
    out
}

fn text_key(state: &mut State, id: Id, key: Key, mods: Modifiers) -> Vec<OutEvent> {
    let now = state.now();
    let mut events = Vec::new();
    let n = match state.nodes.get_mut(id) {
        Some(n) => n,
        None => return events,
    };
    let mut changed = false;
    let len = n.text.text.len();
    let mut cursor = n.text.text.len().min(n.input.cursor);
    while cursor > 0 && !n.text.text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    let mut anchor = n
        .input
        .anchor
        .filter(|a| *a <= len && n.text.text.is_char_boundary(*a));
    let selection = anchor
        .filter(|a| *a != cursor)
        .map(|a| (a.min(cursor), a.max(cursor)));
    let shortcut = mods.ctrl || mods.meta;

    let delete_selection =
        |t: &mut String, cursor: &mut usize, anchor: &mut Option<usize>| -> bool {
            if let Some((a, b)) = selection {
                t.replace_range(a..b, "");
                *cursor = a;
                *anchor = None;
                true
            } else {
                false
            }
        };

    match key {
        Key::Char(s) => {
            if shortcut {
                if s.eq_ignore_ascii_case("a") {
                    anchor = Some(0);
                    cursor = len;
                }
            } else {
                let s: String = s.chars().filter(|c| !c.is_control()).collect();
                if !s.is_empty() {
                    delete_selection(&mut n.text.text, &mut cursor, &mut anchor);
                    let max = n.control.max_length;
                    if max <= 0 || (n.text.text.chars().count() + s.chars().count()) as i64 <= max {
                        n.text.text.insert_str(cursor, &s);
                        cursor += s.len();
                        changed = true;
                    }
                }
            }
        }
        Key::Backspace => {
            if delete_selection(&mut n.text.text, &mut cursor, &mut anchor) {
                changed = true;
            } else if cursor > 0 {
                let to = if shortcut {
                    word_left(&n.text.text, cursor)
                } else {
                    prev_boundary(&n.text.text, cursor)
                };
                n.text.text.replace_range(to..cursor, "");
                cursor = to;
                changed = true;
            }
        }
        Key::Delete => {
            if delete_selection(&mut n.text.text, &mut cursor, &mut anchor) {
                changed = true;
            } else if cursor < n.text.text.len() {
                let to = if shortcut {
                    word_right(&n.text.text, cursor)
                } else {
                    next_boundary(&n.text.text, cursor)
                };
                n.text.text.replace_range(cursor..to, "");
                changed = true;
            }
        }
        Key::Left | Key::Right | Key::Home | Key::End | Key::Up | Key::Down => {
            let collapse_left = matches!(key, Key::Left | Key::Up);
            let collapse = if !mods.shift && matches!(key, Key::Left | Key::Right) {
                selection
            } else {
                None
            };
            if let Some((a, b)) = collapse {
                cursor = if collapse_left { a } else { b };
                anchor = None;
            } else {
                if mods.shift && anchor.is_none() {
                    anchor = Some(cursor);
                }
                cursor = match key {
                    Key::Left => {
                        if shortcut {
                            word_left(&n.text.text, cursor)
                        } else {
                            prev_boundary(&n.text.text, cursor)
                        }
                    }
                    Key::Right => {
                        if shortcut {
                            word_right(&n.text.text, cursor)
                        } else {
                            next_boundary(&n.text.text, cursor)
                        }
                    }
                    Key::Home | Key::Up => 0,
                    _ => n.text.text.len(),
                };
                if !mods.shift {
                    anchor = None;
                }
            }
        }
        Key::Enter => {
            events.push(OutEvent::new(id, EV_SUBMIT).text(n.text.text.clone()));
        }
        _ => {}
    }
    n.input.cursor = cursor.min(n.text.text.len());
    n.input.anchor = anchor.filter(|a| *a != n.input.cursor);
    n.input.blink_epoch = now;
    if changed {
        events.push(OutEvent::new(id, EV_TEXT_CHANGED).text(n.text.text.clone()));
    }
    events
}

/// Used by the FFI when the host replaces the text of an input.
pub fn reset_input(n: &mut Node) {
    n.input.cursor = n.text.text.len();
    n.input.anchor = None;
    n.input.scroll_x = 0.0;
}

#[allow(dead_code)]
fn _assert_text_system_used(_t: &TextSystem) {}
