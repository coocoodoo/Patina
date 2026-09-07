package patina

import (
	"strconv"
	"strings"
)

// Color is either a literal RGBA color (see RGB, RGBA and Hex) or one of the theme tokens
// below. Tokens resolve against the active theme, so a widget colored with Accent follows
// SetAccent and switches correctly between light and dark mode.
type Color int64

const (
	// ColorDefault uses the widget's default color.
	ColorDefault Color = -1
	// Background is the window background.
	Background Color = -2
	// Surface is the color of cards and inputs.
	Surface Color = -3
	// SurfaceVariant is a subtle contrasting fill.
	SurfaceVariant Color = -4
	// Text is the primary text color.
	Text Color = -5
	// TextSecondary is a softer text color.
	TextSecondary Color = -6
	// TextMuted is the faintest text color.
	TextMuted Color = -7
	// Accent is the brand color used by primary controls.
	Accent Color = -8
	// OnAccent is the text color that reads well on Accent.
	OnAccent Color = -9
	// Outline is the hairline border color.
	Outline Color = -10
	// Danger is used for destructive actions and errors.
	Danger Color = -11
	// Success is a positive status color.
	Success Color = -12
	// Warning is a cautionary status color.
	Warning Color = -13

	// Transparent is a fully transparent literal color.
	Transparent Color = 0
)

// RGB creates an opaque color.
func RGB(r, g, b uint8) Color { return RGBA(r, g, b, 255) }

// RGBA creates a color with an alpha channel (255 = opaque).
func RGBA(r, g, b, a uint8) Color {
	return Color(uint32(r)<<24 | uint32(g)<<16 | uint32(b)<<8 | uint32(a))
}

// Hex parses "#RRGGBB", "#RRGGBBAA", "#RGB" or the same without the hash. It returns
// ColorDefault for anything it cannot parse.
func Hex(s string) Color {
	s = strings.TrimPrefix(strings.TrimSpace(s), "#")
	switch len(s) {
	case 3:
		s = string([]byte{s[0], s[0], s[1], s[1], s[2], s[2]}) + "ff"
	case 6:
		s += "ff"
	case 8:
	default:
		return ColorDefault
	}
	v, err := strconv.ParseUint(s, 16, 32)
	if err != nil {
		return ColorDefault
	}
	return Color(v)
}

// IsToken reports whether the color is a theme token rather than a literal.
func (c Color) IsToken() bool { return c < 0 }

// WithAlpha returns the same literal color with a different alpha. Tokens are returned unchanged.
func (c Color) WithAlpha(a uint8) Color {
	if c.IsToken() {
		return c
	}
	return Color(uint32(c)&0xffffff00 | uint32(a))
}

// Components returns the channels of a literal color.
func (c Color) Components() (r, g, b, a uint8) {
	v := uint32(c)
	return uint8(v >> 24), uint8(v >> 16), uint8(v >> 8), uint8(v)
}
