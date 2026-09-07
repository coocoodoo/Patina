//! Painting of the widget tree. All coordinates are physical pixels.

use std::f32::consts::TAU;

use tiny_skia::{Pixmap, PixmapMut, Transform};

use crate::anim::{Motion, Tween, ease_out_cubic};
use crate::color::{ColorRef, Rgba};
use crate::geom::Rect;
use crate::layout::metrics::*;
use crate::layout::{font_face, font_px};
use crate::node::*;
use crate::paint::{Painter, ShadowCache, arc_path};
use crate::props::*;
use crate::state::{Arena, Id, State};
use crate::svg::{SvgCache, SvgDoc};
use crate::text::{TextLayout, TextSystem};
use crate::theme::Palette;

/// How long a click ripple lasts, in animation-clock seconds.
pub const RIPPLE_SECS: f64 = 0.55;
const ENTER_SPEED: f32 = 9.0;
const GLOW_PERIOD: f32 = 1.8;

pub struct Frame<'a> {
    pub p: Painter<'a>,
    pub nodes: &'a Arena,
    pub text: &'a mut TextSystem,
    pub shadows: &'a mut ShadowCache,
    pub svgs: &'a mut SvgCache,
    pub pal: Palette,
    /// Wall-clock seconds (caret blinking).
    pub now: f64,
    /// Animation-clock seconds (everything that moves; scaled by the animation speed).
    pub clock: f64,
    pub focused: Option<Id>,
    pub focus_visible: bool,
    pub hovered: Option<Id>,
    pub pressed: Option<Id>,
    /// Menu item highlighted with the keyboard.
    pub menu_focus: Option<Id>,
}

fn tick(t: &mut Tween, dt: f32, speed: f32, motion: Motion, snap: bool) -> bool {
    if snap {
        let v = t.target;
        t.snap(v);
        false
    } else {
        t.advance(dt, speed, motion)
    }
}

/// Step every tween of a window (content, open menus and the tooltip) and report whether
/// anything still needs frames. `snap` makes every transition finish immediately (reduced
/// motion). `motion` is the default style for value, toggle and entrance tweens.
pub fn advance_animations(
    nodes: &mut Arena,
    win: Id,
    dt: f32,
    clock: f64,
    snap: bool,
    motion: Motion,
) -> bool {
    let (content, overlays) = match nodes.get(win).and_then(|n| n.win()) {
        Some(w) => (w.content, w.overlays.clone()),
        None => return false,
    };
    let mut active = false;
    let mut stack: Vec<Id> = content
        .into_iter()
        .chain(overlays.iter().copied())
        .collect();
    let mut closed: Vec<Id> = Vec::new();
    while let Some(id) = stack.pop() {
        let n = match nodes.get_mut(id) {
            Some(n) => n,
            None => continue,
        };
        let m = n.motion_override().unwrap_or(motion);
        active |= tick(&mut n.anim.hover, dt, 16.0, Motion::Ease, snap);
        active |= tick(&mut n.anim.press, dt, 28.0, Motion::Ease, snap);
        active |= tick(&mut n.anim.focus, dt, 18.0, Motion::Ease, snap);
        active |= tick(&mut n.anim.check, dt, 14.0, m, snap);
        active |= tick(&mut n.anim.value, dt, 12.0, m, snap);
        active |= tick(&mut n.scroll.bar, dt, 10.0, Motion::Ease, snap);
        if !n.entered && !n.rect.is_empty() {
            n.entered = true;
            if n.visual.enter != 0 && !snap {
                n.anim.enter.snap(0.0);
                n.anim.enter.set(1.0);
            } else {
                n.anim.enter.snap(1.0);
                n.press_wobble = None;
            }
        }
        active |= tick(&mut n.anim.enter, dt, ENTER_SPEED, m, snap);
        if n.closing && (snap || n.anim.enter.value <= 0.02) {
            closed.push(id);
        }
        if !n.entered && n.visible && n.visual.enter != 0 {
            // Not laid out yet: the entrance starts on the frame after layout.
            active = true;
        }
        if n.visible {
            let animated_svg = n.control.svg_animate
                && n.svg.as_ref().is_some_and(|d| {
                    d.is_animated() && d.end().is_none_or(|e| clock - n.svg_epoch <= e + 0.05)
                });
            if (n.kind == Kind::Progress && n.control.indeterminate)
                || n.kind == Kind::Spinner
                || n.visual.glow
                || n.visual.color_cycle > 0.0
                || n.control.spin > 0.0
                || n.control.pulse > 0.0
                || animated_svg
            {
                active = true;
            }
            if let Some(start) = n.press_wobble {
                if clock - start < wobble_secs(n.visual.press_effect) {
                    active = true;
                } else {
                    n.press_wobble = None;
                }
            }
            if let Some(r) = &n.ripple {
                if clock - r.start < RIPPLE_SECS {
                    active = true;
                } else {
                    n.ripple = None;
                }
            }
        }
        if n.visible {
            stack.extend(n.children.iter().copied());
        }
    }
    for id in closed {
        finish_close(nodes, id);
    }
    if let Some(w) = nodes.get_mut(win).and_then(|n| n.win_mut()) {
        if let Some(tip) = &mut w.tooltip {
            if tip.shown || tip.closing {
                active |= tick(&mut tip.anim, dt, 18.0, Motion::Ease, snap);
            }
            if tip.closing && tip.anim.value <= 0.01 {
                w.tooltip = None;
            }
        }
    }
    active && !snap
}

/// Remove a closed menu from its window's overlays and reset it for the next open.
pub fn finish_close(nodes: &mut Arena, id: Id) {
    let parent = nodes.get(id).and_then(|n| n.parent);
    if let Some(w) = parent
        .and_then(|p| nodes.get_mut(p))
        .and_then(|n| n.win_mut())
    {
        w.overlays.retain(|o| *o != id);
        if w.hovered.is_some_and(|h| h == id) {
            w.hovered = None;
        }
    }
    if let Some(n) = nodes.get_mut(id) {
        n.parent = None;
        n.closing = false;
        n.entered = false;
        n.anim.enter.snap(1.0);
        n.rect = Rect::default();
    }
}

