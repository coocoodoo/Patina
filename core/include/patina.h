/*
 * Patina core C ABI.
 *
 * Rules that keep this ABI simple to bind from any language:
 *   - Every argument is an integer or a pointer. Floating point values are passed as the
 *     IEEE-754 bit pattern of a double in a uint64_t (see patina_set_f64 / patina_get_f64).
 *   - Strings are (pointer, length) pairs of UTF-8 bytes and are copied by the callee.
 *   - Handles are uint64_t; 0 is never a valid handle.
 *   - Thread safety: every function may be called from any thread. Events are delivered on
 *     the thread that called patina_run, never while an API call is in progress, and event
 *     handlers may call any API function.
 *   - Only 64-bit targets are supported.
 */
#ifndef PATINA_H
#define PATINA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define PATINA_ABI_VERSION 1

/* ---- node kinds ---------------------------------------------------------- */
#define PATINA_KIND_BOX 1
#define PATINA_KIND_LABEL 2
#define PATINA_KIND_BUTTON 3
#define PATINA_KIND_TEXT_INPUT 4
#define PATINA_KIND_CHECKBOX 5
#define PATINA_KIND_SWITCH 6
#define PATINA_KIND_SLIDER 7
#define PATINA_KIND_PROGRESS 8
#define PATINA_KIND_DIVIDER 9
#define PATINA_KIND_SPACER 10
#define PATINA_KIND_IMAGE 11
#define PATINA_KIND_SCROLL 12
#define PATINA_KIND_SPINNER 13
#define PATINA_KIND_MENU 14
#define PATINA_KIND_MENU_ITEM 15
#define PATINA_KIND_WINDOW 100

/* ---- properties ------------------------------------------------------------
 * i64 = patina_set_i64 / patina_get_i64, f64 = patina_set_f64 / patina_get_f64,
 * str = patina_set_str / patina_get_str, color = i64 (see color encoding below).
 * Lengths (WIDTH..MAX_HEIGHT) use an f64 encoding: NaN = auto, >= 0 = px, < 0 = percent
 * (so -50.0 means 50%).
 */
