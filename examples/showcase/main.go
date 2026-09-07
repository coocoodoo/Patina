// Showcase: every widget, theming, timers and background work.
//
//	go run ./examples/showcase                 # interactive
//	go run ./examples/showcase -snapshot out.png   # render offscreen and exit
package main

import (
	"flag"
	"fmt"
	"log"
	"time"

	"github.com/patina-ui/patina"
	"github.com/patina-ui/patina/icons"
)

func main() {
	snapshot := flag.String("snapshot", "", "render the window to this PNG file and exit")
	dark := flag.Bool("dark", false, "start in dark mode")
	flag.Parse()

	if *dark {
		patina.SetTheme(patina.Dark)
	}

	win := patina.NewWindow("Patina showcase", 980, 720).MinSize(640, 480).SetIconSVG(logo)

	// ---- header ---------------------------------------------------------------
	darkMode := patina.Switch("Dark mode").SetOn(*dark).OnChange(func(on bool) {
		if on {
			patina.SetTheme(patina.Dark)
		} else {
			patina.SetTheme(patina.Light)
		}
	})
	accent := func(name string, c patina.Color) patina.Widget {
		return patina.Button(name).Ghost().OnClick(func() { patina.SetAccent(c) })
	}
	palettes := patina.Row().Wrap(true).Gap(6)
	for _, p := range patina.Palettes() {
		p := p
		palettes.Add(patina.Button(p.Name).Tonal().FontSize(12).Background(p.Accent).Color(p.OnAccent).
			OnClick(func() {
				_ = patina.SetPalette(p.Name)
				darkMode.SetOn(p.Dark)
			}))
	}
	header := patina.Row(
		patina.Column(
			patina.Label("Patina").FontSize(26).Bold(),
			patina.Label("Beautiful UIs for Go, rendered by Rust.").Color(patina.TextSecondary),
		).Gap(4),
		patina.Spacer(),
		accent("Indigo", patina.ColorDefault),
		accent("Sky", patina.Hex("#0EA5E9")),
		accent("Emerald", patina.Hex("#10B981")),
		accent("Rose", patina.Hex("#F43F5E")),
		patina.Button("").Ghost().Icon(icons.Settings),
		darkMode,
	).Gap(10)

	// ---- sign-in card ---------------------------------------------------------
	greeting := patina.Label("Type your name and press Enter.").Color(patina.TextSecondary)
	name := patina.TextInput("Your name").
		OnChange(func(v string) {
			if v == "" {
				greeting.SetText("Type your name and press Enter.")
			} else {
				greeting.SetText("Hello, " + v + "!")
			}
		}).
		OnSubmit(func(v string) { greeting.SetText("Submitted: " + v) })
	password := patina.TextInput("Password").Password()
	remember := patina.Checkbox("Remember me").SetChecked(true)
	terms := patina.Checkbox("I agree to the terms")
	status := patina.Caption("")
	signIn := patina.Card(
		patina.Heading("Sign in"),
		greeting,
		name,
		password,
		remember,
		terms,
		patina.Row(
			status,
			patina.Spacer(),
			patina.Button("Cancel").Ghost().OnClick(func() {
				name.SetValue("")
				password.SetValue("")
				status.SetText("Cleared.")
			}),
			patina.Button("Continue").OnClick(func() {
				if !terms.Checked() {
					status.SetText("Please accept the terms first.").Color(patina.Danger)
					return
				}
				status.SetText("Signed in as " + name.Value()).Color(patina.Success)
			}),
		).Gap(10),
	).Expand()

	// ---- preferences card -----------------------------------------------------
	volumeLabel := patina.Caption("Volume: 62%").Semibold()
	volume := patina.Slider(0, 100).SetValue(62).Step(1).OnChange(func(v float64) {
		volumeLabel.SetText(fmt.Sprintf("Volume: %.0f%%", v))
	})
	upload := patina.Progress(0)
	uploadLabel := patina.Caption("Upload: idle").Semibold()
	var uploadBtn *patina.ButtonWidget
	uploadBtn = patina.Button("Simulate upload").Tonal().Icon(icons.Upload).OnClick(func() {
		uploadBtn.Disable()
		go func() { // background work reports back through Post
			for i := 0; i <= 100; i += 4 {
				time.Sleep(60 * time.Millisecond)
				frac := float64(i) / 100
				patina.Post(func() {
					upload.SetValue(frac)
					uploadLabel.SetText(fmt.Sprintf("Upload: %.0f%%", frac*100))
				})
			}
			patina.Post(func() {
				uploadLabel.SetText("Upload: done")
				uploadBtn.Enable()
			})
		}()
	})
	uptime := patina.Caption("Uptime: 0s")
	start := time.Now()
	patina.Every(time.Second, func() {
		uptime.SetText("Uptime: " + time.Since(start).Round(time.Second).String())
	})
	prefs := patina.Card(
		patina.Heading("Preferences"),
		patina.Switch("Notifications").SetOn(true),
		patina.Switch("Sync over cellular"),
		patina.Divider(),
		volumeLabel,
		volume,
		uploadLabel,
		upload,
		patina.Progress(0).Indeterminate(true),
		patina.Row(uploadBtn, patina.Button("Disabled").Disable(), patina.Spacer(), uptime).Gap(10),
	).Expand()

	// ---- activity list --------------------------------------------------------
	list := patina.Scroll().Gap(6).Height(170)
	counter := 0
	addItem := func() {
		counter++
		var row *patina.Box
		row = patina.Row(
			patina.Icon(icons.File).Size(18, 18).Tint(patina.TextSecondary),
			patina.Label(fmt.Sprintf("Item %d", counter)).Weight(patina.Medium),
			patina.Caption("Synced a few minutes ago"),
			patina.Spacer(),
			patina.Button("Remove").Ghost().OnClick(func() { row.Destroy() }),
		).Gap(10)
		list.Add(row)
	}
	for i := 0; i < 8; i++ {
		addItem()
	}
	activity := patina.Card(
		patina.Row(
			patina.Heading("Activity"),
			patina.Spacer(),
			patina.Button("Add item").Outlined().Icon(icons.Plus).OnClick(func() { addItem(); list.ScrollToEnd() }),
			patina.Button("Clear").Danger().Icon(icons.Trash).OnClick(func() { list.Clear() }),
		).Gap(10),
		list,
	)

	win.SetContent(
		patina.Column(
			header,
			palettes,
			patina.Row(signIn, prefs).Gap(20).Align(patina.AlignStretch),
			activity,
		).Padding(28).Gap(20),
	)
	name.Focus()

	if *snapshot != "" {
		if err := win.Snapshot(*snapshot, 980, 720, 1); err != nil {
			log.Fatal(err)
		}
		fmt.Println("wrote", *snapshot)
		return
	}
	if err := patina.Run(); err != nil {
		log.Fatal(err)
	}
}

// logo is the window icon: a gradient badge that stays visible on light and dark title bars.
const logo = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0" stop-color="#5B5FEF"/><stop offset="1" stop-color="#0EA5E9"/>
  </linearGradient></defs>
  <rect x="4" y="4" width="56" height="56" rx="16" fill="url(#g)"/>
  <path d="M35 12 20 36h12l-3 16 15-24H32z" fill="#fff"/>
</svg>`