/// Snap all tweens of a window to their targets (used for offscreen snapshots).
pub fn settle_animations(nodes: &mut Arena, win: Id) {
    let (content, overlays) = match nodes.get(win).and_then(|n| n.win()) {
        Some(w) => (w.content, w.overlays.clone()),
        None => return,
    };
    for o in &overlays {
        if nodes.get(*o).map(|n| n.closing).unwrap_or(false) {
            finish_close(nodes, *o);
        }
    }
    let mut stack: Vec<Id> = content
        .into_iter()
        .chain(overlays.iter().copied())
        .collect();
    while let Some(id) = stack.pop() {
        if let Some(n) = nodes.get_mut(id) {
            for t in [
                &mut n.anim.hover,
                &mut n.anim.press,
                &mut n.anim.focus,
                &mut n.anim.check,
                &mut n.anim.value,
                &mut n.scroll.bar,
            ] {
                let v = t.target;
                t.snap(v);
            }
            n.entered = true;
            n.anim.enter.snap(1.0);
            n.press_wobble = None;
            stack.extend(n.children.iter().copied());
        }
    }
    if let Some(w) = nodes.get_mut(win).and_then(|n| n.win_mut()) {
        if let Some(tip) = &mut w.tooltip {
            if tip.closing {
                w.tooltip = None;
            } else {
                tip.shown = true;
                tip.anim.snap(1.0);
            }
        }
    }
}

/// Render one window's tree into `pm` (already sized to the window).
pub fn render_window(state: &mut State, win: Id, pm: PixmapMut, scale: f32) {
    let clock = state.clock;
    let pal = state.theme.palette_at(clock);
    let now = state.now();
    let (content, overlays, focused, focus_visible, hovered, pressed, menu_focus, bg, tooltip) =
        match state.window_data(win) {
            Some(w) => (
                w.content,
                w.overlays.clone(),
                w.focused,
                w.focus_visible,
                w.hovered,
                w.pressed,
                w.menu_focus,
                w.background,
                w.tooltip.clone(),
            ),
            None => return,
        };
    let State {
        nodes,
        text,
        shadows,
        svgs,
        ..
    } = state;
    let mut p = Painter::new(pm, scale);
    p.pm.fill(pal.resolve(bg, pal.background).to_skia());
    if !text.ensure_loaded() {
        return;
    }
    let mut f = Frame {
        p,
        nodes,
        text,
        shadows,
        svgs,
        pal,
        now,
        clock,
        focused,
        focus_visible,
        hovered,
        pressed,
        menu_focus,
    };
    if let Some(c) = content {
        paint_node(&mut f, c);
    }
    for o in overlays {
        paint_node(&mut f, o);
    }
    if let Some(tip) = tooltip {
        paint_tooltip(&mut f, &tip);
    }
}

fn paint_node(f: &mut Frame, id: Id) {
    let nodes = f.nodes;
    let n = match nodes.get(id) {
        Some(n) => n,
        None => return,
    };
    if !n.visible || n.rect.is_empty() {
        return;
    }
    if n.visual.enter != 0 && !n.entered {
        return;
    }
    let prev_alpha = f.p.alpha;
    let prev_offset = f.p.offset;
    let mut alpha = n.visual.opacity.clamp(0.0, 1.0);
    if n.disabled {
        alpha *= 0.5;
    }
    let enter = n.anim.enter.value;
    if (enter - 1.0).abs() > 0.001 {
        alpha *= ease_out_cubic(enter);
        // Raw progress, so a spring entrance overshoots its resting place and settles back.
        let d = (1.0 - enter) * 18.0 * f.p.scale;
        match n.visual.enter {
            2 => f.p.offset.1 += d,
            3 => f.p.offset.1 -= d,
            4 => f.p.offset.0 += d,
            5 => f.p.offset.0 -= d,
            6 => f.p.offset.1 += d * 0.4,
            _ => {}
        }
    }
    if n.visual.hover_lift {
        f.p.offset.1 -= 2.0 * f.p.scale * n.anim.hover.value;
    }
    if f.p.shifted(n.rect).intersect(&f.p.clip_rect()).is_empty() {
        f.p.offset = prev_offset;
        return;
    }
    f.p.alpha = prev_alpha * alpha;
    match press_transform(n, f.clock) {
        Some((sx, sy, kx)) => paint_deformed(f, n, id, sx, sy, kx),
        None => paint_kind(f, n, id),
    }
    f.p.alpha = prev_alpha;
    f.p.offset = prev_offset;
}

/// How long a press effect wobbles after the click, in animation-clock seconds.
pub fn wobble_secs(effect: i64) -> f64 {
    match effect {
        PRESS_RUBBER => 0.9,
        PRESS_GELATIN => 0.8,
        PRESS_BOUNCE => 0.6,
        _ => 0.0,
    }
}

/// The deformation (x scale, y scale, x skew) of a widget with a press effect: a squash
/// while pressed and a damped wobble after the click. `None` when it is at rest.
pub fn press_transform(n: &Node, clock: f64) -> Option<(f32, f32, f32)> {
    let effect = n.visual.press_effect;
    if effect == 0 {
        return None;
    }
    let press = n.anim.press.value;
    let (mut sx, mut sy, mut kx) = (1.0f32, 1.0f32, 0.0f32);
    match effect {
        PRESS_RUBBER => {
            sx -= 0.04 * press;
            sy -= 0.04 * press;
        }
        PRESS_GELATIN => {
            sx += 0.04 * press;
            sy -= 0.08 * press;
        }
        _ => {
            sx -= 0.06 * press;
            sy -= 0.06 * press;
        }
    }
    if let Some(start) = n.press_wobble {
        let t = (clock - start) as f32;
        if t >= 0.0 && (t as f64) < wobble_secs(effect) {
            match effect {
                PRESS_RUBBER => {
                    // Stretch sideways past the rest size, then settle like a rubber band.
                    let a = 0.22 * (-3.6 * t).exp() * (t * 5.8).sin();
                    sx += a;
                    sy -= a * 0.6;
                }
                PRESS_GELATIN => {
                    // Fast squash-and-stretch with a lean, like jelly.
                    let e = (-5.0 * t).exp();
                    let a = 0.12 * e * (t * 14.0).sin();
                    sx += a;
                    sy -= a;
                    kx = 0.10 * e * (t * 14.0 + 1.2).sin();
                }
                _ => {
                    let a = 0.16 * (-6.0 * t).exp() * (t * 12.5).sin();
                    sx += a;
                    sy += a;
                }
            }
        }
    }
    if (sx - 1.0).abs() < 1e-3 && (sy - 1.0).abs() < 1e-3 && kx.abs() < 1e-3 {
        None
    } else {
        Some((sx, sy, kx))
    }
}

