package patina

import "github.com/patina-ui/patina/internal/native"

// Enter is an entrance animation, played when a widget first appears or is shown again.
type Enter int

const (
	// EnterNone shows the widget immediately.
	EnterNone Enter = iota
	// FadeIn fades the widget in.
	FadeIn
	// SlideUp fades in while moving up into place.
	SlideUp
	// SlideDown fades in while moving down into place.
	SlideDown
	// SlideLeft fades in while moving left into place.
	SlideLeft
	// SlideRight fades in while moving right into place.
	SlideRight
	// Pop fades in with a short upward nudge (a good default for toasts and cards).
	Pop
)

// SetAnimationSpeed scales every animation in the toolkit: 0.5 plays everything at half
// speed, 2 at double speed. 0 turns motion off (changes apply instantly). The default is 1.
func SetAnimationSpeed(factor float64) {
	mustInit()
	native.SetAnimationSpeed(factor)
}

// AnimationSpeed returns the current global animation speed factor.
func AnimationSpeed() float64 {
	mustInit()
	return native.AnimationSpeed()
}

// ReducedMotion turns all animations off (on = true) or restores normal speed.
func ReducedMotion(on bool) {
	if on {
		SetAnimationSpeed(0)
	} else {
		SetAnimationSpeed(1)
	}
}

// PressEffect deforms a widget when it is pressed and clicked.
type PressEffect int

const (
	// NoPressEffect leaves the widget rigid.
	NoPressEffect PressEffect = iota
	// RubberBand squashes the widget while pressed and stretches it past its size on release,
	// snapping back like a rubber band.
	RubberBand
	// Gelatin squishes the widget under the pointer and lets it wobble and lean like jelly
	// after the click.
	Gelatin
	// Bounce shrinks the widget while pressed and pops it with a springy overshoot on release.
	Bounce
)

// Motion is the default way values, toggles and entrances move towards their targets.
type Motion int

const (
	// EaseMotion is a smooth ease-out that never overshoots (the default).
	EaseMotion Motion = iota
	// SpringMotion is a gentle spring with a small overshoot.
	SpringMotion
	// BouncyMotion is a lively spring that bounces before it settles.
	BouncyMotion
)

// SetMotion sets the motion style for every slider, progress bar, toggle and entrance
// animation. Individual widgets can override it with Spring.
func SetMotion(m Motion) {
	mustInit()
	native.SetMotion(uint32(m))
}

// CurrentMotion returns the global motion style.
func CurrentMotion() Motion {
	mustInit()
	return Motion(native.Motion())
}
