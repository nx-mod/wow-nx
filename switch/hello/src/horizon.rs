//! What Rust's `std` asks of the C library on Horizon that devkitA64's newlib does not have; the 3DS target gets the
//! same from its shim crate. Provided from libnx.

use core::ffi::{c_int, c_long, c_uint, c_void};

extern "C" {
    // libnx (switch/kernel/random.h): bytes from the kernel's entropy, seeded once by libnx.
    fn randomGet(buf: *mut c_void, len: usize);
}

/// `getrandom(2)`: std seeds its hash maps with it.
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: usize, _flags: c_uint) -> isize {
    randomGet(buf, buflen);
    buflen as isize
}

// newlib's sys/unistd.h.
const SC_PAGESIZE: c_int = 8;
const SC_NPROCESSORS_CONF: c_int = 9;
const SC_NPROCESSORS_ONLN: c_int = 10;

/// `sysconf(3)`: std's threads ask for the page size and the core count. An application has 3 of the 4 cores.
#[no_mangle]
pub extern "C" fn sysconf(name: c_int) -> c_long {
    match name {
        SC_PAGESIZE => 0x1000,
        SC_NPROCESSORS_CONF | SC_NPROCESSORS_ONLN => 3,
        _ => -1,
    }
}
