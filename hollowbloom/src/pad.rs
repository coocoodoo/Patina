//! Game controllers: the Steam Deck's own controls and any Xbox-style pad, read straight from
//! the system with no extra libraries. On Linux that's the kernel's event devices (which is
//! how SteamOS hands the Deck's controls to a game, through Steam Input's virtual pad); on
//! Windows (and under Proton) it's XInput.

use glam::Vec2;

/// The Steam Deck's buttons. An Xbox pad has the same ones: its bumpers and triggers are
/// L1/R1 and L2/R2 here, and View and Menu are Back and Start.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PadButton {
    A,
    B,
    X,
    Y,
    L1,
    R1,
    L2,
    R2,
    /// Clicking the left and right sticks in.
    L3,
    R3,
    View,
    Menu,
    Up,
    Down,
    Left,
    Right,
}

pub const PAD_BUTTONS: usize = 16;

impl PadButton {
    pub fn bit(self) -> u16 {
        1 << self as u16
    }
}

/// Everything a pad is doing right now. Sticks run -1..1 with y pointing down the screen;
/// triggers run 0..1.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct PadState {
    pub connected: bool,
    pub buttons: u16,
    pub left: Vec2,
    pub right: Vec2,
    pub lt: f32,
    pub rt: f32,
}

impl PadState {
    pub fn held(&self, b: PadButton) -> bool {
        self.buttons & b.bit() != 0
    }

    fn merge(&mut self, o: &PadState) {
        self.connected |= o.connected;
        self.buttons |= o.buttons;
        if o.left.length_squared() > self.left.length_squared() {
            self.left = o.left;
        }
        if o.right.length_squared() > self.right.length_squared() {
            self.right = o.right;
        }
        self.lt = self.lt.max(o.lt);
        self.rt = self.rt.max(o.rt);
    }
}

/// Every pad plugged in, read together as one.
pub struct Pads {
    sys: sys::Backend,
}

impl Pads {
    pub fn new() -> Pads {
        Pads {
            sys: sys::Backend::new(),
        }
    }

    /// Reads what every pad did since last time. Pads plugged in later are picked up
    /// every couple of seconds.
    pub fn poll(&mut self, dt: f32) -> PadState {
        self.sys.poll(dt)
    }
}

impl Default for Pads {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
mod sys {
    //! The kernel's event devices: any device whose keys include the south face button
    //! (`BTN_SOUTH`) is a gamepad. Steam Input's virtual pad on the Deck is one.

    use std::ffi::{c_int, c_ulong};
    use std::fs::{File, OpenOptions};
    use std::io::{ErrorKind, Read};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::PathBuf;

    use super::{PadButton as P, PadState};

    const O_NONBLOCK: i32 = 0o4000;
    const EV_KEY: u16 = 1;
    const EV_ABS: u16 = 3;
    const BTN_SOUTH: usize = 0x130;

    unsafe extern "C" {
        fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }

    /// `struct input_absinfo`.
    #[repr(C)]
    #[derive(Default)]
    struct AbsInfo {
        value: i32,
        minimum: i32,
        maximum: i32,
        fuzz: i32,
        flat: i32,
        resolution: i32,
    }

    /// `EVIOCGABS(axis)`: an axis's range.
    fn eviocgabs(axis: u16) -> c_ulong {
        let size = std::mem::size_of::<AbsInfo>() as c_ulong;
        (2 << 30) | (size << 16) | (0x45 << 8) | (0x40 + axis as c_ulong)
    }

    /// The Xbox layout's buttons, as the kernel numbers them (the Deck's too).
    fn button(code: u16) -> Option<P> {
        Some(match code {
            0x130 => P::A,
            0x131 => P::B,
            0x133 => P::X,
            0x134 => P::Y,
            0x136 => P::L1,
            0x137 => P::R1,
            0x138 => P::L2,
            0x139 => P::R2,
            0x13a => P::View,
            0x13b => P::Menu,
            0x13d => P::L3,
            0x13e => P::R3,
            0x220 => P::Up,
            0x221 => P::Down,
            0x222 => P::Left,
            0x223 => P::Right,
            _ => return None,
        })
    }

