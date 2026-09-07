package patina

import "github.com/patina-ui/patina/internal/native"

// Weight is a font weight.
type Weight int

const (
	Regular  Weight = 400
	Medium   Weight = 500
	Semibold Weight = 600
	Bold     Weight = 700
)

// TextAlign is the horizontal alignment of text inside its box.
type TextAlign int

const (
	TextLeft TextAlign = iota
	TextCenter
	TextRight
)

// textStyle is shared by every widget that shows text.
type textStyle[T any] struct{ Base[T] }

// FontSize sets the font size in logical pixels (default 14).
func (t *textStyle[T]) FontSize(px float64) *T { return t.setF(native.PropFontSize, px) }

// Weight sets the font weight.
func (t *textStyle[T]) Weight(w Weight) *T { return t.setI(native.PropFontWeight, int64(w)) }

// Bold is shorthand for Weight(Bold).
func (t *textStyle[T]) Bold() *T { return t.Weight(Bold) }

// Semibold is shorthand for Weight(Semibold).
func (t *textStyle[T]) Semibold() *T { return t.Weight(Semibold) }

// Color sets the text color.
func (t *textStyle[T]) Color(c Color) *T { return t.setI(native.PropTextColor, int64(c)) }

// Mono switches to the monospace system font.
func (t *textStyle[T]) Mono() *T { return t.setI(native.PropFontFamily, 1) }

// SetText replaces the text.
func (t *textStyle[T]) SetText(s string) *T {
	native.SetStr(t.n.id, native.PropText, s)
	return t.self
}

// Text returns the current text.
func (t *textStyle[T]) Text() string { return native.GetStr(t.n.id, native.PropText) }

// LabelWidget is a piece of static text. Create it with Label, Heading, Title or Caption.
type LabelWidget struct{ textStyle[LabelWidget] }

// Label creates a text label. Long text wraps to the available width.
func Label(text string) *LabelWidget {
	l := &LabelWidget{}
	l.attach(l, native.KindLabel)
	l.SetText(text)
	return l
}

// Title creates a large bold label (28px).
func Title(text string) *LabelWidget { return Label(text).FontSize(28).Bold() }

// Heading creates a section heading (20px, semibold).
func Heading(text string) *LabelWidget { return Label(text).FontSize(20).Semibold() }

// Caption creates small secondary text (12px).
func Caption(text string) *LabelWidget { return Label(text).FontSize(12).Color(TextSecondary) }

// Align sets the horizontal alignment of the text within the label's box.
func (l *LabelWidget) Align(a TextAlign) *LabelWidget { return l.setI(native.PropTextAlign, int64(a)) }

// Wrap enables or disables word wrapping (enabled by default).
func (l *LabelWidget) Wrap(wrap bool) *LabelWidget { return l.setB(native.PropTextWrap, wrap) }
