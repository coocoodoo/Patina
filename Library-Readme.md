# Using Patina

Patina is a Go UI toolkit. You write ordinary Go, and a Rust core (loaded at runtime, no
cgo) does the windowing, layout, text and painting. This guide walks through everything you
need to build an application with it: setting up, building a screen, reacting to input,
styling, theming, animation, menus, images and threading.

The reference for individual methods is the Go doc comments (`go doc github.com/patina-ui/patina`).
The project README covers the architecture and the build of the core.

## 1. Setup

Requirements: Go 1.26 or newer to build your program, and Rust 1.85 or newer once to build the
core (a prebuilt core is embedded in the package when you build it with the helper below).

```bash
git clone <this repository> patina
cd patina
go run ./tools/build        # compiles the Rust core and embeds it for your OS/arch
go run ./examples/hello     # opens a window
```

In your own module:

```bash
go get github.com/patina-ui/patina
```

Nothing else is needed at runtime. The core is extracted from the embedded copy into the
user cache directory on first run. To use a core you built yourself, set `PATINA_LIBRARY` to
the path of `patina_core.dll`, `libpatina_core.so` or `libpatina_core.dylib`.

## 2. A first window

```go
package main

import (
    "log"

    "github.com/patina-ui/patina"
)

func main() {
    win := patina.NewWindow("Hello", 480, 320)

    name := patina.TextInput("Your name")
    greeting := patina.Label("")

    win.SetContent(
        patina.Column(
            patina.Title("Welcome"),
            name,
            patina.Button("Greet").OnClick(func() {
                greeting.SetText("Hello, " + name.Value() + "!")
            }),
            greeting,
        ).Padding(24).Gap(12),
    )

    if err := patina.Run(); err != nil {
        log.Fatal(err)
    }
}
```

Three rules to keep in mind:

- Build widgets with constructors (`patina.Label`, `patina.Button`, `patina.Column`) and
  configure them with chained modifiers. Every modifier returns the widget's concrete type,
  so you can keep a variable and update it later.
- Call `patina.Run()` from `main`. It blocks until the last window closes. All callbacks run
  on that thread.
- Setters are safe from any goroutine. Only widget construction and `patina.Run` belong to
  the main goroutine (see section 10 for background work).

## 3. Layout

Patina uses flexbox. Two containers cover almost everything:

```go
patina.Column(a, b, c)   // vertical; children stretch to the column's width
patina.Row(a, b, c)      // horizontal; children are vertically centered
```

Modifiers on containers:

| Modifier | Effect |
| --- | --- |
| `.Gap(px)` | space between children |
| `.Padding(px)`, `.PaddingXY(x, y)`, `.PaddingEach(l, t, r, b)` | inner spacing |
| `.Align(patina.AlignStart / AlignCenter / AlignEnd / AlignStretch)` | cross-axis alignment |
| `.Justify(patina.JustifyStart / Center / End / SpaceBetween / SpaceAround / SpaceEvenly)` | main-axis distribution |
| `.Wrap(true)` | wrap onto new lines |
| `.Add(...)`, `.Insert(i, w)`, `.Remove(w)`, `.Clear()`, `.Children()` | edit at runtime |

Sizing modifiers exist on every widget:

```go
w.Width(200).Height(40)      // fixed size in logical pixels
w.WidthPercent(50)           // percent of the parent
w.MinWidth(120).MaxWidth(400)
w.Expand()                   // flex-grow 1: take the remaining space
w.Grow(2)                    // share remaining space proportionally
w.Shrink(0)                  // never shrink below the natural size
w.AlignSelf(patina.AlignEnd)
w.Margin(8)
```

Helpers:

- `patina.Spacer()` is flexible empty space (push siblings apart in a Row).
- `patina.Fixed(px)` is a fixed gap.
- `patina.Divider()` is a hairline, horizontal in a Column and vertical in a Row.
- `patina.Card(children...)` is a Column on an elevated surface (padding, radius, border,
  shadow).
- `patina.Scroll(children...)` is a vertically scrolling column with an overlay scrollbar.
  Give it a height or let it expand; `ScrollTo(y)`, `ScrollToEnd()` and `Offset()` control it.

Logical pixels are scaled by the window's DPI automatically, so a 200px wide button is 400
device pixels on a 200% display.

## 4. Widgets

