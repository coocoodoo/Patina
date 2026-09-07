package patina

import (
	"fmt"
	"image"
	"image/draw"
	"os"
	"time"

	"github.com/patina-ui/patina/internal/native"
)

// ---- Button ----------------------------------------------------------------------------

// Variant selects a button style.
type Variant int

const (
	// Primary is a filled accent-colored button for the main action.
	Primary Variant = iota
	// Tonal is a soft accent-tinted button for secondary actions.
	Tonal
	// Outlined is a bordered button on a surface background.
	Outlined
	// Ghost is a borderless button that only shows a fill on hover.
	Ghost
	// DangerButton is a filled red button for destructive actions.
	DangerButton
)

// ButtonWidget is a clickable button. Create it with Button.
type ButtonWidget struct{ textStyle[ButtonWidget] }

// Button creates a primary button with the given caption.
func Button(text string) *ButtonWidget {
	b := &ButtonWidget{}
	b.attach(b, native.KindButton)
	b.SetText(text)
	return b
}

// OnClick registers the click handler.
func (b *ButtonWidget) OnClick(fn func()) *ButtonWidget {
	b.n.onClick = fn
	return b
}

// Variant sets the button style.
func (b *ButtonWidget) Variant(v Variant) *ButtonWidget { return b.setI(native.PropVariant, int64(v)) }

func (b *ButtonWidget) Primary() *ButtonWidget  { return b.Variant(Primary) }
func (b *ButtonWidget) Tonal() *ButtonWidget    { return b.Variant(Tonal) }
func (b *ButtonWidget) Outlined() *ButtonWidget { return b.Variant(Outlined) }
func (b *ButtonWidget) Ghost() *ButtonWidget    { return b.Variant(Ghost) }
func (b *ButtonWidget) Danger() *ButtonWidget   { return b.Variant(DangerButton) }

// Icon shows an SVG icon before the caption, tinted with the caption color. A button with
// an empty caption becomes a square icon button. Pass "" to remove the icon.
func (b *ButtonWidget) Icon(svg string) *ButtonWidget {
	if err := native.SetSVG(b.n.id, svg); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	return b
}

// IconSize sets the icon size in logical pixels (default 18).
func (b *ButtonWidget) IconSize(px float64) *ButtonWidget { return b.setF(native.PropIconSize, px) }

// IconEnd draws the icon after the caption instead of before it (dropdown chevrons).
func (b *ButtonWidget) IconEnd(v bool) *ButtonWidget { return b.setB(native.PropIconEnd, v) }

// Menu opens the popup menu below the button when it is clicked.
func (b *ButtonWidget) Menu(m *MenuWidget) *ButtonWidget {
	return b.OnClick(func() { m.OpenBelow(b) })
}

// Glow surrounds the button with a soft, slowly pulsing halo in its own color.
func (b *ButtonWidget) Glow(on bool) *ButtonWidget { return b.setB(native.PropGlow, on) }

// ColorCycle continuously shifts the button through the color wheel, one full rotation per
// period. Zero restores the normal fill.
func (b *ButtonWidget) ColorCycle(period time.Duration) *ButtonWidget {
	return b.setF(native.PropColorCycle, period.Seconds())
}

// RubberBand stretches the button past its size on release and snaps it back.
func (b *ButtonWidget) RubberBand() *ButtonWidget { return b.PressEffect(RubberBand) }

// Gelatin squishes the button under the pointer and lets it wobble like jelly after a click.
func (b *ButtonWidget) Gelatin() *ButtonWidget { return b.PressEffect(Gelatin) }

// Bounce shrinks the button while pressed and pops it back with a springy overshoot.
func (b *ButtonWidget) Bounce() *ButtonWidget { return b.PressEffect(Bounce) }

// ---- TextInput -------------------------------------------------------------------------

// TextInputWidget is a single-line text field. Create it with TextInput.
type TextInputWidget struct{ textStyle[TextInputWidget] }

// TextInput creates an empty text field showing placeholder when empty.
func TextInput(placeholder string) *TextInputWidget {
	t := &TextInputWidget{}
	t.attach(t, native.KindTextInput)
	if placeholder != "" {
		t.Placeholder(placeholder)
	}
	return t
}

// Placeholder sets the hint shown while the field is empty.
func (t *TextInputWidget) Placeholder(s string) *TextInputWidget {
	native.SetStr(t.n.id, native.PropPlaceholder, s)
	return t
}

// Value returns the current contents.
func (t *TextInputWidget) Value() string { return t.Text() }

// SetValue replaces the contents and moves the caret to the end.
func (t *TextInputWidget) SetValue(s string) *TextInputWidget { return t.SetText(s) }

// Password masks the typed characters.
func (t *TextInputWidget) Password() *TextInputWidget { return t.setB(native.PropSecure, true) }

// MaxLength limits the number of characters (0 = unlimited).
func (t *TextInputWidget) MaxLength(n int) *TextInputWidget {
	return t.setI(native.PropMaxLength, int64(n))
}

