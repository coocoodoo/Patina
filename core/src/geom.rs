//! Minimal geometry helpers (physical pixels unless stated otherwise).

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }

    pub fn inset(&self, d: f32) -> Rect {
        Rect::new(
            self.x + d,
            self.y + d,
            (self.w - 2.0 * d).max(0.0),
            (self.h - 2.0 * d).max(0.0),
        )
    }

    pub fn outset(&self, d: f32) -> Rect {
        self.inset(-d)
    }

    pub fn intersect(&self, o: &Rect) -> Rect {
        let x1 = self.x.max(o.x);
        let y1 = self.y.max(o.y);
        let x2 = self.right().min(o.right());
        let y2 = self.bottom().min(o.bottom());
        Rect::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
    }

    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    pub fn to_skia(&self) -> Option<tiny_skia::Rect> {
        tiny_skia::Rect::from_xywh(self.x, self.y, self.w, self.h)
    }

    /// Snap to whole pixels (expanding), which keeps 1px borders crisp.
    pub fn round_out(&self) -> Rect {
        let x = self.x.floor();
        let y = self.y.floor();
        Rect::new(x, y, self.right().ceil() - x, self.bottom().ceil() - y)
    }
}
