// Package native loads the Patina core shared library (patina-core, written in Rust) and
// exposes its C ABI to the rest of the module.
//
// It uses purego, so no C compiler or cgo is required. Every ABI function takes only
// integers and pointers; floating point values travel as the bit pattern of a float64.
package native

import (
	"errors"
	"fmt"
	"math"
	"runtime"
	"sync"
	"sync/atomic"
	"unsafe"

	"github.com/ebitengine/purego"
)

// ABIVersion must match PATINA_ABI_VERSION in core/include/patina.h.
const ABIVersion = 1

// Node kinds.
const (
	KindBox       uint32 = 1
	KindLabel     uint32 = 2
	KindButton    uint32 = 3
	KindTextInput uint32 = 4
	KindCheckbox  uint32 = 5
	KindSwitch    uint32 = 6
	KindSlider    uint32 = 7
	KindProgress  uint32 = 8
	KindDivider   uint32 = 9
	KindSpacer    uint32 = 10
	KindImage     uint32 = 11
	KindScroll    uint32 = 12
	KindSpinner   uint32 = 13
	KindMenu      uint32 = 14
	KindMenuItem  uint32 = 15
	KindWindow    uint32 = 100
)

// Properties.
const (
	PropVisible         uint32 = 1
	PropDisabled        uint32 = 2
	PropOpacity         uint32 = 3
	PropBackground      uint32 = 10
	PropRadius          uint32 = 11
	PropBorderWidth     uint32 = 12
	PropBorderColor     uint32 = 13
	PropShadow          uint32 = 14
	PropTint            uint32 = 15
	PropGradientEnd     uint32 = 16
	PropGlow            uint32 = 17
	PropColorCycle      uint32 = 18
	PropRipple          uint32 = 19
	PropDirection       uint32 = 20
	PropGap             uint32 = 21
	PropPaddingLeft     uint32 = 22
	PropPaddingTop      uint32 = 23
	PropPaddingRight    uint32 = 24
	PropPaddingBottom   uint32 = 25
	PropMarginLeft      uint32 = 26
	PropMarginTop       uint32 = 27
	PropMarginRight     uint32 = 28
	PropMarginBottom    uint32 = 29
	PropWidth           uint32 = 30
	PropHeight          uint32 = 31
	PropMinWidth        uint32 = 32
	PropMinHeight       uint32 = 33
	PropMaxWidth        uint32 = 34
	PropMaxHeight       uint32 = 35
	PropGrow            uint32 = 36
	PropShrink          uint32 = 37
	PropAlignItems      uint32 = 38
	PropJustify         uint32 = 39
	PropAlignSelf       uint32 = 40
	PropWrap            uint32 = 41
	PropEnter           uint32 = 42
	PropHoverLift       uint32 = 43
	PropClickable       uint32 = 44
	PropPressEffect     uint32 = 45
	PropSpringStiffness uint32 = 46
	PropSpringDamping   uint32 = 47
	PropContextMenu     uint32 = 48
	PropMatchAnchor     uint32 = 49
	PropText            uint32 = 50
	PropPlaceholder     uint32 = 51
	PropFontSize        uint32 = 52
	PropFontWeight      uint32 = 53
	PropTextColor       uint32 = 54
	PropTextAlign       uint32 = 55
	PropTextWrap        uint32 = 56
	PropFontFamily      uint32 = 57
	PropVariant         uint32 = 60
	PropChecked         uint32 = 61
	PropValue           uint32 = 62
	PropMin             uint32 = 63
	PropMax             uint32 = 64
	PropStep            uint32 = 65
	PropSecure          uint32 = 66
	PropIndeterminate   uint32 = 67
	PropOrientation     uint32 = 68
	PropFit             uint32 = 69
	PropScrollY         uint32 = 70
	PropFocus           uint32 = 71
	PropMaxLength       uint32 = 72
	PropIconSize        uint32 = 73
	PropSpin            uint32 = 74
	PropPulse           uint32 = 75
	PropAnimate         uint32 = 76
	PropIconEnd         uint32 = 77
	PropTooltip         uint32 = 78
	PropShortcut        uint32 = 79
	PropTitle           uint32 = 80
	PropWindowWidth     uint32 = 81
	PropWindowHeight    uint32 = 82
	PropWindowMinWidth  uint32 = 83
	PropWindowMinHeight uint32 = 84
	PropResizable       uint32 = 85
	PropDecorations     uint32 = 86
	PropAlwaysOnTop     uint32 = 87
	PropMaximized       uint32 = 88
	PropScale           uint32 = 89
	PropInterceptClose  uint32 = 90
)