| Constructor | Notes |
| --- | --- |
| `Label(text)`, `Title(text)`, `Heading(text)`, `Caption(text)` | text; `.FontSize(px)`, `.Weight(patina.Semibold)`, `.Bold()`, `.Color(c)`, `.Mono()`, `.Align(...)`, `.Wrap(false)` |
| `Button(text)` | `.OnClick(fn)`; styles `.Primary()` (default), `.Tonal()`, `.Outlined()`, `.Ghost()`, `.Danger()`; `.Icon(svg)`, `.IconSize(px)`, `.IconEnd(true)` |
| `TextInput(placeholder)` | `.OnChange(func(string))`, `.OnSubmit(func(string))`, `.Password()`, `.MaxLength(n)`, `.Value()`, `.SetValue(s)`, `.Focus()` |
| `Checkbox(label)` | `.OnChange(func(bool))`, `.SetChecked(v)`, `.Checked()` |
| `Switch(label)` | `.OnChange(func(bool))`, `.SetOn(v)`, `.On()` |
| `Slider(min, max)` | `.Step(s)`, `.OnChange(func(float64))`, `.SetValue(v)`, `.Value()` |
| `Progress(fraction)` | `.SetValue(f)`, `.Indeterminate(true)`, `.Color(c)` |
| `Spinner()` | activity indicator; `.Size(w, h)`, `.Color(c)` |
| `Select(options...)` | dropdown; `.OnChange(func(i int, v string))`, `.SetSelected(i)`, `.Selected()`, `.Value()` |
| `Image(img)` | any `image.Image`; `.Fit(patina.Contain / Cover / Fill)`, `.SetImage(img)` |
| `SVG(markup)`, `Icon(markup)` | vector images; see section 8 |

Text inputs support selection with the mouse or Shift+arrows, word jumps with Ctrl, Ctrl+A,
Home/End and Enter for submit. Tab moves keyboard focus between interactive widgets, and
Space/Enter activate buttons and toggles.

Common modifiers on every widget:

```go
w.Visible(false)  w.Show()  w.Hide()
w.Disabled(true)  w.Enable()  w.Disable()
w.Focus()  w.HasFocus()
w.OnHover(enter, leave)  w.OnFocusChange(focus, blur)
w.Tooltip("Shown after the pointer rests on the widget")
w.Destroy()       // removes it from the tree and frees it
```

## 5. Styling

```go
box.Background(patina.Surface)          // any Color
box.Radius(12)
box.Border(1, patina.Outline)
box.Shadow(2)                           // 0..3
box.Opacity(0.8)
box.Gradient(patina.Hex("#5B5FEF"), patina.Hex("#0EA5E9"))   // horizontal gradient fill
```

Colors are either literals or theme tokens:

```go
patina.RGB(99, 102, 241)
patina.RGBA(0, 0, 0, 128)
patina.Hex("#6366F1")            // also "#6366F180" for alpha
patina.Accent, patina.OnAccent   // brand color and text on it
patina.Background, patina.Surface, patina.SurfaceVariant
patina.Text, patina.TextSecondary, patina.TextMuted
patina.Outline, patina.Danger, patina.Success, patina.Warning
```

Tokens resolve against the active palette, so a `Card` with `Background(patina.Surface)`
looks right in every theme without changes.

## 6. Themes and palettes

```go
patina.SetTheme(patina.Dark)          // Light, Dark or System (follows the OS)
patina.SetPalette("Nord")             // one of 20 built-in palettes; switches light/dark to match
patina.SetAccent(patina.Hex("#0EA5E9"))
patina.SetThemeTransition(0)          // switch instantly instead of the 280 ms cross-fade
patina.IsDark(), patina.CurrentPalette(), patina.Palettes()
```

Light palettes: Indigo, Ocean, Forest, Sunset, Rose, Lavender, Sand, Slate, Mint, Mono.
Dark palettes: Midnight, Nord, Dracula, Ocean Dark, Forest Dark, Ember, Rose Dark, Amethyst,
Graphite, Carbon.

Define your own:

```go
patina.DefinePalette("Brand", false, patina.PaletteColors{
    Background: patina.Hex("#FAFAFC"), Surface: patina.Hex("#FFFFFF"), SurfaceVariant: patina.Hex("#F0F1F5"),
    Text: patina.Hex("#14161C"), TextSecondary: patina.Hex("#5B6170"), TextMuted: patina.Hex("#9AA0AE"),
    Accent: patina.Hex("#D9480F"), OnAccent: patina.Hex("#FFFFFF"),
    Outline: patina.Hex("#E4E6EC"), OutlineStrong: patina.Hex("#CBD0DA"),
    Danger: patina.Hex("#E03131"), Success: patina.Hex("#2F9E44"), Warning: patina.Hex("#E8590C"),
    Shadow: patina.Hex("#000000"),
})
patina.SetPalette("Brand")
```

Fonts: the core uses the system UI font (Segoe UI, Helvetica or DejaVu/Noto/Liberation).
Load your own with `patina.SetFont(regular, bold, mono)` (TrueType or OpenType files).

