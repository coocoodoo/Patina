//! Platform audio output. Each backend takes interleaved stereo i16 frames and blocks until
//! it has room, which paces the mixer thread.

pub trait Sink {
    fn write(&mut self, frames: &[i16]) -> Result<(), ()>;
}

#[cfg(target_os = "linux")]
mod alsa {
    use std::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

    use libloading::Library;

    type Open = unsafe extern "C" fn(*mut *mut c_void, *const c_char, c_int, c_int) -> c_int;
    type SetParams =
        unsafe extern "C" fn(*mut c_void, c_int, c_int, c_uint, c_uint, c_int, c_uint) -> c_int;
    type Writei = unsafe extern "C" fn(*mut c_void, *const c_void, c_ulong) -> c_long;
    type Recover = unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int;
    type Close = unsafe extern "C" fn(*mut c_void) -> c_int;

    const STREAM_PLAYBACK: c_int = 0;
    const FORMAT_S16_LE: c_int = 2;
    const ACCESS_RW_INTERLEAVED: c_int = 3;

    pub struct Alsa {
        _lib: Library,
        pcm: *mut c_void,
        writei: Writei,
        recover: Recover,
        close: Close,
    }

    // The PCM handle is only ever used from the audio thread that created it.
    unsafe impl Send for Alsa {}

    impl Alsa {
        pub fn open(rate: u32) -> Option<Alsa> {
            // SAFETY: loading the system ALSA library and resolving documented symbols with
            // their documented C signatures.
            unsafe {
                let lib = Library::new("libasound.so.2").ok()?;
                let open: Open = *lib.get(b"snd_pcm_open\0").ok()?;
                let set_params: SetParams = *lib.get(b"snd_pcm_set_params\0").ok()?;
                let writei: Writei = *lib.get(b"snd_pcm_writei\0").ok()?;
                let recover: Recover = *lib.get(b"snd_pcm_recover\0").ok()?;
                let close: Close = *lib.get(b"snd_pcm_close\0").ok()?;
                let mut pcm = std::ptr::null_mut();
                if open(&mut pcm, c"default".as_ptr(), STREAM_PLAYBACK, 0) < 0 || pcm.is_null() {
                    return None;
                }
                if set_params(
                    pcm,
                    FORMAT_S16_LE,
                    ACCESS_RW_INTERLEAVED,
                    2,
                    rate,
                    1,
                    60_000,
                ) < 0
                {
                    close(pcm);
                    return None;
                }
                Some(Alsa {
                    _lib: lib,
                    pcm,
                    writei,
                    recover,
                    close,
                })
            }
        }
    }

    impl super::Sink for Alsa {
        fn write(&mut self, frames: &[i16]) -> Result<(), ()> {
            let total = frames.len() / 2;
            let mut done = 0;
            let mut failures = 0;
            while done < total {
                // SAFETY: the pointer and length describe the unwritten part of `frames`.
                let n = unsafe {
                    (self.writei)(
                        self.pcm,
                        frames[done * 2..].as_ptr() as *const c_void,
                        (total - done) as c_ulong,
                    )
                };
                if n < 0 {
                    failures += 1;
                    // SAFETY: recovering the stream we own after an xrun or suspend.
                    if failures > 8 || unsafe { (self.recover)(self.pcm, n as c_int, 1) } < 0 {
                        return Err(());
                    }
                } else {
                    done += n as usize;
                }
            }
            Ok(())
        }
    }

    impl Drop for Alsa {
        fn drop(&mut self) {
            // SAFETY: closing the handle we opened.
            unsafe {
                (self.close)(self.pcm);
            }
        }
    }
}

#[cfg(windows)]
mod winmm {
    use windows_sys::Win32::Media::Audio::{
        CALLBACK_NULL, HWAVEOUT, WAVE_FORMAT_PCM, WAVE_MAPPER, WAVEFORMATEX, WAVEHDR, WHDR_DONE,
        waveOutClose, waveOutOpen, waveOutPrepareHeader, waveOutReset, waveOutUnprepareHeader,
        waveOutWrite,
    };

    const BUFFERS: usize = 4;

    /// `WAVEHDR` is declared packed; keeping it 8-aligned makes its fields aligned too, so
    /// the flag the device writes can be read with an aligned volatile load.
    #[repr(C, align(8))]
    #[derive(Default)]
    struct Hdr(WAVEHDR);