// Events.
const (
	EvClick         uint32 = 1
	EvTextChanged   uint32 = 2
	EvToggled       uint32 = 3
	EvValueChanged  uint32 = 4
	EvSubmit        uint32 = 5
	EvFocus         uint32 = 6
	EvBlur          uint32 = 7
	EvHoverEnter    uint32 = 8
	EvHoverLeave    uint32 = 9
	EvMenuClosed    uint32 = 10
	EvWindowClose   uint32 = 20
	EvWindowResized uint32 = 21
	EvWindowFocus   uint32 = 22
	EvTimer         uint32 = 30
	EvPosted        uint32 = 31
	EvStarted       uint32 = 32
)

// Button variants.
const (
	VariantPrimary  int64 = 0
	VariantTonal    int64 = 1
	VariantOutlined int64 = 2
	VariantGhost    int64 = 3
	VariantDanger   int64 = 4
)

// EventHandler receives every event from the core, on the thread that called Run.
type EventHandler func(node uint64, event uint32, a, b int64, text string)

var (
	loadOnce sync.Once
	loadErr  error
	handler  atomic.Pointer[EventHandler]

	// LibraryPath is the file the core was loaded from (set by Load).
	LibraryPath string
)

var procs struct {
	abiVersion, setEventHandler, run, quit, isRunning, lastError                           uintptr
	nodeNew, windowNew, nodeFree, nodeAppend, nodeInsert, nodeRemove, nodeClear            uintptr
	nodeChildCount, nodeKind, nodeParent, windowSetContent, windowClose, windowSnapshot    uintptr
	setI64, setF64, setStr, getI64, getF64, getStr, setImage                               uintptr
	themeSetMode, themeIsDark, themeSetAccent, themeSetFont, timerStart, timerCancel, post uintptr
	setSvg, svgRender, windowSetIcon                                                       uintptr
	themeUse, themeCurrent, themeList, themeDefine, themeSetTransition                     uintptr
	setAnimationSpeed, getAnimationSpeed                                                   uintptr
	menuOpen, menuClose, menuIsOpen, setMotion, getMotion                                  uintptr
}

// Load locates and loads the shared library. It is safe to call repeatedly.
func Load() error {
	loadOnce.Do(func() { loadErr = load() })
	return loadErr
}

func load() error {
	path, err := locate()
	if err != nil {
		return err
	}
	h, err := openLibrary(path)
	if err != nil {
		return fmt.Errorf("patina: cannot load %s: %w", path, err)
	}
	syms := []struct {
		name string
		dst  *uintptr
	}{
		{"patina_abi_version", &procs.abiVersion},
		{"patina_set_event_handler", &procs.setEventHandler},
		{"patina_run", &procs.run},
		{"patina_quit", &procs.quit},
		{"patina_is_running", &procs.isRunning},
		{"patina_last_error", &procs.lastError},
		{"patina_node_new", &procs.nodeNew},
		{"patina_window_new", &procs.windowNew},
		{"patina_node_free", &procs.nodeFree},
		{"patina_node_append", &procs.nodeAppend},
		{"patina_node_insert", &procs.nodeInsert},
		{"patina_node_remove", &procs.nodeRemove},
		{"patina_node_clear", &procs.nodeClear},
		{"patina_node_child_count", &procs.nodeChildCount},
		{"patina_node_kind", &procs.nodeKind},
		{"patina_node_parent", &procs.nodeParent},
		{"patina_window_set_content", &procs.windowSetContent},
		{"patina_window_close", &procs.windowClose},
		{"patina_window_snapshot", &procs.windowSnapshot},
		{"patina_set_i64", &procs.setI64},
		{"patina_set_f64", &procs.setF64},
		{"patina_set_str", &procs.setStr},
		{"patina_get_i64", &procs.getI64},
		{"patina_get_f64", &procs.getF64},
		{"patina_get_str", &procs.getStr},
		{"patina_set_image", &procs.setImage},
		{"patina_theme_set_mode", &procs.themeSetMode},
		{"patina_theme_is_dark", &procs.themeIsDark},
		{"patina_theme_set_accent", &procs.themeSetAccent},
		{"patina_theme_set_font", &procs.themeSetFont},
		{"patina_timer_start", &procs.timerStart},
		{"patina_timer_cancel", &procs.timerCancel},
		{"patina_post", &procs.post},
		{"patina_set_svg", &procs.setSvg},
		{"patina_svg_render", &procs.svgRender},
		{"patina_window_set_icon", &procs.windowSetIcon},
		{"patina_theme_use", &procs.themeUse},
		{"patina_theme_current", &procs.themeCurrent},
		{"patina_theme_list", &procs.themeList},
		{"patina_theme_define", &procs.themeDefine},
		{"patina_theme_set_transition", &procs.themeSetTransition},
		{"patina_set_animation_speed", &procs.setAnimationSpeed},
		{"patina_get_animation_speed", &procs.getAnimationSpeed},
		{"patina_menu_open", &procs.menuOpen},
		{"patina_menu_close", &procs.menuClose},
		{"patina_menu_is_open", &procs.menuIsOpen},
		{"patina_set_motion", &procs.setMotion},
		{"patina_get_motion", &procs.getMotion},
	}
	for _, s := range syms {
		addr, err := lookup(h, s.name)
		if err != nil {
			return fmt.Errorf("patina: symbol %s missing in %s (library out of date?): %w", s.name, path, err)
		}
		*s.dst = addr
	}
	if v := uint32(call(procs.abiVersion)); v != ABIVersion {
		return fmt.Errorf("patina: ABI mismatch: %s is version %d, this package expects %d", path, v, ABIVersion)
	}
	cb := purego.NewCallback(onEvent)
	call(procs.setEventHandler, cb, 0)
	LibraryPath = path
	return nil
}

