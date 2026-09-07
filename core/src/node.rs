//! The retained widget tree: node kinds, properties and per-node runtime state.

use std::sync::Arc;

use crate::anim::{Motion, Tween};
use crate::color::ColorRef;
use crate::geom::Rect;
use crate::props::*;
use crate::svg::SvgDoc;

/// Opaque node handle handed to the host language. Never 0.
pub type Id = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Window,
    Box,
    Label,
    Button,
    TextInput,
    Checkbox,
    Switch,
    Slider,
    Progress,
    Divider,
    Spacer,
    Image,
    Scroll,
    Spinner,
    Menu,
    MenuItem,
}

impl Kind {
    pub fn from_u32(v: u32) -> Option<Kind> {
        Some(match v {
            KIND_SPINNER => Kind::Spinner,
            KIND_MENU => Kind::Menu,
            KIND_MENU_ITEM => Kind::MenuItem,
            KIND_BOX => Kind::Box,
            KIND_LABEL => Kind::Label,
            KIND_BUTTON => Kind::Button,
            KIND_TEXT_INPUT => Kind::TextInput,
            KIND_CHECKBOX => Kind::Checkbox,
            KIND_SWITCH => Kind::Switch,
            KIND_SLIDER => Kind::Slider,
            KIND_PROGRESS => Kind::Progress,
            KIND_DIVIDER => Kind::Divider,
            KIND_SPACER => Kind::Spacer,
            KIND_IMAGE => Kind::Image,
            KIND_SCROLL => Kind::Scroll,
            KIND_WINDOW => Kind::Window,
            _ => return None,
        })
    }

    pub fn to_u32(self) -> u32 {
        match self {
            Kind::Box => KIND_BOX,
            Kind::Label => KIND_LABEL,
            Kind::Button => KIND_BUTTON,
            Kind::TextInput => KIND_TEXT_INPUT,
            Kind::Checkbox => KIND_CHECKBOX,
            Kind::Switch => KIND_SWITCH,
            Kind::Slider => KIND_SLIDER,
            Kind::Progress => KIND_PROGRESS,
            Kind::Divider => KIND_DIVIDER,
            Kind::Spacer => KIND_SPACER,
            Kind::Image => KIND_IMAGE,
            Kind::Scroll => KIND_SCROLL,
            Kind::Spinner => KIND_SPINNER,
            Kind::Menu => KIND_MENU,
            Kind::MenuItem => KIND_MENU_ITEM,
            Kind::Window => KIND_WINDOW,
        }
    }

    /// Container kinds lay out their children with flexbox.
    pub fn is_container(self) -> bool {
        matches!(self, Kind::Window | Kind::Box | Kind::Scroll | Kind::Menu)
    }

    /// Kinds that react to the pointer and can take keyboard focus.
    pub fn is_interactive(self) -> bool {
        matches!(
            self,
            Kind::Button
                | Kind::TextInput
                | Kind::Checkbox
                | Kind::Switch
                | Kind::Slider
                | Kind::MenuItem
        )
    }
}

/// A CSS-like length. Percent is stored as a fraction (0.5 == 50%).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

impl Length {
    /// ABI encoding: NaN = auto, `>= 0` = px, `< 0` = percent (-50 == 50%).
    pub fn from_f64(v: f64) -> Length {
        if v.is_nan() {
            Length::Auto
        } else if v < 0.0 {
            Length::Percent((-v as f32) / 100.0)
        } else {
            Length::Px(v as f32)
        }
    }