    pub struct WinMm {
        dev: HWAVEOUT,
        /// Headers and sample buffers live at fixed heap addresses for as long as the device
        /// may be reading them.
        hdrs: Box<[Hdr]>,
        data: Box<[Box<[i16]>]>,
        queued: usize,
        next: usize,
    }

    // The device handle is only ever used from the audio thread that created it.
    unsafe impl Send for WinMm {}

    impl WinMm {
        pub fn open(rate: u32, frames: usize) -> Option<WinMm> {
            let fmt = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM as u16,
                nChannels: 2,
                nSamplesPerSec: rate,
                nAvgBytesPerSec: rate * 4,
                nBlockAlign: 4,
                wBitsPerSample: 16,
                cbSize: 0,
            };
            let mut dev: HWAVEOUT = std::ptr::null_mut();
            // SAFETY: standard waveOut initialisation with a valid format description.
            let r = unsafe { waveOutOpen(&mut dev, WAVE_MAPPER, &fmt, 0, 0, CALLBACK_NULL) };
            if r != 0 {
                return None;
            }
            let data: Box<[Box<[i16]>]> = (0..BUFFERS)
                .map(|_| vec![0i16; frames * 2].into_boxed_slice())
                .collect();
            let hdrs: Box<[Hdr]> = (0..BUFFERS).map(|_| Hdr::default()).collect();
            Some(WinMm {
                dev,
                hdrs,
                data,
                queued: 0,
                next: 0,
            })
        }
    }

    impl super::Sink for WinMm {
        fn write(&mut self, frames: &[i16]) -> Result<(), ()> {
            let size = std::mem::size_of::<WAVEHDR>() as u32;
            let i = self.next;
            if self.queued >= BUFFERS {
                // Wait for the oldest buffer to finish. The device sets the flag from another
                // thread, so it must be read with a volatile load.
                let mut waited = 0;
                let flags = std::ptr::addr_of!(self.hdrs[i].0.dwFlags);
                // SAFETY: `flags` points into a header we own and is aligned (see `Hdr`).
                while unsafe { std::ptr::read_volatile(flags) } & WHDR_DONE == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    waited += 1;
                    if waited > 2000 {
                        return Err(());
                    }
                }
                // SAFETY: the header has finished playing, so it can be released and reused.
                unsafe {
                    waveOutUnprepareHeader(self.dev, &mut self.hdrs[i].0, size);
                }
            } else {
                self.queued += 1;
            }
            let buf = &mut self.data[i];
            let n = buf.len().min(frames.len());
            buf[..n].copy_from_slice(&frames[..n]);
            let hdr = &mut self.hdrs[i].0;
            *hdr = WAVEHDR::default();
            hdr.lpData = buf.as_mut_ptr() as *mut u8;
            hdr.dwBufferLength = (n * 2) as u32;
            // SAFETY: the header and its buffer stay at fixed addresses inside `self`.
            unsafe {
                waveOutPrepareHeader(self.dev, hdr, size);
                waveOutWrite(self.dev, hdr, size);
            }
            self.next = (i + 1) % BUFFERS;
            Ok(())
        }
    }

    impl Drop for WinMm {
        fn drop(&mut self) {
            let size = std::mem::size_of::<WAVEHDR>() as u32;
            // SAFETY: stopping playback and releasing headers before freeing buffers.
            unsafe {
                waveOutReset(self.dev);
                for h in self.hdrs.iter_mut().take(self.queued) {
                    waveOutUnprepareHeader(self.dev, &mut h.0, size);
                }
                waveOutClose(self.dev);
            }
        }
    }
}

pub fn open(rate: u32, frames: usize) -> Option<Box<dyn Sink>> {
    #[cfg(target_os = "linux")]
    {
        let _ = frames;
        alsa::Alsa::open(rate).map(|s| Box::new(s) as Box<dyn Sink>)
    }
    #[cfg(windows)]
    {
        winmm::WinMm::open(rate, frames).map(|s| Box::new(s) as Box<dyn Sink>)
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (rate, frames);
        None
    }
}

pub fn precise_timers() {
    #[cfg(windows)]
    // SAFETY: requesting 1 ms timer resolution for the process.
    unsafe {
        windows_sys::Win32::Media::timeBeginPeriod(1);
    }
}
