package patina_test

import (
	"image/png"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/patina-ui/patina"
)

func requireCore(t *testing.T) {
	t.Helper()
	if err := patina.Init(); err != nil {
		t.Skipf("native core unavailable: %v", err)
	}
}

// TestSnapshot renders a window offscreen, which exercises the whole Go -> native path:
// node creation, properties, layout, text and painting, without needing a display.
func TestSnapshot(t *testing.T) {
	requireCore(t)

	win := patina.NewWindow("snapshot test", 400, 300)
	label := patina.Label("Hello, Patina")
	check := patina.Checkbox("Remember me").SetChecked(true)
	slider := patina.Slider(0, 10).SetValue(2.5)
	input := patina.TextInput("Name").SetValue("Ada")
	win.SetContent(
		patina.Column(
			label,
			patina.Button("Continue"),
			input,
			check,
			patina.Switch("Dark").SetOn(true),
			slider,
			patina.Progress(0.4),
			patina.Divider(),
			patina.Card(patina.Caption("card")),
		).Padding(16).Gap(8),
	)

	path := filepath.Join(t.TempDir(), "out.png")
	if err := win.Snapshot(path, 400, 300, 2); err != nil {
		t.Fatalf("snapshot: %v", err)
	}
	f, err := os.Open(path)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	img, err := png.Decode(f)
	if err != nil {
		t.Fatalf("decode: %v", err)
	}
	if got := img.Bounds(); got.Dx() != 800 || got.Dy() != 600 {
		t.Fatalf("snapshot size = %dx%d, want 800x600 (2x scale)", got.Dx(), got.Dy())
	}

	// Property round trips.
	label.SetText("Changed")
	if got := label.Text(); got != "Changed" {
		t.Errorf("label text = %q, want %q", got, "Changed")
	}
	if !check.Checked() {
		t.Error("checkbox should be checked")
	}
	if got := slider.Value(); got != 2.5 {
		t.Errorf("slider value = %v, want 2.5", got)
	}
	if got := input.Value(); got != "Ada" {
		t.Errorf("input value = %q, want %q", got, "Ada")
	}
	if got := win.Title(); got != "snapshot test" {
		t.Errorf("title = %q", got)
	}
	label.Hide()
	if label.IsVisible() {
		t.Error("label should be hidden")
	}
}

func TestTree(t *testing.T) {
	requireCore(t)

	box := patina.Column()
	a := patina.Label("a")
	b := patina.Label("b")
	box.Add(a, b)
	if n := len(box.Children()); n != 2 {
		t.Fatalf("children = %d, want 2", n)
	}
	box.Remove(a)
	if n := len(box.Children()); n != 1 || box.Children()[0] != b {
		t.Fatalf("after Remove: children = %v", box.Children())
	}
	if a.IsDestroyed() {
		t.Fatal("Remove must not destroy the child")
	}
	box.Insert(0, a)
	if box.Children()[0] != a {
		t.Fatal("Insert(0) should put the child first")
	}
	box.Clear()
	if len(box.Children()) != 0 || !a.IsDestroyed() || !b.IsDestroyed() {
		t.Fatal("Clear should destroy all children")
	}
	box.Destroy()
	if !box.IsDestroyed() {
		t.Fatal("box should be destroyed")
	}
}

const circleSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>`

func TestSVG(t *testing.T) {
	requireCore(t)

	img, err := patina.RenderSVG(circleSVG, 32, 32)
	if err != nil {
		t.Fatal(err)
	}
	if got := img.Bounds(); got.Dx() != 32 || got.Dy() != 32 {
		t.Fatalf("size = %v", got)
	}
	if _, _, _, a := img.At(16, 16).RGBA(); a == 0 {
		t.Error("center of the circle should be opaque")
	}
	if _, _, _, a := img.At(0, 0).RGBA(); a != 0 {
		t.Error("corner should be transparent")
	}
	tinted, err := patina.RenderSVGTinted(circleSVG, 32, 32, patina.RGB(0, 255, 0))
	if err != nil {
		t.Fatal(err)
	}
	if r, g, _, _ := tinted.At(16, 16).RGBA(); g < 0xF000 || r > 0x0FFF {
		t.Errorf("tinted center = r%d g%d, want green", r, g)
	}
	if _, err := patina.RenderSVG("<svg", 8, 8); err == nil {
		t.Error("invalid SVG must fail")
	}

	icon := patina.Icon(circleSVG).Size(20, 20)
	if err := icon.SetSVG("<svg"); err == nil {
		t.Error("SetSVG must report parse errors")
	}
	btn := patina.Button("Go").Icon(circleSVG)
	win := patina.NewWindow("svg", 200, 120)
	win.SetContent(patina.Row(icon, btn, patina.SVG(circleSVG).Size(24, 24)).Gap(8).Padding(12))
	path := filepath.Join(t.TempDir(), "svg.png")
	if err := win.Snapshot(path, 200, 120, 1); err != nil {
		t.Fatalf("snapshot with SVG content: %v", err)
	}
}

func TestHex(t *testing.T) {
	cases := map[string]patina.Color{
		"#6366F1":   patina.RGB(0x63, 0x66, 0xF1),
		"6366f1":    patina.RGB(0x63, 0x66, 0xF1),
		"#6366F180": patina.RGBA(0x63, 0x66, 0xF1, 0x80),
		"#FFF":      patina.RGB(255, 255, 255),
		"nope":      patina.ColorDefault,
	}
	for in, want := range cases {
		if got := patina.Hex(in); got != want {
			t.Errorf("Hex(%q) = %v, want %v", in, got, want)
		}
	}
	if got := patina.RGB(1, 2, 3).WithAlpha(9); got != patina.RGBA(1, 2, 3, 9) {
		t.Errorf("WithAlpha = %v", got)
	}
	if patina.Accent.WithAlpha(1) != patina.Accent {
		t.Error("WithAlpha must leave tokens unchanged")
	}
}

const animatedTestSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill="#5B5FEF"><animate attributeName="r" values="4;8;4" dur="1s" repeatCount="indefinite"/></circle></svg>`

func TestPalettes(t *testing.T) {
	requireCore(t)
	t.Cleanup(func() {
		_ = patina.SetPalette("Indigo")
		patina.SetTheme(patina.Light)
	})

	var light, dark int
	for _, p := range patina.Palettes() {
		if p.Dark {
			dark++
		} else {
			light++
		}
		if p.Name == "" || p.Accent.IsToken() || p.OnAccent.IsToken() {
			t.Fatalf("bad palette entry %+v", p)
		}
	}
	if light < 10 || dark < 10 {
		t.Fatalf("got %d light and %d dark palettes, want at least 10 of each", light, dark)
	}
	if err := patina.SetPalette("Nord"); err != nil {
		t.Fatal(err)
	}
	if !patina.IsDark() || patina.CurrentPalette() != "Nord" {
		t.Fatalf("after SetPalette(Nord): dark=%v current=%q", patina.IsDark(), patina.CurrentPalette())
	}
	if err := patina.SetPalette("ocean"); err != nil {
		t.Fatal(err)
	}
	if patina.IsDark() || patina.CurrentPalette() != "Ocean" {
		t.Fatalf("after SetPalette(ocean): dark=%v current=%q", patina.IsDark(), patina.CurrentPalette())
	}
	if err := patina.SetPalette("no such palette"); err == nil {
		t.Fatal("expected an error for an unknown palette")
	}
	custom := patina.PaletteColors{
		Background: patina.Hex("#101418"), Surface: patina.Hex("#181d23"), SurfaceVariant: patina.Hex("#222932"),
		Text: patina.Hex("#f2f4f8"), TextSecondary: patina.Hex("#b8c0cc"), TextMuted: patina.Hex("#7c8794"),
		Accent: patina.Hex("#ff7a1a"), OnAccent: patina.Hex("#1a0d00"),
		Outline: patina.Hex("#2c3540"), OutlineStrong: patina.Hex("#3d4856"),
		Danger: patina.Hex("#ff5c5c"), Success: patina.Hex("#3ddc97"), Warning: patina.Hex("#ffc857"),
		Shadow: patina.Hex("#000000"),
	}
	if err := patina.DefinePalette("Test Orange", true, custom); err != nil {
		t.Fatal(err)
	}
	if err := patina.SetPalette("test orange"); err != nil {
		t.Fatal(err)
	}
	if patina.CurrentPalette() != "Test Orange" || !patina.IsDark() {
		t.Fatalf("custom palette not active: %q dark=%v", patina.CurrentPalette(), patina.IsDark())
	}
	found := false
	for _, p := range patina.Palettes() {
		if p.Name == "Test Orange" && p.Dark && p.Accent == patina.Hex("#ff7a1a") {
			found = true
		}
	}
	if !found {
		t.Fatal("custom palette missing from Palettes()")
	}
	if err := patina.DefinePalette("Bad", false, patina.PaletteColors{Accent: patina.Accent}); err == nil {
		t.Fatal("expected an error for token colors")
	}
}

