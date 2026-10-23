use super::ffi::*;

/// A buffer is a memory block of its own, CPU-uncached so mappings are coherent.
// TODO(wow-nx): sub-allocate small buffers from shared blocks; Bevy makes many of them.
#[derive(Debug)]
pub struct Buffer {
    pub(super) mem_block: DkMemBlock,
    pub(super) gpu: DkGpuAddr,
    pub(super) cpu: *mut u8,
    pub(super) size: u64,
    _shared: alloc::sync::Arc<super::Shared>,
}

// SAFETY: the block is only freed by `Device::destroy_buffer`; wgpu-core orders access to it.
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
    pub(super) unsafe fn new(
        shared: &alloc::sync::Arc<super::Shared>,
        desc: &crate::BufferDescriptor,
    ) -> Result<Self, crate::DeviceError> {
        let device = shared.device;
        let size = super::align_up(desc.size.max(1), DK_MEMBLOCK_ALIGNMENT as u64);
        let size = u32::try_from(size).map_err(|_| crate::DeviceError::OutOfMemory)?;
        let mem_block = unsafe {
            super::mem_block(device, size, DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached)
        }
        .ok_or(crate::DeviceError::OutOfMemory)?;
        Ok(Buffer {
            mem_block,
            gpu: unsafe { dkMemBlockGetGpuAddr(mem_block) },
            cpu: unsafe { dkMemBlockGetCpuAddr(mem_block) }.cast(),
            size: desc.size,
            _shared: shared.clone(),
        })
    }

    pub(super) fn ptr(&self, range: &crate::MemoryRange) -> *mut u8 {
        assert!(range.start <= range.end && range.end <= self.size, "range out of bounds");
        // SAFETY: bounds checked against the block's size just above.
        unsafe { self.cpu.add(range.start as usize) }
    }
}