#define PATINA_PROP_VISIBLE 1          /* i64 bool */
#define PATINA_PROP_DISABLED 2         /* i64 bool */
#define PATINA_PROP_OPACITY 3          /* f64 0..1 */
#define PATINA_PROP_BACKGROUND 10      /* color */
#define PATINA_PROP_RADIUS 11          /* f64 */
#define PATINA_PROP_BORDER_WIDTH 12    /* f64 */
#define PATINA_PROP_BORDER_COLOR 13    /* color */
#define PATINA_PROP_SHADOW 14          /* i64 0..3 */
#define PATINA_PROP_TINT 15            /* color: recolor an SVG image using it as a mask */
#define PATINA_PROP_GRADIENT_END 16    /* color: right-hand color of a horizontal gradient fill */
#define PATINA_PROP_GLOW 17            /* i64 bool: pulsing glow around a button */
#define PATINA_PROP_COLOR_CYCLE 18     /* f64 seconds per hue rotation (0 = off) */
#define PATINA_PROP_RIPPLE 19          /* i64 bool: click ripple */
#define PATINA_PROP_DIRECTION 20       /* i64 0 column, 1 row */
#define PATINA_PROP_GAP 21             /* f64 */
#define PATINA_PROP_PADDING_LEFT 22    /* f64 */
#define PATINA_PROP_PADDING_TOP 23
#define PATINA_PROP_PADDING_RIGHT 24
#define PATINA_PROP_PADDING_BOTTOM 25
#define PATINA_PROP_MARGIN_LEFT 26
#define PATINA_PROP_MARGIN_TOP 27
#define PATINA_PROP_MARGIN_RIGHT 28
#define PATINA_PROP_MARGIN_BOTTOM 29
#define PATINA_PROP_WIDTH 30           /* f64 length */
#define PATINA_PROP_HEIGHT 31
#define PATINA_PROP_MIN_WIDTH 32
#define PATINA_PROP_MIN_HEIGHT 33
#define PATINA_PROP_MAX_WIDTH 34
#define PATINA_PROP_MAX_HEIGHT 35
#define PATINA_PROP_GROW 36            /* f64 (NaN = kind default) */
#define PATINA_PROP_SHRINK 37          /* f64 (NaN = default) */
#define PATINA_PROP_ALIGN_ITEMS 38     /* i64 0 start 1 center 2 end 3 stretch, -1 default */
#define PATINA_PROP_JUSTIFY 39         /* i64 0 start 1 center 2 end 3 between 4 around 5 evenly */
#define PATINA_PROP_ALIGN_SELF 40      /* i64 0 auto 1 start 2 center 3 end 4 stretch */
#define PATINA_PROP_WRAP 41            /* i64 bool (flex wrap) */
#define PATINA_PROP_ENTER 42           /* i64: 0 none 1 fade 2 up 3 down 4 left 5 right 6 pop */
#define PATINA_PROP_HOVER_LIFT 43      /* i64 bool: lift with a deeper shadow on hover */
#define PATINA_PROP_CLICKABLE 44       /* i64 bool: a box reacts to hover and emits CLICK */
#define PATINA_PROP_PRESS_EFFECT 45    /* i64: 0 none 1 rubber band 2 gelatin 3 bounce */
#define PATINA_PROP_SPRING_STIFFNESS 46 /* f64: per-widget spring (0 = global motion style) */
#define PATINA_PROP_SPRING_DAMPING 47  /* f64 */
#define PATINA_PROP_CONTEXT_MENU 48    /* i64: menu node opened by a right click (0 = none) */
#define PATINA_PROP_MATCH_ANCHOR 49    /* i64 bool: a menu is at least as wide as its anchor */
#define PATINA_PROP_TEXT 50            /* str */
#define PATINA_PROP_PLACEHOLDER 51     /* str */
#define PATINA_PROP_FONT_SIZE 52       /* f64 (0 = default) */
#define PATINA_PROP_FONT_WEIGHT 53     /* i64 100..900 (0 = default) */
#define PATINA_PROP_TEXT_COLOR 54      /* color */
#define PATINA_PROP_TEXT_ALIGN 55      /* i64 0 left 1 center 2 right */
#define PATINA_PROP_TEXT_WRAP 56       /* i64 bool */
#define PATINA_PROP_FONT_FAMILY 57     /* i64 0 sans, 1 mono */
#define PATINA_PROP_VARIANT 60         /* i64 button variant */
#define PATINA_PROP_CHECKED 61         /* i64 bool */
#define PATINA_PROP_VALUE 62           /* f64 */
#define PATINA_PROP_MIN 63             /* f64 */
#define PATINA_PROP_MAX 64             /* f64 */
#define PATINA_PROP_STEP 65            /* f64 (0 = continuous) */
#define PATINA_PROP_SECURE 66          /* i64 bool */
#define PATINA_PROP_INDETERMINATE 67   /* i64 bool */
#define PATINA_PROP_ORIENTATION 68     /* i64 0 auto/horizontal 1 vertical */
#define PATINA_PROP_FIT 69             /* i64 0 contain 1 cover 2 fill */
#define PATINA_PROP_SCROLL_Y 70        /* f64 logical px */
#define PATINA_PROP_FOCUS 71           /* i64: set 1 to focus; get = is focused */
#define PATINA_PROP_MAX_LENGTH 72      /* i64 (0 = unlimited) */
#define PATINA_PROP_ICON_SIZE 73       /* f64 logical px: button icon size (default 18) */
#define PATINA_PROP_SPIN 74            /* f64 seconds per rotation of an image (0 = off) */
#define PATINA_PROP_PULSE 75           /* f64 seconds per pulse cycle of an image (0 = off) */
#define PATINA_PROP_ANIMATE 76         /* i64 bool: play SMIL animations of an SVG (default 1) */
#define PATINA_PROP_ICON_END 77        /* i64 bool: draw a button icon after the caption */
#define PATINA_PROP_TOOLTIP 78         /* str: hover text */
#define PATINA_PROP_SHORTCUT 79        /* str: shortcut hint of a menu item */
#define PATINA_PROP_TITLE 80           /* str (window) */
#define PATINA_PROP_WINDOW_WIDTH 81    /* f64 logical (window) */
#define PATINA_PROP_WINDOW_HEIGHT 82
#define PATINA_PROP_WINDOW_MIN_WIDTH 83
#define PATINA_PROP_WINDOW_MIN_HEIGHT 84
#define PATINA_PROP_RESIZABLE 85       /* i64 bool */
#define PATINA_PROP_DECORATIONS 86     /* i64 bool */
#define PATINA_PROP_ALWAYS_ON_TOP 87   /* i64 bool */
#define PATINA_PROP_MAXIMIZED 88       /* i64 bool */
#define PATINA_PROP_SCALE 89           /* f64, get only */
#define PATINA_PROP_INTERCEPT_CLOSE 90 /* i64 bool: only emit WINDOW_CLOSE, do not close */

