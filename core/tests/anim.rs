//! SMIL animation, palettes and theme transitions.

use patina_core::palettes;
use patina_core::smil::Timeline;
use patina_core::svg::SvgDoc;
use patina_core::theme::{Mode, Theme};

const ANIMATED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill="#000"><animate attributeName="r" from="4" to="8" dur="1s" repeatCount="indefinite"/><animate attributeName="fill" values="#ff0000;#0000ff" dur="2s" repeatCount="indefinite"/></circle><g><animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="1s" repeatCount="indefinite"/><rect x="10" y="2" width="4" height="4"/></g></svg>"##;

#[test]
fn animate_numbers_colors_and_transforms() {
    let tl = Timeline::parse(ANIMATED).expect("has animations");
    let s = tl.apply(ANIMATED, 0.5);
    assert!(s.contains(r##"r="6""##), "{s}");
    assert!(s.contains(r##"fill="#bf0040""##), "{s}");
    assert!(s.contains(r##"transform="rotate(180 12 12)""##), "{s}");
    assert!(!s.contains("<animate"), "{s}");
    assert_eq!(tl.period(), None, "durations differ, so no common loop");

    // Looping: 1.25s is the same frame as 0.25s for the 1s tracks.
    let s = tl.apply(ANIMATED, 1.25);
    assert!(s.contains(r##"r="5""##), "{s}");
    assert!(s.contains(r##"rotate(90 12 12)"##), "{s}");

    let doc = SvgDoc::parse(ANIMATED).expect("parse");
    assert!(doc.is_animated());
    let pm = doc
        .render_at(48, 48, patina_core::color::Rgba::BLACK, None, 0.5)
        .expect("render");
    assert_eq!(pm.width(), 48);

    assert!(Timeline::parse(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"><rect width="8" height="8"/></svg>"##).is_none());
}

#[test]
fn keytimes_freeze_set_and_splines() {
    let src = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" opacity="0.2"><animate attributeName="opacity" values="0;1;0" keyTimes="0;0.25;1" dur="4s" fill="freeze"/><set attributeName="fill" to="#00ff00" begin="1s"/><animate attributeName="x" from="0" to="10" dur="2s" calcMode="spline" keySplines="0.42 0 0.58 1"/></rect></svg>"##;
    let tl = Timeline::parse(src).expect("animations");
    let at = |t: f64| tl.apply(src, t);

    assert!(at(1.0).contains(r##"opacity="1""##), "{}", at(1.0));
    assert!(at(2.5).contains(r##"opacity="0.5""##), "{}", at(2.5));
    assert!(
        at(10.0).contains(r##"opacity="0""##),
        "frozen at the last value: {}",
        at(10.0)
    );
    assert!(
        !at(0.5).contains("#00ff00"),
        "set has not begun: {}",
        at(0.5)
    );
    assert!(at(2.0).contains(r##"fill="#00ff00""##), "{}", at(2.0));
    // Ease-in-out spline: halfway in time is halfway in value, a quarter is less than a quarter.
    assert!(at(1.0).contains(r##"x="5""##), "{}", at(1.0));
    let quarter = at(0.5);
    let x: f32 = quarter
        .split(r##" x=""##)
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse().ok())
        .expect("x attribute");
    assert!(x > 0.5 && x < 2.5, "eased x = {x}");
    assert_eq!(tl.end(), Some(4.0));
}

#[test]
fn builtin_palettes() {
    assert_eq!(palettes::BUILTIN.len(), 20);
    let light = palettes::BUILTIN.iter().filter(|(_, p)| !p.is_dark).count();
    assert_eq!(light, 10);
    let mut names: Vec<&str> = palettes::BUILTIN.iter().map(|(n, _)| *n).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 20, "names must be unique");
    assert!(palettes::builtin("nord").unwrap().is_dark);
    assert!(!palettes::builtin("Mint").unwrap().is_dark);
    assert!(palettes::builtin("nope").is_none());
    for (name, p) in palettes::BUILTIN {
        // Text must contrast with the background it sits on.
        let contrast = (p.text.luminance() - p.background.luminance()).abs();
        assert!(
            contrast > 0.5,
            "{name}: text/background contrast {contrast}"
        );
        assert!(
            (p.on_accent.luminance() - p.accent.luminance()).abs() > 0.3,
            "{name}: on_accent must be readable on accent"
        );
    }
}

#[test]
fn theme_transition_blends_palettes() {
    let mut theme = Theme::default();
    assert_eq!(theme.mode, Mode::Light);
    theme.use_palette("Nord", 0.0);
    assert_eq!(theme.mode, Mode::Dark);
    assert!(theme.transitioning(0.1));
    let mid = theme.palette_at(0.15);
    let end = theme.palette_at(10.0);
    assert!(!theme.transitioning(10.0));
    assert_eq!(
        end.background.to_rgba_u32(),
        palettes::NORD.background.to_rgba_u32()
    );
    // Halfway through, the background is between the two palettes.
    assert!(mid.background.luminance() > end.background.luminance());
    assert!(mid.background.luminance() < palettes::INDIGO.background.luminance());
}

#[test]
fn press_effects_deform_and_settle() {
    use patina_core::node::{Kind, Node};
    use patina_core::props::*;
    use patina_core::render::{press_transform, wobble_secs};
    let mut n = Node::new(Kind::Button);
    assert!(press_transform(&n, 0.0).is_none(), "no effect configured");
    n.visual.press_effect = PRESS_RUBBER;
    assert!(press_transform(&n, 0.0).is_none(), "at rest");
    n.press_wobble = Some(0.0);
    let (sx, sy, _) = press_transform(&n, 0.18).expect("stretching");
    assert!(
        sx > 1.05 && sy < 1.0,
        "rubber band stretches sideways: {sx} {sy}"
    );
    assert!(
        press_transform(&n, wobble_secs(PRESS_RUBBER) + 0.1).is_none(),
        "settled after the wobble"
    );
    n.visual.press_effect = PRESS_GELATIN;
    let (sx, sy, kx) = press_transform(&n, 0.09).expect("wobbling");
    assert!(
        (sx - 1.0).abs() > 0.02 && (sy - 1.0).abs() > 0.02 && kx.abs() > 0.01,
        "gelatin squashes and leans: {sx} {sy} {kx}"
    );
    n.press_wobble = None;
    n.anim.press.snap(1.0);
    let (sx, sy, _) = press_transform(&n, 5.0).expect("squashed while pressed");
    assert!(sx > 1.0 && sy < 1.0, "{sx} {sy}");
}

#[test]
fn animate_motion_follows_a_path() {
    use patina_core::color::Rgba;
    let src = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect id="box" width="4" height="4"><animateMotion path="M0 0 L100 0" dur="2s" rotate="auto" repeatCount="indefinite"/></rect><circle r="2" transform="scale(2)"><animateMotion dur="1s" values="0,0; 0,10" fill="freeze"/></circle><path id="track" d="M0 50 C 25 0, 75 0, 100 50"/><g><animateMotion dur="4s"><mpath href="#track"/></animateMotion><rect width="1" height="1"/></g></svg>"##;
    let tl = Timeline::parse(src).expect("animations");
    let at = |t: f64| tl.apply(src, t);
    let s = at(1.0);
    assert!(
        s.contains(r##"transform="translate(50 0)""##),
        "halfway along a straight path: {s}"
    );
    assert!(
        s.contains("translate(0 10) scale(2)"),
        "the element keeps its own transform after the motion: {s}"
    );
    let s0 = at(0.0);
    assert!(
        s0.contains("translate(0 50)"),
        "mpath starts at the first point of the track: {s0}"
    );
    assert!(
        !s0.contains("<animateMotion") && !s0.contains("<mpath"),
        "{s0}"
    );

    // rotate="auto" turns the element along the tangent: a vertical path yields 90 degrees.
    let vert = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="2" height="2"><animateMotion path="M0 0 L0 10" dur="1s" rotate="auto" fill="freeze"/></rect></svg>"##;
    let tl = Timeline::parse(vert).unwrap();
    let s = tl.apply(vert, 0.5);
    assert!(s.contains("translate(0 5) rotate(90)"), "{s}");
    assert_eq!(tl.end(), Some(1.0));

    // keyPoints/keyTimes pin the distance along the path.
    let kp = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="1" height="1"><animateMotion path="M0 0 H 10" dur="1s" keyPoints="0;0.2;1" keyTimes="0;0.5;1" calcMode="linear" fill="freeze"/></rect></svg>"##;
    let tl = Timeline::parse(kp).unwrap();
    assert!(
        tl.apply(kp, 0.5).contains("translate(2 0)"),
        "{}",
        tl.apply(kp, 0.5)
    );
    assert!(
        tl.apply(kp, 0.75).contains("translate(6 0)"),
        "{}",
        tl.apply(kp, 0.75)
    );

    // Arcs and relative commands flatten too, and the frame renders.
    let arc = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><circle r="2" fill="#000"><animateMotion path="m2 10 a8 8 0 1 1 16 0" dur="2s" repeatCount="indefinite"/></circle></svg>"##;
    let doc = SvgDoc::parse(arc).expect("valid");
    assert!(doc.is_animated());
    assert_eq!(doc.period(), Some(2.0));
    let pm = doc
        .render_at(20, 20, Rgba::BLACK, None, 1.0)
        .expect("render");
    assert!(pm.pixels().iter().any(|p| p.alpha() > 0));
    let tl = Timeline::parse(arc).unwrap();
    let s = tl.apply(arc, 1.0);
    let nums: Vec<f32> = s
        .split("translate(")
        .nth(1)
        .and_then(|r| r.split(')').next())
        .map(|inner| inner.split(' ').filter_map(|v| v.parse().ok()).collect())
        .unwrap_or_default();
    assert!(
        nums.len() == 2 && (nums[0] - 10.0).abs() < 0.05 && (nums[1] - 2.0).abs() < 0.05,
        "halfway round the arc is its top: {s}"
    );
}

#[test]
fn spring_tweens_overshoot_and_settle() {
    use patina_core::anim::{Motion, Tween};
    let mut t = Tween::default();
    t.set(1.0);
    let mut max = 0.0f32;
    let mut steps = 0;
    while t.advance(1.0 / 60.0, 12.0, Motion::BOUNCY) && steps < 600 {
        max = max.max(t.value);
        steps += 1;
    }
    assert!(max > 1.05, "a bouncy spring overshoots: max {max}");
    assert!(
        (t.value - 1.0).abs() < 0.01 && steps < 600,
        "settles at the target: {} after {steps} frames",
        t.value
    );
    let mut e = Tween::default();
    e.set(1.0);
    let mut max_e = 0.0f32;
    while e.advance(1.0 / 60.0, 12.0, Motion::Ease) {
        max_e = max_e.max(e.value);
    }
    assert!(max_e <= 1.0, "easing never overshoots");
    assert_eq!(Motion::from_style(2), Motion::BOUNCY);
    assert_eq!(Motion::BOUNCY.style(), 2);
    assert_eq!(Motion::Ease.style(), 0);
}
