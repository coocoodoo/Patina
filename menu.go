package patina

import (
	"fmt"
	"os"

	"github.com/patina-ui/patina/internal/native"
)

// ---- Menu ------------------------------------------------------------------------------

// MenuWidget is a popup list of MenuItems and MenuSeparators. It is not part of the layout
// tree: open it below a widget with OpenBelow (or Button.Menu), at a point with OpenAt, or
// attach it to a widget as its right-click ContextMenu. Choosing an item calls its handler
// and closes the menu; Escape, a click outside it, or Close dismiss it. Up/Down/Home/End
// move a keyboard highlight and Enter or Space choose the highlighted item.
type MenuWidget struct{ Base[MenuWidget] }

// Menu creates a popup menu with the given items.
func Menu(items ...Widget) *MenuWidget {
	m := &MenuWidget{}
	m.attach(m, native.KindMenu)
	m.Add(items...)
	return m
}

// Add appends items.
func (m *MenuWidget) Add(items ...Widget) *MenuWidget {
	for _, it := range items {
		if it != nil {
			m.n.adopt(it.node(), -1)
		}
	}
	return m
}

// Insert adds an item at the given position.
func (m *MenuWidget) Insert(index int, item Widget) *MenuWidget {
	if item != nil {
		m.n.adopt(item.node(), index)
	}
	return m
}

// Remove detaches an item without destroying it.
func (m *MenuWidget) Remove(item Widget) *MenuWidget {
	if item != nil {
		n := item.node()
		if n.parent == m.n {
			n.detach()
			native.NodeRemove(m.n.id, n.id)
		}
	}
	return m
}

// Clear destroys every item.
func (m *MenuWidget) Clear() *MenuWidget {
	m.n.clear()
	return m
}

// Items returns the current items in order.
func (m *MenuWidget) Items() []Widget {
	out := make([]Widget, 0, len(m.n.children))
	for _, c := range m.n.children {
		out = append(out, c.widget)
	}
	return out
}

// OpenBelow shows the menu under anchor, left-aligned with it (or above it when there is no
// room below). The anchor must be inside a window.
func (m *MenuWidget) OpenBelow(anchor Widget) *MenuWidget {
	if anchor == nil {
		return m
	}
	if err := native.MenuOpen(m.n.id, anchor.node().id, 0, 0); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	return m
}

// OpenAt shows the menu at a point in the window's client area, in logical pixels.
func (m *MenuWidget) OpenAt(win *Window, x, y float64) *MenuWidget {
	if win == nil {
		return m
	}
	if err := native.MenuOpen(m.n.id, win.n.id, x, y); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	return m
}

// Close dismisses the menu.
func (m *MenuWidget) Close() { native.MenuClose(m.n.id) }

// IsOpen reports whether the menu is showing.
func (m *MenuWidget) IsOpen() bool { return native.MenuIsOpen(m.n.id) }

// MatchWidth makes the menu at least as wide as the widget it opens below (dropdowns).
func (m *MenuWidget) MatchWidth(v bool) *MenuWidget { return m.setB(native.PropMatchAnchor, v) }

// OnClose registers a callback for when the menu closes, whatever the reason.
func (m *MenuWidget) OnClose(fn func()) *MenuWidget {
	m.n.onClose = fn
	return m
}

// ---- MenuItem ---------------------------------------------------------------------------

// MenuItemWidget is a row of a Menu. Create it with MenuItem.
type MenuItemWidget struct{ textStyle[MenuItemWidget] }

// MenuItem creates a menu row that calls onSelect when chosen (nil is allowed).
func MenuItem(label string, onSelect func()) *MenuItemWidget {
	i := &MenuItemWidget{}
	i.attach(i, native.KindMenuItem)
	i.SetText(label)
	i.n.onClick = onSelect
	return i
}

// OnSelect replaces the selection handler.
func (i *MenuItemWidget) OnSelect(fn func()) *MenuItemWidget {
	i.n.onClick = fn
	return i
}

// Icon shows an SVG icon in the column before the label.
func (i *MenuItemWidget) Icon(svg string) *MenuItemWidget {
	if err := native.SetSVG(i.n.id, svg); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	return i
}

// Shortcut shows a keyboard shortcut hint at the right, such as "Ctrl+S". It is only a
// hint: bind the key yourself.
func (i *MenuItemWidget) Shortcut(s string) *MenuItemWidget {
	native.SetStr(i.n.id, native.PropShortcut, s)
	return i
}

// Checked shows a check mark in place of the icon.
func (i *MenuItemWidget) Checked(v bool) *MenuItemWidget { return i.setB(native.PropChecked, v) }

// IsChecked reports the check mark state.
func (i *MenuItemWidget) IsChecked() bool { return native.GetBool(i.n.id, native.PropChecked) }

// MenuSeparator creates a thin line between groups of items.
func MenuSeparator() *DividerWidget { return Divider().MarginEach(6, 4, 6, 4) }

// ---- Select -----------------------------------------------------------------------------

const chevronDownSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>`

// SelectWidget is a dropdown: a button showing the current option that opens a menu of all
// the options. Create it with Select.
type SelectWidget struct {
	textStyle[SelectWidget]
	menu     *MenuWidget
	options  []string
	items    []*MenuItemWidget
	index    int
	onChange func(index int, value string)
}

// Select creates a dropdown with the given options; the first one starts selected.
func Select(options ...string) *SelectWidget {
	s := &SelectWidget{options: options, index: -1}
	s.attach(s, native.KindButton)
	s.setI(native.PropVariant, native.VariantOutlined)
	s.setB(native.PropIconEnd, true)
	s.setF(native.PropIconSize, 16)
	if err := native.SetSVG(s.n.id, chevronDownSVG); err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
	s.menu = Menu().MatchWidth(true)
	for i, opt := range options {
		i := i
		item := MenuItem(opt, func() { s.choose(i, true) })
		s.items = append(s.items, item)
		s.menu.Add(item)
	}
	s.n.onClick = func() { s.menu.OpenBelow(s) }
	if len(options) > 0 {
		s.choose(0, false)
	}
	return s
}

func (s *SelectWidget) choose(i int, notify bool) {
	if i < 0 || i >= len(s.options) {
		return
	}
	changed := i != s.index
	s.index = i
	s.SetText(s.options[i])
	for j, it := range s.items {
		it.Checked(j == i)
	}
	if notify && changed && s.onChange != nil {
		s.onChange(i, s.options[i])
	}
}

// SetSelected selects an option by index without calling the OnChange handler.
func (s *SelectWidget) SetSelected(i int) *SelectWidget {
	s.choose(i, false)
	return s
}

// Selected returns the index of the current option (-1 when there are none).
func (s *SelectWidget) Selected() int { return s.index }

// Value returns the text of the current option.
func (s *SelectWidget) Value() string {
	if s.index < 0 || s.index >= len(s.options) {
		return ""
	}
	return s.options[s.index]
}

// OnChange registers a handler called when the user picks a different option.
func (s *SelectWidget) OnChange(fn func(index int, value string)) *SelectWidget {
	s.onChange = fn
	return s
}

// Menu returns the underlying popup, for tweaks such as OnClose.
func (s *SelectWidget) Menu() *MenuWidget { return s.menu }