// SetEventHandler installs the Go-side event dispatcher.
func SetEventHandler(h EventHandler) {
	handler.Store(&h)
}

func onEvent(_ uintptr, node uint64, event uint32, a int64, b int64, text *byte, textLen uintptr) uintptr {
	var s string
	if text != nil && textLen > 0 {
		s = string(unsafe.Slice(text, textLen))
	}
	if h := handler.Load(); h != nil && *h != nil {
		(*h)(node, event, a, b, s)
	}
	return 0
}

func call(fn uintptr, args ...uintptr) uintptr {
	r1, _, _ := purego.SyscallN(fn, args...)
	return r1
}

func bytesArg(b []byte) (uintptr, uintptr) {
	if len(b) == 0 {
		return 0, 0
	}
	return uintptr(unsafe.Pointer(&b[0])), uintptr(len(b))
}

func f64Arg(v float64) uintptr {
	return uintptr(math.Float64bits(v))
}

// ---- lifecycle ---------------------------------------------------------------------------

// Run blocks running the event loop. It returns when the last window closes or Quit is called.
func Run() error {
	if call(procs.run) != 0 {
		msg := LastError()
		if msg == "" {
			msg = "event loop failed"
		}
		return errors.New("patina: " + msg)
	}
	return nil
}

func Quit() { call(procs.quit) }

func IsRunning() bool { return call(procs.isRunning) != 0 }

// LastError returns the core's last error message.
func LastError() string {
	buf := make([]byte, 512)
	p, l := bytesArg(buf)
	n := call(procs.lastError, p, l)
	runtime.KeepAlive(buf)
	if n > uintptr(len(buf)) {
		buf = make([]byte, n)
		p, l = bytesArg(buf)
		n = call(procs.lastError, p, l)
		runtime.KeepAlive(buf)
	}
	return string(buf[:n])
}

// ---- tree --------------------------------------------------------------------------------

func NodeNew(kind uint32) uint64 { return uint64(call(procs.nodeNew, uintptr(kind))) }

func WindowNew() uint64 { return uint64(call(procs.windowNew)) }

func NodeFree(node uint64) { call(procs.nodeFree, uintptr(node)) }

func NodeAppend(parent, child uint64) bool {
	return call(procs.nodeAppend, uintptr(parent), uintptr(child)) != 0
}

func NodeInsert(parent, child uint64, index int) bool {
	return call(procs.nodeInsert, uintptr(parent), uintptr(child), uintptr(index)) != 0
}

func NodeRemove(parent, child uint64) { call(procs.nodeRemove, uintptr(parent), uintptr(child)) }

func NodeClear(parent uint64) { call(procs.nodeClear, uintptr(parent)) }

func NodeChildCount(node uint64) int { return int(call(procs.nodeChildCount, uintptr(node))) }

func NodeKind(node uint64) uint32 { return uint32(call(procs.nodeKind, uintptr(node))) }

func NodeParent(node uint64) uint64 { return uint64(call(procs.nodeParent, uintptr(node))) }

func WindowSetContent(window, node uint64) bool {
	return call(procs.windowSetContent, uintptr(window), uintptr(node)) != 0
}

func WindowClose(window uint64) { call(procs.windowClose, uintptr(window)) }

