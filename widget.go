package patina

import (
	"math"

	"github.com/patina-ui/patina/internal/native"
)

// Widget is implemented by every Patina widget.
type Widget interface {
	node() *node
}

// Align positions children along the cross axis of a Column or Row.
type Align int

const (
	AlignStart Align = iota
	AlignCenter
	AlignEnd
	AlignStretch
)

// Justify distributes children along the main axis of a Column or Row.
type Justify int

const (
	JustifyStart Justify = iota
	JustifyCenter
	JustifyEnd
	SpaceBetween
	SpaceAround
	SpaceEvenly
)

// Base carries the behaviour shared by every widget. T is the concrete widget type, which
// lets chained modifiers keep their static type:
//
//	label := patina.Label("Hi").Padding(8).Color(patina.Accent) // still a *LabelWidget
type Base[T any] struct {
	n    *node
	self *T
}

func (b *Base[T]) node() *node { return b.n }

func (b *Base[T]) attach(self *T, kind uint32) {
	b.self = self
	w, _ := any(self).(Widget)
	b.n = newNode(kind, w)
}

// ID returns the native handle of the widget.
func (b *Base[T]) ID() uint64 { return b.n.id }

// Destroy removes the widget (and its children) from its parent and frees it. The widget
// must not be used afterwards.
func (b *Base[T]) Destroy() { b.n.destroy() }

// IsDestroyed reports whether Destroy was called on the widget or an ancestor.
func (b *Base[T]) IsDestroyed() bool { return b.n.destroyed }

func (b *Base[T]) setF(prop uint32, v float64) *T {
	native.SetF64(b.n.id, prop, v)
	return b.self
}

func (b *Base[T]) setI(prop uint32, v int64) *T {
	native.SetI64(b.n.id, prop, v)
	return b.self
}

func (b *Base[T]) setB(prop uint32, v bool) *T {
	native.SetBool(b.n.id, prop, v)
	return b.self
}

// ---- size ------------------------------------------------------------------------------

// Width sets a fixed width in logical pixels.
func (b *Base[T]) Width(px float64) *T { return b.setF(native.PropWidth, px) }

// Height sets a fixed height in logical pixels.
func (b *Base[T]) Height(px float64) *T { return b.setF(native.PropHeight, px) }

// Size sets both a fixed width and height.
func (b *Base[T]) Size(width, height float64) *T {
	b.Width(width)
	return b.Height(height)
}

// WidthPercent sets the width relative to the parent (0-100).
func (b *Base[T]) WidthPercent(percent float64) *T { return b.setF(native.PropWidth, -percent) }

// HeightPercent sets the height relative to the parent (0-100).
func (b *Base[T]) HeightPercent(percent float64) *T { return b.setF(native.PropHeight, -percent) }

// AutoWidth removes a fixed width.
func (b *Base[T]) AutoWidth() *T { return b.setF(native.PropWidth, math.NaN()) }

// AutoHeight removes a fixed height.
func (b *Base[T]) AutoHeight() *T { return b.setF(native.PropHeight, math.NaN()) }

func (b *Base[T]) MinWidth(px float64) *T  { return b.setF(native.PropMinWidth, px) }
func (b *Base[T]) MinHeight(px float64) *T { return b.setF(native.PropMinHeight, px) }
func (b *Base[T]) MaxWidth(px float64) *T  { return b.setF(native.PropMaxWidth, px) }
func (b *Base[T]) MaxHeight(px float64) *T { return b.setF(native.PropMaxHeight, px) }

// Grow sets the flex grow factor: how much of the parent's free space the widget takes.
func (b *Base[T]) Grow(factor float64) *T { return b.setF(native.PropGrow, factor) }

// Expand is shorthand for Grow(1).
func (b *Base[T]) Expand() *T { return b.Grow(1) }

// Shrink sets the flex shrink factor (default 1).
func (b *Base[T]) Shrink(factor float64) *T { return b.setF(native.PropShrink, factor) }

// AlignSelf overrides the parent's cross-axis alignment for this widget.
func (b *Base[T]) AlignSelf(a Align) *T { return b.setI(native.PropAlignSelf, int64(a)+1) }

// ---- spacing ---------------------------------------------------------------------------

// Padding sets the same inner spacing on all four sides.
func (b *Base[T]) Padding(all float64) *T { return b.PaddingEach(all, all, all, all) }

// PaddingXY sets horizontal and vertical inner spacing.
func (b *Base[T]) PaddingXY(x, y float64) *T { return b.PaddingEach(x, y, x, y) }

// PaddingEach sets the inner spacing per side.
func (b *Base[T]) PaddingEach(left, top, right, bottom float64) *T {
	b.setF(native.PropPaddingLeft, left)
	b.setF(native.PropPaddingTop, top)
	b.setF(native.PropPaddingRight, right)
	return b.setF(native.PropPaddingBottom, bottom)
}

// Margin sets the same outer spacing on all four sides.
func (b *Base[T]) Margin(all float64) *T { return b.MarginEach(all, all, all, all) }

// MarginXY sets horizontal and vertical outer spacing.
func (b *Base[T]) MarginXY(x, y float64) *T { return b.MarginEach(x, y, x, y) }

