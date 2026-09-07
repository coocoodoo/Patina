//! Pointer and keyboard handling exercised against a laid-out tree, without a window.

use patina_core::ffi::*;
use patina_core::input::{Key, cursor_moved, key_down, mouse_button};
use patina_core::layout::layout_window;
use patina_core::node::Modifiers;
use patina_core::props::*;
use patina_core::state::{self, OutEvent, State};

fn set_text(node: u64, text: &str) {
    unsafe { patina_set_str(node, PROP_TEXT, text.as_ptr(), text.len()) };
}

fn has(events: &[OutEvent], node: u64, event: u32) -> bool {
    events.iter().any(|e| e.node == node && e.event == event)
}

/// Builds a window with a button, a checkbox and a text input and lays it out at 400x300.
fn setup() -> (u64, u64, u64, u64) {
    let win = patina_window_new();
    let col = patina_node_new(KIND_BOX);
    let btn = patina_node_new(KIND_BUTTON);
    set_text(btn, "Go");
    let cb = patina_node_new(KIND_CHECKBOX);
    set_text(cb, "Check");
    let input = patina_node_new(KIND_TEXT_INPUT);
    patina_node_append(col, btn);
    patina_node_append(col, cb);
    patina_node_append(col, input);
    patina_window_set_content(win, col);
    {
        let mut s = state::lock();
        let State { nodes, text, .. } = &mut *s;
        assert!(
            text.ensure_loaded(),
            "system fonts must be available: {:?}",
            text.error()
        );
        layout_window(nodes, text, win, 400.0, 300.0, 1.0);
    }
    (win, btn, cb, input)
}

fn center(node: u64) -> (f32, f32) {
    let s = state::lock();
    let r = s.nodes.get(node).expect("node").rect;
    assert!(r.w > 0.0 && r.h > 0.0, "node must have a laid out size");
    r.center()
}

#[test]
fn click_toggle_and_type() {
    let (win, btn, cb, input) = setup();
    let (bx, by) = center(btn);
    let (cx, cy) = center(cb);
    let (ix, iy) = center(input);

    let mut s = state::lock();

    // Hover then click the button.
    let out = cursor_moved(&mut s, win, bx, by);
    assert!(has(&out.events, btn, EV_HOVER_ENTER), "{:?}", out.events);
    let out = mouse_button(&mut s, win, true, 0);
    assert!(has(&out.events, btn, EV_FOCUS), "{:?}", out.events);
    let out = mouse_button(&mut s, win, false, 0);
    assert!(has(&out.events, btn, EV_CLICK), "{:?}", out.events);

    // Click the checkbox: toggled on, then Space toggles it off again.
    cursor_moved(&mut s, win, cx, cy);
    mouse_button(&mut s, win, true, 0);
    let out = mouse_button(&mut s, win, false, 0);
    let toggled = out
        .events
        .iter()
        .find(|e| e.node == cb && e.event == EV_TOGGLED)
        .expect("toggled");
    assert_eq!(toggled.a, 1);
    assert!(s.nodes.get(cb).unwrap().control.checked);
    let out = key_down(&mut s, win, Key::Char(" ".into()), Modifiers::default());
    let toggled = out
        .events
        .iter()
        .find(|e| e.node == cb && e.event == EV_TOGGLED)
        .expect("toggled");
    assert_eq!(toggled.a, 0);

    // Pressing on the button but releasing elsewhere must not click.
    cursor_moved(&mut s, win, bx, by);
    mouse_button(&mut s, win, true, 0);
    cursor_moved(&mut s, win, bx, by + 200.0);
    let out = mouse_button(&mut s, win, false, 0);
    assert!(!has(&out.events, btn, EV_CLICK));

    // Type into the input.
    cursor_moved(&mut s, win, ix, iy);
    mouse_button(&mut s, win, true, 0);
    mouse_button(&mut s, win, false, 0);
    for ch in ["h", "i"] {
        let out = key_down(&mut s, win, Key::Char(ch.into()), Modifiers::default());
        assert!(has(&out.events, input, EV_TEXT_CHANGED), "{:?}", out.events);
    }
    assert_eq!(s.nodes.get(input).unwrap().text.text, "hi");
    let out = key_down(&mut s, win, Key::Backspace, Modifiers::default());
    assert!(has(&out.events, input, EV_TEXT_CHANGED));
    assert_eq!(s.nodes.get(input).unwrap().text.text, "h");
    let out = key_down(&mut s, win, Key::Enter, Modifiers::default());
    let submit = out
        .events
        .iter()
        .find(|e| e.node == input && e.event == EV_SUBMIT)
        .expect("submit");
    assert_eq!(submit.text.as_deref(), Some("h"));

    // Tab moves focus to the next control.
    let out = key_down(&mut s, win, Key::Tab, Modifiers::default());
    assert!(has(&out.events, input, EV_BLUR), "{:?}", out.events);
    assert!(has(&out.events, btn, EV_FOCUS), "{:?}", out.events);
}

