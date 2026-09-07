//! Flexbox layout via taffy. Layout runs in physical pixels; node styles are logical and are
//! multiplied by the window scale factor here.

use std::collections::HashMap;

use taffy::compute::compute_leaf_layout;
use taffy::tree::{LayoutInput, LayoutOutput};
use taffy::{
    AlignItems, AlignSelf, AvailableSpace, Dimension, Display, FlexDirection, FlexWrap,
    JustifyContent, LengthPercentage, LengthPercentageAuto, NodeId, Overflow, Point, Size, Style,
    TaffyTree,
};

use crate::geom::Rect;
use crate::node::*;
use crate::state::Arena;
use crate::text::{Face, TextSystem};

/// Widget metrics in logical pixels.
pub mod metrics {
    pub const FONT_SIZE: f32 = 14.0;
    pub const BUTTON_HEIGHT: f32 = 38.0;
    pub const BUTTON_PAD_X: f32 = 18.0;
    pub const BUTTON_MIN_WIDTH: f32 = 64.0;
    pub const BUTTON_ICON_GAP: f32 = 8.0;
    pub const INPUT_HEIGHT: f32 = 38.0;
    pub const INPUT_PAD_X: f32 = 12.0;
    pub const INPUT_MIN_WIDTH: f32 = 160.0;
    pub const CHECK_SIZE: f32 = 20.0;
    pub const CHECK_GAP: f32 = 10.0;
    pub const SWITCH_W: f32 = 44.0;
    pub const SWITCH_H: f32 = 24.0;
    pub const SLIDER_H: f32 = 24.0;
    pub const SLIDER_MIN_W: f32 = 140.0;
    pub const THUMB_R: f32 = 9.0;
    pub const PROGRESS_H: f32 = 6.0;
    pub const SCROLLBAR_W: f32 = 6.0;
    pub const SCROLLBAR_PAD: f32 = 3.0;
    pub const SPINNER_SIZE: f32 = 24.0;
    pub const MENU_ITEM_H: f32 = 32.0;
    pub const MENU_PAD_X: f32 = 12.0;
    pub const MENU_ICON_COL: f32 = 26.0;
    pub const TOOLTIP_MAX_W: f32 = 280.0;
}

use metrics::*;

pub fn default_weight(kind: Kind) -> u16 {
    match kind {
        Kind::Button => 600,
        _ => 400,
    }
}

pub fn font_px(node: &Node, scale: f32) -> f32 {
    node.text.size.filter(|s| *s > 0.0).unwrap_or(FONT_SIZE) * scale
}

pub fn font_face(node: &Node) -> Face {
    let w = if node.text.weight == 0 {
        default_weight(node.kind)
    } else {
        node.text.weight
    };
    TextSystem::face_for(w, node.text.family)
}

/// Lay out the content of window `win` into a `width` x `height` (physical px) area.
pub fn layout_window(
    nodes: &mut Arena,
    text: &mut TextSystem,
    win: Id,
    width: f32,
    height: f32,
    scale: f32,
) {
    let content = match nodes.get(win).and_then(|n| n.win()).and_then(|w| w.content) {
        Some(c) => c,
        None => return,
    };
    let mut tree: TaffyTree<Id> = TaffyTree::new();
    let mut map: HashMap<Id, NodeId> = HashMap::new();
    let child = match build(
        &mut tree,
        nodes,
        content,
        Direction::Column,
        true,
        scale,
        &mut map,
    ) {
        Some(c) => c,
        None => {
            clear_rects(nodes, content);
            layout_overlays(nodes, text, win, width, height, scale);
            return;
        }
    };
    let mut root_style = Style::default();
    root_style.display = Display::Flex;
    root_style.flex_direction = FlexDirection::Column;
    root_style.size = Size {
        width: Dimension::length(width),
        height: Dimension::length(height),
    };
    root_style.align_items = Some(AlignItems::STRETCH);
    let root = match tree.new_with_children(root_style, &[child]) {
        Ok(r) => r,
        Err(_) => return,
    };

    {
        let nodes_ro: &Arena = nodes;
        let _ = tree.compute_layout_with_measure(
            root,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
            |input: LayoutInput,
             _tid: NodeId,
             ctx: Option<&mut Id>,
             style: &Style|
             -> LayoutOutput {
                let id = ctx.map(|c| *c);
                compute_leaf_layout(
                    input,
                    style,
                    |_, _| 0.0,
                    |known, avail| match id {
                        Some(id) => measure(nodes_ro, text, id, known, avail, scale),
                        None => Size::ZERO,
                    },
                )
            },
        );
    }

    place(&tree, nodes, &map, content, 0.0, 0.0, scale);
    layout_overlays(nodes, text, win, width, height, scale);
}

