package patina

import (
	"math"
	"sync"
	"sync/atomic"
	"time"

	"github.com/patina-ui/patina/internal/native"
)

// node is the Go-side shadow of a native node: its id, tree links and handlers.
type node struct {
	id        uint64
	kind      uint32
	parent    *node
	children  []*node
	widget    Widget
	destroyed bool

	onClick       func()
	onText        func(string)
	onSubmit      func(string)
	onToggle      func(bool)
	onValue       func(float64)
	onFocus       func()
	onBlur        func()
	onHoverEnter  func()
	onHoverLeave  func()
	onClose       func()
	onCloseReq    func() bool
	onResize      func(width, height int)
	onWindowFocus func(bool)
}

var registry = struct {
	mu sync.RWMutex
	m  map[uint64]*node
}{m: map[uint64]*node{}}

func newNode(kind uint32, w Widget) *node {
	mustInit()
	var id uint64
	if kind == native.KindWindow {
		id = native.WindowNew()
	} else {
		id = native.NodeNew(kind)
	}
	if id == 0 {
		panic("patina: the native core refused to create a node")
	}
	n := &node{id: id, kind: kind, widget: w}
	registry.mu.Lock()
	registry.m[id] = n
	registry.mu.Unlock()
	return n
}

func lookupNode(id uint64) *node {
	registry.mu.RLock()
	n := registry.m[id]
	registry.mu.RUnlock()
	return n
}

func (n *node) forget() {
	delete(registry.m, n.id)
	n.destroyed = true
	for _, c := range n.children {
		c.forget()
	}
	n.children = nil
}

// destroy frees the node and its whole subtree on both sides.
func (n *node) destroy() {
	if n.destroyed {
		return
	}
	n.detach()
	registry.mu.Lock()
	n.forget()
	registry.mu.Unlock()
	native.NodeFree(n.id)
}

func (n *node) detach() {
	p := n.parent
	if p == nil {
		return
	}
	for i, c := range p.children {
		if c == n {
			p.children = append(p.children[:i], p.children[i+1:]...)
			break
		}
	}
	n.parent = nil
}

// adopt links child under n at index (or at the end when index < 0) and mirrors it natively.
func (n *node) adopt(child *node, index int) {
	if child == nil || child == n || child.destroyed {
		return
	}
	child.detach()
	child.parent = n
	if index < 0 || index >= len(n.children) {
		n.children = append(n.children, child)
		native.NodeAppend(n.id, child.id)
		return
	}
	n.children = append(n.children, nil)
	copy(n.children[index+1:], n.children[index:])
	n.children[index] = child
	native.NodeInsert(n.id, child.id, index)
}

// clear destroys every child.
func (n *node) clear() {
	kids := n.children
	n.children = nil
	registry.mu.Lock()
	for _, c := range kids {
		c.parent = nil
		c.forget()
	}
	registry.mu.Unlock()
	native.NodeClear(n.id)
}

// dispatch is the single entry point for events coming from the core (UI thread).
func dispatch(id uint64, ev uint32, a, b int64, text string) {
	switch ev {
	case native.EvTimer:
		fireTimer(uint64(a))
		return
	case native.EvPosted:
		firePost(uint64(a))
		return
	case native.EvStarted:
		if startHook != nil {
			startHook()
		}
		return
	}
	n := lookupNode(id)
	if n == nil {
		return
	}
	switch ev {
	case native.EvClick:
		if n.onClick != nil {
			n.onClick()
		}
	case native.EvTextChanged:
		if n.onText != nil {
			n.onText(text)
		}
	case native.EvSubmit:
		if n.onSubmit != nil {
			n.onSubmit(text)
		}
	case native.EvToggled:
		if n.onToggle != nil {
			n.onToggle(a != 0)
		}
	case native.EvValueChanged:
		if n.onValue != nil {
			n.onValue(math.Float64frombits(uint64(a)))
		}
	case native.EvFocus:
		if n.onFocus != nil {
			n.onFocus()
		}
	case native.EvBlur:
		if n.onBlur != nil {
			n.onBlur()
		}
	case native.EvHoverEnter:
		if n.onHoverEnter != nil {
			n.onHoverEnter()
		}
	case native.EvHoverLeave:
		if n.onHoverLeave != nil {
			n.onHoverLeave()
		}
	case native.EvMenuClosed:
		if n.onClose != nil {
			n.onClose()
		}
	case native.EvWindowClose:
		allowed := true
		if n.onCloseReq != nil {
			allowed = n.onCloseReq()
			if allowed {
				native.WindowClose(n.id)
			}
		}
		if allowed && n.onClose != nil {
			n.onClose()
		}
	case native.EvWindowResized:
		if n.onResize != nil {
			n.onResize(int(a), int(b))
		}
	case native.EvWindowFocus:
		if n.onWindowFocus != nil {
			n.onWindowFocus(a != 0)
		}
	}
}

// ---- timers and cross-goroutine posting ---------------------------------------------------

var (
	tokenSeq atomic.Uint64
	timers   sync.Map // token -> *Timer
	posts    sync.Map // token -> func()
)

// Timer is a pending After or Every callback.
type Timer struct {
	token   uint64
	fn      func()
	repeat  bool
	stopped atomic.Bool
}

// Stop cancels the timer. It is safe to call more than once and from any goroutine.
func (t *Timer) Stop() {
	if t != nil && t.stopped.CompareAndSwap(false, true) {
		timers.Delete(t.token)
		native.TimerCancel(t.token)
	}
}

func startTimer(d time.Duration, repeat bool, fn func()) *Timer {
	mustInit()
	ms := uint64(d / time.Millisecond)
	if ms == 0 {
		ms = 1
	}
	t := &Timer{token: tokenSeq.Add(1), fn: fn, repeat: repeat}
	timers.Store(t.token, t)
	native.TimerStart(t.token, ms, repeat)
	return t
}

// After runs fn on the UI thread once, after d.
func After(d time.Duration, fn func()) *Timer { return startTimer(d, false, fn) }

// Every runs fn on the UI thread repeatedly, every d, until the returned Timer is stopped.
func Every(d time.Duration, fn func()) *Timer { return startTimer(d, true, fn) }

func fireTimer(token uint64) {
	v, ok := timers.Load(token)
	if !ok {
		return
	}
	t := v.(*Timer)
	if t.stopped.Load() {
		return
	}
	if !t.repeat {
		timers.Delete(token)
	}
	t.fn()
}

// Post schedules fn to run on the UI thread as soon as possible. Use it to apply the results
// of background work; widget setters are already safe from any goroutine, so Post is mainly
// useful for batching several updates or for building new widgets.
func Post(fn func()) {
	mustInit()
	token := tokenSeq.Add(1)
	posts.Store(token, fn)
	native.Post(token)
}

func firePost(token uint64) {
	if v, ok := posts.LoadAndDelete(token); ok {
		v.(func())()
	}
}