fn paint_kind(f: &mut Frame, n: &Node, id: Id) {
    match n.kind {
        Kind::Box => paint_box(f, n),
        Kind::Scroll => paint_scroll(f, n),
        Kind::Label => paint_label(f, n),
        Kind::Button => paint_button(f, n),
        Kind::TextInput => paint_input(f, n, id),
        Kind::Checkbox => paint_checkbox(f, n),
        Kind::Switch => paint_switch(f, n),
        Kind::Slider => paint_slider(f, n),
        Kind::Progress => paint_progress(f, n),
        Kind::Divider => paint_divider(f, n),
        Kind::Image => paint_image(f, n),
        Kind::Spinner => paint_spinner(f, n),
        Kind::Menu => paint_menu(f, n),
        Kind::MenuItem => paint_menu_item(f, n, id),
        Kind::Spacer | Kind::Window => {}
    }
}

fn sub_frame<'b>(f: &'b mut Frame<'_>, pm: PixmapMut<'b>) -> Frame<'b> {
    Frame {
        p: Painter::new(pm, f.p.scale),
        nodes: f.nodes,
        text: &mut *f.text,
        shadows: &mut *f.shadows,
        svgs: &mut *f.svgs,
        pal: f.pal,
        now: f.now,
        clock: f.clock,
        focused: f.focused,
        focus_visible: f.focus_visible,
        hovered: f.hovered,
        pressed: f.pressed,
        menu_focus: f.menu_focus,
    }
}

/// Paint a widget through a scale/skew about its center: it is rendered offscreen (with room
/// for its shadow) and blitted with the transform, so text and icons deform with it.
fn paint_deformed(f: &mut Frame, n: &Node, id: Id, sx: f32, sy: f32, kx: f32) {
    let r = n.rect;
    let margin = (56.0 * f.p.scale).ceil();
    let w = (r.w + 2.0 * margin).ceil() as u32;
    let h = (r.h + 2.0 * margin).ceil() as u32;
    let Some(mut tmp) = Pixmap::new(w.max(1), h.max(1)) else {
        paint_kind(f, n, id);
        return;
    };
    let x0 = (r.x - margin).floor();
    let y0 = (r.y - margin).floor();
    {
        let mut sub = sub_frame(f, tmp.as_mut());
        sub.p.offset = (-x0, -y0);
        paint_kind(&mut sub, n, id);
    }
    let (cx, cy) = r.center();
    let about = Transform::from_translate(-cx, -cy)
        .post_concat(Transform::from_row(sx, 0.0, kx, sy, 0.0, 0.0))
        .post_translate(cx, cy);
    let t = Transform::from_translate(x0, y0).post_concat(about);
    f.p.draw_pixmap_transformed(&tmp, t, 1.0);
}

fn radius_of(f: &Frame, n: &Node, default: f32) -> f32 {
    n.visual
        .radius
        .map(|r| r * f.p.scale)
        .unwrap_or(default * f.p.scale)
}

fn text_clip(f: &Frame, r: Rect) -> Rect {
    let r = f.p.shifted(r);
    f.p.clip_rect()
        .intersect(&Rect::new(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0))
}

fn draw_text_in(f: &mut Frame, layout: &TextLayout, r: Rect, align: i64, color: Rgba) {
    let y = if layout.lines.len() <= 1 {
        r.y + (r.h - layout.height) * 0.5
    } else {
        r.y
    };
    let clip = text_clip(f, r);
    let color = color.mul_alpha(f.p.alpha);
    let (ox, oy) = f.p.offset;
    f.text.draw(
        &mut f.p.pm,
        layout,
        r.x + ox,
        y + oy,
        r.w,
        align,
        color,
        clip,
    );
}

/// Quantize an alpha so animated shadows reuse a small number of cached sprites.
fn quantize(a: f32) -> f32 {
    (a * 32.0).round() / 32.0
}

fn hue_of(c: Rgba) -> f32 {
    let max = c.r.max(c.g).max(c.b);
    let min = c.r.min(c.g).min(c.b);
    let d = max - min;
    if d < 1e-4 {
        return 0.0;
    }
    let h = if max == c.r {
        ((c.g - c.b) / d) % 6.0
    } else if max == c.g {
        (c.b - c.r) / d + 2.0
    } else {
        (c.r - c.g) / d + 4.0
    };
    (h * 60.0 + 360.0) % 360.0
}

fn hsl(h: f32, s: f32, l: f32) -> Rgba {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c * 0.5;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Rgba::new(r + m, g + m, b + m, 1.0)
}

/// The expanding circle drawn after a press on widgets with the ripple effect.
fn paint_ripple(f: &mut Frame, n: &Node, r: Rect, radius: f32, color: Rgba) {
    let Some(rp) = &n.ripple else {
        return;
    };
    let t = ((f.clock - rp.start) / RIPPLE_SECS) as f32;
    if !(0.0..1.0).contains(&t) {
        return;
    }
    let e = ease_out_cubic(t);
    let max_r = r.w.max(r.h) * 1.1;
    f.p.push_clip(r, radius);
    f.p.fill_circle(rp.x, rp.y, max_r * e, color.with_alpha(0.28 * (1.0 - t)));
    f.p.pop_clip();
}

fn paint_visual(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let radius = radius_of(f, n, 0.0);
    let pal = f.pal;
    let scale = f.p.scale;
    if n.visual.shadow > 0 {
        if let Some((c, blur, dy)) = pal.shadow(n.visual.shadow) {
            f.p.draw_shadow(f.shadows, r, radius, blur * scale, dy * scale, c);
        }
    }
    if n.visual.hover_lift && n.anim.hover.value > 0.01 {
        let level = (n.visual.shadow + 1).clamp(1, 3);
        if let Some((c, blur, dy)) = pal.shadow(level) {
            let a = quantize(c.a * n.anim.hover.value);
            f.p.draw_shadow(
                f.shadows,
                r,
                radius,
                blur * scale,
                dy * scale,
                c.with_alpha(a),
            );
        }
    }
    let bg = pal.resolve(n.visual.background, Rgba::TRANSPARENT);
    if !n.visual.gradient_end.is_default() {
        let end = pal.resolve(n.visual.gradient_end, bg);
        f.p.fill_rrect_hgradient(r, radius, bg, end);
    } else if bg.is_visible() {
        f.p.fill_rrect(r, radius, bg);
    }
    if n.visual.ripple {
        paint_ripple(f, n, r, radius, pal.text);
    }
    let bw = n.visual.border_width.unwrap_or(0.0) * scale;
    if bw > 0.0 {
        let bc = pal.resolve(n.visual.border_color, pal.outline);
        f.p.stroke_rrect(r, radius, bw, bc);
    }
}