func TestAnimationSpeed(t *testing.T) {
	requireCore(t)
	t.Cleanup(func() { patina.SetAnimationSpeed(1) })
	patina.SetAnimationSpeed(0.5)
	if got := patina.AnimationSpeed(); got != 0.5 {
		t.Fatalf("AnimationSpeed() = %v, want 0.5", got)
	}
	patina.ReducedMotion(true)
	if got := patina.AnimationSpeed(); got != 0 {
		t.Fatalf("ReducedMotion: AnimationSpeed() = %v, want 0", got)
	}
	patina.ReducedMotion(false)
	if got := patina.AnimationSpeed(); got != 1 {
		t.Fatalf("AnimationSpeed() = %v, want 1", got)
	}
}

// TestAnimatedWidgets renders every animated element offscreen.
func TestAnimatedWidgets(t *testing.T) {
	requireCore(t)
	win := patina.NewWindow("animations", 360, 300)
	clicked := false
	win.SetContent(
		patina.Column(
			patina.Row(
				patina.SVG(animatedTestSVG).Size(48, 48),
				patina.SVG(animatedTestSVG).Size(48, 48).Animate(false),
				patina.Icon(animatedTestSVG).Size(32, 32).Spin(time.Second).Pulse(time.Second),
				patina.Spinner(),
				patina.Spinner().Size(40, 40).Color(patina.Danger),
			).Gap(8),
			patina.Row(
				patina.Button("Glow").Glow(true),
				patina.Button("Cycle").ColorCycle(2*time.Second).Ripple(true),
				patina.Button("Gradient").Gradient(patina.Hex("#5B5FEF"), patina.Hex("#0EA5E9")),
				patina.Button("Jelly").Gelatin(),
				patina.Button("Rubber").RubberBand(),
			).Gap(8),
			patina.Card(patina.Label("card")).Lift().OnClick(func() { clicked = true }).Enter(patina.Pop).PressEffect(patina.Bounce),
			patina.Progress(0.6).Enter(patina.FadeIn),
		).Padding(12).Gap(8),
	)
	path := filepath.Join(t.TempDir(), "anim.png")
	if err := win.Snapshot(path, 360, 300, 1); err != nil {
		t.Fatalf("snapshot: %v", err)
	}
	f, err := os.Open(path)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	img, err := png.Decode(f)
	if err != nil {
		t.Fatalf("decode: %v", err)
	}
	if got := img.Bounds(); got.Dx() != 360 || got.Dy() != 300 {
		t.Fatalf("snapshot size = %dx%d", got.Dx(), got.Dy())
	}
	_ = clicked
}

func TestMenusSelectAndMotion(t *testing.T) {
	requireCore(t)
	t.Cleanup(func() { patina.SetMotion(patina.EaseMotion) })
	win := patina.NewWindow("menus", 400, 300)
	menu := patina.Menu(
		patina.MenuItem("New", nil).Shortcut("Ctrl+N"),
		patina.MenuItem("Open", nil).Icon(animatedTestSVG),
		patina.MenuSeparator(),
		patina.MenuItem("Checked", nil).Checked(true),
		patina.MenuItem("Disabled", nil).Disable(),
	)
	btn := patina.Button("File").Menu(menu).Tooltip("Open the file menu")
	sel := patina.Select("Small", "Medium", "Large").SetSelected(1)
	card := patina.Card(patina.Label("context")).ContextMenu(menu).Spring(250, 12)
	win.SetContent(patina.Column(btn, sel, card).Padding(16).Gap(8))
	if menu.IsOpen() {
		t.Fatal("menu should start closed")
	}
	menu.OpenBelow(btn)
	if !menu.IsOpen() {
		t.Fatal("menu should be open")
	}
	path := filepath.Join(t.TempDir(), "menu.png")
	if err := win.Snapshot(path, 400, 300, 1); err != nil {
		t.Fatal(err)
	}
	menu.Close()
	if menu.IsOpen() {
		t.Fatal("menu should be closed")
	}
	if sel.Selected() != 1 || sel.Value() != "Medium" {
		t.Fatalf("select: %d %q", sel.Selected(), sel.Value())
	}
	if len(sel.Menu().Items()) != 3 {
		t.Fatalf("select menu has %d items", len(sel.Menu().Items()))
	}
	patina.SetMotion(patina.BouncyMotion)
	if patina.CurrentMotion() != patina.BouncyMotion {
		t.Fatal("motion style did not round-trip")
	}
}
