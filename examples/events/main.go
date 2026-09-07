// Events: prints every callback the toolkit delivers while you click, type and hover.
package main

import (
	"fmt"
	"log"
	"time"

	"github.com/patina-ui/patina"
)

func main() {
	logf := func(format string, args ...any) {
		fmt.Printf("%s "+format+"\n", append([]any{time.Now().Format("15:04:05.000")}, args...)...)
	}

	win := patina.NewWindow("Patina events", 520, 360).
		OnFocusChange(func(f bool) { logf("window focus=%v", f) }).
		OnResize(func(w, h int) { logf("window resize %dx%d", w, h) }).
		OnClose(func() { logf("window closed") })

	button := patina.Button("Click me").
		OnClick(func() { logf("button click") }).
		OnHover(func() { logf("button hover enter") }, func() { logf("button hover leave") }).
		OnFocusChange(func() { logf("button focus") }, func() { logf("button blur") })
	check := patina.Checkbox("Check me").OnChange(func(on bool) { logf("checkbox %v", on) })
	toggle := patina.Switch("Switch me").OnChange(func(on bool) { logf("switch %v", on) })
	input := patina.TextInput("Type here").
		OnChange(func(v string) { logf("input change %q", v) }).
		OnSubmit(func(v string) { logf("input submit %q", v) })
	slider := patina.Slider(0, 100).Step(1).OnChange(func(v float64) { logf("slider %.0f", v) })

	win.SetContent(patina.Column(button, check, toggle, input, slider).Padding(24).Gap(16))
	patina.OnStart(func() { logf("started") })

	if err := patina.Run(); err != nil {
		log.Fatal(err)
	}
}
