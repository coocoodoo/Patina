// Icons: a gallery of the bundled icon set, tinting, sizing, multi-color SVG and icon buttons.
//
//	go run ./examples/icons
//	go run ./examples/icons -snapshot icons.png
package main

import (
	"flag"
	"fmt"
	"log"

	"github.com/patina-ui/patina"
	"github.com/patina-ui/patina/icons"
)

// A multi-color SVG with a gradient: rendered as-is by patina.SVG (no tint).
const logo = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0" stop-color="#5B5FEF"/><stop offset="1" stop-color="#0EA5E9"/>
  </linearGradient></defs>
  <circle cx="32" cy="32" r="28" fill="url(#g)"/>
  <path d="M20 34l8 8 16-18" stroke="#fff" stroke-width="6" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`

func main() {
	snapshot := flag.String("snapshot", "", "render the window to this PNG file and exit")
	flag.Parse()

	win := patina.NewWindow("Patina icons", 780, 640).SetIconSVG(logo)

	grid := patina.Row().Wrap(true).Gap(8).Align(patina.AlignStart)
	for _, ic := range icons.All {
		grid.Add(
			patina.Column(
				patina.Icon(ic.SVG).Size(24, 24),
				patina.Caption(ic.Name),
			).Align(patina.AlignCenter).Gap(6).Width(86).PaddingXY(0, 6),
		)
	}

	sizes := patina.Row(
		patina.Icon(icons.Heart).Size(16, 16).Tint(patina.Danger),
		patina.Icon(icons.Heart).Size(24, 24).Tint(patina.Danger),
		patina.Icon(icons.Heart).Size(40, 40).Tint(patina.Danger),
		patina.Icon(icons.Star).Size(40, 40).Tint(patina.Warning),
		patina.Icon(icons.Check).Size(40, 40).Tint(patina.Success),
		patina.Icon(icons.Zap).Size(40, 40).Tint(patina.Accent),
		patina.SVG(logo).Size(40, 40),
		patina.SVG(logo).Size(40, 40).Radius(8).Background(patina.SurfaceVariant).Padding(4),
	).Gap(16)

	buttons := patina.Row(
		patina.Button("Download").Icon(icons.Download),
		patina.Button("Refresh").Tonal().Icon(icons.Refresh),
		patina.Button("Edit").Outlined().Icon(icons.Edit),
		patina.Button("").Ghost().Icon(icons.Settings),
		patina.Button("").Outlined().Icon(icons.Plus),
		patina.Button("Delete").Danger().Icon(icons.Trash),
	).Gap(10)

	win.SetContent(
		patina.Scroll(
			patina.Heading(fmt.Sprintf("%d bundled icons", len(icons.All))),
			patina.Caption("Line icons on a 24px grid, drawn in currentColor so they follow the theme."),
			grid,
			patina.Divider(),
			patina.Heading("Sizes, tints and multi-color SVG"),
			sizes,
			patina.Heading("Icons on buttons"),
			buttons,
		).Gap(14).Padding(24),
	)

	if *snapshot != "" {
		if err := win.Snapshot(*snapshot, 780, 900, 1); err != nil {
			log.Fatal(err)
		}
		fmt.Println("wrote", *snapshot)
		return
	}
	if err := patina.Run(); err != nil {
		log.Fatal(err)
	}
}
