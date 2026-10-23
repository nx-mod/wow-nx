//! wow-nx: the Switch's device layer, behind the same name as `cpal.rs` and `coreaudio.rs`. No output yet:
//! [`default_output`] reports none and the client runs silent, as on a machine with no audio device.
// TODO(wow-nx): an output on libnx's audout or audren, so the mixer is heard.

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Result};

pub(super) const CHANNELS: u32 = 2;

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
    bail!("no audio output on the Switch yet")
}

pub(super) struct Cycle<'a> {
    pub buffer: &'a mut [f32],
    pub frames: usize,
    pub output_time_ns: u64,
}

pub(super) struct Stream;

impl Stream {
    pub(super) fn buffer_frames(&self) -> u32 {
        0
    }

    pub(super) fn open<F>(_device: &Device, _buffer_frames: u32, _notices: &Arc<Notices>, _on_cycle: F) -> Result<Self>
    where
        F: FnMut(Cycle<'_>) + Send + 'static,
    {
        bail!("no audio output on the Switch yet")
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
