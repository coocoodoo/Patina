//! Builds a showcase window through the C ABI and renders it offscreen to PNG files.
//!
//! Run with `cargo run --example snapshot` from the `core` directory.

use patina_core::ffi::*;
use patina_core::props::*;
use patina_core::render;
use std::path::Path;

fn set_str(node: u64, prop: u32, s: &str) {
    unsafe { patina_set_str(node, prop, s.as_ptr(), s.len()) };
}

fn set_f(node: u64, prop: u32, v: f64) {
    patina_set_f64(node, prop, v.to_bits());
}

fn set_i(node: u64, prop: u32, v: i64) {
    patina_set_i64(node, prop, v);
}

fn node(kind: u32) -> u64 {
    patina_node_new(kind)
}

fn label(text: &str, size: f64, weight: i64, color: i64) -> u64 {
    let n = node(KIND_LABEL);
    set_str(n, PROP_TEXT, text);
    if size > 0.0 {
        set_f(n, PROP_FONT_SIZE, size);
    }
    if weight > 0 {
        set_i(n, PROP_FONT_WEIGHT, weight);
    }
    if color != COLOR_DEFAULT {
        set_i(n, PROP_TEXT_COLOR, color);
    }
    n
}

fn button(text: &str, variant: i64) -> u64 {
    let n = node(KIND_BUTTON);
    set_str(n, PROP_TEXT, text);
    set_i(n, PROP_VARIANT, variant);
    n
}

fn column(gap: f64, children: &[u64]) -> u64 {
    let n = node(KIND_BOX);
    set_f(n, PROP_GAP, gap);
    for c in children {
        patina_node_append(n, *c);
    }
    n
}

fn row(gap: f64, children: &[u64]) -> u64 {
    let n = column(gap, children);
    set_i(n, PROP_DIRECTION, 1);
    set_i(n, PROP_ALIGN_ITEMS, 1);
    n
}

fn card(children: &[u64]) -> u64 {
    let n = column(14.0, children);
    set_i(n, PROP_BACKGROUND, COLOR_SURFACE);
    set_f(n, PROP_RADIUS, 14.0);
    set_f(n, PROP_BORDER_WIDTH, 1.0);
    set_i(n, PROP_SHADOW, 1);
    for p in [
        PROP_PADDING_LEFT,
        PROP_PADDING_TOP,
        PROP_PADDING_RIGHT,
        PROP_PADDING_BOTTOM,
    ] {
        set_f(n, p, 20.0);
    }
    n
}

fn spacer() -> u64 {
    node(KIND_SPACER)
}

fn main() {
    let win = patina_window_new();
    set_str(win, PROP_TITLE, "Patina showcase");

    let title = column(
        4.0,
        &[
            label("Patina", 26.0, 700, COLOR_DEFAULT),
            label(
                "Beautiful UIs for Go, rendered by Rust.",
                14.0,
                0,
                COLOR_TEXT_SECONDARY,
            ),
        ],
    );
    let header = row(
        10.0,
        &[
            title,
            spacer(),
            button("Primary", VARIANT_PRIMARY),
            button("Tonal", VARIANT_TONAL),
            button("Outlined", VARIANT_OUTLINED),
            button("Ghost", VARIANT_GHOST),
            button("Delete", VARIANT_DANGER),
        ],
    );

    let email = node(KIND_TEXT_INPUT);
    set_str(email, PROP_PLACEHOLDER, "Email address");
    set_str(email, PROP_TEXT, "ada@example.com");
    let password = node(KIND_TEXT_INPUT);
    set_str(password, PROP_PLACEHOLDER, "Password");
    set_i(password, PROP_SECURE, 1);
    let remember = node(KIND_CHECKBOX);
    set_str(remember, PROP_TEXT, "Remember me");
    set_i(remember, PROP_CHECKED, 1);
    let terms = node(KIND_CHECKBOX);
    set_str(terms, PROP_TEXT, "I agree to the terms");
    let sign_in = card(&[
        label("Sign in", 18.0, 600, COLOR_DEFAULT),
        label(
            "Welcome back. Enter your details to continue.",
            0.0,
            0,
            COLOR_TEXT_SECONDARY,
        ),
        email,
        password,
        remember,
        terms,
        row(
            10.0,
            &[
                spacer(),
                button("Cancel", VARIANT_GHOST),
                button("Continue", VARIANT_PRIMARY),
            ],
        ),
    ]);
    set_f(sign_in, PROP_GROW, 1.0);

    let dark = node(KIND_SWITCH);
    set_str(dark, PROP_TEXT, "Dark mode");
    set_i(dark, PROP_CHECKED, 1);
    let notify = node(KIND_SWITCH);
    set_str(notify, PROP_TEXT, "Notifications");
    let divider = node(KIND_DIVIDER);
    let slider = node(KIND_SLIDER);
    set_f(slider, PROP_VALUE, 0.62);
    let progress = node(KIND_PROGRESS);
    set_f(progress, PROP_VALUE, 0.45);
    let busy = node(KIND_PROGRESS);
    set_i(busy, PROP_INDETERMINATE, 1);
    let disabled = button("Disabled", VARIANT_PRIMARY);
    set_i(disabled, PROP_DISABLED, 1);
    let prefs = card(&[
        label("Preferences", 18.0, 600, COLOR_DEFAULT),
        dark,
        notify,
        divider,
        label("Volume", 12.0, 600, COLOR_TEXT_SECONDARY),
        slider,
        label("Upload progress", 12.0, 600, COLOR_TEXT_SECONDARY),
        progress,
        busy,
        row(10.0, &[disabled, button("Tonal", VARIANT_TONAL)]),
    ]);
    set_f(prefs, PROP_GROW, 1.0);

    let cards = row(20.0, &[sign_in, prefs]);
    set_i(cards, PROP_ALIGN_ITEMS, 3);

    let list = node(KIND_SCROLL);
    set_f(list, PROP_GAP, 6.0);
    set_f(list, PROP_HEIGHT, 150.0);
    for i in 1..=12 {
        let item = row(
            10.0,
            &[
                label(&format!("Item {i}"), 0.0, 500, COLOR_DEFAULT),
                label("Synced a few minutes ago", 12.0, 0, COLOR_TEXT_MUTED),
                spacer(),
                button("Open", VARIANT_GHOST),
            ],
        );
        patina_node_append(list, item);
    }
    let list_card = card(&[label("Activity", 18.0, 600, COLOR_DEFAULT), list]);

    let root = column(20.0, &[header, cards, list_card]);
    for p in [
        PROP_PADDING_LEFT,
        PROP_PADDING_TOP,
        PROP_PADDING_RIGHT,
        PROP_PADDING_BOTTOM,
    ] {
        set_f(root, p, 28.0);
    }
    patina_window_set_content(win, root);

    // Give the first field focus so the caret and focus ring show up in the snapshot.
    set_i(email, PROP_FOCUS, 1);

    let out = Path::new("snapshots");
    std::fs::create_dir_all(out).expect("create snapshots dir");
    render::snapshot(win, 980, 720, 1.0, &out.join("snapshot_light.png")).expect("light snapshot");
    patina_theme_set_mode(1);
    render::snapshot(win, 980, 720, 1.0, &out.join("snapshot_dark.png")).expect("dark snapshot");
    patina_theme_set_mode(0);
    patina_theme_set_accent(0x0EA5E9FF);
    render::snapshot(win, 980, 720, 2.0, &out.join("snapshot_hidpi.png")).expect("hidpi snapshot");
    println!("wrote snapshots/snapshot_light.png, snapshot_dark.png and snapshot_hidpi.png");
}
