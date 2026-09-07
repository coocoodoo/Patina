//! Constants shared with `include/patina.h` and the Go package.
//!
//! These values are part of the ABI: never renumber them, only append.

pub const ABI_VERSION: u32 = 1;

// ---- Node kinds -----------------------------------------------------------
pub const KIND_BOX: u32 = 1;
pub const KIND_LABEL: u32 = 2;
pub const KIND_BUTTON: u32 = 3;
pub const KIND_TEXT_INPUT: u32 = 4;
pub const KIND_CHECKBOX: u32 = 5;
pub const KIND_SWITCH: u32 = 6;
pub const KIND_SLIDER: u32 = 7;
pub const KIND_PROGRESS: u32 = 8;
pub const KIND_DIVIDER: u32 = 9;
pub const KIND_SPACER: u32 = 10;
pub const KIND_IMAGE: u32 = 11;
pub const KIND_SCROLL: u32 = 12;
pub const KIND_SPINNER: u32 = 13;
pub const KIND_MENU: u32 = 14;
pub const KIND_MENU_ITEM: u32 = 15;
pub const KIND_WINDOW: u32 = 100;

// ---- Properties -----------------------------------------------------------
// Generic
pub const PROP_VISIBLE: u32 = 1; // i64 bool
pub const PROP_DISABLED: u32 = 2; // i64 bool
pub const PROP_OPACITY: u32 = 3; // f64 0..1
// Visual
pub const PROP_BACKGROUND: u32 = 10; // color
pub const PROP_RADIUS: u32 = 11; // f64 px
pub const PROP_BORDER_WIDTH: u32 = 12; // f64 px
pub const PROP_BORDER_COLOR: u32 = 13; // color
pub const PROP_SHADOW: u32 = 14; // i64 0..3
pub const PROP_TINT: u32 = 15; // color: recolor an SVG image using it as a mask
pub const PROP_GRADIENT_END: u32 = 16; // color: right-hand color of a horizontal gradient fill
pub const PROP_GLOW: u32 = 17; // i64 bool: pulsing glow around a button
pub const PROP_COLOR_CYCLE: u32 = 18; // f64 seconds per hue rotation (0 = off)
pub const PROP_RIPPLE: u32 = 19; // i64 bool: click ripple
// Layout
pub const PROP_DIRECTION: u32 = 20; // i64 0 column, 1 row
pub const PROP_GAP: u32 = 21; // f64
pub const PROP_PADDING_LEFT: u32 = 22;
pub const PROP_PADDING_TOP: u32 = 23;
pub const PROP_PADDING_RIGHT: u32 = 24;
pub const PROP_PADDING_BOTTOM: u32 = 25;
pub const PROP_MARGIN_LEFT: u32 = 26;
pub const PROP_MARGIN_TOP: u32 = 27;
pub const PROP_MARGIN_RIGHT: u32 = 28;
pub const PROP_MARGIN_BOTTOM: u32 = 29;
pub const PROP_WIDTH: u32 = 30; // f64 length encoding (see `Length::from_f64`)
pub const PROP_HEIGHT: u32 = 31;
pub const PROP_MIN_WIDTH: u32 = 32;
pub const PROP_MIN_HEIGHT: u32 = 33;
pub const PROP_MAX_WIDTH: u32 = 34;
pub const PROP_MAX_HEIGHT: u32 = 35;
pub const PROP_GROW: u32 = 36; // f64
pub const PROP_SHRINK: u32 = 37; // f64
pub const PROP_ALIGN_ITEMS: u32 = 38; // i64 0 start 1 center 2 end 3 stretch
pub const PROP_JUSTIFY: u32 = 39; // i64 0 start 1 center 2 end 3 between 4 around 5 evenly
pub const PROP_ALIGN_SELF: u32 = 40; // i64 0 auto 1 start 2 center 3 end 4 stretch
pub const PROP_WRAP: u32 = 41; // i64 bool
pub const PROP_ENTER: u32 = 42; // i64 entrance animation: 0 none 1 fade 2 up 3 down 4 left 5 right 6 pop
pub const PROP_HOVER_LIFT: u32 = 43; // i64 bool: lift with a deeper shadow on hover
pub const PROP_CLICKABLE: u32 = 44; // i64 bool: a Box/Scroll reacts to hover and emits CLICK
pub const PROP_PRESS_EFFECT: u32 = 45; // i64: 0 none 1 rubber band 2 gelatin 3 bounce
pub const PROP_SPRING_STIFFNESS: u32 = 46; // f64: per-widget spring (0 = global motion style)
pub const PROP_SPRING_DAMPING: u32 = 47; // f64
pub const PROP_CONTEXT_MENU: u32 = 48; // i64 menu node opened by a right click (0 = none)
pub const PROP_MATCH_ANCHOR: u32 = 49; // i64 bool: a menu is at least as wide as its anchor
// Text
pub const PROP_TEXT: u32 = 50; // str
pub const PROP_PLACEHOLDER: u32 = 51; // str
pub const PROP_FONT_SIZE: u32 = 52; // f64 (0 = default)
pub const PROP_FONT_WEIGHT: u32 = 53; // i64 100..900 (0 = default)
pub const PROP_TEXT_COLOR: u32 = 54; // color
pub const PROP_TEXT_ALIGN: u32 = 55; // i64 0 left 1 center 2 right
pub const PROP_TEXT_WRAP: u32 = 56; // i64 bool
pub const PROP_FONT_FAMILY: u32 = 57; // i64 0 sans, 1 mono
// Controls
pub const PROP_VARIANT: u32 = 60; // i64 button variant
pub const PROP_CHECKED: u32 = 61; // i64 bool
pub const PROP_VALUE: u32 = 62; // f64
pub const PROP_MIN: u32 = 63; // f64
pub const PROP_MAX: u32 = 64; // f64
pub const PROP_STEP: u32 = 65; // f64 (0 = continuous)
pub const PROP_SECURE: u32 = 66; // i64 bool (password field)
pub const PROP_INDETERMINATE: u32 = 67; // i64 bool
pub const PROP_ORIENTATION: u32 = 68; // i64 0 auto/horizontal 1 vertical
pub const PROP_FIT: u32 = 69; // i64 0 contain 1 cover 2 fill
pub const PROP_SCROLL_Y: u32 = 70; // f64 (get/set)
pub const PROP_FOCUS: u32 = 71; // i64: set 1 to request focus; get returns focused
pub const PROP_MAX_LENGTH: u32 = 72; // i64 (0 = unlimited)
pub const PROP_ICON_SIZE: u32 = 73; // f64 logical px: button icon size (default 18)
pub const PROP_SPIN: u32 = 74; // f64 seconds per full rotation of an image (0 = off)
pub const PROP_PULSE: u32 = 75; // f64 seconds per pulse cycle of an image (0 = off)
pub const PROP_ANIMATE: u32 = 76; // i64 bool: play SMIL animations of an SVG (default 1)
pub const PROP_ICON_END: u32 = 77; // i64 bool: draw a button icon after the caption
pub const PROP_TOOLTIP: u32 = 78; // str: hover text
pub const PROP_SHORTCUT: u32 = 79; // str: shortcut hint of a menu item
// Window
pub const PROP_TITLE: u32 = 80; // str
pub const PROP_WINDOW_WIDTH: u32 = 81; // f64 logical
pub const PROP_WINDOW_HEIGHT: u32 = 82; // f64 logical
pub const PROP_WINDOW_MIN_WIDTH: u32 = 83;
pub const PROP_WINDOW_MIN_HEIGHT: u32 = 84;
pub const PROP_RESIZABLE: u32 = 85; // i64 bool
pub const PROP_DECORATIONS: u32 = 86; // i64 bool
pub const PROP_ALWAYS_ON_TOP: u32 = 87; // i64 bool
pub const PROP_MAXIMIZED: u32 = 88; // i64 bool
pub const PROP_SCALE: u32 = 89; // f64 (get only)
pub const PROP_INTERCEPT_CLOSE: u32 = 90; // i64 bool: only emit WINDOW_CLOSE, do not close

