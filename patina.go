// Package patina is a toolkit for building beautiful desktop user interfaces in Go.
//
// Windows, layout, text and drawing are handled by patina-core, a Rust library that is loaded
// at runtime (no cgo needed). The Go API is a small retained widget tree with chainable
// modifiers:
//
//	win := patina.NewWindow("Hello", 480, 320)
//	win.SetContent(
//		patina.Column(
//			patina.Heading("Hello, Patina"),
//			patina.Button("Click me").OnClick(func() { fmt.Println("clicked") }),
//		).Padding(24).Gap(12),
//	)
//	patina.Run()
//
// Run must be called from the main goroutine; package init locks it to the main OS thread.
// Widget methods may be called from any goroutine, and all callbacks run on the UI thread.
package patina

import (
	"runtime"
	"sync"

	"github.com/patina-ui/patina/internal/native"
)

func init() {
	// The event loop has to live on the main thread (a hard requirement on macOS).
	runtime.LockOSThread()
}

var (
	initOnce sync.Once
	initErr  error
)

// Init loads the native core. It runs automatically on first use; call it explicitly to
// handle a missing or incompatible library without a panic.
func Init() error {
	initOnce.Do(func() {
		initErr = native.Load()
		if initErr == nil {
			native.SetEventHandler(dispatch)
		}
	})
	return initErr
}

func mustInit() {
	if err := Init(); err != nil {
		panic(err)
	}
}

// Run shows the windows created so far and runs the event loop. It returns when the last
// window has been closed or Quit was called. Call it once, from the main goroutine.
func Run() error {
	mustInit()
	return native.Run()
}

// Quit asks the event loop to stop. Safe from any goroutine.
func Quit() {
	mustInit()
	native.Quit()
}

// IsRunning reports whether the event loop is active.
func IsRunning() bool {
	mustInit()
	return native.IsRunning()
}

// LibraryPath returns the location of the native core that was loaded.
func LibraryPath() string {
	mustInit()
	return native.LibraryPath
}

var startHook func()

// OnStart registers a function that runs on the UI thread as soon as the event loop starts.
func OnStart(fn func()) {
	startHook = fn
}