    pub fn to_f64(self) -> f64 {
        match self {
            Length::Auto => f64::NAN,
            Length::Px(v) => v as f64,
            Length::Percent(p) => -(p as f64) * 100.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Column,
    Row,
}

#[derive(Clone, Debug, Default)]
pub struct LayoutStyle {
    pub direction: Direction,
    pub gap: f32,
    /// left, top, right, bottom (logical px)
    pub padding: [f32; 4],
    pub margin: [f32; 4],
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub align_items: Option<i64>,
    pub justify: Option<i64>,
    pub align_self: Option<i64>,
    pub wrap: bool,
}

#[derive(Clone, Debug)]
pub struct Visual {
    pub background: ColorRef,
    pub radius: Option<f32>,
    pub border_width: Option<f32>,
    pub border_color: ColorRef,
    pub shadow: i64,
    pub opacity: f32,
    /// Recolors an SVG image using it as a mask (`Default` = keep the document's colors).
    pub tint: ColorRef,
    /// Right-hand color of a horizontal gradient fill (`Default` = solid background).
    pub gradient_end: ColorRef,
    /// Pulsing glow around a button.
    pub glow: bool,
    /// Seconds per hue rotation of a color-cycling fill (0 = off).
    pub color_cycle: f32,
    /// Click ripple.
    pub ripple: bool,
    /// Lift with a deeper shadow while hovered.
    pub hover_lift: bool,
    /// Entrance animation kind (see `PROP_ENTER`).
    pub enter: i64,
    /// Press effect (see PROP_PRESS_EFFECT): squash while pressed, wobble after a click.
    pub press_effect: i64,
}

impl Default for Visual {
    fn default() -> Self {
        Visual {
            background: ColorRef::Default,
            radius: None,
            border_width: None,
            border_color: ColorRef::Default,
            shadow: 0,
            opacity: 1.0,
            tint: ColorRef::Default,
            gradient_end: ColorRef::Default,
            glow: false,
            color_cycle: 0.0,
            ripple: false,
            hover_lift: false,
            enter: 0,
            press_effect: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub text: String,
    pub placeholder: String,
    pub size: Option<f32>,
    /// 0 = kind default, otherwise 100..900
    pub weight: u16,
    pub color: ColorRef,
    pub align: i64,
    pub wrap: bool,
    pub family: i64,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            text: String::new(),
            placeholder: String::new(),
            size: None,
            weight: 0,
            color: ColorRef::Default,
            align: 0,
            wrap: true,
            family: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Control {
    pub variant: i64,
    pub checked: bool,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub secure: bool,
    pub indeterminate: bool,
    pub orientation: i64,
    pub fit: i64,
    pub max_length: i64,
    /// Button icon size in logical px.
    pub icon_size: f32,
    /// Seconds per full rotation of an image (0 = off).
    pub spin: f32,
    /// Seconds per pulse cycle of an image (0 = off).
    pub pulse: f32,
    /// Whether an SVG's SMIL animations play.
    pub svg_animate: bool,
    /// Draw a button's icon after the caption instead of before it.
    pub icon_end: bool,
}

impl Default for Control {
    fn default() -> Self {
        Control {
            variant: VARIANT_PRIMARY,
            checked: false,
            value: 0.0,
            min: 0.0,
            max: 1.0,
            step: 0.0,
            secure: false,
            indeterminate: false,
            orientation: 0,
            fit: 0,
            max_length: 0,
            icon_size: 18.0,
            spin: 0.0,
            pulse: 0.0,
            svg_animate: true,
            icon_end: false,
        }
    }
}

/// Editing state of a text input. Indices are byte offsets on char boundaries.
#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub cursor: usize,
    pub anchor: Option<usize>,
    pub scroll_x: f32,
    /// Time (seconds) the caret blink cycle was last restarted.
    pub blink_epoch: f64,
}

impl InputState {
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.anchor?;
        if a == self.cursor {
            return None;
        }
        Some((a.min(self.cursor), a.max(self.cursor)))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScrollState {
    /// Vertical offset in physical px.
    pub offset: f32,
    /// Total content height in physical px (valid after layout).
    pub content_h: f32,
    /// Scrollbar visibility tween.
    pub bar: Tween,
    /// While dragging the scrollbar thumb: offset between pointer and thumb top.
    pub bar_grab: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Anim {
    pub hover: Tween,
    pub press: Tween,
    pub focus: Tween,
    pub check: Tween,
    /// Entrance progress (0 = just appeared, 1 = settled).
    pub enter: Tween,
    /// Displayed fraction of a progress bar or slider (eases towards the real value).
    pub value: Tween,
}

impl Default for Anim {
    fn default() -> Self {
        Anim {
            hover: Tween::default(),
            press: Tween::default(),
            focus: Tween::default(),
            check: Tween::default(),
            enter: Tween::at(1.0),
            value: Tween::default(),
        }
    }
}

/// A tooltip that is pending, shown or fading out.
#[derive(Clone, Debug)]
pub struct TooltipState {
    /// The widget the tooltip belongs to.
    pub node: Id,
    pub text: String,
    /// When the pointer settled on the widget (seconds, `State::now`).
    pub since: f64,
    pub shown: bool,
    /// Visibility (0..1).
    pub anim: Tween,
    pub closing: bool,
}

/// A click ripple in progress: press position (physical px) and its start on the animation clock.
#[derive(Clone, Copy, Debug)]
pub struct Ripple {
    pub x: f32,
    pub y: f32,
    pub start: f64,
}

pub struct ImageData {
    pub pixmap: tiny_skia::Pixmap,
}

/// Straight-alpha RGBA8 pixels for a window's title bar / taskbar icon.
pub struct WindowIcon {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Drag {
    #[default]
    None,
    Slider(Id),
    ScrollBar(Id),
    Select(Id),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorKind {
    #[default]
    Default,
    Pointer,
    Text,
}

pub struct WindowData {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub maximized: bool,
    pub visible: bool,
    pub intercept_close: bool,
    pub background: ColorRef,
    pub content: Option<Id>,
    pub icon: Option<Arc<WindowIcon>>,
    pub icon_dirty: bool,

    pub winit: Option<Arc<winit::window::Window>>,
    pub scale: f32,
    pub phys_w: u32,
    pub phys_h: u32,

    pub layout_dirty: bool,
    pub needs_redraw: bool,
    pub props_dirty: bool,
    pub close_requested: bool,
    pub closed: bool,

    pub hovered: Option<Id>,
    pub pressed: Option<Id>,
    pub focused: Option<Id>,
    pub focus_visible: bool,
    pub cursor: Option<(f32, f32)>,
    pub drag: Drag,
    pub modifiers: Modifiers,
    pub cursor_icon: CursorKind,
    pub animating: bool,
    /// Open popup menus, bottom to top.
    pub overlays: Vec<Id>,
    /// Menu item highlighted with the keyboard.
    pub menu_focus: Option<Id>,
    pub tooltip: Option<TooltipState>,
}

impl Default for WindowData {
    fn default() -> Self {
        WindowData {
            title: "Patina".to_string(),
            width: 800.0,
            height: 600.0,
            min_width: 0.0,
            min_height: 0.0,
            resizable: true,
            decorations: true,
            always_on_top: false,
            maximized: false,
            visible: true,
            intercept_close: false,
            background: ColorRef::Default,
            content: None,
            icon: None,
            icon_dirty: false,
            winit: None,
            scale: 1.0,
            phys_w: 0,
            phys_h: 0,
            layout_dirty: true,
            needs_redraw: true,
            props_dirty: false,
            close_requested: false,
            closed: false,
            hovered: None,
            pressed: None,
            focused: None,
            focus_visible: false,
            cursor: None,
            drag: Drag::None,
            modifiers: Modifiers::default(),
            cursor_icon: CursorKind::Default,
            animating: false,
            overlays: Vec::new(),
            menu_focus: None,
            tooltip: None,
        }
    }
}

pub struct Node {
    pub kind: Kind,
    pub parent: Option<Id>,
    pub children: Vec<Id>,
    pub layout: LayoutStyle,
    pub visual: Visual,
    pub text: TextStyle,
    pub control: Control,
    pub image: Option<Arc<ImageData>>,
    /// Vector content of an image node, or the icon of a button.
    pub svg: Option<Arc<SvgDoc>>,
    pub window: Option<Box<WindowData>>,
    /// Layout result in physical px, absolute within the window.
    pub rect: Rect,
    pub anim: Anim,
    pub scroll: ScrollState,
    pub input: InputState,
    pub visible: bool,
    pub disabled: bool,
    /// A Box/Scroll that reacts to hover and emits click events.
    pub clickable: bool,
    /// Whether the entrance animation has been triggered for the current appearance.
    pub entered: bool,
    pub ripple: Option<Ripple>,
    /// Animation-clock time the SVG content was set (start of its timeline).
    pub svg_epoch: f64,
    /// Animation-clock time of the last click on a widget with a press effect.
    pub press_wobble: Option<f64>,
    /// Hover text shown after the pointer rests on the widget (empty = none).
    pub tooltip: String,
    /// Shortcut hint drawn at the right of a menu item.
    pub shortcut: String,
    /// Menu opened by a right click on the widget or its descendants.
    pub context_menu: Option<Id>,
    /// Per-widget spring override for value, toggle and entrance tweens (0 = global style).
    pub spring_stiffness: f32,
    pub spring_damping: f32,
    /// A menu that is fading out.
    pub closing: bool,
    /// Widget a menu is anchored below (None = at `anchor_pos`).
    pub anchor: Option<Id>,
    /// Logical position of a menu opened at a point.
    pub anchor_pos: (f32, f32),
    /// Make a menu at least as wide as its anchor.
    pub match_anchor: bool,
}

impl Node {
    pub fn new(kind: Kind) -> Node {
        let mut n = Node {
            kind,
            parent: None,
            children: Vec::new(),
            layout: LayoutStyle::default(),
            visual: Visual::default(),
            text: TextStyle::default(),
            control: Control::default(),
            image: None,
            svg: None,
            window: None,
            rect: Rect::default(),
            anim: Anim::default(),
            scroll: ScrollState::default(),
            input: InputState::default(),
            visible: true,
            disabled: false,
            clickable: false,
            entered: false,
            ripple: None,
            svg_epoch: 0.0,
            press_wobble: None,
            tooltip: String::new(),
            shortcut: String::new(),
            context_menu: None,
            spring_stiffness: 0.0,
            spring_damping: 0.0,
            closing: false,
            anchor: None,
            anchor_pos: (0.0, 0.0),
            match_anchor: false,
        };
        match kind {
            Kind::Window => n.window = Some(Box::new(WindowData::default())),
            Kind::TextInput | Kind::Button | Kind::Checkbox | Kind::Switch => n.text.wrap = false,
            Kind::Slider => {
                n.control.min = 0.0;
                n.control.max = 1.0;
            }
            Kind::Menu => {
                n.layout.padding = [6.0; 4];
                n.layout.gap = 2.0;
                n.layout.min_width = Length::Px(180.0);
            }
            Kind::MenuItem => n.text.wrap = false,
            _ => {}
        }
        n
    }

    /// The spring configured on this widget, if any.
    pub fn motion_override(&self) -> Option<Motion> {
        if self.spring_stiffness > 0.0 {
            Some(Motion::Spring {
                stiffness: self.spring_stiffness,
                damping: self.spring_damping.max(0.0),
            })
        } else {
            None
        }
    }

    pub fn win(&self) -> Option<&WindowData> {
        self.window.as_deref()
    }

    pub fn win_mut(&mut self) -> Option<&mut WindowData> {
        self.window.as_deref_mut()
    }

    /// Display text of a text input (masked when secure).
    pub fn display_text(&self) -> String {
        if self.kind == Kind::TextInput && self.control.secure {
            self.text.text.chars().map(|_| '\u{2022}').collect()
        } else {
            self.text.text.clone()
        }
    }
}