/* ---- button variants ------------------------------------------------------ */
#define PATINA_VARIANT_PRIMARY 0
#define PATINA_VARIANT_TONAL 1
#define PATINA_VARIANT_OUTLINED 2
#define PATINA_VARIANT_GHOST 3
#define PATINA_VARIANT_DANGER 4

/* ---- events --------------------------------------------------------------- */
#define PATINA_EV_CLICK 1          /* node */
#define PATINA_EV_TEXT_CHANGED 2   /* node, text */
#define PATINA_EV_TOGGLED 3        /* node, a = checked */
#define PATINA_EV_VALUE_CHANGED 4  /* node, a = double bits */
#define PATINA_EV_SUBMIT 5         /* node, text */
#define PATINA_EV_FOCUS 6          /* node */
#define PATINA_EV_BLUR 7           /* node */
#define PATINA_EV_HOVER_ENTER 8    /* node */
#define PATINA_EV_HOVER_LEAVE 9    /* node */
#define PATINA_EV_MENU_CLOSED 10   /* node = menu */
#define PATINA_EV_WINDOW_CLOSE 20  /* window */
#define PATINA_EV_WINDOW_RESIZED 21 /* window, a = width, b = height (logical) */
#define PATINA_EV_WINDOW_FOCUS 22  /* window, a = focused */
#define PATINA_EV_TIMER 30         /* node = 0, a = token */
#define PATINA_EV_POSTED 31        /* node = 0, a = token */
#define PATINA_EV_STARTED 32       /* node = 0: the event loop is running */

/* ---- colors ------------------------------------------------------------------
 * A color is an int64: values >= 0 are literal 0xRRGGBBAA; negative values are theme tokens.
 */
#define PATINA_COLOR_DEFAULT (-1)
#define PATINA_COLOR_BACKGROUND (-2)
#define PATINA_COLOR_SURFACE (-3)
#define PATINA_COLOR_SURFACE_VARIANT (-4)
#define PATINA_COLOR_TEXT (-5)
#define PATINA_COLOR_TEXT_SECONDARY (-6)
#define PATINA_COLOR_TEXT_MUTED (-7)
#define PATINA_COLOR_ACCENT (-8)
#define PATINA_COLOR_ON_ACCENT (-9)
#define PATINA_COLOR_OUTLINE (-10)
#define PATINA_COLOR_DANGER (-11)
#define PATINA_COLOR_SUCCESS (-12)
#define PATINA_COLOR_WARNING (-13)

/*
 * Event callback. `text` is only valid for the duration of the call.
 */
typedef void (*patina_event_fn)(uintptr_t user, uint64_t node, uint32_t event, int64_t a, int64_t b,
                                const uint8_t* text, size_t text_len);

/* ---- lifecycle ------------------------------------------------------------ */
uint32_t patina_abi_version(void);
void patina_set_event_handler(patina_event_fn fn, uintptr_t user);
/* Runs the event loop on the calling thread (must be the main thread on macOS). Returns 0 on
 * a normal exit, 1 on error (see patina_last_error). */
int32_t patina_run(void);
void patina_quit(void);
int32_t patina_is_running(void);
size_t patina_last_error(uint8_t* buf, size_t cap);

/* ---- tree ----------------------------------------------------------------- */
uint64_t patina_node_new(uint32_t kind);
uint64_t patina_window_new(void);
void patina_node_free(uint64_t node); /* frees the whole subtree */
int32_t patina_node_append(uint64_t parent, uint64_t child);
int32_t patina_node_insert(uint64_t parent, uint64_t child, uint64_t index);
void patina_node_remove(uint64_t parent, uint64_t child); /* detach only */
void patina_node_clear(uint64_t parent);                  /* detach and free children */
uint64_t patina_node_child_count(uint64_t node);
uint32_t patina_node_kind(uint64_t node);
uint64_t patina_node_parent(uint64_t node);
int32_t patina_window_set_content(uint64_t window, uint64_t node);
void patina_window_close(uint64_t window);
/* Offscreen render to PNG; works without a running event loop. scale_bits = double bits. */
int32_t patina_window_snapshot(uint64_t window, uint32_t width, uint32_t height, uint64_t scale_bits,
                               const uint8_t* path, size_t path_len);