/// Lay out the open menus of a window. Each is sized to its content (at least its minimum
/// width, or the anchor's width when requested) and placed below its anchor widget or at
/// its point, flipped upwards when there is no room below and kept inside the window.
pub fn layout_overlays(
    nodes: &mut Arena,
    text: &mut TextSystem,
    win: Id,
    width: f32,
    height: f32,
    scale: f32,
) {
    let overlays: Vec<Id> = match nodes.get(win).and_then(|n| n.win()) {
        Some(w) => w.overlays.clone(),
        None => return,
    };
    for menu in overlays {
        let (anchor, pos, match_anchor) = match nodes.get(menu) {
            Some(n) => (n.anchor, n.anchor_pos, n.match_anchor),
            None => continue,
        };
        let anchor_rect = anchor.and_then(|a| nodes.get(a)).map(|n| n.rect);
        let margin = 6.0 * scale;
        let avail_w = (width - 2.0 * margin).max(1.0);
        let avail_h = (height - 2.0 * margin).max(1.0);
        let min_w = match (match_anchor, anchor_rect) {
            (true, Some(r)) => r.w,
            _ => 0.0,
        };
        let Some((tree, map, root)) = compute(nodes, text, menu, avail_w, avail_h, min_w, scale)
        else {
            clear_rects(nodes, menu);
            continue;
        };
        let (mw, mh) = map
            .get(&menu)
            .and_then(|t| tree.layout(*t).ok())
            .map(|l| (l.size.width, l.size.height))
            .unwrap_or((0.0, 0.0));
        let gap = 4.0 * scale;
        let (mut x, mut y) = match anchor_rect {
            Some(r) => (r.x, r.bottom() + gap),
            None => (pos.0 * scale, pos.1 * scale),
        };
        if y + mh > height - margin {
            y = match anchor_rect {
                Some(r) => r.y - mh - gap,
                None => height - margin - mh,
            };
        }
        x = x.min(width - margin - mw).max(margin);
        y = y.max(margin);
        let _ = root;
        place(&tree, nodes, &map, menu, x.round(), y.round(), scale);
    }
}

/// Build and solve the taffy tree for `root_id` inside an `avail_w` x `avail_h` area. The
/// root is sized to its content (at least `min_w` wide) and positioned at the origin.
fn compute(
    nodes: &Arena,
    text: &mut TextSystem,
    root_id: Id,
    avail_w: f32,
    avail_h: f32,
    min_w: f32,
    scale: f32,
) -> Option<(TaffyTree<Id>, HashMap<Id, NodeId>, NodeId)> {
    let mut tree: TaffyTree<Id> = TaffyTree::new();
    let mut map: HashMap<Id, NodeId> = HashMap::new();
    let child = build(
        &mut tree,
        nodes,
        root_id,
        Direction::Column,
        false,
        scale,
        &mut map,
    )?;
    if min_w > 0.0 {
        if let Ok(mut style) = tree.style(child).cloned() {
            style.min_size.width = LengthPercentageAuto::length(min_w);
            let _ = tree.set_style(child, style);
        }
    }
    let mut root_style = Style::default();
    root_style.display = Display::Flex;
    root_style.flex_direction = FlexDirection::Column;
    root_style.max_size = Size {
        width: LengthPercentageAuto::length(avail_w),
        height: LengthPercentageAuto::length(avail_h),
    };
    root_style.align_items = Some(AlignItems::START);
    let root = tree.new_with_children(root_style, &[child]).ok()?;
    let nodes_ro: &Arena = nodes;
    tree.compute_layout_with_measure(
        root,
        Size {
            width: AvailableSpace::Definite(avail_w),
            height: AvailableSpace::Definite(avail_h),
        },
        |input: LayoutInput, _tid: NodeId, ctx: Option<&mut Id>, style: &Style| -> LayoutOutput {
            let id = ctx.map(|c| *c);
            compute_leaf_layout(
                input,
                style,
                |_, _| 0.0,
                |known, avail| match id {
                    Some(id) => measure(nodes_ro, text, id, known, avail, scale),
                    None => Size::ZERO,
                },
            )
        },
    )
    .ok()?;
    Some((tree, map, root))
}