// OnChange is called with the new value after every edit.
func (t *TextInputWidget) OnChange(fn func(value string)) *TextInputWidget {
	t.n.onText = fn
	return t
}

// OnSubmit is called with the value when Enter is pressed.
func (t *TextInputWidget) OnSubmit(fn func(value string)) *TextInputWidget {
	t.n.onSubmit = fn
	return t
}

// ---- Checkbox and Switch ----------------------------------------------------------------

// CheckboxWidget is a labelled check box. Create it with Checkbox.
type CheckboxWidget struct{ textStyle[CheckboxWidget] }

// Checkbox creates a check box with a label (which may be empty).
func Checkbox(label string) *CheckboxWidget {
	c := &CheckboxWidget{}
	c.attach(c, native.KindCheckbox)
	c.SetText(label)
	return c
}

// SetChecked sets the state without firing OnChange.
func (c *CheckboxWidget) SetChecked(v bool) *CheckboxWidget { return c.setB(native.PropChecked, v) }

// Checked reports the current state.
func (c *CheckboxWidget) Checked() bool { return native.GetBool(c.n.id, native.PropChecked) }

// OnChange is called with the new state when the user toggles the box.
func (c *CheckboxWidget) OnChange(fn func(checked bool)) *CheckboxWidget {
	c.n.onToggle = fn
	return c
}

// SwitchWidget is a labelled toggle switch. Create it with Switch.
type SwitchWidget struct{ textStyle[SwitchWidget] }

// Switch creates a toggle switch with a label (which may be empty).
func Switch(label string) *SwitchWidget {
	s := &SwitchWidget{}
	s.attach(s, native.KindSwitch)
	s.SetText(label)
	return s
}

// SetOn sets the state without firing OnChange.
func (s *SwitchWidget) SetOn(v bool) *SwitchWidget { return s.setB(native.PropChecked, v) }

// On reports the current state.
func (s *SwitchWidget) On() bool { return native.GetBool(s.n.id, native.PropChecked) }

// OnChange is called with the new state when the user flips the switch.
func (s *SwitchWidget) OnChange(fn func(on bool)) *SwitchWidget {
	s.n.onToggle = fn
	return s
}

// ---- Slider and Progress ----------------------------------------------------------------

// SliderWidget lets the user pick a number in a range. Create it with Slider.
type SliderWidget struct{ Base[SliderWidget] }

// Slider creates a slider ranging from min to max, initially at min.
func Slider(min, max float64) *SliderWidget {
	s := &SliderWidget{}
	s.attach(s, native.KindSlider)
	s.setF(native.PropMin, min)
	s.setF(native.PropMax, max)
	s.setF(native.PropValue, min)
	return s
}

// SetValue moves the slider without firing OnChange.
func (s *SliderWidget) SetValue(v float64) *SliderWidget { return s.setF(native.PropValue, v) }

// Value returns the current value.
func (s *SliderWidget) Value() float64 { return native.GetF64(s.n.id, native.PropValue) }

// Step snaps values to multiples of step (0 = continuous).
func (s *SliderWidget) Step(step float64) *SliderWidget { return s.setF(native.PropStep, step) }

// OnChange is called with the new value while the user drags the slider.
func (s *SliderWidget) OnChange(fn func(value float64)) *SliderWidget {
	s.n.onValue = fn
	return s
}

// ProgressWidget is a horizontal progress bar. Create it with Progress.
type ProgressWidget struct{ Base[ProgressWidget] }

// Progress creates a progress bar showing fraction (0-1).
func Progress(fraction float64) *ProgressWidget {
	p := &ProgressWidget{}
	p.attach(p, native.KindProgress)
	p.SetValue(fraction)
	return p
}

// SetValue sets the completed fraction (0-1).
func (p *ProgressWidget) SetValue(fraction float64) *ProgressWidget {
	return p.setF(native.PropValue, fraction)
}

// Value returns the completed fraction.
func (p *ProgressWidget) Value() float64 { return native.GetF64(p.n.id, native.PropValue) }

// Indeterminate shows a sweeping animation instead of a fraction.
func (p *ProgressWidget) Indeterminate(on bool) *ProgressWidget {
	return p.setB(native.PropIndeterminate, on)
}

// Color sets the fill color.
func (p *ProgressWidget) Color(c Color) *ProgressWidget { return p.Background(c) }

// ---- Divider, Spacer, Image ---------------------------------------------------------------

// DividerWidget is a thin separator line. Create it with Divider.
type DividerWidget struct{ Base[DividerWidget] }

// Divider creates a separator. It is horizontal in a Column and vertical in a Row.
func Divider() *DividerWidget {
	d := &DividerWidget{}
	d.attach(d, native.KindDivider)
	return d
}

// Vertical forces a vertical orientation.
func (d *DividerWidget) Vertical() *DividerWidget { return d.setI(native.PropOrientation, 1) }

// Color sets the line color.
func (d *DividerWidget) Color(c Color) *DividerWidget { return d.Background(c) }