/* ---- properties ----------------------------------------------------------- */
void patina_set_i64(uint64_t node, uint32_t prop, int64_t value);
void patina_set_f64(uint64_t node, uint32_t prop, uint64_t value_bits);
void patina_set_str(uint64_t node, uint32_t prop, const uint8_t* s, size_t len);
int64_t patina_get_i64(uint64_t node, uint32_t prop);
uint64_t patina_get_f64(uint64_t node, uint32_t prop);
size_t patina_get_str(uint64_t node, uint32_t prop, uint8_t* buf, size_t cap);
int32_t patina_set_image(uint64_t node, uint32_t width, uint32_t height, const uint8_t* rgba, size_t len);

/* ---- vector graphics ------------------------------------------------------ */
/* Sets SVG markup as the content of an image node or the icon of a button. An empty string
 * clears it. Returns 1 on success, 0 on a parse error (see patina_last_error). */
int32_t patina_set_svg(uint64_t node, const uint8_t* svg, size_t len);
/* Rasterizes SVG markup to straight-alpha RGBA8 at width x height. `tint` uses the color
 * encoding above: PATINA_COLOR_DEFAULT keeps the document's colors (currentColor = theme text),
 * anything else recolors the result using the document as a mask. Returns the number of bytes
 * needed (width * height * 4), writing them only when `cap` is large enough; 0 on error. */
size_t patina_svg_render(const uint8_t* svg, size_t len, uint32_t width, uint32_t height, int64_t tint,
                         uint8_t* out, size_t cap);
/* Sets the window and taskbar icon from straight-alpha RGBA8 pixels. */
int32_t patina_window_set_icon(uint64_t window, uint32_t width, uint32_t height, const uint8_t* rgba,
                               size_t len);

/* ---- theme ---------------------------------------------------------------- */
void patina_theme_set_mode(uint32_t mode); /* 0 light, 1 dark, 2 system */
int32_t patina_theme_is_dark(void);
void patina_theme_set_accent(uint32_t rgba); /* 0 = default */
/* Activates a built-in or custom palette by name (mode follows the palette; cross-fades). */
int32_t patina_theme_use(const uint8_t* name, size_t len);
/* Name of the active palette; same buffer protocol as patina_get_str. */
size_t patina_theme_current(uint8_t* buf, size_t cap);
/* Every palette as newline-separated "name	dark	accent	on_accent" records (hex RRGGBBAA). */
size_t patina_theme_list(uint8_t* buf, size_t cap);
#define PATINA_PALETTE_COLORS 14
/* Registers a custom palette: background, surface, surface_variant, text, text_secondary,
 * text_muted, accent, on_accent, outline, outline_strong, danger, success, warning, shadow. */
int32_t patina_theme_define(const uint8_t* name, size_t len, int32_t dark, const uint32_t* colors, size_t count);
/* Seconds a palette or mode change cross-fades over (double bits; default 0.28). */
void patina_theme_set_transition(uint64_t secs_bits);
int32_t patina_theme_set_font(const uint8_t* regular, size_t regular_len, const uint8_t* bold, size_t bold_len,
                              const uint8_t* mono, size_t mono_len);

/* ---- timers / posting ----------------------------------------------------- */
void patina_timer_start(uint64_t token, uint64_t interval_ms, int32_t repeat);
void patina_timer_cancel(uint64_t token);
void patina_post(uint64_t token);

/* ---- animation ------------------------------------------------------------ */
/* Global speed factor as double bits: 1 normal, 0.5 slow motion, 0 = no motion (instant). */
void patina_set_animation_speed(uint64_t speed_bits);
uint64_t patina_get_animation_speed(void);
/* Default motion for value, toggle and entrance animations: 0 ease, 1 spring, 2 bouncy. */
void patina_set_motion(uint32_t style);
uint32_t patina_get_motion(void);

/* ---- menus ---------------------------------------------------------------- */
/* Opens a menu below `anchor` (a widget), or at the logical point x,y (double bits) when
 * `anchor` is the window itself. Items emit EV_CLICK; the menu emits EV_MENU_CLOSED. */
int32_t patina_menu_open(uint64_t menu, uint64_t anchor, uint64_t x_bits, uint64_t y_bits);
void patina_menu_close(uint64_t menu);
int32_t patina_menu_is_open(uint64_t menu);

#ifdef __cplusplus
}
#endif

#endif /* PATINA_H */