#[test]
fn menus_open_select_and_close() {
    use patina_core::anim::Motion;
    use patina_core::input::{close_menu, hit_window, menu_is_open, open_menu};
    use patina_core::render::advance_animations;
    let (win, btn, _cb, _input) = setup();
    let menu = patina_node_new(KIND_MENU);
    let item1 = patina_node_new(KIND_MENU_ITEM);
    set_text(item1, "New");
    let item2 = patina_node_new(KIND_MENU_ITEM);
    set_text(item2, "Open");
    patina_node_append(menu, item1);
    patina_node_append(menu, item2);
    let (bx, by) = center(btn);
    let relayout = |s: &mut State| {
        let State { nodes, text, .. } = s;
        layout_window(nodes, text, win, 400.0, 300.0, 1.0);
    };

    let mut s = state::lock();
    open_menu(&mut s, menu, btn, 0.0, 0.0).expect("opens below the button");
    assert!(menu_is_open(&s, menu));
    relayout(&mut s);
    let mr = s.nodes.get(menu).unwrap().rect;
    let br = s.nodes.get(btn).unwrap().rect;
    assert!(
        mr.y >= br.bottom(),
        "menu below its anchor: {mr:?} vs {br:?}"
    );
    assert!(mr.w >= 180.0 && mr.h > 0.0, "{mr:?}");
    let ir = s.nodes.get(item1).unwrap().rect;
    assert!(ir.w > 0.0 && mr.contains(ir.x + 1.0, ir.y + 1.0), "{ir:?}");
    let (ix, iy) = ir.center();
    assert_eq!(
        hit_window(&s, win, ix, iy),
        Some(item1),
        "the menu is on top"
    );

    // Clicking an item emits a click and closes the menu.
    cursor_moved(&mut s, win, ix, iy);
    mouse_button(&mut s, win, true, 0);
    let out = mouse_button(&mut s, win, false, 0);
    assert!(has(&out.events, item1, EV_CLICK), "{:?}", out.events);
    assert!(has(&out.events, menu, EV_MENU_CLOSED), "{:?}", out.events);
    assert!(!menu_is_open(&s, menu));
    advance_animations(&mut s.nodes, win, 0.016, 0.0, true, Motion::Ease);
    assert!(s.nodes.get(menu).unwrap().parent.is_none());
    assert!(s.window_data(win).unwrap().overlays.is_empty());

    // Keyboard: Down twice then Enter selects the second item.
    open_menu(&mut s, menu, btn, 0.0, 0.0).unwrap();
    relayout(&mut s);
    key_down(&mut s, win, Key::Down, Modifiers::default());
    key_down(&mut s, win, Key::Down, Modifiers::default());
    let out = key_down(&mut s, win, Key::Enter, Modifiers::default());
    assert!(has(&out.events, item2, EV_CLICK), "{:?}", out.events);
    assert!(!menu_is_open(&s, menu));
    advance_animations(&mut s.nodes, win, 0.016, 0.0, true, Motion::Ease);

    // A click outside an open menu closes it without reaching the widget underneath.
    open_menu(&mut s, menu, btn, 0.0, 0.0).unwrap();
    relayout(&mut s);
    cursor_moved(&mut s, win, bx, by);
    let out = mouse_button(&mut s, win, true, 0);
    assert!(has(&out.events, menu, EV_MENU_CLOSED), "{:?}", out.events);
    let out = mouse_button(&mut s, win, false, 0);
    assert!(
        !has(&out.events, btn, EV_CLICK),
        "swallowed: {:?}",
        out.events
    );
    advance_animations(&mut s.nodes, win, 0.016, 0.0, true, Motion::Ease);

    // Escape closes too.
    open_menu(&mut s, menu, btn, 0.0, 0.0).unwrap();
    let out = key_down(&mut s, win, Key::Escape, Modifiers::default());
    assert!(has(&out.events, menu, EV_MENU_CLOSED), "{:?}", out.events);
    assert!(close_menu(&mut s, menu).is_empty(), "already closing");
    advance_animations(&mut s.nodes, win, 0.016, 0.0, true, Motion::Ease);
}

#[test]
fn tooltips_show_after_a_delay() {
    use patina_core::input::{TOOLTIP_DELAY, tooltips_due};
    let (win, btn, _cb, _input) = setup();
    let tip = "Go go";
    unsafe { patina_set_str(btn, PROP_TOOLTIP, tip.as_ptr(), tip.len()) };
    let (bx, by) = center(btn);
    let mut s = state::lock();
    cursor_moved(&mut s, win, bx, by);
    let pending = s
        .window_data(win)
        .unwrap()
        .tooltip
        .clone()
        .expect("pending tooltip");
    assert_eq!(pending.node, btn);
    assert!(!pending.shown);
    let since = pending.since;
    assert!(!tooltips_due(&mut s, since + TOOLTIP_DELAY * 0.5).contains(&win));
    assert!(tooltips_due(&mut s, since + TOOLTIP_DELAY + 0.01).contains(&win));
    assert!(s.window_data(win).unwrap().tooltip.as_ref().unwrap().shown);
    // Pressing the button hides it.
    mouse_button(&mut s, win, true, 0);
    let hidden = s
        .window_data(win)
        .unwrap()
        .tooltip
        .as_ref()
        .map(|t| t.closing)
        .unwrap_or(true);
    assert!(hidden);
    mouse_button(&mut s, win, false, 0);
}