fn paint_box(f: &mut Frame, n: &Node) {
    paint_visual(f, n);
    for c in &n.children {
        paint_node(f, *c);
    }
}

/// A popup menu: an elevated surface with a border, then its items.
fn paint_menu(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let scale = f.p.scale;
    let radius = radius_of(f, n, 10.0);
    let pal = f.pal;
    let level = if n.visual.shadow > 0 {
        n.visual.shadow
    } else {
        3
    };
    if let Some((c, blur, dy)) = pal.shadow(level) {
        f.p.draw_shadow(f.shadows, r, radius, blur * scale, dy * scale, c);
    }
    let bg = pal.resolve(n.visual.background, pal.surface);
    f.p.fill_rrect(r, radius, bg);
    let bw = n.visual.border_width.unwrap_or(1.0) * scale;
    if bw > 0.0 {
        let bc = pal.resolve(n.visual.border_color, pal.outline);
        f.p.stroke_rrect(r, radius, bw, bc);
    }
    for c in &n.children {
        paint_node(f, *c);
    }
}

/// A menu row: hover/keyboard highlight, a check mark or icon column, the label and an
/// optional shortcut hint at the right.
fn paint_menu_item(f: &mut Frame, n: &Node, id: Id) {
    let r = n.rect;
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let pal = f.pal;
    let hover = n.anim.hover.value;
    let press = n.anim.press.value;
    let keyed = f.menu_focus == Some(id);
    let radius = radius_of(f, n, 6.0);
    let highlight = (0.08 * hover + 0.06 * press).min(0.16) + if keyed { 0.12 } else { 0.0 };
    if highlight > 0.005 {
        f.p.fill_rrect(r, radius, pal.accent.with_alpha(highlight));
    }
    let tc = pal.resolve(n.text.color, pal.text);
    let col_x = r.x + s(MENU_PAD_X);
    let icon_px = s(18.0).round();
    let cy = r.y + r.h * 0.5;
    if n.control.checked {
        let k = icon_px;
        let cx = col_x + k * 0.5;
        let pt = |px: f32, py: f32| (cx + (px - 0.5) * k, cy + (py - 0.5) * k);
        let pts = [pt(0.2, 0.55), pt(0.42, 0.75), pt(0.82, 0.3)];
        f.p.stroke_polyline(&pts, s(2.0), pal.accent, true);
    } else if let Some(doc) = &n.svg {
        let size = icon_px.max(1.0) as u32;
        if let Some(pm) = f.svgs.get(doc, size, size, tc, Some(tc)) {
            let ix = col_x.round() as i32;
            let iy = (cy - icon_px * 0.5).round() as i32;
            f.p.draw_pixmap(&pm, ix, iy, 1.0);
        }
    }
    let px = font_px(n, scale);
    let face = font_face(n);
    let text_x = col_x + s(MENU_ICON_COL);
    if !n.text.text.is_empty() {
        let layout = f.text.layout(&n.text.text, face, px, f32::INFINITY, false);
        let tr = Rect::new(text_x, r.y, layout.width + s(2.0), r.h);
        draw_text_in(f, &layout, tr, 0, tc);
    }
    if !n.shortcut.is_empty() {
        let layout = f
            .text
            .layout(&n.shortcut, face, px * 0.92, f32::INFINITY, false);
        let sx = r.right() - s(MENU_PAD_X) - layout.width;
        let tr = Rect::new(sx, r.y, layout.width + s(2.0), r.h);
        draw_text_in(f, &layout, tr, 0, pal.text_muted);
    }
}

/// The tooltip of a window: a small inverted bubble near the widget it belongs to.
fn paint_tooltip(f: &mut Frame, tip: &TooltipState) {
    let a = tip.anim.value.clamp(0.0, 1.0);
    if a <= 0.005 || tip.text.is_empty() {
        return;
    }
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let Some(anchor) = f.nodes.get(tip.node).map(|n| n.rect) else {
        return;
    };
    if anchor.is_empty() {
        return;
    }
    let px = s(12.5);
    let face = TextSystem::face_for(400, 0);
    let layout = f.text.layout(&tip.text, face, px, s(TOOLTIP_MAX_W), true);
    let (pad_x, pad_y) = (s(9.0), s(6.0));
    let w = (layout.width + 2.0 * pad_x).ceil();
    let h = (layout.height + 2.0 * pad_y).ceil();
    let (win_w, win_h) = (f.p.width(), f.p.height());
    let margin = s(6.0);
    let gap = s(8.0);
    let x = (anchor.x + (anchor.w - w) * 0.5).clamp(margin, (win_w - w - margin).max(margin));
    let fits_below = anchor.bottom() + gap + h <= win_h - margin;
    let y = if fits_below {
        anchor.bottom() + gap
    } else {
        (anchor.y - h - gap).max(margin)
    };
    // Rise into place from the anchor's side.
    let rise = (1.0 - a) * s(4.0);
    let y = if fits_below { y - rise } else { y + rise };
    let r = Rect::new(x.round(), y.round(), w, h);
    let pal = f.pal;
    let prev_alpha = f.p.alpha;
    f.p.alpha = a;
    let radius = s(6.0);
    f.p.draw_shadow(
        f.shadows,
        r,
        radius,
        s(12.0),
        s(3.0),
        pal.shadow.with_alpha(0.2),
    );
    f.p.fill_rrect(r, radius, pal.text.with_alpha(0.96));
    let tr = Rect::new(
        r.x + pad_x,
        r.y + pad_y,
        layout.width + s(1.0),
        layout.height,
    );
    draw_text_in(f, &layout, tr, 0, pal.background);
    f.p.alpha = prev_alpha;
}

