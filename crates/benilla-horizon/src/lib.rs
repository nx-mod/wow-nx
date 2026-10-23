//! wow-nx: the Switch port's platform seam (docs/SWITCH.md). On Horizon, what libnx gives Rust that devkitA64's newlib
//! does not: the C functions `std` calls (`getrandom`, `sysconf`) and the random source for the `getrandom` crate.
//! Elsewhere it is empty, so a crate can depend on it for the Switch only.

#![cfg_attr(not(target_os = "horizon"), allow(unused))]

#[cfg(target_os = "horizon")]
mod horizon {
    use core::ffi::{c_int, c_long, c_uint, c_void};

    #[link(name = "nx")]
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

    fn aleatorio(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        unsafe { randomGet(buf.as_mut_ptr().cast(), buf.len()) };
        Ok(())
    }
    getrandom::register_custom_getrandom!(aleatorio);

    /// getrandom 0.3 and 0.4's custom backend (both name this symbol).
    #[no_mangle]
    unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), getrandom03::Error> {
        unsafe { randomGet(dest.cast(), len) };
        Ok(())
    }
}
