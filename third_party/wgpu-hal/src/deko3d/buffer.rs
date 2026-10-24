use super::ffi::*;

/// A buffer is CPU-uncached memory, so mappings are coherent: a chunk of a shared block when small (`Pool`), a memory
/// block of its own when large.
#[derive(Debug)]
pub struct Buffer {
    pub(super) memory: Memory,
    pub(super) gpu: DkGpuAddr,
    pub(super) cpu: *mut u8,
    pub(super) size: u64,
    pub(super) _shared: alloc::sync::Arc<super::Shared>,
    _counted: super::Counted,
}

#[derive(Debug)]
pub(super) enum Memory {
    Own(DkMemBlock),
    Pooled { class: usize, chunk: Chunk },
}

// SAFETY: the memory is only freed by `Device::destroy_buffer`; wgpu-core orders access to it.
unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Buffer {
    pub(super) unsafe fn new(
        shared: &alloc::sync::Arc<super::Shared>,
        desc: &crate::BufferDescriptor,
    ) -> Result<Self, crate::DeviceError> {
        let size = desc.size.max(1);
        let (memory, gpu, cpu) = match Pool::class(size) {
            Some(class) => {
                let chunk = unsafe { shared.buffers.lock().take(shared.device, class) }.ok_or(crate::DeviceError::OutOfMemory)?;
                (Memory::Pooled { class, chunk }, chunk.gpu, chunk.cpu)
            }
            None => {
                let size = super::align_up(size, DK_MEMBLOCK_ALIGNMENT as u64);
                let size = u32::try_from(size).map_err(|_| crate::DeviceError::OutOfMemory)?;
                let block = unsafe { super::mem_block(shared.device, size, BUFFER_MEMORY) }
                    .ok_or(crate::DeviceError::OutOfMemory)?;
                let (gpu, cpu) = unsafe { (dkMemBlockGetGpuAddr(block), dkMemBlockGetCpuAddr(block).cast()) };
                (Memory::Own(block), gpu, cpu)
            }
        };
        Ok(Buffer {
            memory,
            gpu,
            cpu,
            size: desc.size,
            _shared: shared.clone(),
            _counted: super::Counted::new(&super::live::BUFFERS),
        })
    }

    /// Gives the memory back: its block to the system, or its chunk to the pool.
    pub(super) unsafe fn free(self) {
        match self.memory {
            Memory::Own(block) => unsafe { dkMemBlockDestroy(block) },
            Memory::Pooled { class, chunk } => self._shared.buffers.lock().give(class, chunk),
        }
    }

    pub(super) fn ptr(&self, range: &crate::MemoryRange) -> *mut u8 {
        assert!(range.start <= range.end && range.end <= self.size, "range out of bounds");
        // SAFETY: bounds checked against the buffer's size just above.
        unsafe { self.cpu.add(range.start as usize) }
    }
}

const BUFFER_MEMORY: u32 = DkMemBlockFlags_CpuUncached | DkMemBlockFlags_GpuCached;

/// One piece of a pool block.
#[derive(Clone, Copy, Debug)]
pub(super) struct Chunk {
    gpu: DkGpuAddr,
    cpu: *mut u8,
}

/// Small buffers in shared blocks, by power-of-two size class, each chunk aligned to its size. Every memory block is
/// a few kernel calls to make and to free, and wgpu makes a staging buffer for each `write_buffer`: Bevy's frames
/// made and freed dozens of blocks. Blocks stay for the device's life; their chunks go round the free lists.
#[derive(Debug, Default)]
pub(super) struct Pool {
    blocks: alloc::vec::Vec<DkMemBlock>,
    free: [alloc::vec::Vec<Chunk>; CLASSES],
}

/// 256 bytes (WebGPU's uniform offset alignment) to 1 MiB, in 4 MiB blocks.
const SMALLEST: u32 = 8;
const CLASSES: usize = 13;
const POOL_BLOCK: u32 = 4 << 20;

// SAFETY: the pool is only used under `Shared::buffers`' lock.
unsafe impl Send for Pool {}

impl Pool {
    /// The size class of a buffer, or none when it is too large to pool.
    fn class(size: u64) -> Option<usize> {
        let bits = size.next_power_of_two().trailing_zeros().max(SMALLEST);
        let class = (bits - SMALLEST) as usize;
        (class < CLASSES).then_some(class)
    }

    unsafe fn take(&mut self, device: DkDevice, class: usize) -> Option<Chunk> {
        if self.free[class].is_empty() {
            let block = unsafe { super::mem_block(device, POOL_BLOCK, BUFFER_MEMORY) }?;
            self.blocks.push(block);
            let (gpu, cpu) = unsafe { (dkMemBlockGetGpuAddr(block), dkMemBlockGetCpuAddr(block).cast::<u8>()) };
            let size = 1u32 << (class as u32 + SMALLEST);
            for offset in (0..POOL_BLOCK).step_by(size as usize).rev() {
                // SAFETY: inside the block just made.
                self.free[class].push(Chunk { gpu: gpu + offset as u64, cpu: unsafe { cpu.add(offset as usize) } });
            }
        }
        self.free[class].pop()
    }

    fn give(&mut self, class: usize, chunk: Chunk) {
        self.free[class].push(chunk);
    }

    /// Frees the blocks; the device goes next (`Shared`'s drop).
    pub(super) unsafe fn destroy(&mut self) {
        for block in self.blocks.drain(..) {
            unsafe { dkMemBlockDestroy(block) };
        }
    }
}