/// Scrollbar thumb rectangle for a scroll container, if it is scrollable.
pub fn scrollbar_thumb_rect(n: &Node, scale: f32) -> Option<Rect> {
    if n.kind != Kind::Scroll {
        return None;
    }
    let r = n.rect;
    let content_h = n.scroll.content_h;
    if content_h <= r.h + 0.5 || r.h <= 0.0 {
        return None;
    }
    let pad = SCROLLBAR_PAD * scale;
    let track_h = (r.h - 2.0 * pad).max(1.0);
    let thumb_h = (r.h / content_h * track_h).max((24.0 * scale).min(track_h));
    let max_off = (content_h - r.h).max(1.0);
    let frac = (n.scroll.offset / max_off).clamp(0.0, 1.0);
    let ty = r.y + pad + frac * (track_h - thumb_h);
    let tx = r.right() - SCROLLBAR_W * scale - pad;
    Some(Rect::new(tx, ty, SCROLLBAR_W * scale, thumb_h))
}

fn paint_scroll(f: &mut Frame, n: &Node) {
    paint_visual(f, n);
    let radius = radius_of(f, n, 0.0);
    f.p.push_clip(n.rect, radius);
    for c in &n.children {
        paint_node(f, *c);
    }
    f.p.pop_clip();
    let alpha = n.scroll.bar.value;
    if alpha > 0.01 {
        if let Some(thumb) = scrollbar_thumb_rect(n, f.p.scale) {
            let strength = if n.scroll.bar_grab.is_some() {
                0.5
            } else {
                0.3
            };
            let c = f.pal.text.with_alpha(strength * alpha);
            f.p.fill_rrect(thumb, thumb.w * 0.5, c);
        }
    }
}

fn paint_label(f: &mut Frame, n: &Node) {
    if n.text.text.is_empty() {
        return;
    }
    let px = font_px(n, f.p.scale);
    let face = font_face(n);
    let color = f.pal.resolve(n.text.color, f.pal.text);
    let mw = if n.text.wrap { n.rect.w } else { f32::INFINITY };
    let layout = f.text.layout(&n.text.text, face, px, mw, n.text.wrap);
    draw_text_in(f, &layout, n.rect, n.text.align, color);
}

fn paint_button(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let radius = radius_of(f, n, 10.0);
    let hover = n.anim.hover.value;
    let press = if f.hovered == f.pressed {
        n.anim.press.value
    } else {
        0.0
    };
    let focus = n.anim.focus.value;
    let pal = f.pal;

    let mut border: Option<Rgba> = None;
    let mut shadow = false;
    let (mut top, mut bottom, mut text_color) = match n.control.variant {
        VARIANT_TONAL => {
            let c = pal.accent.with_alpha(0.13 + 0.06 * hover + 0.08 * press);
            (c, c, pal.accent)
        }
        VARIANT_OUTLINED => {
            border = Some(pal.outline_strong);
            let c = pal
                .surface
                .mix(pal.surface_variant, (hover + press * 0.6).min(1.0));
            (c, c, pal.text)
        }
        VARIANT_GHOST => {
            let c = pal.text.with_alpha(0.06 * hover + 0.06 * press);
            (c, c, pal.text)
        }
        v => {
            shadow = true;
            let base = if v == VARIANT_DANGER {
                pal.danger
            } else {
                pal.accent
            };
            let base = base.lighten(0.07 * hover).darken(0.10 * press);
            (base.lighten(0.09), base, pal.on_accent)
        }
    };
    if !n.visual.background.is_default() {
        let c = pal
            .resolve(n.visual.background, bottom)
            .lighten(0.07 * hover)
            .darken(0.10 * press);
        top = c.lighten(0.06);
        bottom = c;
        shadow = c.a > 0.99;
        if c.a > 0.5 {
            text_color = if c.luminance() > 0.6 {
                Rgba::hex(0x14161C)
            } else {
                Rgba::WHITE
            };
        }
    }
    // Animated hue cycling replaces the fill with a moving two-tone gradient.
    let mut horizontal: Option<(Rgba, Rgba)> = None;
    if n.visual.color_cycle > 0.0 {
        let base_hue = hue_of(bottom);
        let hue = base_hue + (f.clock as f32 / n.visual.color_cycle) * 360.0;
        let (sat, lum) = if pal.is_dark {
            (0.72, 0.6)
        } else {
            (0.78, 0.52)
        };
        let a = hsl(hue, sat, lum).lighten(0.06 * hover).darken(0.1 * press);
        let b = hsl(hue + 45.0, sat, lum + 0.06)
            .lighten(0.06 * hover)
            .darken(0.1 * press);
        bottom = a;
        top = a;
        horizontal = Some((a, b));
        text_color = Rgba::WHITE;
        shadow = true;
    } else if !n.visual.gradient_end.is_default() {
        let end = pal
            .resolve(n.visual.gradient_end, bottom)
            .lighten(0.07 * hover)
            .darken(0.10 * press);
        horizontal = Some((bottom, end));
        text_color = if bottom.mix(end, 0.5).luminance() > 0.6 {
            Rgba::hex(0x14161C)
        } else {
            Rgba::WHITE
        };
        shadow = true;
    }
    if n.visual.glow {
        let pulse = 0.5 + 0.5 * (f.clock as f32 * TAU / GLOW_PERIOD).sin();
        let glow_color = horizontal.map(|(a, b)| a.mix(b, 0.5)).unwrap_or(bottom);
        let a = quantize(0.22 + 0.34 * pulse);
        f.p.draw_shadow(
            f.shadows,
            r.outset(s(2.0)),
            radius + s(2.0),
            s(22.0),
            0.0,
            glow_color.with_alpha(a),
        );
    }
    if shadow && (!pal.is_dark || bottom.luminance() > 0.3) {
        let sc = bottom.with_alpha(0.30 * (1.0 - press * 0.7));
        f.p.draw_shadow(f.shadows, r, radius, s(9.0), s(2.5), sc);
    }
    match horizontal {
        Some((a, b)) => f.p.fill_rrect_hgradient(r, radius, a, b),
        None if top == bottom => f.p.fill_rrect(r, radius, bottom),
        None => f.p.fill_rrect_vgradient(r, radius, top, bottom),
    }
    let tc = pal.resolve(n.text.color, text_color);
    if n.visual.ripple {
        paint_ripple(f, n, r, radius, tc);
    }
    if let Some(b) = border {
        let b = b.mix(pal.accent.with_alpha(0.7), hover * 0.5);
        f.p.stroke_rrect(r, radius, s(1.0), b);
    }
    let bw = n.visual.border_width.unwrap_or(0.0) * scale;
    if bw > 0.0 {
        let bc = pal.resolve(n.visual.border_color, pal.outline_strong);
        f.p.stroke_rrect(r, radius, bw, bc);
    }
    if focus > 0.01 && f.focus_visible {
        f.p.stroke_rrect_outside(r, radius, s(2.0), pal.accent.with_alpha(0.55 * focus));
    }
    // Content: an optional icon and the caption, centered as a group. The icon sits before
    // the caption unless `icon_end` moves it after (dropdown chevrons).
    let has_text = !n.text.text.is_empty();
    let icon_px = if n.svg.is_some() {
        s(n.control.icon_size).round()
    } else {
        0.0
    };
    let gap = if icon_px > 0.0 && has_text {
        s(BUTTON_ICON_GAP)
    } else {
        0.0
    };
    let layout = if has_text {
        let px = font_px(n, scale);
        let face = font_face(n);
        Some(f.text.layout(&n.text.text, face, px, f32::INFINITY, false))
    } else {
        None
    };
    let text_w = layout.as_ref().map(|l| l.width).unwrap_or(0.0);
    let start_x = r.x + (r.w - (icon_px + gap + text_w)) * 0.5;
    let (icon_x, text_x) = if n.control.icon_end {
        (start_x + text_w + gap, start_x)
    } else {
        (start_x, start_x + icon_px + gap)
    };
    if let Some(doc) = &n.svg {
        let size = icon_px.max(1.0) as u32;
        if let Some(pm) = f.svgs.get(doc, size, size, tc, Some(tc)) {
            let ix = icon_x.round() as i32;
            let iy = (r.y + (r.h - icon_px) * 0.5).round() as i32;
            f.p.draw_pixmap(&pm, ix, iy, 1.0);
        }
    }
    if let Some(layout) = &layout {
        let tr = Rect::new(text_x, r.y, text_w + s(2.0), r.h);
        draw_text_in(f, layout, tr, 0, tc);
    }
}

