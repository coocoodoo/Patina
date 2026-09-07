package patina

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/patina-ui/patina/internal/native"
)

// Theme selects the light or dark palette.
type Theme int

const (
	// Light is the default light palette.
	Light Theme = iota
	// Dark is the dark palette.
	Dark
	// System follows the operating system's appearance setting.
	System
)

// SetTheme switches every window to the given palette. Safe from any goroutine.
func SetTheme(t Theme) {
	mustInit()
	native.ThemeSetMode(uint32(t))
}

// IsDark reports whether the dark palette is currently active.
func IsDark() bool {
	mustInit()
	return native.ThemeIsDark()
}

// SetAccent changes the brand color used by primary controls. Pass ColorDefault to restore
// the built-in indigo.
func SetAccent(c Color) {
	mustInit()
	if c.IsToken() {
		native.ThemeSetAccent(0)
		return
	}
	native.ThemeSetAccent(uint32(c))
}

// SetFont loads custom font files (TrueType or OpenType). bold and mono may be empty, in
// which case the regular face is used for them.
func SetFont(regular, bold, mono string) error {
	mustInit()
	if regular == "" {
		return errors.New("patina: SetFont needs at least a regular font file")
	}
	return native.ThemeSetFont(regular, bold, mono)
}

// ---- palettes --------------------------------------------------------------------------

// PaletteInfo describes a named palette.
type PaletteInfo struct {
	// Name is the palette's name, as accepted by SetPalette.
	Name string
	// Dark reports whether it is a dark palette.
	Dark bool
	// Accent is its brand color and OnAccent the text color that reads on it.
	Accent, OnAccent Color
}

// Palettes lists every palette: the ten built-in light ones (Indigo, Ocean, Forest, Sunset,
// Rose, Lavender, Sand, Slate, Mint, Mono), the ten dark ones (Midnight, Nord, Dracula,
// Ocean Dark, Forest Dark, Ember, Rose Dark, Amethyst, Graphite, Carbon) and any registered
// with DefinePalette.
func Palettes() []PaletteInfo {
	mustInit()
	var out []PaletteInfo
	for _, line := range strings.Split(native.ThemeList(), "\n") {
		f := strings.Split(line, "\t")
		if len(f) < 4 {
			continue
		}
		accent, err1 := strconv.ParseUint(f[2], 16, 32)
		on, err2 := strconv.ParseUint(f[3], 16, 32)
		if err1 != nil || err2 != nil {
			continue
		}
		out = append(out, PaletteInfo{
			Name:     f[0],
			Dark:     f[1] == "1",
			Accent:   Color(accent),
			OnAccent: Color(on),
		})
	}
	return out
}

// SetPalette activates a palette by name (case-insensitive). Dark palettes switch the theme
// to Dark and light ones to Light; every color cross-fades to the new palette. It returns
// an error for unknown names.
func SetPalette(name string) error {
	mustInit()
	return native.ThemeUse(name)
}

// CurrentPalette returns the name of the palette in effect.
func CurrentPalette() string {
	mustInit()
	return native.ThemeCurrent()
}

// PaletteColors is a full set of theme colors for DefinePalette.
type PaletteColors struct {
	Background, Surface, SurfaceVariant Color
	Text, TextSecondary, TextMuted      Color
	Accent, OnAccent                    Color
	Outline, OutlineStrong              Color
	Danger, Success, Warning            Color
	Shadow                              Color
}

// DefinePalette registers a custom palette (or replaces one with the same name) that can
// then be activated with SetPalette. Colors must be literal colors, not theme tokens.
func DefinePalette(name string, dark bool, c PaletteColors) error {
	mustInit()
	list := []Color{
		c.Background, c.Surface, c.SurfaceVariant,
		c.Text, c.TextSecondary, c.TextMuted,
		c.Accent, c.OnAccent,
		c.Outline, c.OutlineStrong,
		c.Danger, c.Success, c.Warning,
		c.Shadow,
	}
	raw := make([]uint32, len(list))
	for i, col := range list {
		if col.IsToken() {
			return fmt.Errorf("patina: DefinePalette %q: color %d is a theme token, not a literal color", name, i)
		}
		raw[i] = uint32(col)
	}
	return native.ThemeDefine(name, dark, raw)
}

// SetThemeTransition sets how long palette, mode and accent changes take to cross-fade.
// Zero switches instantly; the default is 280ms.
func SetThemeTransition(d time.Duration) {
	mustInit()
	native.ThemeSetTransition(d.Seconds())
}