fn clear_rects(nodes: &mut Arena, id: Id) {
    let mut stack = vec![id];
    while let Some(n) = stack.pop() {
        if let Some(node) = nodes.get_mut(n) {
            node.rect = Rect::default();
            stack.extend(node.children.iter().copied());
        }
    }
}

fn build(
    tree: &mut TaffyTree<Id>,
    nodes: &Arena,
    id: Id,
    parent_dir: Direction,
    root_content: bool,
    scale: f32,
    map: &mut HashMap<Id, NodeId>,
) -> Option<NodeId> {
    let n = nodes.get(id)?;
    if !n.visible {
        return None;
    }
    let style = to_style(n, parent_dir, root_content, scale);
    let tid = if n.kind.is_container() {
        let mut kids = Vec::with_capacity(n.children.len());
        for c in &n.children {
            if let Some(t) = build(tree, nodes, *c, n.layout.direction, false, scale, map) {
                kids.push(t);
            }
        }
        tree.new_with_children(style, &kids).ok()?
    } else {
        tree.new_leaf_with_context(style, id).ok()?
    };
    map.insert(id, tid);
    Some(tid)
}

fn dim(l: Length, scale: f32) -> Dimension {
    match l {
        Length::Auto => Dimension::auto(),
        Length::Px(v) => Dimension::length(v * scale),
        Length::Percent(p) => Dimension::percent(p),
    }
}

fn lpa(l: Length, scale: f32) -> LengthPercentageAuto {
    match l {
        Length::Auto => LengthPercentageAuto::auto(),
        Length::Px(v) => LengthPercentageAuto::length(v * scale),
        Length::Percent(p) => LengthPercentageAuto::percent(p),
    }
}

/// Default flex-grow: the root container and scroll areas fill their parent; inputs, sliders and
/// progress bars fill a Row (their main axis is horizontal) but never stretch vertically in a
/// Column. An explicit main-axis size disables growing.
fn default_grow(n: &Node, parent_dir: Direction, root_content: bool) -> f32 {
    let main_size = match parent_dir {
        Direction::Row => n.layout.width,
        Direction::Column => n.layout.height,
    };
    if main_size != Length::Auto {
        return 0.0;
    }
    if root_content && n.kind.is_container() {
        return 1.0;
    }
    match n.kind {
        Kind::Scroll | Kind::Spacer => 1.0,
        Kind::Slider | Kind::Progress | Kind::TextInput if parent_dir == Direction::Row => 1.0,
        _ => 0.0,
    }
}

fn align_items(v: i64) -> AlignItems {
    match v {
        1 => AlignItems::CENTER,
        2 => AlignItems::END,
        3 => AlignItems::STRETCH,
        _ => AlignItems::START,
    }
}

fn justify(v: i64) -> JustifyContent {
    match v {
        1 => JustifyContent::CENTER,
        2 => JustifyContent::END,
        3 => JustifyContent::SPACE_BETWEEN,
        4 => JustifyContent::SPACE_AROUND,
        5 => JustifyContent::SPACE_EVENLY,
        _ => JustifyContent::START,
    }
}

fn align_self(v: i64) -> Option<AlignSelf> {
    match v {
        1 => Some(AlignSelf::START),
        2 => Some(AlignSelf::CENTER),
        3 => Some(AlignSelf::END),
        4 => Some(AlignSelf::STRETCH),
        _ => None,
    }
}