/// Horizontal scroll offset that keeps the caret visible in a text field.
pub fn input_scroll_x(layout: &TextLayout, caret_x: f32, inner_w: f32, current: f32) -> f32 {
    let max_sx = (layout.width - inner_w).max(0.0);
    let mut sx = current.clamp(0.0, max_sx);
    if caret_x - sx > inner_w {
        sx = caret_x - inner_w;
    }
    if caret_x < sx {
        sx = caret_x;
    }
    sx.max(0.0)
}

/// Map a byte offset in the real text to the byte offset in the displayed (possibly masked) text.
pub fn real_to_display(n: &Node, byte: usize) -> usize {
    if n.kind == Kind::TextInput && n.control.secure {
        let chars = n.text.text[..byte.min(n.text.text.len())].chars().count();
        chars * '\u{2022}'.len_utf8()
    } else {
        byte
    }
}

fn paint_input(f: &mut Frame, n: &Node, id: Id) {
    let r = n.rect;
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let radius = radius_of(f, n, 10.0);
    let hover = n.anim.hover.value;
    let focus = n.anim.focus.value;
    let pal = f.pal;

    if focus > 0.01 {
        let ring = s(3.0) * focus;
        f.p.fill_rrect(
            r.outset(ring),
            radius + ring,
            pal.accent.with_alpha(0.22 * focus),
        );
    }
    let bg = pal.resolve(n.visual.background, pal.surface);
    f.p.fill_rrect(r, radius, bg);
    let border = pal
        .outline_strong
        .mix(pal.text.with_alpha(0.45), hover * 0.5)
        .mix(pal.accent, focus);
    f.p.stroke_rrect(r, radius, s(1.0 + 0.5 * focus), border);

    let pad = s(INPUT_PAD_X);
    let inner = Rect::new(r.x + pad, r.y, (r.w - 2.0 * pad).max(1.0), r.h);
    let px = font_px(n, scale);
    let face = font_face(n);
    let display = n.display_text();
    let placeholder = display.is_empty();
    let content: &str = if placeholder {
        &n.text.placeholder
    } else {
        &display
    };
    let layout = f.text.layout(content, face, px, f32::INFINITY, false);
    let text_y = r.y + (r.h - layout.line_height) * 0.5;
    let caret_x = if placeholder {
        0.0
    } else {
        layout.caret(real_to_display(n, n.input.cursor)).1
    };
    let sx = if placeholder {
        0.0
    } else {
        input_scroll_x(&layout, caret_x, inner.w, n.input.scroll_x)
    };

    f.p.push_clip(Rect::new(inner.x - s(1.0), r.y, inner.w + s(2.0), r.h), 0.0);
    if !placeholder && focus > 0.01 {
        if let Some((a, b)) = n.input.selection() {
            let xa = layout.caret(real_to_display(n, a)).1;
            let xb = layout.caret(real_to_display(n, b)).1;
            let sel = Rect::new(inner.x - sx + xa, text_y, xb - xa, layout.line_height);
            f.p.fill_rrect(sel, s(2.0), pal.accent.with_alpha(0.28 * focus));
        }
    }
    let color = if placeholder {
        pal.text_muted
    } else {
        pal.resolve(n.text.color, pal.text)
    };
    let clip = f.p.clip_rect();
    let (ox, oy) = f.p.offset;
    f.text.draw(
        &mut f.p.pm,
        &layout,
        inner.x - sx + ox,
        text_y + oy,
        inner.w,
        0,
        color.mul_alpha(f.p.alpha),
        clip,
    );
    if f.focused == Some(id) && !n.disabled {
        let t = (f.now - n.input.blink_epoch).max(0.0);
        if t % 1.06 < 0.53 {
            let cx = inner.x - sx + caret_x;
            let caret = Rect::new(
                cx - s(0.5),
                text_y + s(2.0),
                s(1.5),
                layout.line_height - s(4.0),
            );
            f.p.fill_rrect(caret, s(0.75), pal.accent);
        }
    }
    f.p.pop_clip();
}

