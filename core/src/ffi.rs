//! The C ABI. Every entry point is panic-safe, and none of them hold the state lock while
//! calling back into the host, so host callbacks may freely use the API.
//!
//! Floating point values cross the boundary as IEEE-754 bit patterns in `u64` so that hosts
//! without float support in their FFI layer (Go's purego on Windows, for example) can call in.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use crate::app;
use crate::color::{ColorRef, Rgba};
use crate::input;
use crate::node::*;
use crate::props::*;
use crate::render;
use crate::state::{self, EventFn, Handler, State};
use crate::svg::SvgDoc;
use crate::theme::Mode;

fn guard<R: Default>(f: impl FnOnce() -> R) -> R {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(r) => r,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            eprintln!("patina: internal error: {msg}");
            state::lock().set_error(format!("internal error: {msg}"));
            R::default()
        }
    }
}

/// # Safety
/// `ptr` must point to `len` readable bytes (or be null with `len == 0`).
unsafe fn str_arg(ptr: *const u8, len: usize) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(bytes).into_owned()
}

unsafe fn copy_out(src: &[u8], buf: *mut u8, cap: usize) -> usize {
    if !buf.is_null() && cap > 0 {
        let n = src.len().min(cap);
        unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), buf, n) };
    }
    src.len()
}