// WindowSnapshot renders a window offscreen to a PNG file. It does not need a running loop.
func WindowSnapshot(window uint64, width, height int, scale float64, path string) error {
	b := []byte(path)
	p, l := bytesArg(b)
	ok := call(procs.windowSnapshot, uintptr(window), uintptr(width), uintptr(height), f64Arg(scale), p, l)
	runtime.KeepAlive(b)
	if ok == 0 {
		return errors.New("patina: " + LastError())
	}
	return nil
}

// ---- properties --------------------------------------------------------------------------

func SetI64(node uint64, prop uint32, v int64) {
	call(procs.setI64, uintptr(node), uintptr(prop), uintptr(v))
}

func SetBool(node uint64, prop uint32, v bool) {
	var i int64
	if v {
		i = 1
	}
	SetI64(node, prop, i)
}

func SetF64(node uint64, prop uint32, v float64) {
	call(procs.setF64, uintptr(node), uintptr(prop), f64Arg(v))
}

func SetStr(node uint64, prop uint32, s string) {
	b := []byte(s)
	p, l := bytesArg(b)
	call(procs.setStr, uintptr(node), uintptr(prop), p, l)
	runtime.KeepAlive(b)
}

func GetI64(node uint64, prop uint32) int64 {
	return int64(call(procs.getI64, uintptr(node), uintptr(prop)))
}

func GetBool(node uint64, prop uint32) bool { return GetI64(node, prop) != 0 }

func GetF64(node uint64, prop uint32) float64 {
	return math.Float64frombits(uint64(call(procs.getF64, uintptr(node), uintptr(prop))))
}

func GetStr(node uint64, prop uint32) string {
	buf := make([]byte, 256)
	p, l := bytesArg(buf)
	n := call(procs.getStr, uintptr(node), uintptr(prop), p, l)
	runtime.KeepAlive(buf)
	if n > uintptr(len(buf)) {
		buf = make([]byte, n)
		p, l = bytesArg(buf)
		n = call(procs.getStr, uintptr(node), uintptr(prop), p, l)
		runtime.KeepAlive(buf)
	}
	return string(buf[:n])
}

// SetImage uploads straight-alpha RGBA8 pixels to an image node.
func SetImage(node uint64, width, height int, rgba []byte) bool {
	p, l := bytesArg(rgba)
	ok := call(procs.setImage, uintptr(node), uintptr(width), uintptr(height), p, l)
	runtime.KeepAlive(rgba)
	return ok != 0
}

// ---- theme -------------------------------------------------------------------------------

func ThemeSetMode(mode uint32) { call(procs.themeSetMode, uintptr(mode)) }

func ThemeIsDark() bool { return call(procs.themeIsDark) != 0 }

func ThemeSetAccent(rgba uint32) { call(procs.themeSetAccent, uintptr(rgba)) }

func ThemeSetFont(regular, bold, mono string) error {
	r, b, m := []byte(regular), []byte(bold), []byte(mono)
	rp, rl := bytesArg(r)
	bp, bl := bytesArg(b)
	mp, ml := bytesArg(m)
	ok := call(procs.themeSetFont, rp, rl, bp, bl, mp, ml)
	runtime.KeepAlive(r)
	runtime.KeepAlive(b)
	runtime.KeepAlive(m)
	if ok == 0 {
		return errors.New("patina: " + LastError())
	}
	return nil
}

// ---- timers / posting --------------------------------------------------------------------

func TimerStart(token uint64, intervalMs uint64, repeat bool) {
	var r uintptr
	if repeat {
		r = 1
	}
	call(procs.timerStart, uintptr(token), uintptr(intervalMs), r)
}

func TimerCancel(token uint64) { call(procs.timerCancel, uintptr(token)) }

func Post(token uint64) { call(procs.post, uintptr(token)) }

// ---- vector graphics ---------------------------------------------------------------------

// SetSVG sets SVG markup as the content of an image node or the icon of a button. An empty
// string clears it.
func SetSVG(node uint64, svg string) error {
	b := []byte(svg)
	p, l := bytesArg(b)
	ok := call(procs.setSvg, uintptr(node), p, l)
	runtime.KeepAlive(b)
	if ok == 0 {
		return errors.New("patina: " + LastError())
	}
	return nil
}