fn paint_toggle_label(f: &mut Frame, n: &Node, control_w: f32) {
    if n.text.text.is_empty() {
        return;
    }
    let scale = f.p.scale;
    let px = font_px(n, scale);
    let face = font_face(n);
    let layout = f.text.layout(&n.text.text, face, px, f32::INFINITY, false);
    let x = n.rect.x + control_w + CHECK_GAP * scale;
    let r = Rect::new(x, n.rect.y, (n.rect.right() - x).max(0.0), n.rect.h);
    let color = f.pal.resolve(n.text.color, f.pal.text);
    draw_text_in(f, &layout, r, 0, color);
}

fn paint_checkbox(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let bs = s(CHECK_SIZE);
    let bx = r.x;
    let by = r.y + (r.h - bs) * 0.5;
    let box_r = Rect::new(bx, by, bs, bs);
    let radius = radius_of(f, n, 6.0);
    // Raw for geometry (a spring may overshoot), clamped for colors.
    let t = n.anim.check.value;
    let tc = t.clamp(0.0, 1.0);
    let hover = n.anim.hover.value;
    let press = n.anim.press.value;
    let focus = n.anim.focus.value;
    let pal = f.pal;

    let (cx, cy) = box_r.center();
    let halo = (hover * 0.10 + press * 0.10).min(0.2);
    if halo > 0.005 {
        f.p.fill_circle(cx, cy, bs * 0.95, pal.accent.with_alpha(halo));
    }
    if focus > 0.01 && f.focus_visible {
        f.p.stroke_rrect_outside(box_r, radius, s(2.0), pal.accent.with_alpha(0.55 * focus));
    }
    let fill = pal.surface.mix(pal.accent, tc);
    f.p.fill_rrect(box_r, radius, fill);
    if tc < 0.999 {
        let border = pal.outline_strong.mix(pal.accent, hover * 0.6);
        f.p.stroke_rrect(
            box_r,
            radius,
            s(1.5),
            border.with_alpha(border.a * (1.0 - tc)),
        );
    }
    if t > 0.01 {
        let k = (0.6 + 0.4 * t).max(0.1);
        let pt = |px: f32, py: f32| (cx + (px - 0.5) * bs * k, cy + (py - 0.5) * bs * k);
        let pts = [pt(0.26, 0.53), pt(0.43, 0.70), pt(0.75, 0.33)];
        f.p.stroke_polyline(&pts, s(2.2), pal.on_accent.with_alpha(tc), true);
    }
    paint_toggle_label(f, n, bs);
}

fn paint_switch(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let (sw, sh) = (s(SWITCH_W), s(SWITCH_H));
    let sx = r.x;
    let sy = r.y + (r.h - sh) * 0.5;
    let track = Rect::new(sx, sy, sw, sh);
    let t = n.anim.check.value;
    let tc = t.clamp(0.0, 1.0);
    let hover = n.anim.hover.value;
    let press = n.anim.press.value;
    let focus = n.anim.focus.value;
    let pal = f.pal;

    if focus > 0.01 && f.focus_visible {
        f.p.stroke_rrect_outside(track, sh * 0.5, s(2.0), pal.accent.with_alpha(0.55 * focus));
    }
    let off = pal.text.with_alpha(0.18 + 0.06 * hover);
    let on = pal.accent.lighten(0.06 * hover);
    f.p.fill_rrect(track, sh * 0.5, off.mix(on, tc));
    let kr = s(9.0) + s(1.0) * press;
    let inset = s(3.0);
    // The knob may overshoot the track ends a little with a springy motion style.
    let travel = sw - 2.0 * inset - 2.0 * s(9.0);
    let kx = sx + inset + s(9.0) + t.clamp(-0.15, 1.15) * travel;
    let ky = sy + sh * 0.5;
    f.p.draw_shadow(
        f.shadows,
        Rect::new(kx - kr, ky - kr, 2.0 * kr, 2.0 * kr),
        kr,
        s(4.0),
        s(1.0),
        pal.shadow.with_alpha(0.25),
    );
    f.p.fill_circle(kx, ky, kr, Rgba::WHITE);
    paint_toggle_label(f, n, sw);
}

fn paint_slider(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let scale = f.p.scale;
    let s = |v: f32| v * scale;
    let th = s(THUMB_R);
    let track_h = s(4.0);
    let cy = r.y + r.h * 0.5;
    let x0 = r.x + th;
    let w = (r.w - 2.0 * th).max(1.0);
    let frac = n.anim.value.value.clamp(0.0, 1.0);
    let tx = x0 + frac * w;
    let hover = n.anim.hover.value;
    let press = n.anim.press.value;
    let focus = n.anim.focus.value;
    let pal = f.pal;

    f.p.fill_rrect(
        Rect::new(r.x, cy - track_h * 0.5, r.w, track_h),
        track_h * 0.5,
        pal.text.with_alpha(0.12),
    );
    f.p.fill_rrect(
        Rect::new(r.x, cy - track_h * 0.5, (tx - r.x).max(track_h), track_h),
        track_h * 0.5,
        pal.accent,
    );
    let rad = th * (1.0 + 0.12 * hover + 0.08 * press);
    let glow = focus.max(press);
    if glow > 0.01 {
        f.p.fill_circle(
            tx,
            cy,
            rad + s(6.0) * glow,
            pal.accent.with_alpha(0.18 * glow),
        );
    }
    f.p.draw_shadow(
        f.shadows,
        Rect::new(tx - rad, cy - rad, 2.0 * rad, 2.0 * rad),
        rad,
        s(6.0),
        s(2.0),
        pal.shadow.with_alpha(0.22),
    );
    f.p.fill_circle(tx, cy, rad, Rgba::WHITE);
    let ring = pal.outline_strong.mix(pal.accent, press.max(hover * 0.5));
    f.p.stroke_circle(tx, cy, rad - s(0.5), s(1.0), ring);
}

