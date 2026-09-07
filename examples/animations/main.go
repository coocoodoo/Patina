// Animations: animated SVG (SMIL), spinning and pulsing icons, glowing and color-cycling
// buttons, ripples, smooth progress, entrance animations, hover-lift cards, the twenty
// built-in palettes with cross-fades, and the global animation speed.
//
//	go run ./examples/animations
//	go run ./examples/animations -snapshot anim.png
package main

import (
	"flag"
	"fmt"
	"log"
	"time"

	"github.com/patina-ui/patina"
	"github.com/patina-ui/patina/icons"
)

// A loader drawn with SMIL: a rotating arc, a pulsing dot and a color-shifting ring.
const loaderSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none" stroke-linecap="round">
  <circle cx="24" cy="24" r="18" stroke="currentColor" stroke-opacity="0.15" stroke-width="4"/>
  <path d="M24 6a18 18 0 0 1 18 18" stroke="currentColor" stroke-width="4">
    <animateTransform attributeName="transform" type="rotate" from="0 24 24" to="360 24 24" dur="1s" repeatCount="indefinite"/>
  </path>
  <circle cx="24" cy="24" r="4" fill="currentColor">
    <animate attributeName="r" values="3;6;3" dur="1.2s" repeatCount="indefinite"/>
    <animate attributeName="fill-opacity" values="1;0.4;1" dur="1.2s" repeatCount="indefinite"/>
  </circle>
</svg>`

// A check mark that draws itself, then a ring that changes color.
const drawSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none" stroke-width="4" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="24" cy="24" r="20" stroke="#5B5FEF">
    <animate attributeName="stroke" values="#5B5FEF;#0EA5E9;#10B981;#5B5FEF" dur="4s" repeatCount="indefinite"/>
  </circle>
  <path d="M14 25l7 7 13-14" stroke="#10B981" stroke-dasharray="40" stroke-dashoffset="40">
    <animate attributeName="stroke-dashoffset" values="40;0;0;40" keyTimes="0;0.4;0.7;1" dur="2.4s" repeatCount="indefinite"/>
  </path>
</svg>`

// A blob whose shape morphs by animating polygon points.
const morphSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48">
  <polygon fill="#F43F5E" points="24,4 44,24 24,44 4,24">
    <animate attributeName="points" values="24,4 44,24 24,44 4,24; 8,8 40,8 40,40 8,40; 24,4 44,24 24,44 4,24" dur="3s" calcMode="spline" keySplines="0.42 0 0.58 1;0.42 0 0.58 1" repeatCount="indefinite"/>
    <animate attributeName="fill" values="#F43F5E;#7C3AED;#F43F5E" dur="3s" repeatCount="indefinite"/>
  </polygon>
</svg>`

// A planet on an elliptical orbit and a comet that turns with its path (animateMotion).
const orbitSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48" fill="none">
  <ellipse cx="24" cy="24" rx="20" ry="11" stroke="currentColor" stroke-opacity="0.25" stroke-width="1"/>
  <circle cx="24" cy="24" r="6" fill="#F59E0B"/>
  <circle r="3" fill="#0EA5E9">
    <animateMotion dur="3s" repeatCount="indefinite" path="M4 24 A20 11 0 1 1 44 24 A20 11 0 1 1 4 24"/>
  </circle>
  <path d="M-4 0 L2 -2 L0 0 L2 2 Z" fill="#F43F5E">
    <animateMotion dur="5s" repeatCount="indefinite" rotate="auto" path="M6 40 C 14 8, 34 8, 42 40 C 34 30, 14 30, 6 40"/>
  </path>
</svg>`