// SVGRender rasterizes SVG markup to straight-alpha RGBA8 pixels. tint uses the color
// encoding of the ABI; the default color leaves the document's own colors in place.
func SVGRender(svg string, width, height int, tint int64) ([]byte, error) {
	s := []byte(svg)
	sp, sl := bytesArg(s)
	buf := make([]byte, width*height*4)
	bp, bl := bytesArg(buf)
	n := call(procs.svgRender, sp, sl, uintptr(width), uintptr(height), uintptr(tint), bp, bl)
	runtime.KeepAlive(s)
	runtime.KeepAlive(buf)
	if n == 0 {
		return nil, errors.New("patina: " + LastError())
	}
	return buf, nil
}

// WindowSetIcon sets the window and taskbar icon from straight-alpha RGBA8 pixels.
func WindowSetIcon(window uint64, width, height int, rgba []byte) bool {
	p, l := bytesArg(rgba)
	ok := call(procs.windowSetIcon, uintptr(window), uintptr(width), uintptr(height), p, l)
	runtime.KeepAlive(rgba)
	return ok != 0
}

// ---- palettes ----------------------------------------------------------------------------

// PaletteColors is the number of colors a custom palette needs, in the order background,
// surface, surface variant, text, secondary text, muted text, accent, on-accent, outline,
// strong outline, danger, success, warning, shadow.
const PaletteColors = 14

// ThemeUse activates a palette by name, cross-fading to it.
func ThemeUse(name string) error {
	b := []byte(name)
	p, l := bytesArg(b)
	ok := call(procs.themeUse, p, l)
	runtime.KeepAlive(b)
	if ok == 0 {
		return errors.New("patina: " + LastError())
	}
	return nil
}

// ThemeCurrent returns the name of the palette in effect.
func ThemeCurrent() string { return readString(procs.themeCurrent) }

// ThemeList returns the known palettes as newline-separated "name\tdark\taccent\ton_accent"
// records (dark is 0/1, colors are 0xRRGGBBAA hex).
func ThemeList() string { return readString(procs.themeList) }

// readString calls a `(buf, cap) -> len` export with a growing buffer.
func readString(fn uintptr) string {
	buf := make([]byte, 512)
	p, l := bytesArg(buf)
	n := call(fn, p, l)
	runtime.KeepAlive(buf)
	if n > uintptr(len(buf)) {
		buf = make([]byte, n)
		p, l = bytesArg(buf)
		n = call(fn, p, l)
		runtime.KeepAlive(buf)
	}
	return string(buf[:n])
}

// ThemeDefine registers (or replaces) a custom palette from PaletteColors 0xRRGGBBAA values.
func ThemeDefine(name string, dark bool, colors []uint32) error {
	if len(colors) < PaletteColors {
		return fmt.Errorf("patina: a palette needs %d colors, got %d", PaletteColors, len(colors))
	}
	b := []byte(name)
	p, l := bytesArg(b)
	var d uintptr
	if dark {
		d = 1
	}
	ok := call(procs.themeDefine, p, l, d, uintptr(unsafe.Pointer(&colors[0])), uintptr(len(colors)))
	runtime.KeepAlive(b)
	runtime.KeepAlive(colors)
	if ok == 0 {
		return errors.New("patina: " + LastError())
	}
	return nil
}

// ThemeSetTransition sets how long palette and mode changes take to cross-fade.
func ThemeSetTransition(secs float64) {
	call(procs.themeSetTransition, uintptr(math.Float64bits(secs)))
}

// ---- animation ---------------------------------------------------------------------------

// SetAnimationSpeed sets the global animation speed factor (1 normal, 0 = no motion).
func SetAnimationSpeed(factor float64) {
	call(procs.setAnimationSpeed, uintptr(math.Float64bits(factor)))
}

// AnimationSpeed returns the global animation speed factor.
func AnimationSpeed() float64 {
	return math.Float64frombits(uint64(call(procs.getAnimationSpeed)))
}

// ---- menus -------------------------------------------------------------------------------

// MenuOpen opens a menu below anchor (a widget), or at the logical point x, y when anchor is
// the window itself.
func MenuOpen(menu, anchor uint64, x, y float64) error {
	if call(procs.menuOpen, uintptr(menu), uintptr(anchor), f64Arg(x), f64Arg(y)) == 0 {
		return errors.New("patina: " + LastError())
	}
	return nil
}

func MenuClose(menu uint64) { call(procs.menuClose, uintptr(menu)) }

func MenuIsOpen(menu uint64) bool { return call(procs.menuIsOpen, uintptr(menu)) != 0 }

// ---- motion style ------------------------------------------------------------------------

// SetMotion sets the default motion style: 0 ease, 1 spring, 2 bouncy.
func SetMotion(style uint32) { call(procs.setMotion, uintptr(style)) }

func Motion() uint32 { return uint32(call(procs.getMotion)) }