## 7. Animation

Everything that moves is driven by one animation clock, so one call slows or disables it all:

```go
patina.SetAnimationSpeed(0.5)   // slow motion; 2 = double speed
patina.ReducedMotion(true)      // no motion: every change applies instantly
```

Motion style. Sliders, progress bars, toggles and entrances ease into place by default.
Switch them to springs, globally or per widget:

```go
patina.SetMotion(patina.SpringMotion)   // EaseMotion (default), SpringMotion, BouncyMotion
progress.Spring(250, 12)                // stiffness, damping for this widget only
```

Entrance animations play when a widget first appears or is shown again:

```go
toast := patina.Card(...).Enter(patina.Pop)   // FadeIn, SlideUp, SlideDown, SlideLeft, SlideRight, Pop
```

Buttons:

```go
patina.Button("Go").Glow(true)                                 // pulsing halo
patina.Button("Party").ColorCycle(6 * time.Second)             // hue rotation
patina.Button("Buy").Ripple(true)                              // wave from the click point
patina.Button("Jelly").Gelatin()                               // press effects: RubberBand(), Gelatin(), Bounce()
card.PressEffect(patina.Bounce)                                // on any clickable widget
```

Cards and boxes:

```go
patina.Card(...).Lift().OnClick(func() {})   // clickable, lifts with a deeper shadow on hover
```

Icons and SVG:

```go
patina.Icon(icons.Refresh).Spin(time.Second)            // rotate continuously
patina.Icon(icons.Heart).Pulse(900 * time.Millisecond)  // breathe
patina.SVG(animatedSVG)                                 // SMIL animations just play
patina.SVG(animatedSVG).Animate(false)                  // freeze on the first frame
```

Supported SMIL: `<animate>`, `<animateTransform>`, `<set>` and `<animateMotion>` with
`from`/`to`/`by`/`values`, `keyTimes`, `keySplines`, `calcMode`, `begin` offsets, `dur`,
`repeatCount`, `fill="freeze"`, `additive="sum"`, and for motion `path`, `<mpath>`,
`keyPoints` and `rotate="auto"`. Animated documents are rasterized per frame and cached on
their loop period, so a 60 fps looping icon costs one render per distinct frame.

## 8. Images and vector graphics

```go
img, _ := png.Decode(file)
patina.Image(img).Size(120, 80).Fit(patina.Cover).Radius(8)
```

SVG is rendered by the core at the exact size and scale it is displayed at:

```go
import "github.com/patina-ui/patina/icons"

patina.Icon(icons.Search).Size(20, 20).Tint(patina.TextSecondary)   // monochrome
patina.SVG(logo).Size(48, 48)                                        // keeps its own colors
patina.Button("Save").Icon(icons.Download)
patina.Button("").Ghost().Icon(icons.Settings)                       // square icon button
win.SetIconSVG(logo)                                                 // window and taskbar icon
pixels, err := patina.RenderSVG(svg, 128, 128)                       // to an *image.NRGBA
```

`currentColor` in an SVG becomes the theme's text color. `Tint(c)` recolors any SVG using
it as a mask. The `icons` package bundles 45 line icons drawn in `currentColor`. SVG `<text>`
is not rendered.

## 9. Menus, tooltips and dropdowns

```go
menu := patina.Menu(
    patina.MenuItem("New file", newFile).Icon(icons.Plus).Shortcut("Ctrl+N"),
    patina.MenuItem("Open…", open).Icon(icons.Folder).Shortcut("Ctrl+O"),
    patina.MenuSeparator(),
    patina.MenuItem("Autosave", toggleAutosave).Checked(true),
    patina.MenuItem("Export", nil).Disable(),
)

patina.Button("File").Menu(menu)        // opens below the button when clicked
box.ContextMenu(menu)                   // opens at the pointer on right click
menu.OpenBelow(anyWidget)               // or open it yourself
menu.OpenAt(win, 100, 200)              // at a point in the window
menu.Close(); menu.IsOpen(); menu.OnClose(fn)
```

Menus pop in with an entrance animation and close after a choice, on Escape, on a click
outside them, or when the window loses focus. Up/Down/Home/End move a keyboard highlight and
Enter or Space choose. `Shortcut` only shows the hint; bind keys yourself.

A `Select` is a dropdown built on a menu:

```go
size := patina.Select("Small", "Medium", "Large").SetSelected(1).
    OnChange(func(i int, v string) { fmt.Println("picked", v) })
```

Tooltips work on any widget and appear after the pointer rests on it for half a second:

```go
patina.Icon(icons.Info).Tooltip("Explains the thing next to it")
```

