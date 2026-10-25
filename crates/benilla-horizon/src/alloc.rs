//! Rust's allocator on the Switch: newlib's malloc behind per-thread caches of small blocks.
//!
//! newlib's malloc takes one recursive lock for every call, a kernel arbitration when it is contended: profiled in the
//! world (switch/profile.py), the main thread spent ~21 % of its time blocked on that lock and ~8 % inside malloc and
//! free, while the workers, the render thread and the asset loaders allocated alongside it. Here a block of up to
//! 4 KiB is rounded up to one of a few size classes and, when freed, kept on its thread's list for that class, which
//! the next allocation of the class on the thread takes back with no lock. Each list holds up to [`CLASS_BYTES`];
//! past that, and for larger or over-aligned blocks, newlib's malloc and free serve as before.
//!
//! A cached block is a plain malloc block of its class size, so a block freed on another thread than the one that
//! made it simply joins that thread's list, and any list's surplus can go back to newlib's free. A thread's lists
//! have no destructor: what one holds when its thread ends stays allocated (tens of KiB at most).

use core::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;
use core::ffi::c_void;
use core::ptr;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn memalign(align: usize, size: usize) -> *mut c_void;
    fn realloc(p: *mut c_void, size: usize) -> *mut c_void;
}

/// newlib's malloc returns 16-byte aligned blocks.
const MALLOC_ALIGN: usize = 16;
/// The size classes: 16-byte steps to 128, then four to each power of two up to 4 KiB.
const CLASS_SIZES: [usize; CLASSES] = [
    16, 32, 48, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 448, 512, 640, 768, 896, 1024, 1280, 1536, 1792,
    2048, 2560, 3072, 3584, 4096,
];
const CLASSES: usize = 28;
const LARGEST: usize = 4096;
/// What one thread keeps per class, at most.
const CLASS_BYTES: usize = 64 * 1024;

/// The class of a block of `size` bytes (1 to [`LARGEST`]).
fn class(size: usize) -> usize {
    if size <= 128 {
        return (size - 1) / 16;
    }
    // Four classes to each power of two above 128: the group by the top bit, the quarter by the next two.
    let top = usize::BITS - 1 - (size - 1).leading_zeros(); // 7 for 129..=256
    let quarter = ((size - 1) >> (top - 2)) & 3;
    8 + (top as usize - 7) * 4 + quarter as usize
}

/// A free block's first word: the next on its list.
struct Free {
    next: *mut Free,
}

struct Cache {
    heads: [Cell<*mut Free>; CLASSES],
    counts: [Cell<u32>; CLASSES],
}

thread_local! {
    static CACHE: Cache = const {
        Cache {
            heads: [const { Cell::new(ptr::null_mut()) }; CLASSES],
            counts: [const { Cell::new(0) }; CLASSES],
        }
    };
}

/// The allocator ([`crate`] installs it as the global one on the Switch).
pub struct Cached;

fn cached(layout: &Layout) -> bool {
    layout.align() <= MALLOC_ALIGN && layout.size() <= LARGEST
}

unsafe impl GlobalAlloc for Cached {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if cached(&layout) {
            let c = class(layout.size());
            let taken = CACHE.with(|cache| {
                let head = cache.heads[c].get();
                if !head.is_null() {
                    // SAFETY: a block on the list is a free block of this class, its first word the next.
                    cache.heads[c].set(unsafe { (*head).next });
                    cache.counts[c].set(cache.counts[c].get() - 1);
                }
                head
            });
            if !taken.is_null() {
                return taken.cast();
            }
            return unsafe { malloc(CLASS_SIZES[c]) }.cast();
        }
        if layout.align() <= MALLOC_ALIGN {
            unsafe { malloc(layout.size()) }.cast()
        } else {
            unsafe { memalign(layout.align(), layout.size()) }.cast()
        }
    }

    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        if cached(&layout) {
            let c = class(layout.size());
            let kept = CACHE.with(|cache| {
                let count = cache.counts[c].get();
                if (count as usize + 1) * CLASS_SIZES[c] > CLASS_BYTES && count >= 4 {
                    return false;
                }
                let block = p.cast::<Free>();
                // SAFETY: the block is ours again and at least 16 bytes.
                unsafe { (*block).next = cache.heads[c].get() };
                cache.heads[c].set(block);
                cache.counts[c].set(count + 1);
                true
            });
            if kept {
                return;
            }
        }
        unsafe { free(p.cast()) };
    }

    unsafe fn realloc(&self, p: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() <= MALLOC_ALIGN {
            // Within a class the block already fits; above the classes newlib grows it in place when it can.
            if layout.size() <= LARGEST && new_size <= LARGEST && class(layout.size()) == class(new_size) {
                return p;
            }
            if layout.size() > LARGEST && new_size > LARGEST {
                return unsafe { realloc(p.cast(), new_size) }.cast();
            }
        }
        // SAFETY: as GlobalAlloc::realloc's default: a new block of the new size, the contents, the old one freed.
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        let q = unsafe { self.alloc(new_layout) };
        if !q.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(p, q, layout.size().min(new_size));
                self.dealloc(p, layout);
            }
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_size_fits_its_class_and_the_classes_rise() {
        for size in 1..=LARGEST {
            let c = class(size);
            assert!(CLASS_SIZES[c] >= size, "{size} in class {c} of {}", CLASS_SIZES[c]);
            assert!(c == 0 || CLASS_SIZES[c - 1] < size, "{size} could take class {}", c - 1);
        }
        assert!(CLASS_SIZES.windows(2).all(|w| w[0] < w[1]));
    }
}