// SpacerWidget is empty space. Create it with Spacer.
type SpacerWidget struct{ Base[SpacerWidget] }

// Spacer creates flexible empty space that pushes its siblings apart.
func Spacer() *SpacerWidget {
	s := &SpacerWidget{}
	s.attach(s, native.KindSpacer)
	return s
}

// Fixed creates empty space of a fixed size along both axes.
func Fixed(size float64) *SpacerWidget {
	return Spacer().Grow(0).Size(size, size)
}

// Fit controls how an image is scaled into its box.
type Fit int

const (
	// Contain scales the image to fit entirely inside the box, keeping its aspect ratio.
	Contain Fit = iota
	// Cover scales the image to fill the box, cropping the overflow.
	Cover
	// Fill stretches the image to the box.
	Fill
)

// ImageWidget shows a raster image. Create it with Image.
type ImageWidget struct{ Base[ImageWidget] }

// Image creates an image widget from any image.Image. The intrinsic size is the pixel size
// in logical pixels; use Width/Height to scale it.
func Image(img image.Image) *ImageWidget {
	w := &ImageWidget{}
	w.attach(w, native.KindImage)
	w.SetImage(img)
	return w
}

// SetImage replaces the pixels.
func (w *ImageWidget) SetImage(img image.Image) *ImageWidget {
	nrgba := toNRGBA(img)
	if nrgba == nil {
		return w
	}
	native.SetImage(w.n.id, nrgba.Rect.Dx(), nrgba.Rect.Dy(), nrgba.Pix)
	return w
}

// toNRGBA converts any image to a tightly packed, straight-alpha NRGBA image (nil if empty).
func toNRGBA(img image.Image) *image.NRGBA {
	if img == nil {
		return nil
	}
	b := img.Bounds()
	if b.Dx() <= 0 || b.Dy() <= 0 {
		return nil
	}
	nrgba, ok := img.(*image.NRGBA)
	if !ok || nrgba.Stride != b.Dx()*4 || b.Min != (image.Point{}) {
		nrgba = image.NewNRGBA(image.Rect(0, 0, b.Dx(), b.Dy()))
		draw.Draw(nrgba, nrgba.Bounds(), img, b.Min, draw.Src)
	}
	return nrgba
}

// Fit sets how the image is scaled into its box (default Contain).
func (w *ImageWidget) Fit(f Fit) *ImageWidget { return w.setI(native.PropFit, int64(f)) }

// SVG creates a vector image from SVG markup. It is rasterized by the core at whatever size
// and scale it is displayed at, so it stays crisp on HiDPI displays. The document keeps its
// own colors; `currentColor` resolves to the theme's text color. Its natural size is the
// SVG's width/height (or viewBox); use Size to change it. Parse errors are printed to
// stderr; use SetSVG to handle them.
func SVG(svg string) *ImageWidget {
	w := &ImageWidget{}
	w.attach(w, native.KindImage)
	if err := w.SetSVG(svg); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	return w
}

// Icon creates a monochrome vector icon from SVG markup: the document's shapes are
// recolored with the theme's text color, whatever colors the SVG uses. Change the color
// with Tint and the size with Size (the default is the SVG's own size, 24px for the
// bundled icons package).
func Icon(svg string) *ImageWidget { return SVG(svg).Tint(Text) }

// SetSVG replaces the content with SVG markup.
func (w *ImageWidget) SetSVG(svg string) error {
	return native.SetSVG(w.n.id, svg)
}

// Tint recolors an SVG image with c, using the document only as a mask. Pass ColorDefault to
// show the document's own colors again. Raster images are not affected.
func (w *ImageWidget) Tint(c Color) *ImageWidget { return w.setI(native.PropTint, int64(c)) }

// Spin rotates the image continuously, one full turn per period. Zero stops it.
func (w *ImageWidget) Spin(period time.Duration) *ImageWidget {
	return w.setF(native.PropSpin, period.Seconds())
}

// Pulse scales the image gently in and out, one cycle per period. Zero stops it.
func (w *ImageWidget) Pulse(period time.Duration) *ImageWidget {
	return w.setF(native.PropPulse, period.Seconds())
}

// Animate plays (true, the default) or freezes (false) the SMIL animations of an SVG:
// <animate>, <animateTransform> and <set> elements with values, keyTimes, keySplines,
// begin, dur, repeatCount and fill="freeze" are supported.
func (w *ImageWidget) Animate(on bool) *ImageWidget { return w.setB(native.PropAnimate, on) }

// ---- Spinner ---------------------------------------------------------------------------

// SpinnerWidget is an indeterminate activity indicator. Create it with Spinner.
type SpinnerWidget struct{ Base[SpinnerWidget] }

// Spinner creates a 24px spinning arc in the accent color. Change its size with Size and
// its color with Color.
func Spinner() *SpinnerWidget {
	s := &SpinnerWidget{}
	s.attach(s, native.KindSpinner)
	return s
}

// Color sets the arc color (default Accent).
func (s *SpinnerWidget) Color(c Color) *SpinnerWidget { return s.Background(c) }
