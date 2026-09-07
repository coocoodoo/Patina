//! A practical subset of SMIL animation for SVG.
//!
//! Supported: `<animate>`, `<animateTransform>`, `<set>` and `<animateMotion>` with
//! `from`/`to`/`by`/`values`, `dur`, `begin` (offsets only), `repeatCount`/`repeatDur`,
//! `fill="freeze"`, `calcMode` (discrete, linear, paced, spline), `keyTimes`, `keySplines`,
//! `additive="sum"` for transforms, and for motion: `path`, `<mpath>`, `keyPoints` and
//! `rotate` (`auto`, `auto-reverse` or an angle). Numbers, number lists (points, transform
//! arguments) and colors interpolate; anything else steps.
//!
//! Rather than animating a parsed tree, the document is re-materialized as text for a given
//! time, so the regular SVG renderer draws each frame unchanged.

use std::collections::HashMap;
use std::f32::consts::TAU;
use std::ops::Range;

use crate::color::Rgba;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Numbers(Vec<f32>),
    Color(Rgba),
    Text(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CalcMode {
    Discrete,
    Linear,
    Spline,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TransformKind {
    Rotate,
    Scale,
    Translate,
    SkewX,
    SkewY,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Rotate {
    None,
    Auto,
    AutoReverse,
    Fixed(f32),
}

/// A motion path flattened to a polyline with cumulative lengths.
#[derive(Clone, Debug)]
struct MotionPath {
    pts: Vec<(f32, f32)>,
    cum: Vec<f32>,
    total: f32,
}

impl MotionPath {
    /// Build from points; a `true` flag marks the start of a new subpath (a jump).
    fn from_polyline(poly: &[(f32, f32, bool)]) -> Option<MotionPath> {
        if poly.is_empty() {
            return None;
        }
        let mut pts = Vec::with_capacity(poly.len());
        let mut cum = Vec::with_capacity(poly.len());
        let mut total = 0.0f32;
        for (i, &(x, y, jump)) in poly.iter().enumerate() {
            if i > 0 && !jump {
                let (px, py): (f32, f32) = pts[i - 1];
                total += ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            }
            pts.push((x, y));
            cum.push(total);
        }
        Some(MotionPath { pts, cum, total })
    }

    fn segment_angle(&self, i: usize) -> Option<f32> {
        if i + 1 < self.pts.len() && self.cum[i + 1] > self.cum[i] {
            let (x0, y0) = self.pts[i];
            let (x1, y1) = self.pts[i + 1];
            Some((y1 - y0).atan2(x1 - x0).to_degrees())
        } else {
            None
        }
    }

    /// Position and tangent angle (degrees) at `dist` along the path.
    fn at(&self, dist: f32) -> (f32, f32, f32) {
        let n = self.pts.len();
        if n == 1 || self.total <= 0.0 {
            let (x, y) = self.pts[n - 1];
            return (x, y, 0.0);
        }
        let d = dist.clamp(0.0, self.total);
        let mut i = 0;
        while i + 1 < n && self.cum[i + 1] < d {
            i += 1;
        }
        // Skip zero-length jumps that sit exactly at `d`.
        while i + 1 < n && self.cum[i + 1] == self.cum[i] && self.cum[i] <= d {
            i += 1;
        }
        if i + 1 >= n {
            let (x, y) = self.pts[n - 1];
            let angle = (0..n - 1)
                .rev()
                .find_map(|j| self.segment_angle(j))
                .unwrap_or(0.0);
            return (x, y, angle);
        }
        let (x0, y0) = self.pts[i];
        let (x1, y1) = self.pts[i + 1];
        let seg = self.cum[i + 1] - self.cum[i];
        let t = if seg > 0.0 {
            ((d - self.cum[i]) / seg).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let angle = self
            .segment_angle(i)
            .or_else(|| (0..i).rev().find_map(|j| self.segment_angle(j)))
            .unwrap_or(0.0);
        (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, angle)
    }
}

#[derive(Clone, Debug)]
struct MotionTrack {
    path: MotionPath,
    rotate: Rotate,
}

#[derive(Clone, Debug)]
struct Track {
    /// Byte offset right after the parent's tag name, where a new attribute can be inserted.
    insert_at: usize,
    /// Range of the existing attribute's value (between the quotes), if the parent has it.
    existing: Option<Range<usize>>,
    existing_value: String,
    attr: String,
    transform: Option<TransformKind>,
    motion: Option<MotionTrack>,
    additive: bool,
    values: Vec<Value>,
    key_times: Vec<f32>,
    key_splines: Vec<[f32; 4]>,
    calc_mode: CalcMode,
    begin: f64,
    dur: f64,
    repeat: f64,
    freeze: bool,
}

/// The animations of one document.
#[derive(Clone, Debug, Default)]
pub struct Timeline {
    tracks: Vec<Track>,
    removals: Vec<Range<usize>>,
}

fn parse_pairs(s: &str) -> Vec<(f32, f32)> {
    s.split(';')
        .filter_map(|p| match parse_value(p) {
            Value::Numbers(v) if v.len() >= 2 => Some((v[0], v[1])),
            _ => None,
        })
        .collect()
}

fn parse_floats(s: &str) -> Vec<f32> {
    s.split(';')
        .filter_map(|v| v.trim().parse::<f32>().ok())
        .collect()
}

impl Timeline {
    /// Extract the animations from SVG markup. Returns `None` when there are none.
    pub fn parse(src: &str) -> Option<Timeline> {
        let doc = roxmltree::Document::parse(src).ok()?;
        let mut tl = Timeline::default();
        for node in doc.descendants().filter(|n| n.is_element()) {
            let tag = node.tag_name().name();
            let (is_transform, is_motion) = match tag {
                "animate" | "set" => (false, false),
                "animateTransform" => (true, false),
                "animateMotion" => (false, true),
                "animateColor" | "discard" => {
                    // Unsupported: drop the element so usvg does not choke on it.
                    tl.removals.push(node.range());
                    continue;
                }
                _ => continue,
            };
            tl.removals.push(node.range());
            let Some(parent) = node.parent_element() else {
                continue;
            };
            let attr = match node.attribute("attributeName") {
                Some(a) if !is_motion => a.trim().to_string(),
                _ if is_transform || is_motion => "transform".to_string(),
                _ => continue,
            };
            if attr.is_empty() {
                continue;
            }
            let transform = if is_transform {
                Some(match node.attribute("type").map(|t| t.trim()) {
                    Some("rotate") => TransformKind::Rotate,
                    Some("scale") => TransformKind::Scale,
                    Some("skewX") => TransformKind::SkewX,
                    Some("skewY") => TransformKind::SkewY,
                    _ => TransformKind::Translate,
                })
            } else {
                None
            };

            // Where the parent's start tag ends its name, and its existing attribute value.
            let pstart = parent.range().start;
            let mut insert_at = pstart + 1;
            for (i, ch) in src[pstart + 1..].char_indices() {
                if ch.is_whitespace() || ch == '/' || ch == '>' {
                    insert_at = pstart + 1 + i;
                    break;
                }
            }
            let mut existing = None;
            let mut existing_value = String::new();
            for a in parent.attributes() {
                if a.name() == attr {
                    if let Some(r) = value_range(src, a.range()) {
                        existing_value = src[r.clone()].to_string();
                        existing = Some(r);
                    }
                }
            }

            let calc_mode_attr = node.attribute("calcMode").map(str::trim);

            // Keyframes.
            let mut values: Vec<Value> = Vec::new();
            let mut motion: Option<MotionTrack> = None;
            let mut motion_key_times: Option<Vec<f32>> = None;
            if is_motion {
                // The path: a `path` attribute, an `<mpath>` reference, or coordinate pairs.
                let mut path: Option<MotionPath> = None;
                if let Some(d) = node.attribute("path") {
                    path = MotionPath::from_polyline(&flatten_path(d));
                }
                if path.is_none() {
                    if let Some(mpath) = node
                        .children()
                        .find(|c| c.is_element() && c.tag_name().name() == "mpath")
                    {
                        let href = mpath
                            .attributes()
                            .find(|a| a.name() == "href")
                            .map(|a| a.value().trim());
                        if let Some(id) = href.and_then(|h| h.strip_prefix('#')) {
                            if let Some(target) =
                                doc.descendants().find(|n| n.attribute("id") == Some(id))
                            {
                                if let Some(d) = target.attribute("d") {
                                    path = MotionPath::from_polyline(&flatten_path(d));
                                }
                            }
                        }
                    }
                }
                let mut points: Vec<(f32, f32)> = Vec::new();
                if path.is_none() {
                    if let Some(v) = node.attribute("values") {
                        points = parse_pairs(v);
                    } else if let Some(to) = node.attribute("to") {
                        let from = node
                            .attribute("from")
                            .and_then(|f| parse_pairs(f).first().copied())
                            .unwrap_or((0.0, 0.0));
                        if let Some(t) = parse_pairs(to).first().copied() {
                            points = vec![from, t];
                        }
                    } else if let Some(by) = node.attribute("by") {
                        let from = node
                            .attribute("from")
                            .and_then(|f| parse_pairs(f).first().copied())
                            .unwrap_or((0.0, 0.0));
                        if let Some((dx, dy)) = parse_pairs(by).first().copied() {
                            points = vec![from, (from.0 + dx, from.1 + dy)];
                        }
                    }
                    if points.len() >= 2 {
                        let poly: Vec<(f32, f32, bool)> = points
                            .iter()
                            .enumerate()
                            .map(|(i, &(x, y))| (x, y, i == 0))
                            .collect();
                        path = MotionPath::from_polyline(&poly);
                    }
                }
                let Some(path) = path else {
                    continue;
                };
                let rotate = match node.attribute("rotate").map(str::trim) {
                    Some("auto") => Rotate::Auto,
                    Some("auto-reverse") => Rotate::AutoReverse,
                    Some(v) => v.parse::<f32>().map(Rotate::Fixed).unwrap_or(Rotate::None),
                    None => Rotate::None,
                };
                let key_points: Vec<f32> = node
                    .attribute("keyPoints")
                    .map(parse_floats)
                    .unwrap_or_default();
                let key_times_attr: Vec<f32> = node
                    .attribute("keyTimes")
                    .map(parse_floats)
                    .unwrap_or_default();
                let paced = !matches!(calc_mode_attr, Some("linear") | Some("discrete"));
                if key_points.len() >= 2 && key_times_attr.len() == key_points.len() {
                    values = key_points
                        .iter()
                        .map(|f| Value::Numbers(vec![f.clamp(0.0, 1.0)]))
                        .collect();
                    motion_key_times = Some(key_times_attr);
                } else if !points.is_empty() && !paced && path.total > 0.0 {
                    // Equal time per segment between the listed points.
                    values = path
                        .cum
                        .iter()
                        .map(|c| Value::Numbers(vec![c / path.total]))
                        .collect();
                } else {
                    values = vec![Value::Numbers(vec![0.0]), Value::Numbers(vec![1.0])];
                }
                motion = Some(MotionTrack { path, rotate });
            } else if let Some(v) = node.attribute("values") {
                values = v
                    .split(';')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(parse_value)
                    .collect();
            } else if let Some(to) = node.attribute("to") {
                if let Some(from) = node.attribute("from") {
                    values.push(parse_value(from));
                } else if !existing_value.is_empty() && tag != "set" {
                    values.push(parse_value(&existing_value));
                }
                values.push(parse_value(to));
            } else if let Some(by) = node.attribute("by") {
                let base = if existing_value.is_empty() {
                    Value::Numbers(vec![0.0])
                } else {
                    parse_value(&existing_value)
                };
                if let (Value::Numbers(b), Value::Numbers(d)) = (&base, parse_value(by)) {
                    if b.len() == d.len() {
                        values.push(base.clone());
                        values.push(Value::Numbers(
                            b.iter().zip(&d).map(|(x, y)| x + y).collect(),
                        ));
                    }
                }
            }
            if values.is_empty() {
                continue;
            }

            let n = values.len();
            let mut key_times: Vec<f32> = (0..n)
                .map(|i| {
                    if n > 1 {
                        i as f32 / (n - 1) as f32
                    } else {
                        0.0
                    }
                })
                .collect();
            if let Some(kt) = node.attribute("keyTimes") {
                let parsed = parse_floats(kt);
                if parsed.len() == n && parsed.first().copied().unwrap_or(1.0) == 0.0 {
                    key_times = parsed;
                }
            }
            if let Some(kt) = motion_key_times {
                key_times = kt;
            }
            let calc_mode = match calc_mode_attr {
                Some("discrete") => CalcMode::Discrete,
                Some("spline") => CalcMode::Spline,
                _ => CalcMode::Linear,
            };
            let mut key_splines = Vec::new();
            if calc_mode == CalcMode::Spline {
                if let Some(ks) = node.attribute("keySplines") {
                    for seg in ks.split(';') {
                        let nums: Vec<f32> = seg
                            .split(|c: char| c.is_whitespace() || c == ',')
                            .filter(|s| !s.is_empty())
                            .filter_map(|s| s.parse::<f32>().ok())
                            .collect();
                        if nums.len() == 4 {
                            key_splines.push([nums[0], nums[1], nums[2], nums[3]]);
                        }
                    }
                }
            }

            let begin = node.attribute("begin").and_then(parse_clock).unwrap_or(0.0);
            if node
                .attribute("begin")
                .map(|b| b.trim() == "indefinite")
                .unwrap_or(false)
            {
                continue;
            }
            let dur = match node.attribute("dur").map(str::trim) {
                Some("indefinite") | None => {
                    if tag == "set" {
                        f64::INFINITY
                    } else {
                        continue;
                    }
                }
                Some(d) => match parse_clock(d) {
                    Some(v) if v > 0.0 => v,
                    _ => continue,
                },
            };
            let mut repeat = match node.attribute("repeatCount").map(str::trim) {
                Some("indefinite") => f64::INFINITY,
                Some(c) => c.parse::<f64>().ok().filter(|v| *v > 0.0).unwrap_or(1.0),
                None => 1.0,
            };
            if node
                .attribute("repeatDur")
                .map(|d| d.trim() == "indefinite")
                .unwrap_or(false)
            {
                repeat = f64::INFINITY;
            }
            let freeze = node
                .attribute("fill")
                .map(|f| f.trim() == "freeze")
                .unwrap_or(false)
                || tag == "set";
            let additive = node
                .attribute("additive")
                .map(|a| a.trim() == "sum")
                .unwrap_or(false);

            tl.tracks.push(Track {
                insert_at,
                existing,
                existing_value,
                attr,
                transform,
                motion,
                additive,
                values,
                key_times,
                key_splines,
                calc_mode,
                begin,
                dur,
                repeat,
                freeze,
            });
        }
        if tl.tracks.is_empty() && tl.removals.is_empty() {
            return None;
        }
        Some(tl)
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// The loop length when every track repeats forever with the same duration.
    pub fn period(&self) -> Option<f64> {
        let mut period: Option<f64> = None;
        for t in &self.tracks {
            if !t.repeat.is_infinite() || t.begin != 0.0 {
                return None;
            }
            match period {
                None => period = Some(t.dur),
                Some(p) if (p - t.dur).abs() < 1e-3 => {}
                Some(_) => return None,
            }
        }
        period
    }

    /// Time after which nothing changes anymore, if the animation ends. A track with an
    /// indefinite duration (a `<set>` without `dur`) stops changing once it begins.
    pub fn end(&self) -> Option<f64> {
        let mut end: f64 = 0.0;
        for t in &self.tracks {
            if t.dur.is_infinite() {
                end = end.max(t.begin);
                continue;
            }
            if t.repeat.is_infinite() {
                return None;
            }
            end = end.max(t.begin + t.dur * t.repeat);
        }
        Some(end)
    }

    /// Materialize the document at `t` seconds.
    pub fn apply(&self, src: &str, t: f64) -> String {
        let mut edits: Vec<(Range<usize>, String)> = Vec::new();
        let mut slot: HashMap<(usize, &str), usize> = HashMap::new();
        for tr in &self.tracks {
            let Some(v) = tr.value_at(t) else {
                continue;
            };
            let text = match &tr.motion {
                Some(m) => {
                    let frac = match &v {
                        Value::Numbers(n) => n.first().copied().unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let (x, y, tangent) = m.path.at(frac.clamp(0.0, 1.0) * m.path.total);
                    let angle = match m.rotate {
                        Rotate::None => 0.0,
                        Rotate::Auto => tangent,
                        Rotate::AutoReverse => tangent + 180.0,
                        Rotate::Fixed(a) => a,
                    };
                    let mut s = format!("translate({} {})", fmt_num(x), fmt_num(y));
                    if angle.abs() > 1e-4 {
                        s = format!("{s} rotate({})", fmt_num(angle));
                    }
                    // The motion happens in the parent's coordinate system, so the element's
                    // own transform is applied after it.
                    if !tr.existing_value.trim().is_empty() {
                        s = format!("{s} {}", tr.existing_value.trim());
                    }
                    s
                }
                None => {
                    let mut text = format_value(&v, tr.transform);
                    if tr.transform.is_some() && tr.additive && !tr.existing_value.is_empty() {
                        text = format!("{} {}", tr.existing_value, text);
                    }
                    text
                }
            };
            let text = escape_attr(&text);
            let edit = match &tr.existing {
                Some(r) => (r.clone(), text),
                None => (
                    tr.insert_at..tr.insert_at,
                    format!(" {}=\"{}\"", tr.attr, text),
                ),
            };
            let key = (tr.insert_at, tr.attr.as_str());
            match slot.get(&key) {
                Some(&i) => edits[i] = edit,
                None => {
                    slot.insert(key, edits.len());
                    edits.push(edit);
                }
            }
        }
        for r in &self.removals {
            edits.push((r.clone(), String::new()));
        }
        // Apply from the end so earlier offsets stay valid.
        edits.sort_by(|a, b| b.0.start.cmp(&a.0.start).then(b.0.end.cmp(&a.0.end)));
        let mut out = src.to_string();
        for (r, rep) in edits {
            if r.start <= r.end && r.end <= out.len() {
                out.replace_range(r, &rep);
            }
        }
        out
    }
}

impl Track {
    fn value_at(&self, t: f64) -> Option<Value> {
        let local = t - self.begin;
        if local < 0.0 {
            return None;
        }
        let frac = if self.dur.is_infinite() {
            0.0
        } else {
            let iter = local / self.dur;
            if iter >= self.repeat {
                if self.freeze {
                    1.0
                } else {
                    return None;
                }
            } else {
                iter.fract() as f32
            }
        };
        let n = self.values.len();
        if n == 1 {
            return Some(self.values[0].clone());
        }
        let times = &self.key_times;
        let mut i = 0;
        while i + 2 < n && frac >= times[i + 1] {
            i += 1;
        }
        if frac >= 1.0 {
            return Some(self.values[n - 1].clone());
        }
        let span = (times[i + 1] - times[i]).max(1e-6);
        let mut local_t = ((frac - times[i]) / span).clamp(0.0, 1.0);
        match self.calc_mode {
            CalcMode::Discrete => return Some(self.values[i].clone()),
            CalcMode::Spline => {
                if let Some(ks) = self.key_splines.get(i) {
                    local_t = bezier_y(*ks, local_t);
                }
            }
            CalcMode::Linear => {}
        }
        Some(interpolate(&self.values[i], &self.values[i + 1], local_t))
    }
}

// ---- SVG path flattening (for motion paths) ---------------------------------------------

fn skip_sep(b: &[u8], i: &mut usize) {
    while *i < b.len() && (b[*i].is_ascii_whitespace() || b[*i] == b',') {
        *i += 1;
    }
}

fn num(b: &[u8], i: &mut usize) -> Option<f32> {
    skip_sep(b, i);
    let start = *i;
    let mut j = *i;
    if j < b.len() && (b[j] == b'-' || b[j] == b'+') {
        j += 1;
    }
    let mut digits = 0;
    while j < b.len() && b[j].is_ascii_digit() {
        j += 1;
        digits += 1;
    }
    if j < b.len() && b[j] == b'.' {
        j += 1;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return None;
    }
    if j < b.len() && (b[j] == b'e' || b[j] == b'E') {
        let mut k = j + 1;
        if k < b.len() && (b[k] == b'-' || b[k] == b'+') {
            k += 1;
        }
        let mut ed = 0;
        while k < b.len() && b[k].is_ascii_digit() {
            k += 1;
            ed += 1;
        }
        if ed > 0 {
            j = k;
        }
    }
    let v = std::str::from_utf8(&b[start..j])
        .ok()?
        .parse::<f32>()
        .ok()?;
    *i = j;
    Some(v)
}

fn flag(b: &[u8], i: &mut usize) -> Option<bool> {
    skip_sep(b, i);
    if *i < b.len() && (b[*i] == b'0' || b[*i] == b'1') {
        let v = b[*i] == b'1';
        *i += 1;
        Some(v)
    } else {
        None
    }
}

fn flatten_cubic(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    out: &mut Vec<(f32, f32, bool)>,
) {
    const N: usize = 24;
    for k in 1..=N {
        let t = k as f32 / N as f32;
        let u = 1.0 - t;
        let x =
            u * u * u * p0.0 + 3.0 * u * u * t * p1.0 + 3.0 * u * t * t * p2.0 + t * t * t * p3.0;
        let y =
            u * u * u * p0.1 + 3.0 * u * u * t * p1.1 + 3.0 * u * t * t * p2.1 + t * t * t * p3.1;
        out.push((x, y, false));
    }
}

fn flatten_quad(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), out: &mut Vec<(f32, f32, bool)>) {
    const N: usize = 16;
    for k in 1..=N {
        let t = k as f32 / N as f32;
        let u = 1.0 - t;
        let x = u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0;
        let y = u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1;
        out.push((x, y, false));
    }
}

#[allow(clippy::too_many_arguments)]
fn flatten_arc(
    x1: f32,
    y1: f32,
    rx: f32,
    ry: f32,
    phi_deg: f32,
    large: bool,
    sweep: bool,
    x2: f32,
    y2: f32,
    out: &mut Vec<(f32, f32, bool)>,
) {
    if (x1 - x2).abs() < 1e-6 && (y1 - y2).abs() < 1e-6 {
        return;
    }
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx < 1e-6 || ry < 1e-6 {
        out.push((x2, y2, false));
        return;
    }
    let (sinp, cosp) = phi_deg.to_radians().sin_cos();
    let dx = (x1 - x2) * 0.5;
    let dy = (y1 - y2) * 0.5;
    let x1p = cosp * dx + sinp * dy;
    let y1p = -sinp * dx + cosp * dy;
    let lambda = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }
    let num = (rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p).max(0.0);
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let mut coef = if den > 0.0 { (num / den).sqrt() } else { 0.0 };
    if large == sweep {
        coef = -coef;
    }
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;
    let cx = cosp * cxp - sinp * cyp + (x1 + x2) * 0.5;
    let cy = sinp * cxp + cosp * cyp + (y1 + y2) * 0.5;
    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| -> f32 {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        if len <= 0.0 {
            return 0.0;
        }
        let mut a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };
    let (ux, uy) = ((x1p - cxp) / rx, (y1p - cyp) / ry);
    let (vx, vy) = ((-x1p - cxp) / rx, (-y1p - cyp) / ry);
    let theta1 = angle(1.0, 0.0, ux, uy);
    let mut dtheta = angle(ux, uy, vx, vy);
    if !sweep && dtheta > 0.0 {
        dtheta -= TAU;
    } else if sweep && dtheta < 0.0 {
        dtheta += TAU;
    }
    // About one sample per 5 degrees; the tiny bias keeps exact multiples on a vertex.
    let steps = ((dtheta.abs().to_degrees() / 5.0 - 1e-3).ceil() as usize).clamp(4, 96);
    for k in 1..=steps {
        let t = theta1 + dtheta * k as f32 / steps as f32;
        let (st, ct) = t.sin_cos();
        let x = cosp * rx * ct - sinp * ry * st + cx;
        let y = sinp * rx * ct + cosp * ry * st + cy;
        out.push((x, y, false));
    }
    if let Some(last) = out.last_mut() {
        *last = (x2, y2, false);
    }
}

/// Flatten an SVG path (`d` attribute) into a polyline. A `true` flag marks the first point
/// of a subpath (a jump from the previous position).
fn flatten_path(d: &str) -> Vec<(f32, f32, bool)> {
    let b = d.as_bytes();
    let mut i = 0usize;
    let mut out: Vec<(f32, f32, bool)> = Vec::new();
    let mut cmd: u8 = 0;
    let (mut cx, mut cy) = (0.0f32, 0.0f32);
    let (mut sx, mut sy) = (0.0f32, 0.0f32);
    let mut last_ctrl: Option<(f32, f32)> = None;
    let mut last_cmd: u8 = 0;
    loop {
        skip_sep(b, &mut i);
        if i >= b.len() {
            break;
        }
        if b[i].is_ascii_alphabetic() {
            cmd = b[i];
            i += 1;
        } else if cmd == 0 || cmd.eq_ignore_ascii_case(&b'Z') {
            break;
        } else if cmd == b'M' {
            cmd = b'L';
        } else if cmd == b'm' {
            cmd = b'l';
        }
        let rel = cmd.is_ascii_lowercase();
        let (ox, oy) = if rel { (cx, cy) } else { (0.0, 0.0) };
        let up = cmd.to_ascii_uppercase();
        match up {
            b'M' => {
                let (Some(x), Some(y)) = (num(b, &mut i), num(b, &mut i)) else {
                    break;
                };
                cx = ox + x;
                cy = oy + y;
                sx = cx;
                sy = cy;
                out.push((cx, cy, true));
                last_ctrl = None;
            }
            b'L' => {
                let (Some(x), Some(y)) = (num(b, &mut i), num(b, &mut i)) else {
                    break;
                };
                cx = ox + x;
                cy = oy + y;
                out.push((cx, cy, false));
                last_ctrl = None;
            }
            b'H' => {
                let Some(x) = num(b, &mut i) else { break };
                cx = ox + x;
                out.push((cx, cy, false));
                last_ctrl = None;
            }
            b'V' => {
                let Some(y) = num(b, &mut i) else { break };
                cy = oy + y;
                out.push((cx, cy, false));
                last_ctrl = None;
            }
            b'C' => {
                let mut v = [0.0f32; 6];
                let mut ok = true;
                for slot in v.iter_mut() {
                    match num(b, &mut i) {
                        Some(x) => *slot = x,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    break;
                }
                let p1 = (ox + v[0], oy + v[1]);
                let p2 = (ox + v[2], oy + v[3]);
                let p3 = (ox + v[4], oy + v[5]);
                flatten_cubic((cx, cy), p1, p2, p3, &mut out);
                last_ctrl = Some(p2);
                cx = p3.0;
                cy = p3.1;
            }
            b'S' => {
                let mut v = [0.0f32; 4];
                let mut ok = true;
                for slot in v.iter_mut() {
                    match num(b, &mut i) {
                        Some(x) => *slot = x,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    break;
                }
                let p1 = match (last_cmd, last_ctrl) {
                    (b'C' | b'S', Some((px, py))) => (2.0 * cx - px, 2.0 * cy - py),
                    _ => (cx, cy),
                };
                let p2 = (ox + v[0], oy + v[1]);
                let p3 = (ox + v[2], oy + v[3]);
                flatten_cubic((cx, cy), p1, p2, p3, &mut out);
                last_ctrl = Some(p2);
                cx = p3.0;
                cy = p3.1;
            }
            b'Q' => {
                let mut v = [0.0f32; 4];
                let mut ok = true;
                for slot in v.iter_mut() {
                    match num(b, &mut i) {
                        Some(x) => *slot = x,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok {
                    break;
                }
                let p1 = (ox + v[0], oy + v[1]);
                let p2 = (ox + v[2], oy + v[3]);
                flatten_quad((cx, cy), p1, p2, &mut out);
                last_ctrl = Some(p1);
                cx = p2.0;
                cy = p2.1;
            }
            b'T' => {
                let (Some(x), Some(y)) = (num(b, &mut i), num(b, &mut i)) else {
                    break;
                };
                let p1 = match (last_cmd, last_ctrl) {
                    (b'Q' | b'T', Some((px, py))) => (2.0 * cx - px, 2.0 * cy - py),
                    _ => (cx, cy),
                };
                let p2 = (ox + x, oy + y);
                flatten_quad((cx, cy), p1, p2, &mut out);
                last_ctrl = Some(p1);
                cx = p2.0;
                cy = p2.1;
            }
            b'A' => {
                let (Some(rx), Some(ry), Some(rot)) =
                    (num(b, &mut i), num(b, &mut i), num(b, &mut i))
                else {
                    break;
                };
                let (Some(large), Some(sweep)) = (flag(b, &mut i), flag(b, &mut i)) else {
                    break;
                };
                let (Some(x), Some(y)) = (num(b, &mut i), num(b, &mut i)) else {
                    break;
                };
                let (ex, ey) = (ox + x, oy + y);
                flatten_arc(cx, cy, rx, ry, rot, large, sweep, ex, ey, &mut out);
                cx = ex;
                cy = ey;
                last_ctrl = None;
            }
            b'Z' => {
                if (cx - sx).abs() > 1e-6 || (cy - sy).abs() > 1e-6 {
                    out.push((sx, sy, false));
                }
                cx = sx;
                cy = sy;
                last_ctrl = None;
            }
            _ => break,
        }
        last_cmd = up;
    }
    out
}

// ---- helpers ---------------------------------------------------------------------------

/// The value range (inside the quotes) of an attribute given the range roxmltree reports,
/// which may cover either the whole `name="value"` or just the value.
fn value_range(src: &str, r: Range<usize>) -> Option<Range<usize>> {
    let s = src.get(r.clone())?;
    if let Some(eq) = s.find('=') {
        let rest = &s[eq + 1..];
        let q_off = rest.find(['"', '\''])?;
        let quote = rest.as_bytes()[q_off] as char;
        let start = r.start + eq + 1 + q_off + 1;
        let end_off = src[start..].find(quote)?;
        Some(start..start + end_off)
    } else {
        Some(r)
    }
}

fn parse_clock(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() || s == "indefinite" {
        return None;
    }
    let (num, mult) = if let Some(v) = s.strip_suffix("ms") {
        (v, 0.001)
    } else if let Some(v) = s.strip_suffix("min") {
        (v, 60.0)
    } else if let Some(v) = s.strip_suffix('h') {
        (v, 3600.0)
    } else if let Some(v) = s.strip_suffix('s') {
        (v, 1.0)
    } else if s.contains(':') {
        let parts: Vec<f64> = s
            .split(':')
            .filter_map(|p| p.trim().parse::<f64>().ok())
            .collect();
        return match parts.len() {
            2 => Some(parts[0] * 60.0 + parts[1]),
            3 => Some(parts[0] * 3600.0 + parts[1] * 60.0 + parts[2]),
            _ => None,
        };
    } else {
        (s, 1.0)
    };
    num.trim().parse::<f64>().ok().map(|v| v * mult)
}

fn parse_value(s: &str) -> Value {
    let s = s.trim();
    if let Some(c) = parse_color(s) {
        return Value::Color(c);
    }
    let nums: Option<Vec<f32>> = s
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .map(|p| p.parse::<f32>().ok())
        .collect();
    match nums {
        Some(v) if !v.is_empty() => Value::Numbers(v),
        _ => Value::Text(s.to_string()),
    }
}

fn parse_color(s: &str) -> Option<Rgba> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let v = u32::from_str_radix(hex, 16).ok()?;
        return match hex.len() {
            3 => {
                let r = (v >> 8) & 0xf;
                let g = (v >> 4) & 0xf;
                let b = v & 0xf;
                Some(Rgba::hex((r * 17) << 16 | (g * 17) << 8 | (b * 17)))
            }
            6 => Some(Rgba::hex(v)),
            8 => Some(Rgba::from_rgba_u32(v)),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    if let Some(inner) = lower
        .strip_prefix("rgba(")
        .or_else(|| lower.strip_prefix("rgb("))
    {
        let inner = inner.strip_suffix(')')?;
        let parts: Vec<f32> = inner
            .split([',', ' '])
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.trim_end_matches('%').parse::<f32>().ok())
            .collect();
        if parts.len() < 3 {
            return None;
        }
        let a = parts.get(3).copied().unwrap_or(1.0);
        return Some(Rgba::new(
            parts[0] / 255.0,
            parts[1] / 255.0,
            parts[2] / 255.0,
            a,
        ));
    }
    Some(match lower.as_str() {
        "black" => Rgba::hex(0x000000),
        "white" => Rgba::hex(0xffffff),
        "red" => Rgba::hex(0xff0000),
        "green" => Rgba::hex(0x008000),
        "lime" => Rgba::hex(0x00ff00),
        "blue" => Rgba::hex(0x0000ff),
        "yellow" => Rgba::hex(0xffff00),
        "orange" => Rgba::hex(0xffa500),
        "purple" => Rgba::hex(0x800080),
        "gray" | "grey" => Rgba::hex(0x808080),
        "transparent" => Rgba::TRANSPARENT,
        _ => return None,
    })
}

fn interpolate(a: &Value, b: &Value, t: f32) -> Value {
    match (a, b) {
        (Value::Numbers(x), Value::Numbers(y)) if x.len() == y.len() => {
            Value::Numbers(x.iter().zip(y).map(|(p, q)| p + (q - p) * t).collect())
        }
        (Value::Color(x), Value::Color(y)) => Value::Color(x.mix(*y, t)),
        _ => {
            if t < 1.0 {
                a.clone()
            } else {
                b.clone()
            }
        }
    }
}

fn fmt_num(v: f32) -> String {
    if v.fract() == 0.0 && v.abs() < 1e7 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v:.4}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn format_value(v: &Value, transform: Option<TransformKind>) -> String {
    match v {
        Value::Numbers(n) => {
            let joined = n.iter().map(|x| fmt_num(*x)).collect::<Vec<_>>().join(" ");
            match transform {
                Some(TransformKind::Rotate) => format!("rotate({joined})"),
                Some(TransformKind::Scale) => format!("scale({joined})"),
                Some(TransformKind::Translate) => format!("translate({joined})"),
                Some(TransformKind::SkewX) => format!("skewX({joined})"),
                Some(TransformKind::SkewY) => format!("skewY({joined})"),
                None => joined,
            }
        }
        Value::Color(c) => {
            let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            if c.a >= 0.999 {
                format!("#{:02x}{:02x}{:02x}", ch(c.r), ch(c.g), ch(c.b))
            } else {
                format!("rgba({},{},{},{})", ch(c.r), ch(c.g), ch(c.b), fmt_num(c.a))
            }
        }
        Value::Text(s) => s.clone(),
    }
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

/// Y of a CSS-style cubic bezier (P0 = 0,0 and P3 = 1,1) at the given x.
fn bezier_y(k: [f32; 4], x: f32) -> f32 {
    let (x1, y1, x2, y2) = (k[0], k[1], k[2], k[3]);
    let bx =
        |t: f32| 3.0 * (1.0 - t) * (1.0 - t) * t * x1 + 3.0 * (1.0 - t) * t * t * x2 + t * t * t;
    let by =
        |t: f32| 3.0 * (1.0 - t) * (1.0 - t) * t * y1 + 3.0 * (1.0 - t) * t * t * y2 + t * t * t;
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let mut t = x;
    for _ in 0..24 {
        t = 0.5 * (lo + hi);
        if bx(t) < x {
            lo = t;
        } else {
            hi = t;
        }
    }
    by(t).clamp(0.0, 1.0)
}