    struct Dev {
        path: PathBuf,
        file: File,
        state: PadState,
        /// Each axis's (minimum, maximum), for the first 64 axes.
        range: [(f32, f32); 64],
        hat: (i32, i32),
    }

    impl Dev {
        fn open(path: PathBuf) -> Option<Dev> {
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(O_NONBLOCK)
                .open(&path)
                .ok()?;
            let mut range = [(-32768.0, 32767.0); 64];
            for (axis, r) in range.iter_mut().enumerate() {
                let mut info = AbsInfo::default();
                // SAFETY: EVIOCGABS fills in one `input_absinfo`, which `info` is.
                let ok = unsafe {
                    ioctl(
                        file.as_raw_fd(),
                        eviocgabs(axis as u16),
                        &mut info as *mut AbsInfo,
                    )
                } == 0;
                if ok && info.maximum > info.minimum {
                    *r = (info.minimum as f32, info.maximum as f32);
                }
            }
            Some(Dev {
                path,
                file,
                state: PadState {
                    connected: true,
                    ..Default::default()
                },
                range,
                hat: (0, 0),
            })
        }

        /// Reads everything waiting. False once the pad is gone.
        fn read(&mut self) -> bool {
            // `struct input_event`: a timeval, then type, code and value.
            let size = std::mem::size_of::<usize>() * 2 + 8;
            let mut buf = [0u8; 24 * 64];
            let want = size * (buf.len() / size);
            loop {
                match self.file.read(&mut buf[..want]) {
                    Ok(0) => return true,
                    Ok(n) => {
                        for ev in buf[..n].chunks_exact(size) {
                            let t = u16::from_ne_bytes([ev[size - 8], ev[size - 7]]);
                            let code = u16::from_ne_bytes([ev[size - 6], ev[size - 5]]);
                            let value = i32::from_ne_bytes([
                                ev[size - 4],
                                ev[size - 3],
                                ev[size - 2],
                                ev[size - 1],
                            ]);
                            self.apply(t, code, value);
                        }
                        if n < want {
                            return true;
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => return true,
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(_) => return false,
                }
            }
        }

        fn stick(&self, axis: u16, v: i32) -> f32 {
            let (lo, hi) = self.range[axis as usize & 63];
            let mid = (lo + hi) * 0.5;
            ((v as f32 - mid) / ((hi - lo) * 0.5).max(1.0)).clamp(-1.0, 1.0)
        }

        fn trigger(&self, axis: u16, v: i32) -> f32 {
            let (lo, hi) = self.range[axis as usize & 63];
            ((v as f32 - lo) / (hi - lo).max(1.0)).clamp(0.0, 1.0)
        }

        fn apply(&mut self, t: u16, code: u16, value: i32) {
            let s = &mut self.state;
            match t {
                EV_KEY => {
                    if let Some(b) = button(code) {
                        if value != 0 {
                            s.buttons |= b.bit();
                        } else {
                            s.buttons &= !b.bit();
                        }
                    }
                }
                EV_ABS => match code {
                    0 => self.state.left.x = self.stick(code, value),
                    1 => self.state.left.y = self.stick(code, value),
                    3 => self.state.right.x = self.stick(code, value),
                    4 => self.state.right.y = self.stick(code, value),
                    // Triggers: Z/RZ on Xbox pads, brake/gas on some, the second hat on
                    // the Deck's own driver.
                    2 | 10 | 0x13 => self.state.lt = self.trigger(code, value),
                    5 | 9 | 0x12 => self.state.rt = self.trigger(code, value),
                    0x10 | 0x11 => {
                        if code == 0x10 {
                            self.hat.0 = value.signum();
                        } else {
                            self.hat.1 = value.signum();
                        }
                        let s = &mut self.state;
                        let hat = [
                            (P::Left, self.hat.0 < 0),
                            (P::Right, self.hat.0 > 0),
                            (P::Up, self.hat.1 < 0),
                            (P::Down, self.hat.1 > 0),
                        ];
                        for (b, on) in hat {
                            if on {
                                s.buttons |= b.bit();
                            } else {
                                s.buttons &= !b.bit();
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// Is bit `bit` set in one of `/proc/bus/input/devices`' hex bitmaps (highest word
    /// first)?
    fn has_bit(words: &str, bit: usize) -> bool {
        let ws: Vec<&str> = words.split_whitespace().collect();
        let per = usize::BITS as usize;
        let (i, off) = (bit / per, bit % per);
        if i >= ws.len() {
            return false;
        }
        u64::from_str_radix(ws[ws.len() - 1 - i], 16).is_ok_and(|w| (w >> off) & 1 == 1)
    }

    pub struct Backend {
        devs: Vec<Dev>,
        rescan: f32,
    }

    impl Backend {
        pub fn new() -> Backend {
            let mut b = Backend {
                devs: Vec::new(),
                rescan: 0.0,
            };
            b.scan();
            b
        }

        fn scan(&mut self) {
            let Ok(list) = std::fs::read_to_string("/proc/bus/input/devices") else {
                return;
            };
            for block in list.split("\n\n") {
                let mut event = None;
                let mut gamepad = false;
                for line in block.lines() {
                    if let Some(h) = line.strip_prefix("H: Handlers=") {
                        event = h.split_whitespace().find(|w| w.starts_with("event"));
                    } else if let Some(k) = line.strip_prefix("B: KEY=") {
                        gamepad = has_bit(k, BTN_SOUTH);
                    }
                }
                let Some(event) = event.filter(|_| gamepad) else {
                    continue;
                };
                let path = PathBuf::from("/dev/input").join(event);
                if self.devs.iter().all(|d| d.path != path) {
                    if let Some(d) = Dev::open(path) {
                        self.devs.push(d);
                    }
                }
            }
        }

        pub fn poll(&mut self, dt: f32) -> PadState {
            self.rescan -= dt;
            if self.rescan <= 0.0 {
                self.rescan = 2.0;
                self.scan();
            }
            self.devs.retain_mut(|d| d.read());
            let mut out = PadState::default();
            for d in &self.devs {
                out.merge(&d.state);
            }
            out
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn reads_capability_bitmaps_highest_word_first() {
            // A typical Xbox pad: BTN_SOUTH (0x130) lives in the fifth word from the right.
            let pad = "7cdb000000000000 0 0 0 0";
            let keyboard = "120013 0 0 0 0 0 0 0 1ff";
            assert!(has_bit(pad, BTN_SOUTH));
            assert!(!has_bit(keyboard, BTN_SOUTH));
            assert!(!has_bit("", BTN_SOUTH));
        }

        #[test]
        fn events_are_read_as_the_kernel_writes_them() {
            // Three `struct input_event`s: A down, the left stick hard right, a sync.
            let size = std::mem::size_of::<usize>() * 2 + 8;
            let mut bytes = Vec::new();
            for (t, code, value) in [(EV_KEY, 0x130u16, 1i32), (EV_ABS, 0, 32767), (0, 0, 0)] {
                let mut ev = vec![0u8; size];
                ev[size - 8..size - 6].copy_from_slice(&t.to_ne_bytes());
                ev[size - 6..size - 4].copy_from_slice(&code.to_ne_bytes());
                ev[size - 4..].copy_from_slice(&value.to_ne_bytes());
                bytes.extend(ev);
            }
            let path = std::env::temp_dir().join(format!("hb-pad-{}", std::process::id()));
            std::fs::write(&path, &bytes).unwrap();
            // A plain file answers no ioctls: the usual Xbox ranges are assumed.
            let mut d = Dev::open(path.clone()).expect("opens");
            assert!(d.read());
            assert!(d.state.held(P::A));
            assert!(d.state.left.x > 0.99);
            let _ = std::fs::remove_file(path);
        }

        #[test]
        fn a_press_and_a_push_come_through() {
            let file = File::open("/dev/null").unwrap();
            let mut d = Dev {
                path: PathBuf::from("/dev/null"),
                file,
                state: PadState::default(),
                range: [(-32768.0, 32767.0); 64],
                hat: (0, 0),
            };
            d.range[2] = (0.0, 255.0);
            d.apply(EV_KEY, 0x130, 1);
            d.apply(EV_ABS, 0, 32767);
            d.apply(EV_ABS, 2, 255);
            d.apply(EV_ABS, 0x11, -1);
            assert!(d.state.held(P::A) && d.state.held(P::Up));
            assert!(d.state.left.x > 0.99 && d.state.lt > 0.99);
            d.apply(EV_KEY, 0x130, 0);
            d.apply(EV_ABS, 0x11, 0);
            assert!(!d.state.held(P::A) && !d.state.held(P::Up));
        }
    }
}

#[cfg(windows)]
mod sys {
    //! XInput: up to four pads. Empty slots are only looked at again every couple of
    //! seconds, as asking about a missing pad is slow.

    use glam::Vec2;
    use windows_sys::Win32::UI::Input::XboxController::*;

    use super::{PadButton as P, PadState};

    pub struct Backend {
        connected: [bool; 4],
        rescan: f32,
    }

    impl Backend {
        pub fn new() -> Backend {
            Backend {
                connected: [false; 4],
                rescan: 0.0,
            }
        }

        pub fn poll(&mut self, dt: f32) -> PadState {
            self.rescan -= dt;
            let look_all = self.rescan <= 0.0;
            if look_all {
                self.rescan = 2.0;
            }
            let mut out = PadState::default();
            for i in 0..4 {
                if !self.connected[i] && !look_all {
                    continue;
                }
                // SAFETY: XInputGetState writes one XINPUT_STATE, which `st` is.
                let mut st: XINPUT_STATE = unsafe { std::mem::zeroed() };
                let ok = unsafe { XInputGetState(i as u32, &mut st) } == 0;
                self.connected[i] = ok;
                if !ok {
                    continue;
                }
                let g = st.Gamepad;
                let mut s = PadState {
                    connected: true,
                    ..Default::default()
                };
                let map = [
                    (XINPUT_GAMEPAD_A, P::A),
                    (XINPUT_GAMEPAD_B, P::B),
                    (XINPUT_GAMEPAD_X, P::X),
                    (XINPUT_GAMEPAD_Y, P::Y),
                    (XINPUT_GAMEPAD_LEFT_SHOULDER, P::L1),
                    (XINPUT_GAMEPAD_RIGHT_SHOULDER, P::R1),
                    (XINPUT_GAMEPAD_LEFT_THUMB, P::L3),
                    (XINPUT_GAMEPAD_RIGHT_THUMB, P::R3),
                    (XINPUT_GAMEPAD_BACK, P::View),
                    (XINPUT_GAMEPAD_START, P::Menu),
                    (XINPUT_GAMEPAD_DPAD_UP, P::Up),
                    (XINPUT_GAMEPAD_DPAD_DOWN, P::Down),
                    (XINPUT_GAMEPAD_DPAD_LEFT, P::Left),
                    (XINPUT_GAMEPAD_DPAD_RIGHT, P::Right),
                ];
                for (flag, b) in map {
                    if g.wButtons & flag != 0 {
                        s.buttons |= b.bit();
                    }
                }
                let axis = |v: i16| (v as f32 / 32767.0).clamp(-1.0, 1.0);
                // XInput's sticks point up; ours point down the screen.
                s.left = Vec2::new(axis(g.sThumbLX), -axis(g.sThumbLY));
                s.right = Vec2::new(axis(g.sThumbRX), -axis(g.sThumbRY));
                s.lt = g.bLeftTrigger as f32 / 255.0;
                s.rt = g.bRightTrigger as f32 / 255.0;
                out.merge(&s);
            }
            out
        }
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
mod sys {
    use super::PadState;

    pub struct Backend;

    impl Backend {
        pub fn new() -> Backend {
            Backend
        }

        pub fn poll(&mut self, _dt: f32) -> PadState {
            PadState::default()
        }
    }
}