fn paint_progress(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let scale = f.p.scale;
    let h = r.h.min(PROGRESS_H * scale).max(2.0 * scale);
    let y = r.y + (r.h - h) * 0.5;
    let radius = h * 0.5;
    let pal = f.pal;
    let track = Rect::new(r.x, y, r.w, h);
    f.p.fill_rrect(track, radius, pal.text.with_alpha(0.10));
    let fill = pal.resolve(n.visual.background, pal.accent);
    if n.control.indeterminate {
        let period = 1.6;
        let t = ((f.clock / period) % 1.0) as f32;
        let bw = r.w * 0.35;
        let x = r.x + t * (r.w + bw) - bw;
        f.p.push_clip(track, radius);
        f.p.fill_rrect(Rect::new(x, y, bw, h), radius, fill);
        f.p.pop_clip();
    } else {
        let frac = n.anim.value.value.clamp(0.0, 1.0);
        if frac > 0.001 {
            f.p.fill_rrect(Rect::new(r.x, y, (r.w * frac).max(h), h), radius, fill);
        }
    }
}

fn paint_spinner(f: &mut Frame, n: &Node) {
    let r = n.rect;
    let size = r.w.min(r.h);
    if size <= 2.0 {
        return;
    }
    let (cx, cy) = r.center();
    let width = (size * 0.12).max(1.5);
    let radius = size * 0.5 - width * 0.75;
    let color = f.pal.resolve(n.visual.background, f.pal.accent);
    f.p.stroke_circle(cx, cy, radius, width, color.with_alpha(0.15));
    let clock = f.clock as f32;
    let start = (clock * 360.0) % 360.0;
    let sweep = 90.0 + 170.0 * (0.5 + 0.5 * (clock * TAU / 1.6).sin());
    if let Some(path) = arc_path(cx, cy, radius, start, sweep) {
        f.p.stroke_path(&path, width, color, true);
    }
}

fn paint_divider(f: &mut Frame, n: &Node) {
    let mut r = n.rect;
    if r.h < 2.0 {
        r.y = r.y.round();
        r.h = 1.0;
    }
    if r.w < 2.0 {
        r.x = r.x.round();
        r.w = 1.0;
    }
    let c = f.pal.resolve(n.visual.background, f.pal.outline);
    f.p.fill_rrect(r, 0.0, c);
}

fn paint_image(f: &mut Frame, n: &Node) {
    let radius = radius_of(f, n, 0.0);
    if let Some(doc) = &n.svg {
        paint_svg(f, n, doc, radius);
        return;
    }
    match &n.image {
        Some(img) => f.p.draw_image(&img.pixmap, n.rect, n.control.fit, radius),
        None => {
            let c = f.pal.surface_variant;
            f.p.fill_rrect(n.rect, radius, c);
        }
    }
}

/// Rasterize an SVG at exactly the size it is shown at (so it stays crisp at any scale),
/// advancing its SMIL timeline and applying spin/pulse effects.
fn paint_svg(f: &mut Frame, n: &Node, doc: &SvgDoc, radius: f32) {
    let dst = n.rect;
    if dst.is_empty() || doc.width <= 0.0 || doc.height <= 0.0 {
        return;
    }
    let pal = f.pal;
    let tint = match n.visual.tint {
        ColorRef::Default => None,
        c => Some(pal.resolve(c, pal.text)),
    };
    let current = tint.unwrap_or_else(|| pal.resolve(n.text.color, pal.text));
    let (dw, dh) = match n.control.fit {
        2 => (dst.w, dst.h),
        1 => {
            let k = (dst.w / doc.width).max(dst.h / doc.height);
            (doc.width * k, doc.height * k)
        }
        _ => {
            let k = (dst.w / doc.width).min(dst.h / doc.height);
            (doc.width * k, doc.height * k)
        }
    };
    let (w, h) = (dw.round().max(1.0) as u32, dh.round().max(1.0) as u32);
    let animated = n.control.svg_animate && doc.is_animated();
    let t = if animated {
        (f.clock - n.svg_epoch).max(0.0)
    } else {
        0.0
    };
    let spin_deg = if n.control.spin > 0.0 {
        ((f.clock / n.control.spin as f64) * 360.0 % 360.0) as f32
    } else {
        0.0
    };
    let pulse_k = if n.control.pulse > 0.0 {
        1.0 - 0.08 * (0.5 + 0.5 * (f.clock as f32 * TAU / n.control.pulse).sin())
    } else {
        1.0
    };
    let pm = if animated || spin_deg != 0.0 || pulse_k != 1.0 {
        f.svgs
            .get_animated(doc, w, h, current, tint, t, spin_deg, pulse_k)
    } else {
        f.svgs.get(doc, w, h, current, tint)
    };
    let pm = match pm {
        Some(p) => p,
        None => return,
    };
    let x = (dst.x + (dst.w - w as f32) * 0.5).round() as i32;
    let y = (dst.y + (dst.h - h as f32) * 0.5).round() as i32;
    let clip = radius > 0.0 || w as f32 > dst.w + 0.5 || h as f32 > dst.h + 0.5;
    if clip {
        f.p.push_clip(dst, radius);
    }
    f.p.draw_pixmap(&pm, x, y, 1.0);
    if clip {
        f.p.pop_clip();
    }
}

/// Render a window offscreen (no event loop needed) and save it as a PNG.
pub fn snapshot(
    win: Id,
    width: u32,
    height: u32,
    scale: f32,
    path: &std::path::Path,
) -> Result<(), String> {
    let mut s = crate::state::lock();
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let pw = ((width as f32) * scale).round().max(1.0) as u32;
    let ph = ((height as f32) * scale).round().max(1.0) as u32;
    if s.nodes
        .get(win)
        .map(|n| n.kind != Kind::Window)
        .unwrap_or(true)
    {
        return Err("snapshot: not a window".into());
    }
    {
        let State { nodes, text, .. } = &mut *s;
        if !text.ensure_loaded() {
            return Err(text.error().unwrap_or("fonts unavailable").to_string());
        }
        settle_animations(nodes, win);
        crate::layout::layout_window(nodes, text, win, pw as f32, ph as f32, scale);
        settle_animations(nodes, win);
    }
    s.theme.settle();
    let mut pm =
        tiny_skia::Pixmap::new(pw, ph).ok_or_else(|| "snapshot: invalid size".to_string())?;
    render_window(&mut s, win, pm.as_mut(), scale);
    if let Some(wd) = s.window_data_mut(win) {
        wd.layout_dirty = true;
    }
    pm.save_png(path)
        .map_err(|e| format!("snapshot: failed to write {}: {e}", path.display()))
}
