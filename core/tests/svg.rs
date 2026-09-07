//! SVG parsing, rasterization, tinting and the related ABI entry points.

use patina_core::color::Rgba;
use patina_core::ffi::*;
use patina_core::props::*;
use patina_core::svg::SvgDoc;

const CIRCLE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#;

#[test]
fn parse_render_and_tint() {
    let doc = SvgDoc::parse(CIRCLE).expect("parse");
    assert_eq!((doc.width, doc.height), (24.0, 24.0));

    let red = Rgba::hex(0xFF0000);
    let pm = doc.render(48, 48, red, None).expect("render");
    let center = pm.pixel(24, 24).unwrap();
    assert_eq!(center.alpha(), 255);
    assert!(
        center.red() > 200 && center.green() < 10,
        "currentColor should resolve to red"
    );
    assert_eq!(
        pm.pixel(1, 1).unwrap().alpha(),
        0,
        "corners are outside the circle"
    );

    let pm = doc
        .render(48, 48, red, Some(Rgba::hex(0x00FF00)))
        .expect("render tinted");
    let center = pm.pixel(24, 24).unwrap();
    assert!(
        center.green() > 200 && center.red() < 10,
        "tint must override the fill"
    );

    assert!(SvgDoc::parse("<svg").is_err());
    assert!(SvgDoc::parse("").is_err());
}

#[test]
fn ffi_render_and_set() {
    let (w, h) = (16u32, 16u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    let n = unsafe {
        patina_svg_render(
            CIRCLE.as_ptr(),
            CIRCLE.len(),
            w,
            h,
            COLOR_DEFAULT,
            buf.as_mut_ptr(),
            buf.len(),
        )
    };
    assert_eq!(n, buf.len());
    let center = ((8 * w + 8) * 4) as usize;
    assert_eq!(buf[center + 3], 255);
    assert_eq!(buf[3], 0);

    // A too-small buffer still reports the required size without writing.
    let mut small = vec![7u8; 8];
    let n = unsafe {
        patina_svg_render(
            CIRCLE.as_ptr(),
            CIRCLE.len(),
            w,
            h,
            COLOR_DEFAULT,
            small.as_mut_ptr(),
            small.len(),
        )
    };
    assert_eq!(n, buf.len());
    assert_eq!(small, vec![7u8; 8]);

    let bad = b"<svg";
    let n = unsafe {
        patina_svg_render(
            bad.as_ptr(),
            bad.len(),
            w,
            h,
            COLOR_DEFAULT,
            buf.as_mut_ptr(),
            buf.len(),
        )
    };
    assert_eq!(n, 0);

    let image = patina_node_new(KIND_IMAGE);
    assert_eq!(
        unsafe { patina_set_svg(image, CIRCLE.as_ptr(), CIRCLE.len()) },
        1
    );
    assert_eq!(unsafe { patina_set_svg(image, bad.as_ptr(), bad.len()) }, 0);
    let button = patina_node_new(KIND_BUTTON);
    assert_eq!(
        unsafe { patina_set_svg(button, CIRCLE.as_ptr(), CIRCLE.len()) },
        1
    );
    // Clearing with an empty string succeeds.
    assert_eq!(unsafe { patina_set_svg(button, std::ptr::null(), 0) }, 1);
}
