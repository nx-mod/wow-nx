//! wow-nx: the Switch's device layer, behind the same name as `cpal.rs` and `coreaudio.rs`: libnx's audout, 48 kHz
//! stereo 16-bit. A thread of its own keeps three 1024-frame buffers queued (21.3 ms each, at most ~64 ms ahead, the
//! sizes rexglue-sdk-nx's Switch audio settled on): each one the device hands back is refilled from the mixer and
//! queued again.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Result};

pub(super) const CHANNELS: u32 = 2;
const RATE: u32 = 48_000;
/// Frames per audout buffer, and buffers kept queued.
const FRAMES: usize = 1024;
const BUFFERS: usize = 3;
/// audout wants page-aligned sample memory.
const BUFFER_BYTES: usize = (FRAMES * CHANNELS as usize * 2 + 0xfff) & !0xfff;
/// Bounded, so the thread notices a stop even if the device stops handing buffers back.
const WAIT_NS: u64 = 100_000_000;

#[repr(C)]
struct AudioOutBuffer {
    next: *mut AudioOutBuffer,
    buffer: *mut core::ffi::c_void,
    buffer_size: u64,
    data_size: u64,
    data_offset: u64,
}

#[link(name = "nx")]
unsafe extern "C" {
    fn audoutInitialize() -> u32;
    fn audoutExit();
    fn audoutStartAudioOut() -> u32;
    fn audoutStopAudioOut() -> u32;
    fn audoutAppendAudioOutBuffer(buffer: *mut AudioOutBuffer) -> u32;
    fn audoutWaitPlayFinish(released: *mut *mut AudioOutBuffer, released_count: *mut u32, timeout: u64) -> u32;
}

pub(super) fn now_ns() -> u64 {
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    EPOCH.get_or_init(Instant::now).elapsed().as_nanos() as u64
}

#[derive(Clone, Debug)]
pub(super) struct Device {
    pub name: String,
    pub sample_rate: u32,
    pub buffer_range: (u32, u32),
    pub latency_frames: u32,
    pub safety_frames: u32,
}

pub(super) fn default_output() -> Result<Device> {
    now_ns();
    Ok(Device {
        name: "audout".into(),
        sample_rate: RATE,
        buffer_range: (FRAMES as u32, FRAMES as u32),
        latency_frames: (FRAMES * (BUFFERS - 1)) as u32,
        safety_frames: 0,
    })
}

pub(super) struct Cycle<'a> {
    pub buffer: &'a mut [f32],
    pub frames: usize,
    pub output_time_ns: u64,
}

/// The output thread; dropping the stream stops it and closes audout.
pub(super) struct Stream {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Stream {
    pub(super) fn buffer_frames(&self) -> u32 {
        FRAMES as u32
    }

    pub(super) fn open<F>(_device: &Device, _buffer_frames: u32, notices: &Arc<Notices>, mut on_cycle: F) -> Result<Self>
    where
        F: FnMut(Cycle<'_>) + Send + 'static,
    {
        let stop = Arc::new(AtomicBool::new(false));
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let stopping = Arc::clone(&stop);
        let notices = Arc::clone(notices);
        let thread = std::thread::Builder::new()
            .name("audio-out".into())
            .spawn(move || {
                // SAFETY: audout is opened, used and closed on this thread only.
                let rc = unsafe { audoutInitialize() };
                if rc != 0 {
                    let _ = ready_tx.send(Err(format!("audoutInitialize: {rc:#x}")));
                    return;
                }
                let layout = std::alloc::Layout::from_size_align(BUFFER_BYTES, 0x1000).unwrap();
                let mut buffers: Vec<AudioOutBuffer> = (0..BUFFERS)
                    .map(|_| AudioOutBuffer {
                        next: core::ptr::null_mut(),
                        // SAFETY: a non-zero size; zeroed, so a buffer queued before its first fill is silence.
                        buffer: unsafe { std::alloc::alloc_zeroed(layout) }.cast(),
                        buffer_size: BUFFER_BYTES as u64,
                        data_size: (FRAMES * CHANNELS as usize * 2) as u64,
                        data_offset: 0,
                    })
                    .collect();
                if buffers.iter().any(|b| b.buffer.is_null()) {
                    let _ = ready_tx.send(Err("out of memory for audio buffers".into()));
                    unsafe { audoutExit() };
                    return;
                }
                let mut scratch = vec![0f32; FRAMES * CHANNELS as usize];
                let mut fill = |b: &mut AudioOutBuffer, scratch: &mut [f32]| {
                    scratch.fill(0.0);
                    let ahead = (FRAMES * (BUFFERS - 1)) as u64 * 1_000_000_000 / RATE as u64;
                    on_cycle(Cycle { buffer: scratch, frames: FRAMES, output_time_ns: now_ns() + ahead });
                    // SAFETY: each buffer holds FRAMES stereo 16-bit frames and is not queued while it is filled.
                    let out = unsafe { core::slice::from_raw_parts_mut(b.buffer.cast::<i16>(), FRAMES * CHANNELS as usize) };
                    for (o, s) in out.iter_mut().zip(scratch.iter()) {
                        *o = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                    }
                };
                for b in buffers.iter_mut() {
                    fill(b, &mut scratch);
                    unsafe { audoutAppendAudioOutBuffer(b) };
                }
                let rc = unsafe { audoutStartAudioOut() };
                let _ = ready_tx.send(if rc == 0 { Ok(()) } else { Err(format!("audoutStartAudioOut: {rc:#x}")) });
                if rc == 0 {
                    while !stopping.load(Ordering::Acquire) {
                        let (mut released, mut count) = (core::ptr::null_mut(), 0u32);
                        let rc = unsafe { audoutWaitPlayFinish(&mut released, &mut count, WAIT_NS) };
                        if rc != 0 || released.is_null() {
                            continue;
                        }
                        // SAFETY: the device hands back one of our buffers.
                        let b = unsafe { &mut *released };
                        fill(b, &mut scratch);
                        if unsafe { audoutAppendAudioOutBuffer(b) } != 0 {
                            notices.device_died.store(true, Ordering::Release);
                            break;
                        }
                    }
                    unsafe { audoutStopAudioOut() };
                }
                unsafe { audoutExit() };
                for b in &buffers {
                    unsafe { std::alloc::dealloc(b.buffer.cast(), layout) };
                }
            })?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Stream { stop, thread: Some(thread) }),
            Ok(Err(e)) => {
                let _ = thread.join();
                bail!(e)
            }
            Err(_) => bail!("the audio thread ended before it started"),
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[derive(Default)]
pub(super) struct Notices {
    pub default_changed: AtomicBool,
    pub device_died: AtomicBool,
    pub rate_changed: AtomicBool,
    pub overloads: AtomicU64,
    pub last_overload_ns: AtomicU64,
}

/// One output that never changes: nothing to watch.
pub(super) struct Listeners;

impl Listeners {
    pub(super) fn arm(_device: &Device, _notices: Arc<Notices>) -> Self {
        Self
    }
}

pub(super) struct Workgroup;

impl Workgroup {
    pub(super) fn of_device(_device: &Device) -> Option<Self> {
        None
    }
}

pub(super) struct Joined;

impl Joined {
    pub(super) fn join(_group: Workgroup) -> Result<Self> {
        bail!("no audio workgroups on the Switch")
    }
}

pub(super) struct Realtime;

pub(super) fn set_realtime(_period_ns: u64) -> Result<Realtime> {
    bail!("no realtime thread policy on the Switch")
}
