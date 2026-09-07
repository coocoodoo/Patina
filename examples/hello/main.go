// A minimal Patina program: a window with a counter.
package main

import (
	"fmt"
	"log"

	"github.com/patina-ui/patina"
)

func main() {
	win := patina.NewWindow("Hello, Patina", 480, 320)

	count := 0
	value := patina.Title("0").Align(patina.TextCenter).Expand()
	update := func(delta int) {
		count += delta
		value.SetText(fmt.Sprint(count))
	}

	win.SetContent(
		patina.Column(
			patina.Heading("Counter"),
			patina.Caption("A Go UI, drawn by a Rust core."),
			patina.Card(
				patina.Row(
					patina.Button("−").Tonal().OnClick(func() { update(-1) }),
					value,
					patina.Button("+").OnClick(func() { update(+1) }),
				).Gap(16),
			),
			patina.Row(
				patina.Spacer(),
				patina.Button("Reset").Ghost().OnClick(func() { update(-count) }),
			),
		).Padding(24).Gap(12),
	)

	if err := patina.Run(); err != nil {
		log.Fatal(err)
	}
}