// ---- lifecycle ---------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn patina_abi_version() -> u32 {
    ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_set_event_handler(func: Option<EventFn>, user: usize) {
    guard(|| {
        state::lock().handler = Handler { func, user };
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_run() -> i32 {
    guard(|| match app::run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("patina: {e}");
            state::lock().set_error(e);
            1
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_quit() {
    guard(|| {
        let mut s = state::lock();
        s.quit = true;
        s.wake();
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_is_running() -> i32 {
    guard(|| state::lock().running as i32)
}

/// Copies the last error message into `buf`; returns its full length in bytes.
#[unsafe(no_mangle)]
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
pub unsafe extern "C" fn patina_last_error(buf: *mut u8, cap: usize) -> usize {
    guard(|| {
        let s = state::lock();
        unsafe { copy_out(s.last_error.as_bytes(), buf, cap) }
    })
}

// ---- tree --------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_new(kind: u32) -> u64 {
    guard(|| match Kind::from_u32(kind) {
        Some(Kind::Window) => state::lock().new_window(),
        Some(k) => state::lock().new_node(k),
        None => 0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_window_new() -> u64 {
    guard(|| state::lock().new_window())
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_free(node: u64) {
    guard(|| {
        let mut s = state::lock();
        s.free_subtree(node);
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_append(parent: u64, child: u64) -> i32 {
    guard(|| state::lock().insert(parent, child, None) as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_insert(parent: u64, child: u64, index: u64) -> i32 {
    guard(|| state::lock().insert(parent, child, Some(index as usize)) as i32)
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_remove(parent: u64, child: u64) {
    guard(|| {
        let mut s = state::lock();
        if s.nodes.get(child).and_then(|n| n.parent) == Some(parent) {
            s.detach(child);
        }
    })
}

/// Detach and free all children of `parent`.
#[unsafe(no_mangle)]
pub extern "C" fn patina_node_clear(parent: u64) {
    guard(|| {
        let mut s = state::lock();
        let kids = s
            .nodes
            .get(parent)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for k in kids {
            s.free_subtree(k);
        }
        if let Some(wd) = s.window_data_mut(parent) {
            wd.content = None;
        }
        s.mark_layout_dirty(parent);
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_child_count(node: u64) -> u64 {
    guard(|| {
        state::lock()
            .nodes
            .get(node)
            .map(|n| n.children.len() as u64)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_kind(node: u64) -> u32 {
    guard(|| {
        state::lock()
            .nodes
            .get(node)
            .map(|n| n.kind.to_u32())
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_node_parent(node: u64) -> u64 {
    guard(|| {
        state::lock()
            .nodes
            .get(node)
            .and_then(|n| n.parent)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_window_set_content(window: u64, node: u64) -> i32 {
    guard(|| {
        let mut s = state::lock();
        if s.nodes
            .get(window)
            .map(|n| n.kind != Kind::Window)
            .unwrap_or(true)
        {
            return 0;
        }
        s.insert(window, node, None) as i32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_window_close(window: u64) {
    guard(|| {
        let mut s = state::lock();
        if let Some(wd) = s.window_data_mut(window) {
            wd.close_requested = true;
        }
        s.wake();
    })
}

/// Render a window offscreen at `width` x `height` logical px and `scale` (f64 bits) to a PNG.
#[unsafe(no_mangle)]
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
pub unsafe extern "C" fn patina_window_snapshot(
    window: u64,
    width: u32,
    height: u32,
    scale_bits: u64,
    path: *const u8,
    path_len: usize,
) -> i32 {
    guard(|| {
        let path = unsafe { str_arg(path, path_len) };
        let scale = f64::from_bits(scale_bits) as f32;
        match render::snapshot(window, width, height, scale, Path::new(&path)) {
            Ok(()) => 1,
            Err(e) => {
                state::lock().set_error(e);
                0
            }
        }
    })
}

// ---- properties --------------------------------------------------------------

fn set_i64(s: &mut State, id: u64, prop: u32, v: i64) {
    let b = v != 0;
    if prop == PROP_FOCUS {
        if let Some(win) = s.window_of(id) {
            let currently = s.window_data(win).and_then(|w| w.focused);
            let events = if b {
                s.set_focus(win, Some(id), false)
            } else if currently == Some(id) {
                s.set_focus(win, None, false)
            } else {
                Vec::new()
            };
            s.pending_events.extend(events);
            s.mark_redraw(id);
        }
        return;
    }
    let mut relayout = false;
    let n = match s.nodes.get_mut(id) {
        Some(n) => n,
        None => return,
    };
    let is_window = n.kind == Kind::Window;
    match prop {
        PROP_VISIBLE => {
            if is_window {
                if let Some(wd) = n.win_mut() {
                    wd.visible = b;
                    wd.props_dirty = true;
                }
            } else {
                n.visible = b;
                if b {
                    n.entered = false;
                }
                relayout = true;
            }
        }
        PROP_DISABLED => n.disabled = b,
        PROP_BACKGROUND => {
            if let Some(wd) = n.win_mut() {
                wd.background = ColorRef::from_i64(v);
            } else {
                n.visual.background = ColorRef::from_i64(v);
            }
        }
        PROP_BORDER_COLOR => n.visual.border_color = ColorRef::from_i64(v),
        PROP_SHADOW => n.visual.shadow = v.clamp(0, 3),
        PROP_TINT => n.visual.tint = ColorRef::from_i64(v),
        PROP_GRADIENT_END => n.visual.gradient_end = ColorRef::from_i64(v),
        PROP_GLOW => n.visual.glow = b,
        PROP_RIPPLE => n.visual.ripple = b,
        PROP_HOVER_LIFT => n.visual.hover_lift = b,
        PROP_CLICKABLE => n.clickable = b,
        PROP_PRESS_EFFECT => n.visual.press_effect = v.clamp(0, 3),
        PROP_CONTEXT_MENU => n.context_menu = if v > 0 { Some(v as u64) } else { None },
        PROP_MATCH_ANCHOR => n.match_anchor = b,
        PROP_ICON_END => n.control.icon_end = b,
        PROP_ANIMATE => n.control.svg_animate = b,
        PROP_ENTER => {
            n.visual.enter = v.clamp(0, 6);
            // Replay on the next frame for widgets that are already on screen.
            n.entered = false;
        }
        PROP_DIRECTION => {
            n.layout.direction = if v == 1 {
                Direction::Row
            } else {
                Direction::Column
            };
            relayout = true;
        }
        PROP_ALIGN_ITEMS => {
            n.layout.align_items = if v < 0 { None } else { Some(v) };
            relayout = true;
        }
        PROP_JUSTIFY => {
            n.layout.justify = if v < 0 { None } else { Some(v) };
            relayout = true;
        }
        PROP_ALIGN_SELF => {
            n.layout.align_self = if v <= 0 { None } else { Some(v) };
            relayout = true;
        }
        PROP_WRAP => {
            n.layout.wrap = b;
            relayout = true;
        }
        PROP_FONT_WEIGHT => {
            n.text.weight = v.clamp(0, 900) as u16;
            relayout = true;
        }
        PROP_TEXT_COLOR => n.text.color = ColorRef::from_i64(v),
        PROP_TEXT_ALIGN => n.text.align = v,
        PROP_TEXT_WRAP => {
            n.text.wrap = b;
            relayout = true;
        }
        PROP_FONT_FAMILY => {
            n.text.family = v;
            relayout = true;
        }
        PROP_VARIANT => n.control.variant = v,
        PROP_CHECKED => {
            n.control.checked = b;
            let t = if b { 1.0 } else { 0.0 };
            if n.rect.is_empty() {
                n.anim.check.snap(t);
            } else {
                n.anim.check.set(t);
            }
        }
        PROP_SECURE => n.control.secure = b,
        PROP_INDETERMINATE => n.control.indeterminate = b,
        PROP_ORIENTATION => {
            n.control.orientation = v;
            relayout = true;
        }
        PROP_FIT => n.control.fit = v,
        PROP_MAX_LENGTH => n.control.max_length = v,
        PROP_RESIZABLE | PROP_DECORATIONS | PROP_ALWAYS_ON_TOP | PROP_MAXIMIZED
        | PROP_INTERCEPT_CLOSE => {
            if let Some(wd) = n.win_mut() {
                match prop {
                    PROP_RESIZABLE => wd.resizable = b,
                    PROP_DECORATIONS => wd.decorations = b,
                    PROP_ALWAYS_ON_TOP => wd.always_on_top = b,
                    PROP_MAXIMIZED => wd.maximized = b,
                    _ => wd.intercept_close = b,
                }
                wd.props_dirty = true;
            }
        }
        _ => return,
    }
    if relayout {
        s.mark_layout_dirty(id);
    } else {
        s.mark_redraw(id);
    }
}

fn set_f64(s: &mut State, id: u64, prop: u32, v: f64) {
    if prop == PROP_SCROLL_Y {
        let scale = s
            .window_of(id)
            .and_then(|w| s.window_data(w))
            .map(|w| w.scale)
            .unwrap_or(1.0);
        let laid_out = s.nodes.get(id).is_some_and(|n| !n.rect.is_empty());
        if laid_out {
            input::set_scroll_offset(&mut s.nodes, id, v as f32 * scale);
        } else if let Some(n) = s.nodes.get_mut(id) {
            // Before the first layout: keep the request, layout clamps it to the content.
            n.scroll.offset = (v as f32 * scale).max(0.0);
        }
        s.mark_redraw(id);
        return;
    }
    let mut relayout = true;
    let n = match s.nodes.get_mut(id) {
        Some(n) => n,
        None => return,
    };
    let vf = v as f32;
    match prop {
        PROP_OPACITY => {
            n.visual.opacity = vf.clamp(0.0, 1.0);
            relayout = false;
        }
        PROP_RADIUS => {
            n.visual.radius = Some(vf.max(0.0));
            relayout = false;
        }
        PROP_BORDER_WIDTH => {
            n.visual.border_width = Some(vf.max(0.0));
            relayout = false;
        }
        PROP_GAP => n.layout.gap = vf.max(0.0),
        PROP_PADDING_LEFT => n.layout.padding[0] = vf,
        PROP_PADDING_TOP => n.layout.padding[1] = vf,
        PROP_PADDING_RIGHT => n.layout.padding[2] = vf,
        PROP_PADDING_BOTTOM => n.layout.padding[3] = vf,
        PROP_MARGIN_LEFT => n.layout.margin[0] = vf,
        PROP_MARGIN_TOP => n.layout.margin[1] = vf,
        PROP_MARGIN_RIGHT => n.layout.margin[2] = vf,
        PROP_MARGIN_BOTTOM => n.layout.margin[3] = vf,
        PROP_WIDTH => n.layout.width = Length::from_f64(v),
        PROP_HEIGHT => n.layout.height = Length::from_f64(v),
        PROP_MIN_WIDTH => n.layout.min_width = Length::from_f64(v),
        PROP_MIN_HEIGHT => n.layout.min_height = Length::from_f64(v),
        PROP_MAX_WIDTH => n.layout.max_width = Length::from_f64(v),
        PROP_MAX_HEIGHT => n.layout.max_height = Length::from_f64(v),
        PROP_GROW => n.layout.grow = if v.is_nan() { None } else { Some(vf.max(0.0)) },
        PROP_SHRINK => n.layout.shrink = if v.is_nan() { None } else { Some(vf.max(0.0)) },
        PROP_FONT_SIZE => n.text.size = if v > 0.0 { Some(vf) } else { None },
        PROP_VALUE => {
            let (lo, hi) = (
                n.control.min.min(n.control.max),
                n.control.max.max(n.control.min),
            );
            n.control.value = if hi > lo { v.clamp(lo, hi) } else { v };
            let frac = if hi > lo {
                ((n.control.value - lo) / (hi - lo)) as f32
            } else {
                0.0
            };
            if n.rect.is_empty() {
                n.anim.value.snap(frac);
            } else {
                n.anim.value.set(frac);
            }
            relayout = false;
        }
        PROP_MIN => {
            n.control.min = v;
            relayout = false;
        }
        PROP_MAX => {
            n.control.max = v;
            relayout = false;
        }
        PROP_STEP => {
            n.control.step = v.max(0.0);
            relayout = false;
        }
        PROP_ICON_SIZE => n.control.icon_size = vf.max(0.0),
        PROP_COLOR_CYCLE => {
            n.visual.color_cycle = vf.max(0.0);
            relayout = false;
        }
        PROP_SPIN => {
            n.control.spin = vf.max(0.0);
            relayout = false;
        }
        PROP_PULSE => {
            n.control.pulse = vf.max(0.0);
            relayout = false;
        }
        PROP_SPRING_STIFFNESS => {
            n.spring_stiffness = vf.max(0.0);
            relayout = false;
        }
        PROP_SPRING_DAMPING => {
            n.spring_damping = vf.max(0.0);
            relayout = false;
        }
        PROP_WINDOW_WIDTH | PROP_WINDOW_HEIGHT | PROP_WINDOW_MIN_WIDTH | PROP_WINDOW_MIN_HEIGHT => {
            if let Some(wd) = n.win_mut() {
                match prop {
                    PROP_WINDOW_WIDTH => wd.width = vf.max(1.0),
                    PROP_WINDOW_HEIGHT => wd.height = vf.max(1.0),
                    PROP_WINDOW_MIN_WIDTH => wd.min_width = vf.max(0.0),
                    _ => wd.min_height = vf.max(0.0),
                }
                wd.props_dirty = true;
            }
            relayout = false;
        }
        _ => return,
    }
    if relayout {
        s.mark_layout_dirty(id);
    } else {
        s.mark_redraw(id);
    }
}

fn set_str(s: &mut State, id: u64, prop: u32, v: String) {
    let n = match s.nodes.get_mut(id) {
        Some(n) => n,
        None => return,
    };
    let mut relayout = true;
    match prop {
        PROP_TEXT => {
            n.text.text = v;
            if n.kind == Kind::TextInput {
                input::reset_input(n);
                relayout = false;
            }
        }
        PROP_PLACEHOLDER => {
            n.text.placeholder = v;
            relayout = false;
        }
        PROP_TOOLTIP => {
            n.tooltip = v;
            relayout = false;
        }
        PROP_SHORTCUT => n.shortcut = v,
        PROP_TITLE => {
            if let Some(wd) = n.win_mut() {
                wd.title = v;
                wd.props_dirty = true;
            }
            relayout = false;
        }
        _ => return,
    }
    if relayout {
        s.mark_layout_dirty(id);
    } else {
        s.mark_redraw(id);
    }
}

fn get_i64(s: &State, id: u64, prop: u32) -> i64 {
    if prop == PROP_FOCUS {
        return s
            .window_of(id)
            .and_then(|w| s.window_data(w))
            .map(|w| (w.focused == Some(id)) as i64)
            .unwrap_or(0);
    }
    let n = match s.nodes.get(id) {
        Some(n) => n,
        None => return 0,
    };
    match prop {
        PROP_VISIBLE => n
            .win()
            .map(|w| w.visible as i64)
            .unwrap_or(n.visible as i64),
        PROP_DISABLED => n.disabled as i64,
        PROP_BACKGROUND => n
            .win()
            .map(|w| w.background.to_i64())
            .unwrap_or(n.visual.background.to_i64()),
        PROP_BORDER_COLOR => n.visual.border_color.to_i64(),
        PROP_SHADOW => n.visual.shadow,
        PROP_DIRECTION => (n.layout.direction == Direction::Row) as i64,
        PROP_ALIGN_ITEMS => n.layout.align_items.unwrap_or(-1),
        PROP_JUSTIFY => n.layout.justify.unwrap_or(-1),
        PROP_ALIGN_SELF => n.layout.align_self.unwrap_or(0),
        PROP_WRAP => n.layout.wrap as i64,
        PROP_FONT_WEIGHT => n.text.weight as i64,
        PROP_TEXT_COLOR => n.text.color.to_i64(),
        PROP_TEXT_ALIGN => n.text.align,
        PROP_TEXT_WRAP => n.text.wrap as i64,
        PROP_FONT_FAMILY => n.text.family,
        PROP_VARIANT => n.control.variant,
        PROP_CHECKED => n.control.checked as i64,
        PROP_SECURE => n.control.secure as i64,
        PROP_INDETERMINATE => n.control.indeterminate as i64,
        PROP_ORIENTATION => n.control.orientation,
        PROP_FIT => n.control.fit,
        PROP_MAX_LENGTH => n.control.max_length,
        PROP_TINT => n.visual.tint.to_i64(),
        PROP_GRADIENT_END => n.visual.gradient_end.to_i64(),
        PROP_GLOW => n.visual.glow as i64,
        PROP_RIPPLE => n.visual.ripple as i64,
        PROP_HOVER_LIFT => n.visual.hover_lift as i64,
        PROP_CLICKABLE => n.clickable as i64,
        PROP_PRESS_EFFECT => n.visual.press_effect,
        PROP_CONTEXT_MENU => n.context_menu.map(|m| m as i64).unwrap_or(0),
        PROP_MATCH_ANCHOR => n.match_anchor as i64,
        PROP_ICON_END => n.control.icon_end as i64,
        PROP_ANIMATE => n.control.svg_animate as i64,
        PROP_ENTER => n.visual.enter,
        PROP_RESIZABLE => n.win().map(|w| w.resizable as i64).unwrap_or(0),
        PROP_DECORATIONS => n.win().map(|w| w.decorations as i64).unwrap_or(0),
        PROP_ALWAYS_ON_TOP => n.win().map(|w| w.always_on_top as i64).unwrap_or(0),
        PROP_MAXIMIZED => n.win().map(|w| w.maximized as i64).unwrap_or(0),
        PROP_INTERCEPT_CLOSE => n.win().map(|w| w.intercept_close as i64).unwrap_or(0),
        _ => 0,
    }
}

fn get_f64(s: &State, id: u64, prop: u32) -> f64 {
    let n = match s.nodes.get(id) {
        Some(n) => n,
        None => return 0.0,
    };
    let scale = s
        .window_of(id)
        .and_then(|w| s.window_data(w))
        .map(|w| w.scale)
        .unwrap_or(1.0) as f64;
    match prop {
        PROP_OPACITY => n.visual.opacity as f64,
        PROP_RADIUS => n.visual.radius.unwrap_or(0.0) as f64,
        PROP_BORDER_WIDTH => n.visual.border_width.unwrap_or(0.0) as f64,
        PROP_GAP => n.layout.gap as f64,
        PROP_PADDING_LEFT => n.layout.padding[0] as f64,
        PROP_PADDING_TOP => n.layout.padding[1] as f64,
        PROP_PADDING_RIGHT => n.layout.padding[2] as f64,
        PROP_PADDING_BOTTOM => n.layout.padding[3] as f64,
        PROP_MARGIN_LEFT => n.layout.margin[0] as f64,
        PROP_MARGIN_TOP => n.layout.margin[1] as f64,
        PROP_MARGIN_RIGHT => n.layout.margin[2] as f64,
        PROP_MARGIN_BOTTOM => n.layout.margin[3] as f64,
        PROP_WIDTH => n.layout.width.to_f64(),
        PROP_HEIGHT => n.layout.height.to_f64(),
        PROP_MIN_WIDTH => n.layout.min_width.to_f64(),
        PROP_MIN_HEIGHT => n.layout.min_height.to_f64(),
        PROP_MAX_WIDTH => n.layout.max_width.to_f64(),
        PROP_MAX_HEIGHT => n.layout.max_height.to_f64(),
        PROP_GROW => n.layout.grow.map(|g| g as f64).unwrap_or(f64::NAN),
        PROP_SHRINK => n.layout.shrink.map(|g| g as f64).unwrap_or(f64::NAN),
        PROP_FONT_SIZE => n.text.size.unwrap_or(0.0) as f64,
        PROP_VALUE => n.control.value,
        PROP_MIN => n.control.min,
        PROP_MAX => n.control.max,
        PROP_STEP => n.control.step,
        PROP_ICON_SIZE => n.control.icon_size as f64,
        PROP_COLOR_CYCLE => n.visual.color_cycle as f64,
        PROP_SPIN => n.control.spin as f64,
        PROP_PULSE => n.control.pulse as f64,
        PROP_SPRING_STIFFNESS => n.spring_stiffness as f64,
        PROP_SPRING_DAMPING => n.spring_damping as f64,
        PROP_SCROLL_Y => n.scroll.offset as f64 / scale,
        PROP_SCALE => n.win().map(|w| w.scale as f64).unwrap_or(scale),
        PROP_WINDOW_WIDTH => n
            .win()
            .map(|w| w.width as f64)
            .unwrap_or(n.rect.w as f64 / scale),
        PROP_WINDOW_HEIGHT => n
            .win()
            .map(|w| w.height as f64)
            .unwrap_or(n.rect.h as f64 / scale),
        PROP_WINDOW_MIN_WIDTH => n.win().map(|w| w.min_width as f64).unwrap_or(0.0),
        PROP_WINDOW_MIN_HEIGHT => n.win().map(|w| w.min_height as f64).unwrap_or(0.0),
        _ => 0.0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_set_i64(node: u64, prop: u32, value: i64) {
    guard(|| set_i64(&mut state::lock(), node, prop, value))
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_set_f64(node: u64, prop: u32, value_bits: u64) {
    guard(|| set_f64(&mut state::lock(), node, prop, f64::from_bits(value_bits)))
}

#[unsafe(no_mangle)]
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
pub unsafe extern "C" fn patina_set_str(node: u64, prop: u32, ptr: *const u8, len: usize) {
    guard(|| {
        let v = unsafe { str_arg(ptr, len) };
        set_str(&mut state::lock(), node, prop, v)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_get_i64(node: u64, prop: u32) -> i64 {
    guard(|| get_i64(&state::lock(), node, prop))
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_get_f64(node: u64, prop: u32) -> u64 {
    guard(|| get_f64(&state::lock(), node, prop).to_bits())
}

/// Copies the string property into `buf` (no terminator); returns its full length in bytes.
#[unsafe(no_mangle)]
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
pub unsafe extern "C" fn patina_get_str(node: u64, prop: u32, buf: *mut u8, cap: usize) -> usize {
    guard(|| {
        let s = state::lock();
        let n = match s.nodes.get(node) {
            Some(n) => n,
            None => return 0,
        };
        let v: &str = match prop {
            PROP_TEXT => &n.text.text,
            PROP_PLACEHOLDER => &n.text.placeholder,
            PROP_TOOLTIP => &n.tooltip,
            PROP_SHORTCUT => &n.shortcut,
            PROP_TITLE => n.win().map(|w| w.title.as_str()).unwrap_or(""),
            _ => "",
        };
        unsafe { copy_out(v.as_bytes(), buf, cap) }
    })
}

/// Set the pixels of an image node from straight-alpha RGBA8 data.
#[unsafe(no_mangle)]
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
pub unsafe extern "C" fn patina_set_image(
    node: u64,
    width: u32,
    height: u32,
    rgba: *const u8,
    len: usize,
) -> i32 {
    guard(|| {
        let need = width as usize * height as usize * 4;
        if rgba.is_null() || len < need || width == 0 || height == 0 {
            return 0;
        }
        let src = unsafe { std::slice::from_raw_parts(rgba, need) };
        let mut data = Vec::with_capacity(need);
        for px in src.chunks_exact(4) {
            let a = px[3] as u32;
            data.push(((px[0] as u32 * a + 127) / 255) as u8);
            data.push(((px[1] as u32 * a + 127) / 255) as u8);
            data.push(((px[2] as u32 * a + 127) / 255) as u8);
            data.push(px[3]);
        }
        let size = match tiny_skia::IntSize::from_wh(width, height) {
            Some(s) => s,
            None => return 0,
        };
        let pixmap = match tiny_skia::Pixmap::from_vec(data, size) {
            Some(p) => p,
            None => return 0,
        };
        let mut s = state::lock();
        match s.nodes.get_mut(node) {
            Some(n) => n.image = Some(std::sync::Arc::new(ImageData { pixmap })),
            None => return 0,
        }
        s.mark_layout_dirty(node);
        1
    })
}

// ---- theme -------------------------------------------------------------------

fn redraw_all(s: &mut State) {
    for id in s.windows.clone() {
        if let Some(wd) = s.window_data_mut(id) {
            wd.needs_redraw = true;
        }
    }
    s.wake();
}

fn relayout_all(s: &mut State) {
    for id in s.windows.clone() {
        if let Some(wd) = s.window_data_mut(id) {
            wd.layout_dirty = true;
            wd.needs_redraw = true;
        }
    }
    s.wake();
}

/// `mode`: 0 light, 1 dark, 2 follow the system.
#[unsafe(no_mangle)]
pub extern "C" fn patina_theme_set_mode(mode: u32) {
    guard(|| {
        let mut s = state::lock();
        let clock = s.clock;
        let mode = match mode {
            1 => Mode::Dark,
            2 => Mode::System,
            _ => Mode::Light,
        };
        s.theme.set_mode(mode, clock);
        if !s.started() {
            s.theme.settle();
        }
        redraw_all(&mut s);
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_theme_is_dark() -> i32 {
    guard(|| state::lock().theme.is_dark() as i32)
}

/// `rgba` as 0xRRGGBBAA; 0 restores the default accent.
#[unsafe(no_mangle)]
pub extern "C" fn patina_theme_set_accent(rgba: u32) {
    guard(|| {
        let mut s = state::lock();
        let clock = s.clock;
        let accent = if rgba == 0 {
            None
        } else {
            Some(Rgba::from_rgba_u32(rgba))
        };
        s.theme.set_accent(accent, clock);
        if !s.started() {
            s.theme.settle();
        }
        redraw_all(&mut s);
    })
}

/// Activate a named palette (built-in or defined with `patina_theme_define`), switching the
/// mode to match its kind and cross-fading to it. Returns 1 on success, 0 for unknown names.
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_theme_use(name: *const u8, len: usize) -> i32 {
    guard(|| {
        let name = unsafe { str_arg(name, len) };
        let mut s = state::lock();
        let clock = s.clock;
        if !s.theme.use_palette(&name, clock) {
            s.set_error(format!("unknown palette {name:?}"));
            return 0;
        }
        if !s.started() {
            s.theme.settle();
        }
        redraw_all(&mut s);
        1
    })
}

/// Name of the palette in effect, written to `buf` (returns the byte length; see `patina_get_str`).
/// # Safety
/// `buf` must be valid for `cap` bytes, or null with `cap == 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_theme_current(buf: *mut u8, cap: usize) -> usize {
    guard(|| {
        let name = state::lock().theme.current_name().to_string();
        unsafe { copy_out(name.as_bytes(), buf, cap) }
    })
}

/// List every known palette as newline-separated `name\tdark\taccent\ton_accent` records
/// (`dark` is 0/1, colors are 0xRRGGBBAA hex). Same buffer protocol as `patina_get_str`.
/// # Safety
/// `buf` must be valid for `cap` bytes, or null with `cap == 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_theme_list(buf: *mut u8, cap: usize) -> usize {
    guard(|| {
        let s = state::lock();
        let mut out = String::new();
        for (name, p) in s.theme.list() {
            out.push_str(&format!(
                "{}\t{}\t{:08x}\t{:08x}\n",
                name,
                p.is_dark as u8,
                p.accent.to_rgba_u32(),
                p.on_accent.to_rgba_u32()
            ));
        }
        drop(s);
        unsafe { copy_out(out.as_bytes(), buf, cap) }
    })
}

/// Register (or replace) a custom palette. `colors` holds `PATINA_PALETTE_COLORS` 0xRRGGBBAA
/// values in the order background, surface, surface_variant, text, text_secondary, text_muted,
/// accent, on_accent, outline, outline_strong, danger, success, warning, shadow. Returns 1 on
/// success, 0 for an empty name or too few colors.
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_theme_define(
    name: *const u8,
    len: usize,
    dark: i32,
    colors: *const u32,
    count: usize,
) -> i32 {
    guard(|| {
        let name = unsafe { str_arg(name, len) };
        if name.trim().is_empty() || colors.is_null() {
            return 0;
        }
        let raw = unsafe { std::slice::from_raw_parts(colors, count) };
        let cs: Vec<Rgba> = raw.iter().map(|c| Rgba::from_rgba_u32(*c)).collect();
        let Some(p) = crate::theme::Palette::from_colors(dark != 0, &cs) else {
            state::lock().set_error(format!(
                "palette {name:?} needs {} colors, got {}",
                crate::theme::Palette::FIELD_COUNT,
                cs.len()
            ));
            return 0;
        };
        let mut s = state::lock();
        s.theme.define(&name, p);
        1
    })
}

/// Seconds a palette or mode change takes to cross-fade (0 = instant). Bits of an f64.
#[unsafe(no_mangle)]
pub extern "C" fn patina_theme_set_transition(secs_bits: u64) {
    guard(|| {
        let secs = f64::from_bits(secs_bits);
        let mut s = state::lock();
        s.theme.transition_secs = if secs.is_finite() {
            secs.max(0.0)
        } else {
            0.28
        };
    })
}

// ---- animation ---------------------------------------------------------------

/// Global animation speed: 1 is normal, 0.5 is slow motion, 0 disables motion (every
/// transition completes instantly, for reduced-motion preferences). Bits of an f64.
#[unsafe(no_mangle)]
pub extern "C" fn patina_set_animation_speed(speed_bits: u64) {
    guard(|| {
        let v = f64::from_bits(speed_bits);
        let v = if v.is_finite() {
            v.clamp(0.0, 100.0)
        } else {
            1.0
        };
        let mut s = state::lock();
        s.anim_speed = v as f32;
        redraw_all(&mut s);
    })
}

/// The current global animation speed as f64 bits.
#[unsafe(no_mangle)]
pub extern "C" fn patina_get_animation_speed() -> u64 {
    guard(|| (state::lock().anim_speed as f64).to_bits())
}

/// Load custom font files. Bold and mono are optional (pass null / 0).
#[unsafe(no_mangle)]
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
pub unsafe extern "C" fn patina_theme_set_font(
    regular: *const u8,
    regular_len: usize,
    bold: *const u8,
    bold_len: usize,
    mono: *const u8,
    mono_len: usize,
) -> i32 {
    guard(|| {
        let reg = unsafe { str_arg(regular, regular_len) };
        let bold = unsafe { str_arg(bold, bold_len) };
        let mono = unsafe { str_arg(mono, mono_len) };
        if reg.is_empty() {
            return 0;
        }
        let mut s = state::lock();
        let bold_p = if bold.is_empty() {
            None
        } else {
            Some((Path::new(&bold), 0))
        };
        let mono_p = if mono.is_empty() {
            None
        } else {
            Some((Path::new(&mono), 0))
        };
        match s
            .text
            .load_files((Path::new(&reg), 0), None, bold_p, mono_p)
        {
            Ok(()) => {
                relayout_all(&mut s);
                1
            }
            Err(e) => {
                s.set_error(e);
                0
            }
        }
    })
}

// ---- timers / posting --------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn patina_timer_start(token: u64, interval_ms: u64, repeat: i32) {
    guard(|| state::lock().timer_start(token, interval_ms, repeat != 0))
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_timer_cancel(token: u64) {
    guard(|| state::lock().timer_cancel(token))
}

/// Ask the UI thread to emit `PATINA_EV_POSTED` with `token`. Safe from any thread.
#[unsafe(no_mangle)]
pub extern "C" fn patina_post(token: u64) {
    guard(|| state::lock().post(token))
}

// ---- vector graphics ---------------------------------------------------------

/// Set SVG markup as the content of an image node or the icon of a button. An empty string
/// clears it. Returns 1 on success, 0 on a parse error (see `patina_last_error`).
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_set_svg(node: u64, svg: *const u8, len: usize) -> i32 {
    guard(|| {
        let src = unsafe { str_arg(svg, len) };
        let doc = if src.trim().is_empty() {
            None
        } else {
            match SvgDoc::parse(&src) {
                Ok(d) => Some(std::sync::Arc::new(d)),
                Err(e) => {
                    state::lock().set_error(e);
                    return 0;
                }
            }
        };
        let mut s = state::lock();
        let clock = s.clock;
        match s.nodes.get_mut(node) {
            Some(n) => {
                n.svg = doc;
                n.svg_epoch = clock;
            }
            None => return 0,
        }
        s.mark_layout_dirty(node);
        1
    })
}

/// Rasterize SVG markup to straight-alpha RGBA8 at `width` x `height`. `tint` uses the ABI
/// color encoding: the default color keeps the document's own colors (`currentColor` becomes
/// the theme text color), anything else recolors the result using the document as a mask.
/// Returns `width * height * 4`, writing the pixels only when `cap` is large enough; 0 on error.
/// # Safety
/// Pointer arguments must be valid for the given lengths, or null with length 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_svg_render(
    svg: *const u8,
    len: usize,
    width: u32,
    height: u32,
    tint: i64,
    out: *mut u8,
    cap: usize,
) -> usize {
    guard(|| {
        let src = unsafe { str_arg(svg, len) };
        if width == 0 || height == 0 {
            state::lock().set_error("svg: size must be positive");
            return 0;
        }
        let doc = match SvgDoc::parse(&src) {
            Ok(d) => d,
            Err(e) => {
                state::lock().set_error(e);
                return 0;
            }
        };
        let pal = state::lock().theme.palette();
        let tint = match ColorRef::from_i64(tint) {
            ColorRef::Default => None,
            c => Some(pal.resolve(c, pal.text)),
        };
        let current = tint.unwrap_or(pal.text);
        let pm = match doc.render(width, height, current, tint) {
            Some(p) => p,
            None => {
                state::lock().set_error("svg: rendering failed");
                return 0;
            }
        };
        let data = pm.take_demultiplied();
        let need = data.len();
        if !out.is_null() && cap >= need {
            unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), out, need) };
        }
        need
    })
}

/// Set the window's title bar and taskbar icon from straight-alpha RGBA8 pixels.
/// # Safety
/// `rgba` must point to at least `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn patina_window_set_icon(
    window: u64,
    width: u32,
    height: u32,
    rgba: *const u8,
    len: usize,
) -> i32 {
    guard(|| {
        let need = width as usize * height as usize * 4;
        if rgba.is_null() || len < need || width == 0 || height == 0 {
            return 0;
        }
        let data = unsafe { std::slice::from_raw_parts(rgba, need) }.to_vec();
        let mut s = state::lock();
        match s.window_data_mut(window) {
            Some(wd) => {
                wd.icon = Some(std::sync::Arc::new(WindowIcon {
                    width,
                    height,
                    rgba: data,
                }));
                wd.icon_dirty = true;
                wd.props_dirty = true;
            }
            None => return 0,
        }
        s.wake();
        1
    })
}

// ---- menus -------------------------------------------------------------------------------

/// Open a menu: below `anchor` when it is a widget, or at the logical point (`x`, `y`, as
/// f64 bits) inside the window when `anchor` is the window itself. Returns 1 on success.
#[unsafe(no_mangle)]
pub extern "C" fn patina_menu_open(menu: u64, anchor: u64, x_bits: u64, y_bits: u64) -> i32 {
    guard(|| {
        let x = f64::from_bits(x_bits);
        let y = f64::from_bits(y_bits);
        let mut s = state::lock();
        match input::open_menu(&mut s, menu, anchor, x as f32, y as f32) {
            Ok(()) => 1,
            Err(e) => {
                s.set_error(format!("menu: {e}"));
                0
            }
        }
    })
}

/// Close a menu (it fades out). Emits `EV_MENU_CLOSED` when it was open.
#[unsafe(no_mangle)]
pub extern "C" fn patina_menu_close(menu: u64) {
    guard(|| {
        let mut s = state::lock();
        let events = input::close_menu(&mut s, menu);
        if !events.is_empty() {
            s.pending_events.extend(events);
            s.wake();
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_menu_is_open(menu: u64) -> i32 {
    guard(|| input::menu_is_open(&state::lock(), menu) as i32)
}

// ---- motion style ------------------------------------------------------------------------

/// Default motion for value, toggle and entrance animations: 0 ease, 1 spring, 2 bouncy.
#[unsafe(no_mangle)]
pub extern "C" fn patina_set_motion(style: u32) {
    guard(|| {
        let mut s = state::lock();
        s.motion = crate::anim::Motion::from_style(style);
        redraw_all(&mut s);
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn patina_get_motion() -> u32 {
    guard(|| state::lock().motion.style())
}
