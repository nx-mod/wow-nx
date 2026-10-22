//! The C types `std` and the `libc` crate use, checked at compile time against devkitA64's newlib (sizes and alignments
//! measured with its gcc, 2026-10-22). The `libc` crate's Horizon module was written for the 32-bit 3DS: a type of the
//! wrong size here is a struct read at the wrong offsets (file sizes, modes) or a pthread object that overflows.

use core::mem::{align_of, size_of};

macro_rules! igual {
    ($t:ty, $tam:expr, $al:expr) => {
        const _: () = assert!(size_of::<$t>() == $tam && align_of::<$t>() == $al, stringify!($t));
    };
}

igual!(libc::pthread_t, 8, 8);
igual!(libc::pthread_attr_t, 24, 8);
igual!(libc::pthread_mutex_t, 12, 4);
igual!(libc::pthread_mutexattr_t, 4, 4);
igual!(libc::pthread_cond_t, 16, 8);
igual!(libc::pthread_condattr_t, 8, 8);
igual!(libc::pthread_rwlock_t, 16, 4);
igual!(libc::pthread_key_t, 4, 4);
igual!(libc::time_t, 8, 8);
igual!(libc::clock_t, 8, 8);
igual!(libc::clockid_t, 8, 8);
igual!(libc::suseconds_t, 8, 8);
igual!(libc::uid_t, 2, 2);
igual!(libc::gid_t, 2, 2);
igual!(libc::mode_t, 4, 4);
igual!(libc::nlink_t, 2, 2);
igual!(libc::blksize_t, 8, 8);
igual!(libc::blkcnt_t, 8, 8);
igual!(libc::off_t, 8, 8);
igual!(libc::dev_t, 2, 2);
igual!(libc::ino_t, 2, 2);
igual!(libc::sigset_t, 8, 8);
igual!(libc::timespec, 16, 8);
igual!(libc::stat, 104, 8);
igual!(libc::dirent, 260, 2);
const _: () = assert!(core::mem::offset_of!(libc::stat, st_size) == 16, "stat.st_size");
const _: () = assert!(core::mem::offset_of!(libc::stat, st_mtim) == 40, "stat.st_mtim");
const _: () = assert!(core::mem::offset_of!(libc::stat, st_blocks) == 80, "stat.st_blocks");
const _: () = assert!(core::mem::offset_of!(libc::dirent, d_name) == 3, "dirent.d_name");
// The condition variable's initializer as devkitA64's (CLOCK_REALTIME first); the mutex's and rwlock's are zeros.
const _: () = assert!(unsafe { core::mem::transmute::<libc::pthread_cond_t, [u8; 16]>(libc::PTHREAD_COND_INITIALIZER) }[0] == 1);