// MarginEach sets the outer spacing per side.
func (b *Base[T]) MarginEach(left, top, right, bottom float64) *T {
	b.setF(native.PropMarginLeft, left)
	b.setF(native.PropMarginTop, top)
	b.setF(native.PropMarginRight, right)
	return b.setF(native.PropMarginBottom, bottom)
}

// ---- appearance ------------------------------------------------------------------------

// Background sets the fill color (a literal color or a theme token such as Surface).
func (b *Base[T]) Background(c Color) *T { return b.setI(native.PropBackground, int64(c)) }

// Radius sets the corner radius in logical pixels.
func (b *Base[T]) Radius(px float64) *T { return b.setF(native.PropRadius, px) }

// Border draws an outline of the given width and color inside the widget's bounds.
func (b *Base[T]) Border(width float64, c Color) *T {
	b.setF(native.PropBorderWidth, width)
	return b.setI(native.PropBorderColor, int64(c))
}

// Shadow sets the elevation shadow level: 0 (none) to 3 (large).
func (b *Base[T]) Shadow(level int) *T { return b.setI(native.PropShadow, int64(level)) }

// Opacity sets the opacity of the widget and its children (0-1).
func (b *Base[T]) Opacity(o float64) *T { return b.setF(native.PropOpacity, o) }

// ---- state -----------------------------------------------------------------------------

// Visible shows or hides the widget. Hidden widgets take no space.
func (b *Base[T]) Visible(v bool) *T { return b.setB(native.PropVisible, v) }

// Show makes the widget visible.
func (b *Base[T]) Show() *T { return b.Visible(true) }

// Hide hides the widget.
func (b *Base[T]) Hide() *T { return b.Visible(false) }

// IsVisible reports whether the widget is visible.
func (b *Base[T]) IsVisible() bool { return native.GetBool(b.n.id, native.PropVisible) }

// Disabled greys the widget out and stops it from reacting to input.
func (b *Base[T]) Disabled(d bool) *T { return b.setB(native.PropDisabled, d) }

// Enable is shorthand for Disabled(false).
func (b *Base[T]) Enable() *T { return b.Disabled(false) }

// Disable is shorthand for Disabled(true).
func (b *Base[T]) Disable() *T { return b.Disabled(true) }

// IsDisabled reports whether the widget is disabled.
func (b *Base[T]) IsDisabled() bool { return native.GetBool(b.n.id, native.PropDisabled) }

// Focus requests keyboard focus for the widget.
func (b *Base[T]) Focus() *T { return b.setI(native.PropFocus, 1) }

// HasFocus reports whether the widget currently has keyboard focus.
func (b *Base[T]) HasFocus() bool { return native.GetBool(b.n.id, native.PropFocus) }

// OnHover registers callbacks for the pointer entering and leaving the widget. Either may be nil.
func (b *Base[T]) OnHover(enter, leave func()) *T {
	b.n.onHoverEnter = enter
	b.n.onHoverLeave = leave
	return b.self
}

// OnFocusChange registers callbacks for gaining and losing keyboard focus. Either may be nil.
func (b *Base[T]) OnFocusChange(focus, blur func()) *T {
	b.n.onFocus = focus
	b.n.onBlur = blur
	return b.self
}

// ---- motion ------------------------------------------------------------------------------

// Enter plays an entrance animation when the widget first appears (and each time it is shown
// again after Hide).
func (b *Base[T]) Enter(e Enter) *T { return b.setI(native.PropEnter, int64(e)) }

// Gradient fills the background with a horizontal gradient from one color to another.
func (b *Base[T]) Gradient(from, to Color) *T {
	b.setI(native.PropBackground, int64(from))
	return b.setI(native.PropGradientEnd, int64(to))
}

// Ripple shows an expanding wave from the click point on buttons and clickable boxes.
func (b *Base[T]) Ripple(on bool) *T { return b.setB(native.PropRipple, on) }

// PressEffect makes the widget squash while pressed and wobble after a click: RubberBand,
// Gelatin or Bounce. Text and icons deform with it. It works on buttons and on boxes made
// clickable with OnClick.
func (b *Base[T]) PressEffect(e PressEffect) *T { return b.setI(native.PropPressEffect, int64(e)) }

// Tooltip shows text in a small bubble after the pointer rests on the widget for half a
// second. Pass "" to remove it.
func (b *Base[T]) Tooltip(text string) *T {
	native.SetStr(b.n.id, native.PropTooltip, text)
	return b.self
}

// ContextMenu opens the menu at the pointer when the widget (or anything inside it) is
// right-clicked. Pass nil to remove it.
func (b *Base[T]) ContextMenu(m *MenuWidget) *T {
	var id int64
	if m != nil {
		id = int64(m.n.id)
	}
	return b.setI(native.PropContextMenu, id)
}

// Spring gives this widget's value, toggle and entrance animations their own spring instead
// of the global motion style (see SetMotion). Stiffness around 100-400 and damping around
// 8-25 are sensible; pass 0 stiffness to follow the global style again.
func (b *Base[T]) Spring(stiffness, damping float64) *T {
	b.setF(native.PropSpringStiffness, stiffness)
	return b.setF(native.PropSpringDamping, damping)
}