fn to_style(n: &Node, parent_dir: Direction, root_content: bool, scale: f32) -> Style {
    let ls = &n.layout;
    let mut s = Style::default();
    s.display = Display::Flex;
    s.flex_direction = match ls.direction {
        Direction::Column => FlexDirection::Column,
        Direction::Row => FlexDirection::Row,
    };
    let g = LengthPercentage::length(ls.gap * scale);
    s.gap = Size {
        width: g,
        height: g,
    };
    s.padding = taffy::Rect {
        left: LengthPercentage::length(ls.padding[0] * scale),
        top: LengthPercentage::length(ls.padding[1] * scale),
        right: LengthPercentage::length(ls.padding[2] * scale),
        bottom: LengthPercentage::length(ls.padding[3] * scale),
    };
    s.margin = taffy::Rect {
        left: LengthPercentageAuto::length(ls.margin[0] * scale),
        top: LengthPercentageAuto::length(ls.margin[1] * scale),
        right: LengthPercentageAuto::length(ls.margin[2] * scale),
        bottom: LengthPercentageAuto::length(ls.margin[3] * scale),
    };
    s.size = Size {
        width: dim(ls.width, scale),
        height: dim(ls.height, scale),
    };
    s.min_size = Size {
        width: lpa(ls.min_width, scale),
        height: lpa(ls.min_height, scale),
    };
    s.max_size = Size {
        width: lpa(ls.max_width, scale),
        height: lpa(ls.max_height, scale),
    };
    s.flex_grow = ls
        .grow
        .unwrap_or_else(|| default_grow(n, parent_dir, root_content));
    s.flex_shrink = ls.shrink.unwrap_or(1.0);
    s.align_items = ls.align_items.map(align_items);
    s.justify_content = ls.justify.map(justify);
    s.align_self = ls.align_self.and_then(align_self);
    s.flex_wrap = if ls.wrap {
        FlexWrap::Wrap
    } else {
        FlexWrap::NoWrap
    };

    match n.kind {
        Kind::Scroll => {
            s.overflow = Point {
                x: Overflow::Hidden,
                y: Overflow::Scroll,
            };
            s.scrollbar_width = 0.0;
            if ls.min_height == Length::Auto {
                s.min_size.height = LengthPercentageAuto::length(0.0);
            }
        }
        Kind::Divider => {
            let vertical = n.control.orientation == 1
                || (n.control.orientation == 0 && parent_dir == Direction::Row);
            if vertical {
                if ls.width == Length::Auto {
                    s.size.width = Dimension::length(scale.max(1.0));
                }
                if ls.align_self.is_none() {
                    s.align_self = Some(AlignSelf::STRETCH);
                }
            } else if ls.height == Length::Auto {
                s.size.height = Dimension::length(scale.max(1.0));
            }
        }
        Kind::Spacer => {
            s.flex_basis = Dimension::length(0.0);
        }
        Kind::Slider | Kind::Progress if ls.min_width == Length::Auto => {
            s.min_size.width = LengthPercentageAuto::length(SLIDER_MIN_W * scale);
        }
        Kind::TextInput if ls.min_width == Length::Auto => {
            s.min_size.width = LengthPercentageAuto::length(INPUT_MIN_WIDTH * scale);
        }
        _ => {}
    }
    s
}

