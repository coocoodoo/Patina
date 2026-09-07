package patina

import (
	"fmt"
	"image"
	"os"

	"github.com/patina-ui/patina/internal/native"
)

// Window is a top-level window. Create it with NewWindow; it appears when Run starts (or
// immediately, if the loop is already running).
type Window struct{ Base[Window] }

// NewWindow creates a window with a title and a logical size.
func NewWindow(title string, width, height float64) *Window {
	w := &Window{}
	w.attach(w, native.KindWindow)
	w.SetTitle(title)
	if width > 0 && height > 0 {
		w.SetSize(width, height)
	}
	return w
}

// SetContent sets the widget shown in the window, replacing any previous content.
func (w *Window) SetContent(content Widget) *Window {
	if content != nil {
		w.n.adopt(content.node(), -1)
	}
	return w
}

// Content returns the window's root widget, or nil.
func (w *Window) Content() Widget {
	if len(w.n.children) == 0 {
		return nil
	}
	return w.n.children[0].widget
}

// SetTitle changes the title bar text.
func (w *Window) SetTitle(title string) *Window {
	native.SetStr(w.n.id, native.PropTitle, title)
	return w
}

// Title returns the title bar text.
func (w *Window) Title() string { return native.GetStr(w.n.id, native.PropTitle) }

// SetSize resizes the window (logical pixels).
func (w *Window) SetSize(width, height float64) *Window {
	w.setF(native.PropWindowWidth, width)
	return w.setF(native.PropWindowHeight, height)
}

// Size returns the current logical size.
func (w *Window) Size() (width, height float64) {
	return native.GetF64(w.n.id, native.PropWindowWidth), native.GetF64(w.n.id, native.PropWindowHeight)
}

// MinSize sets the smallest size the user can resize the window to.
func (w *Window) MinSize(width, height float64) *Window {
	w.setF(native.PropWindowMinWidth, width)
	return w.setF(native.PropWindowMinHeight, height)
}

// Resizable controls whether the user can resize the window (default true).
func (w *Window) Resizable(v bool) *Window { return w.setB(native.PropResizable, v) }

// Decorations controls the title bar and borders (default true).
func (w *Window) Decorations(v bool) *Window { return w.setB(native.PropDecorations, v) }

// AlwaysOnTop keeps the window above other windows.
func (w *Window) AlwaysOnTop(v bool) *Window { return w.setB(native.PropAlwaysOnTop, v) }

// Maximized maximizes or restores the window.
func (w *Window) Maximized(v bool) *Window { return w.setB(native.PropMaximized, v) }

// Scale returns the display scale factor (1 on standard displays, 2 on HiDPI).
func (w *Window) Scale() float64 { return native.GetF64(w.n.id, native.PropScale) }

// Close closes the window. When it is the last window, Run returns.
func (w *Window) Close() { native.WindowClose(w.n.id) }

// OnClose is called after the window has been closed.
func (w *Window) OnClose(fn func()) *Window {
	w.n.onClose = fn
	return w
}

// OnCloseRequest is called when the user tries to close the window. Return false to keep it
// open (for example to ask about unsaved changes).
func (w *Window) OnCloseRequest(fn func() bool) *Window {
	w.n.onCloseReq = fn
	return w.setB(native.PropInterceptClose, fn != nil)
}

// OnResize is called with the new logical size whenever the window is resized.
func (w *Window) OnResize(fn func(width, height int)) *Window {
	w.n.onResize = fn
	return w
}

// OnFocusChange is called when the window gains or loses focus.
func (w *Window) OnFocusChange(fn func(focused bool)) *Window {
	w.n.onWindowFocus = fn
	return w
}

// SetIcon sets the window's title bar and taskbar icon. Use a square image; 64x64 or larger
// looks good everywhere.
func (w *Window) SetIcon(img image.Image) *Window {
	if nrgba := toNRGBA(img); nrgba != nil {
		native.WindowSetIcon(w.n.id, nrgba.Rect.Dx(), nrgba.Rect.Dy(), nrgba.Pix)
	}
	return w
}

// SetIconSVG renders SVG markup at 64x64 and uses it as the window icon.
func (w *Window) SetIconSVG(svg string) *Window {
	img, err := RenderSVG(svg, 64, 64)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return w
	}
	return w.SetIcon(img)
}

// Snapshot renders the window's content offscreen at the given logical size and scale and
// writes it as a PNG. It works without a running event loop, which makes it handy for
// visual tests and documentation screenshots.
func (w *Window) Snapshot(path string, width, height int, scale float64) error {
	return native.WindowSnapshot(w.n.id, width, height, scale, path)
}