## 10. Events, timers and background work

Callbacks are registered per widget (`OnClick`, `OnChange`, `OnSubmit`, `OnHover`,
`OnFocusChange`) and run on the UI thread. To run code there from elsewhere:

```go
patina.Post(func() { label.SetText("done") })          // as soon as possible
patina.After(2*time.Second, func() { toast.Destroy() })  // once
ticker := patina.Every(time.Second, tick)                // repeatedly; ticker.Stop()
```

A typical background job:

```go
patina.Button("Download").OnClick(func() {
    progress.Indeterminate(true)
    go func() {
        data, err := fetch()
        patina.Post(func() {
            progress.Indeterminate(false)
            if err != nil {
                status.SetText(err.Error()).Color(patina.Danger)
                return
            }
            status.SetText(fmt.Sprintf("%d bytes", len(data)))
        })
    }()
})
```

Setters such as `SetText` or `SetValue` are already safe from any goroutine; `Post` is for
building widgets or batching several changes.

## 11. Windows

```go
win := patina.NewWindow("Title", 900, 600).MinSize(480, 320)
win.SetContent(root)
win.SetTitle("Editing " + name)
win.Resizable(false); win.Decorations(false); win.AlwaysOnTop(true); win.Maximized(true)
win.OnResize(func(w, h int) {})
win.OnFocusChange(func(focused bool) {})
win.OnCloseRequest(func() bool { return confirmDiscard() })   // false keeps it open
win.OnClose(func() {})
win.Close()
win.Snapshot("shot.png", 900, 600, 2)   // offscreen render at 2x, no event loop needed
```

Several windows can be open at once; `patina.Run` returns when the last one closes. Use
`patina.Quit()` to end early.

`Snapshot` is handy for tests and documentation: it lays out and paints the window to a PNG
without a display, with every animation settled.

## 12. Putting it together

```go
package main

import (
    "fmt"
    "log"
    "time"

    "github.com/patina-ui/patina"
    "github.com/patina-ui/patina/icons"
)

func main() {
    patina.SetPalette("Ocean")
    win := patina.NewWindow("Tasks", 640, 480).MinSize(420, 320)

    list := patina.Column().Gap(6)
    count := patina.Caption("0 tasks")
    refresh := func() { count.SetText(fmt.Sprintf("%d tasks", len(list.Children()))) }

    input := patina.TextInput("What needs doing?")
    add := func(text string) {
        if text == "" {
            return
        }
        var row *patina.Box
        row = patina.Card(
            patina.Row(
                patina.Checkbox(text),
                patina.Spacer(),
                patina.Button("").Ghost().Icon(icons.Trash).Tooltip("Remove").OnClick(func() {
                    row.Destroy()
                    refresh()
                }),
            ).Gap(8),
        ).Padding(10).Enter(patina.SlideUp).Lift()
        list.Add(row)
        input.SetValue("")
        refresh()
    }
    input.OnSubmit(add)

    theme := patina.Select("Ocean", "Sunset", "Nord", "Dracula").
        OnChange(func(_ int, name string) { patina.SetPalette(name) })

    win.SetContent(
        patina.Column(
            patina.Row(patina.Title("Tasks"), patina.Spacer(), theme).Gap(12),
            patina.Row(input, patina.Button("Add").Icon(icons.Plus).OnClick(func() { add(input.Value()) })).Gap(8),
            patina.Scroll(list).Expand(),
            count,
        ).Padding(20).Gap(12),
    )

    patina.After(500*time.Millisecond, func() { input.Focus() })
    if err := patina.Run(); err != nil {
        log.Fatal(err)
    }
}
```

## 13. Troubleshooting

- **"native core unavailable"**: run `go run ./tools/build` once so the core is embedded, or
  point `PATINA_LIBRARY` at a built core.
- **Nothing shows up in a Column**: a `Scroll` or `Spacer` with no siblings takes all the
  space; give fixed heights or use `Expand` deliberately.
- **A widget never animates in**: it needs to be added to a window that is showing; `Enter`
  plays on the frame after layout.
- **Menu opens in the wrong place**: the anchor must be inside a window; `OpenAt` coordinates
  are logical pixels relative to the window's client area.
- **Fonts look wrong**: set `PATINA_FONT`, `PATINA_FONT_BOLD` and `PATINA_FONT_MONO` to font
  files, or call `patina.SetFont`.
- **Smoke tests**: set `PATINA_AUTO_QUIT_MS=3000` to have the event loop exit on its own.

Examples in the repository: `examples/hello`, `examples/showcase` (most widgets),
`examples/animations` (animation, palettes, menus, tooltips), `examples/icons`, `examples/events`.