// ---- Button variants ------------------------------------------------------
pub const VARIANT_PRIMARY: i64 = 0;
pub const VARIANT_TONAL: i64 = 1;
pub const VARIANT_OUTLINED: i64 = 2;
pub const VARIANT_GHOST: i64 = 3;
pub const VARIANT_DANGER: i64 = 4;

// Press effects (PROP_PRESS_EFFECT).
pub const PRESS_RUBBER: i64 = 1;
pub const PRESS_GELATIN: i64 = 2;
pub const PRESS_BOUNCE: i64 = 3;

// ---- Events ---------------------------------------------------------------
pub const EV_CLICK: u32 = 1;
pub const EV_TEXT_CHANGED: u32 = 2; // text = new value
pub const EV_TOGGLED: u32 = 3; // a = checked
pub const EV_VALUE_CHANGED: u32 = 4; // a = f64 bits
pub const EV_SUBMIT: u32 = 5; // text = value
pub const EV_FOCUS: u32 = 6;
pub const EV_BLUR: u32 = 7;
pub const EV_HOVER_ENTER: u32 = 8;
pub const EV_HOVER_LEAVE: u32 = 9;
pub const EV_MENU_CLOSED: u32 = 10; // node = menu
pub const EV_WINDOW_CLOSE: u32 = 20;
pub const EV_WINDOW_RESIZED: u32 = 21; // a = width, b = height (logical)
pub const EV_WINDOW_FOCUS: u32 = 22; // a = focused
pub const EV_TIMER: u32 = 30; // a = token
pub const EV_POSTED: u32 = 31; // a = token
pub const EV_STARTED: u32 = 32; // event loop is running

// ---- Color tokens (negative i64 values) -----------------------------------
pub const COLOR_DEFAULT: i64 = -1;
pub const COLOR_BACKGROUND: i64 = -2;
pub const COLOR_SURFACE: i64 = -3;
pub const COLOR_SURFACE_VARIANT: i64 = -4;
pub const COLOR_TEXT: i64 = -5;
pub const COLOR_TEXT_SECONDARY: i64 = -6;
pub const COLOR_TEXT_MUTED: i64 = -7;
pub const COLOR_ACCENT: i64 = -8;
pub const COLOR_ON_ACCENT: i64 = -9;
pub const COLOR_OUTLINE: i64 = -10;
pub const COLOR_DANGER: i64 = -11;
pub const COLOR_SUCCESS: i64 = -12;
pub const COLOR_WARNING: i64 = -13;