fn measure(
    nodes: &Arena,
    text: &mut TextSystem,
    id: Id,
    known: Size<Option<f32>>,
    avail: Size<AvailableSpace>,
    scale: f32,
) -> Size<f32> {
    let n = match nodes.get(id) {
        Some(n) => n,
        None => return Size::ZERO,
    };
    let s = |v: f32| v * scale;
    let px = font_px(n, scale);
    let face = font_face(n);
    let ready = text.ready();
    let avail_w = || -> f32 {
        match known.width {
            Some(w) => w,
            None => match avail.width {
                AvailableSpace::Definite(w) => w,
                AvailableSpace::MinContent => 0.0,
                AvailableSpace::MaxContent => f32::INFINITY,
            },
        }
    };
    let line_h = |text: &mut TextSystem| {
        if ready {
            text.line_metrics(face, px).line_height
        } else {
            px * 1.3
        }
    };
    let text_w = |text: &mut TextSystem, t: &str| {
        if ready && !t.is_empty() {
            text.measure(t, face, px, f32::INFINITY, false).0
        } else {
            0.0
        }
    };

    let (w, h) = match n.kind {
        Kind::Label => {
            if !ready {
                (0.0, px * 1.3)
            } else if n.text.text.is_empty() {
                (0.0, line_h(text))
            } else {
                let mw = if n.text.wrap {
                    avail_w()
                } else {
                    f32::INFINITY
                };
                let (tw, th) = text.measure(&n.text.text, face, px, mw, n.text.wrap);
                (tw.ceil(), th.ceil())
            }
        }
        Kind::Button => {
            let tw = text_w(text, &n.text.text);
            let lh = line_h(text);
            let h = s(BUTTON_HEIGHT).max(lh + s(12.0)).ceil();
            let icon = if n.svg.is_some() {
                s(n.control.icon_size)
            } else {
                0.0
            };
            let w = if n.text.text.is_empty() {
                // Icon-only buttons are square.
                if icon > 0.0 {
                    (icon + s(20.0)).max(h)
                } else {
                    s(BUTTON_MIN_WIDTH)
                }
            } else {
                let gap = if icon > 0.0 { s(BUTTON_ICON_GAP) } else { 0.0 };
                (tw + icon + gap + s(BUTTON_PAD_X * 2.0)).max(s(BUTTON_MIN_WIDTH))
            };
            (w.ceil(), h)
        }
        Kind::TextInput => {
            let lh = line_h(text);
            (s(INPUT_MIN_WIDTH), s(INPUT_HEIGHT).max(lh + s(14.0)).ceil())
        }
        Kind::Checkbox | Kind::Switch => {
            let (bw, bh) = if n.kind == Kind::Checkbox {
                (s(CHECK_SIZE), s(CHECK_SIZE))
            } else {
                (s(SWITCH_W), s(SWITCH_H))
            };
            let tw = text_w(text, &n.text.text);
            let lh = line_h(text);
            let w = if n.text.text.is_empty() {
                bw
            } else {
                bw + s(CHECK_GAP) + tw
            };
            (w.ceil(), bh.max(lh).ceil())
        }
        Kind::Slider => (s(SLIDER_MIN_W), s(SLIDER_H)),
        Kind::Spinner => (s(SPINNER_SIZE), s(SPINNER_SIZE)),
        Kind::MenuItem => {
            let tw = text_w(text, &n.text.text);
            let sw = if n.shortcut.is_empty() {
                0.0
            } else {
                s(24.0) + text_w(text, &n.shortcut)
            };
            let lh = line_h(text);
            let w = s(MENU_PAD_X) * 2.0 + s(MENU_ICON_COL) + tw + sw;
            (w.ceil(), s(MENU_ITEM_H).max(lh + s(10.0)).ceil())
        }
        Kind::Progress => (s(SLIDER_MIN_W), s(PROGRESS_H)),
        Kind::Divider => (s(1.0), s(1.0)),
        Kind::Spacer => (0.0, 0.0),
        Kind::Image => {
            let intrinsic = match (&n.svg, &n.image) {
                (Some(doc), _) => Some((doc.width * scale, doc.height * scale)),
                (None, Some(img)) => Some((
                    img.pixmap.width() as f32 * scale,
                    img.pixmap.height() as f32 * scale,
                )),
                (None, None) => None,
            };
            match intrinsic {
                Some((iw, ih)) => match (known.width, known.height) {
                    (Some(kw), None) if iw > 0.0 => (kw, kw * ih / iw),
                    (None, Some(kh)) if ih > 0.0 => (kh * iw / ih, kh),
                    _ => (iw, ih),
                },
                None => (s(64.0), s(64.0)),
            }
        }
        _ => (0.0, 0.0),
    };
    Size {
        width: known.width.unwrap_or(w),
        height: known.height.unwrap_or(h),
    }
}

fn place(
    tree: &TaffyTree<Id>,
    nodes: &mut Arena,
    map: &HashMap<Id, NodeId>,
    id: Id,
    ox: f32,
    oy: f32,
    scale: f32,
) {
    let tid = match map.get(&id) {
        Some(t) => *t,
        None => {
            clear_rects(nodes, id);
            return;
        }
    };
    let l = match tree.layout(tid) {
        Ok(l) => *l,
        Err(_) => return,
    };
    let rect = Rect::new(
        ox + l.location.x,
        oy + l.location.y,
        l.size.width,
        l.size.height,
    );
    let (children, kind, pad_bottom) = match nodes.get_mut(id) {
        Some(n) => {
            n.rect = rect;
            (n.children.clone(), n.kind, n.layout.padding[3] * scale)
        }
        None => return,
    };
    if children.is_empty() {
        return;
    }
    let mut child_oy = rect.y;
    if kind == Kind::Scroll {
        // Content height from the children's unscrolled layout boxes.
        let mut content_h: f32 = 0.0;
        for c in &children {
            if let Some(ct) = map.get(c) {
                if let Ok(cl) = tree.layout(*ct) {
                    content_h = content_h.max(cl.location.y + cl.size.height);
                }
            }
        }
        content_h += pad_bottom;
        if let Some(n) = nodes.get_mut(id) {
            n.scroll.content_h = content_h;
            let max_off = (content_h - rect.h).max(0.0);
            n.scroll.offset = n.scroll.offset.clamp(0.0, max_off);
            child_oy -= n.scroll.offset;
        }
    }
    for c in children {
        place(tree, nodes, map, c, rect.x, child_oy, scale);
    }
}