func main() {
	snapshot := flag.String("snapshot", "", "render the window to this PNG file and exit")
	withMenu := flag.Bool("menu", false, "open the File menu and snapshot the whole page")
	scrollTo := flag.Float64("scroll", 0, "initial scroll offset in logical pixels")
	flag.Parse()

	win := patina.NewWindow("Patina animations", 960, 760).MinSize(720, 480)

	// ---- palettes ------------------------------------------------------------
	current := patina.Label("Palette: " + patina.CurrentPalette()).Weight(patina.Medium)
	swatches := patina.Row().Wrap(true).Gap(8)
	for _, p := range patina.Palettes() {
		p := p
		label := p.Name
		swatches.Add(
			patina.Button(label).Outlined().
				Background(p.Accent).Color(p.OnAccent).
				OnClick(func() {
					patina.SetPalette(p.Name)
					current.SetText("Palette: " + p.Name)
				}),
		)
	}
	speed := patina.Caption("Animation speed: 1.0x")
	speedSlider := patina.Slider(0, 3).SetValue(1).Step(0.1).OnChange(func(v float64) {
		patina.SetAnimationSpeed(v)
		speed.SetText(fmt.Sprintf("Animation speed: %.1fx", v))
	})
	palettes := patina.Card(
		patina.Row(patina.Heading("Themes"), patina.Spacer(), current).Gap(10),
		patina.Caption("Ten light and ten dark palettes; switching cross-fades every color."),
		swatches,
		patina.Divider(),
		speed,
		speedSlider,
	)

	// ---- animated vectors ------------------------------------------------------
	vectors := patina.Card(
		patina.Heading("Animated SVG and icons"),
		patina.Caption("SMIL <animate>, <animateTransform> and <set> run natively; Spin and Pulse animate any icon."),
		patina.Row(
			patina.SVG(loaderSVG).Size(56, 56),
			patina.SVG(drawSVG).Size(56, 56),
			patina.SVG(morphSVG).Size(56, 56),
			patina.SVG(orbitSVG).Size(56, 56).Tooltip("<animateMotion> along an elliptical path, with rotate=\"auto\""),
			patina.Fixed(12),
			patina.Icon(icons.Refresh).Size(32, 32).Tint(patina.Accent).Spin(1200*time.Millisecond).Tooltip("Icon.Spin(1.2s)"),
			patina.Icon(icons.Settings).Size(32, 32).Spin(4*time.Second),
			patina.Icon(icons.Heart).Size(32, 32).Tint(patina.Danger).Pulse(900*time.Millisecond).Tooltip("Icon.Pulse(0.9s)"),
			patina.Icon(icons.Bell).Size(32, 32).Tint(patina.Warning).Pulse(2*time.Second),
			patina.Fixed(12),
			patina.Spinner(),
			patina.Spinner().Size(36, 36).Color(patina.Success),
			patina.Spinner().Size(48, 48).Color(patina.Danger),
		).Gap(18),
	)

	// ---- buttons ----------------------------------------------------------------
	clicks := patina.Caption("Ripple buttons show a wave from the click point; rubber band, gelatin and bounce buttons deform when clicked.")
	buttons := patina.Card(
		patina.Heading("Buttons"),
		patina.Row(
			patina.Button("Glow").Glow(true).Icon(icons.Zap),
			patina.Button("Color cycle").ColorCycle(6*time.Second),
			patina.Button("Gradient").Gradient(patina.Hex("#5B5FEF"), patina.Hex("#0EA5E9")),
			patina.Button("Ripple").Tonal().Ripple(true).OnClick(func() { clicks.SetText("Rippled at " + time.Now().Format("15:04:05")) }),
			patina.Button("All of it").Glow(true).ColorCycle(4*time.Second).Ripple(true).Icon(icons.Star),
			patina.Button("Rubber band").Outlined().RubberBand(),
			patina.Button("Gelatin").Tonal().Gelatin(),
			patina.Button("Bounce").Bounce(),
		).Gap(12).Wrap(true),
		clicks,
	)

	// ---- values and entrances -------------------------------------------------------
	progress := patina.Progress(0.3)
	slider := patina.Slider(0, 100).SetValue(30)
	setBoth := func(v float64) func() {
		return func() {
			progress.SetValue(v / 100)
			slider.SetValue(v)
		}
	}
	toasts := patina.Column().Gap(8)
	toastCount := 0
	showToast := func(enter patina.Enter, name string) {
		toastCount++
		var toast *patina.Box
		toast = patina.Card(
			patina.Row(
				patina.Icon(icons.Info).Size(18, 18).Tint(patina.Accent),
				patina.Label(fmt.Sprintf("Toast %d entered with %s", toastCount, name)),
				patina.Spacer(),
				patina.Button("").Ghost().Icon(icons.Close).OnClick(func() { toast.Destroy() }),
			).Gap(10),
		).Padding(10).Enter(enter)
		toasts.Add(toast)
		patina.After(4*time.Second, func() {
			if !toast.IsDestroyed() {
				toast.Destroy()
			}
		})
	}
	values := patina.Card(
		patina.Heading("Smooth values and entrances"),
		patina.Caption("Programmatic value changes ease into place; new widgets can fade, slide or pop in."),
		progress,
		slider,
		patina.Row(
			patina.Button("10%").Tonal().OnClick(setBoth(10)),
			patina.Button("50%").Tonal().OnClick(setBoth(50)),
			patina.Button("100%").Tonal().OnClick(setBoth(100)),
			patina.Spacer(),
			patina.Button("Fade").Outlined().OnClick(func() { showToast(patina.FadeIn, "FadeIn") }),
			patina.Button("Slide").Outlined().OnClick(func() { showToast(patina.SlideUp, "SlideUp") }),
			patina.Button("Pop").Outlined().OnClick(func() { showToast(patina.Pop, "Pop") }),
		).Gap(10).Wrap(true),
		toasts,
	)

	// ---- menus, tooltips and motion --------------------------------------------------
	status := patina.Caption("Pick something from a menu, or right-click the box.")
	report := func(what string) func() { return func() { status.SetText("Selected: " + what) } }
	autosave := patina.MenuItem("Autosave", nil).Checked(true)
	autosave.OnSelect(func() {
		autosave.Checked(!autosave.IsChecked())
		status.SetText(fmt.Sprintf("Autosave: %v", autosave.IsChecked()))
	})
	fileMenu := patina.Menu(
		patina.MenuItem("New file", report("New file")).Icon(icons.Plus).Shortcut("Ctrl+N"),
		patina.MenuItem("Open…", report("Open")).Icon(icons.Folder).Shortcut("Ctrl+O"),
		patina.MenuItem("Save", report("Save")).Icon(icons.Download).Shortcut("Ctrl+S"),
		patina.MenuSeparator(),
		autosave,
		patina.MenuItem("Export (soon)", nil).Disable(),
		patina.MenuSeparator(),
		patina.MenuItem("Quit", func() { patina.Quit() }).Icon(icons.Close).Shortcut("Ctrl+Q"),
	)
	contextMenu := patina.Menu(
		patina.MenuItem("Cut", report("Cut")).Shortcut("Ctrl+X"),
		patina.MenuItem("Copy", report("Copy")).Shortcut("Ctrl+C"),
		patina.MenuItem("Paste", report("Paste")).Shortcut("Ctrl+V"),
	)
	motion := patina.Select("Ease", "Spring", "Bouncy").
		OnChange(func(i int, _ string) { patina.SetMotion(patina.Motion(i)) }).
		Tooltip("How sliders, progress bars, toggles and entrances move")
	fileButton := patina.Button("File").Outlined().Icon(icons.ChevronDown).IconEnd(true).Menu(fileMenu).Tooltip("Button.Menu(menu) opens it below the button")
	menus := patina.Card(
		patina.Heading("Menus, tooltips and motion"),
		patina.Caption("Popup menus with icons, shortcuts and keyboard navigation; a Select; a right-click context menu; hover anything for a tooltip."),
		patina.Row(
			fileButton,
			patina.Label("Motion:"),
			motion,
			patina.Spacer(),
			patina.Card(patina.Label("Right-click me")).Padding(10).ContextMenu(contextMenu).Tooltip("Base.ContextMenu(menu)"),
		).Gap(12).Wrap(true),
		status,
	)

	// ---- hover-lift cards -----------------------------------------------------------
	picked := patina.Caption("Click a card.")
	card := func(title, body string, icon string) patina.Widget {
		return patina.Card(
			patina.Row(patina.Icon(icon).Size(22, 22).Tint(patina.Accent), patina.Label(title).Semibold()).Gap(10),
			patina.Caption(body),
		).Lift().Expand().OnClick(func() { picked.SetText("Picked: " + title) })
	}
	cards := patina.Column(
		patina.Heading("Clickable cards with hover lift"),
		patina.Row(
			card("Documents", "Lifted with a deeper shadow on hover.", icons.Folder),
			card("Messages", "Cards report clicks like buttons.", icons.Message),
			card("Settings", "Everything animates at the global speed.", icons.Settings),
		).Gap(16).Align(patina.AlignStretch),
		picked,
	).Gap(12)

	page := patina.Scroll(palettes, vectors, buttons, values, menus, cards).Gap(20).Padding(28)
	win.SetContent(page)
	if *scrollTo > 0 {
		page.ScrollTo(*scrollTo)
	}

	if *snapshot != "" {
		height := 760
		if *withMenu {
			fileMenu.OpenBelow(fileButton)
			height = 1560
		}
		if err := win.Snapshot(*snapshot, 960, height, 1); err != nil {
			log.Fatal(err)
		}
		fmt.Println("wrote", *snapshot)
		return
	}
	if err := patina.Run(); err != nil {
		log.Fatal(err)
	}
}
