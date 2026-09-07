package patina

import "github.com/patina-ui/patina/internal/native"

// Box is a flexbox container. Create it with Column, Row or Card.
type Box struct{ Base[Box] }

func newBox(children []Widget, row bool) *Box {
	b := &Box{}
	b.attach(b, native.KindBox)
	if row {
		b.setI(native.PropDirection, 1)
		b.setI(native.PropAlignItems, int64(AlignCenter))
	}
	b.Add(children...)
	return b
}

// Column stacks its children vertically. Children stretch to the column's width.
func Column(children ...Widget) *Box { return newBox(children, false) }

// Row places its children side by side, vertically centered.
func Row(children ...Widget) *Box { return newBox(children, true) }

// Card is a Column on an elevated surface: padding, rounded corners, a hairline border and
// a soft shadow.
func Card(children ...Widget) *Box {
	return Column(children...).
		Background(Surface).
		Radius(14).
		Border(1, Outline).
		Shadow(1).
		Padding(20).
		Gap(12)
}

// Gap sets the spacing between children.
func (b *Box) Gap(px float64) *Box { return b.setF(native.PropGap, px) }

// Align sets how children are positioned along the cross axis.
func (b *Box) Align(a Align) *Box { return b.setI(native.PropAlignItems, int64(a)) }

// Justify sets how children are distributed along the main axis.
func (b *Box) Justify(j Justify) *Box { return b.setI(native.PropJustify, int64(j)) }

// Wrap lets children flow onto additional lines when they do not fit.
func (b *Box) Wrap(wrap bool) *Box { return b.setB(native.PropWrap, wrap) }

// Vertical lays children out top to bottom.
func (b *Box) Vertical() *Box { return b.setI(native.PropDirection, 0) }

// Horizontal lays children out left to right.
func (b *Box) Horizontal() *Box { return b.setI(native.PropDirection, 1) }

// Add appends children.
func (b *Box) Add(children ...Widget) *Box {
	for _, c := range children {
		if c != nil {
			b.n.adopt(c.node(), -1)
		}
	}
	return b
}

// Insert adds a child at the given position.
func (b *Box) Insert(index int, child Widget) *Box {
	if child != nil {
		b.n.adopt(child.node(), index)
	}
	return b
}

// Remove detaches a child without destroying it, so it can be added elsewhere later.
func (b *Box) Remove(child Widget) *Box {
	if child != nil {
		n := child.node()
		if n.parent == b.n {
			n.detach()
			native.NodeRemove(b.n.id, n.id)
		}
	}
	return b
}

// Clear destroys all children.
func (b *Box) Clear() *Box {
	b.n.clear()
	return b
}

// Children returns the current children in order.
func (b *Box) Children() []Widget {
	out := make([]Widget, 0, len(b.n.children))
	for _, c := range b.n.children {
		out = append(out, c.widget)
	}
	return out
}

// ScrollView is a vertically scrolling column. Create it with Scroll.
type ScrollView struct{ Base[ScrollView] }

// Scroll creates a scrollable column. It expands to fill its parent unless sized explicitly.
func Scroll(children ...Widget) *ScrollView {
	s := &ScrollView{}
	s.attach(s, native.KindScroll)
	s.Add(children...)
	return s
}

// Gap sets the spacing between children.
func (s *ScrollView) Gap(px float64) *ScrollView { return s.setF(native.PropGap, px) }

// Align sets how children are positioned horizontally.
func (s *ScrollView) Align(a Align) *ScrollView { return s.setI(native.PropAlignItems, int64(a)) }

// Add appends children.
func (s *ScrollView) Add(children ...Widget) *ScrollView {
	for _, c := range children {
		if c != nil {
			s.n.adopt(c.node(), -1)
		}
	}
	return s
}

// Insert adds a child at the given position.
func (s *ScrollView) Insert(index int, child Widget) *ScrollView {
	if child != nil {
		s.n.adopt(child.node(), index)
	}
	return s
}

// Remove detaches a child without destroying it.
func (s *ScrollView) Remove(child Widget) *ScrollView {
	if child != nil {
		n := child.node()
		if n.parent == s.n {
			n.detach()
			native.NodeRemove(s.n.id, n.id)
		}
	}
	return s
}

// Clear destroys all children.
func (s *ScrollView) Clear() *ScrollView {
	s.n.clear()
	return s
}

// Children returns the current children in order.
func (s *ScrollView) Children() []Widget {
	out := make([]Widget, 0, len(s.n.children))
	for _, c := range s.n.children {
		out = append(out, c.widget)
	}
	return out
}

// ScrollTo scrolls to a vertical offset in logical pixels.
func (s *ScrollView) ScrollTo(y float64) *ScrollView { return s.setF(native.PropScrollY, y) }

// ScrollToEnd scrolls to the bottom.
func (s *ScrollView) ScrollToEnd() *ScrollView { return s.ScrollTo(1e9) }

// Offset returns the current vertical scroll offset in logical pixels.
func (s *ScrollView) Offset() float64 { return native.GetF64(s.n.id, native.PropScrollY) }

// OnClick makes the box clickable: it shows a pointer cursor, reacts to hover and calls fn
// when clicked (or activated with the keyboard when focused). Pass nil to turn it off.
func (b *Box) OnClick(fn func()) *Box {
	b.n.onClick = fn
	return b.setB(native.PropClickable, fn != nil)
}

// Lift raises the box with a deeper shadow while the pointer hovers over it, the classic
// card hover effect. It implies a clickable box, so combine it with OnClick.
func (b *Box) Lift() *Box {
	b.setB(native.PropClickable, true)
	return b.setB(native.PropHoverLift, true)
}
