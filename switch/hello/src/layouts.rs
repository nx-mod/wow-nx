//! The C types `std` and the `libc` crate use, checked at compile time against devkitA64's newlib (sizes and alignments
//! measured with its gcc, 2026-10-22). The `libc` crate's Horizon module was written for the 32-bit 3DS: a type of the
//! wrong size here is a struct read at the wrong offsets (file sizes, modes) or a pthread object that overflows.

use core::mem::{align_of, size_of};

macro_rules! layout_is {
    ($t:ty, $size_of_set:expr, $al:expr) => {
        const _: () = assert!(size_of::<$t>() == $size_of_set && align_of::<$t>() == $al, stringify!($t));
    };
}

layout_is!(libc::pthread_t, 8, 8);
layout_is!(libc::pthread_attr_t, 24, 8);
layout_is!(libc::pthread_mutex_t, 12, 4);
layout_is!(libc::pthread_mutexattr_t, 4, 4);
layout_is!(libc::pthread_cond_t, 16, 8);
layout_is!(libc::pthread_condattr_t, 8, 8);
layout_is!(libc::pthread_rwlock_t, 16, 4);
layout_is!(libc::pthread_key_t, 4, 4);
layout_is!(libc::time_t, 8, 8);
layout_is!(libc::clock_t, 8, 8);
layout_is!(libc::clockid_t, 8, 8);
layout_is!(libc::suseconds_t, 8, 8);
layout_is!(libc::uid_t, 2, 2);
layout_is!(libc::gid_t, 2, 2);
layout_is!(libc::mode_t, 4, 4);
layout_is!(libc::nlink_t, 2, 2);
layout_is!(libc::blksize_t, 8, 8);
layout_is!(libc::blkcnt_t, 8, 8);
layout_is!(libc::off_t, 8, 8);
layout_is!(libc::dev_t, 2, 2);
layout_is!(libc::ino_t, 2, 2);
layout_is!(libc::sigset_t, 8, 8);
layout_is!(libc::timespec, 16, 8);
layout_is!(libc::stat, 104, 8);
layout_is!(libc::dirent, 260, 2);
const _: () = assert!(core::mem::offset_of!(libc::stat, st_size) == 16, "stat.st_size");
const _: () = assert!(core::mem::offset_of!(libc::stat, st_mtim) == 40, "stat.st_mtim");
const _: () = assert!(core::mem::offset_of!(libc::stat, st_blocks) == 80, "stat.st_blocks");
const _: () = assert!(core::mem::offset_of!(libc::dirent, d_name) == 3, "dirent.d_name");
// The condition variable's initializer as devkitA64's (CLOCK_REALTIME first); the mutex's and rwlock's are zeros.
const _: () = assert!(unsafe { core::mem::transmute::<libc::pthread_cond_t, [u8; 16]>(libc::PTHREAD_COND_INITIALIZER) }[0] == 1);
// Sockets: libnx's, from FreeBSD (devkitA64 gcc on sys/socket.h, netinet/in.h, netdb.h). The 3DS layouts the crate had
// put the family in the length byte, so every address getaddrinfo returned was skipped.
layout_is!(libc::sa_family_t, 1, 1);
layout_is!(libc::sockaddr, 16, 1);
layout_is!(libc::sockaddr_in, 16, 4);
layout_is!(libc::sockaddr_in6, 28, 4);
layout_is!(libc::sockaddr_storage, 128, 8);
layout_is!(libc::addrinfo, 48, 8);
layout_is!(libc::timeval, 16, 8);
layout_is!(libc::linger, 8, 4);
const _: () = assert!(core::mem::offset_of!(libc::sockaddr_in, sin_port) == 2, "sockaddr_in.sin_port");
const _: () = assert!(core::mem::offset_of!(libc::sockaddr_in6, sin6_port) == 2, "sockaddr_in6.sin6_port");
const _: () = assert!(core::mem::offset_of!(libc::sockaddr_in6, sin6_addr) == 8, "sockaddr_in6.sin6_addr");
const _: () = assert!(core::mem::offset_of!(libc::addrinfo, ai_canonname) == 24, "addrinfo.ai_canonname");
const _: () = assert!(core::mem::offset_of!(libc::addrinfo, ai_addr) == 32, "addrinfo.ai_addr");
const _: () = assert!(core::mem::offset_of!(libc::addrinfo, ai_next) == 40, "addrinfo.ai_next");
const _: () = assert!(libc::AF_INET6 == 28 && libc::FIONBIO == 0x8004_667e && libc::SO_ERROR == 0x1007);
const _: () = assert!(libc::TCP_NODELAY == 1 && libc::EAI_NONAME == 8 && libc::SO_SNDTIMEO == 4101);
