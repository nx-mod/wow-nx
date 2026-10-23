//! uam itself: a thread of its own, and the SD card cache.

use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Stage {
    Vertex = 0,
    Geometry = 3,
    Fragment = 4,
    Compute = 5,
}

extern "C" {
    fn benilla_uam_compile(
        stage: i32,
        glsl: *const core::ffi::c_char,
        dksh: *mut *mut u8,
        size: *mut usize,
        log: *mut *mut core::ffi::c_char,
    ) -> i32;
    fn benilla_uam_free(p: *mut core::ffi::c_void);
}

type Job = (Stage, std::ffi::CString, mpsc::Sender<Result<Vec<u8>, String>>);

fn worker() -> &'static Mutex<mpsc::Sender<Job>> {
    static WORKER: OnceLock<Mutex<mpsc::Sender<Job>>> = OnceLock::new();
    WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("uam".into())
            .stack_size(8 << 20)
            .spawn(move || {
                for (stage, glsl, reply) in rx {
                    let _ = reply.send(unsafe { compile_here(stage, &glsl) });
                }
            })
            .expect("uam thread");
        Mutex::new(tx)
    })
}

unsafe fn compile_here(stage: Stage, glsl: &std::ffi::CStr) -> Result<Vec<u8>, String> {
    let (mut dksh, mut size, mut log) = (core::ptr::null_mut(), 0usize, core::ptr::null_mut());
    let ok = unsafe { benilla_uam_compile(stage as i32, glsl.as_ptr(), &mut dksh, &mut size, &mut log) } != 0;
    let text = if log.is_null() {
        String::new()
    } else {
        let t = unsafe { std::ffi::CStr::from_ptr(log) }.to_string_lossy().into_owned();
        unsafe { benilla_uam_free(log.cast()) };
        t
    };
    if !ok || dksh.is_null() {
        return Err(text);
    }
    let bytes = unsafe { std::slice::from_raw_parts(dksh, size) }.to_vec();
    unsafe { benilla_uam_free(dksh.cast()) };
    Ok(bytes)
}

/// Compiles one stage to a whole DKSH file (what the uam tool writes), or returns uam's log.
pub fn compile(stage: Stage, glsl: &str) -> Result<Vec<u8>, String> {
    let glsl = std::ffi::CString::new(glsl).map_err(|_| String::from("NUL in GLSL"))?;
    let (tx, rx) = mpsc::channel();
    worker().lock().unwrap().send((stage, glsl, tx)).map_err(|_| String::from("uam thread gone"))?;
    rx.recv().map_err(|_| String::from("uam thread gone"))?
}

/// Where compiled shaders are kept.
#[cfg(target_os = "horizon")]
pub const CACHE_DIR: &str = "sdmc:/wow-nx/shaders";
#[cfg(not(target_os = "horizon"))]
pub const CACHE_DIR: &str = "target/shaders";

/// FNV-1a over the stage and the source: the cache key.
fn cache_key(stage: Stage, glsl: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ stage as u64;
    for b in glsl.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// [`compile`], through the SD card cache.
pub fn compile_cached(stage: Stage, glsl: &str) -> Result<Vec<u8>, String> {
    let path = format!("{CACHE_DIR}/{:016x}", cache_key(stage, glsl));
    if let Ok(dksh) = std::fs::read(format!("{path}.dksh")) {
        if dksh.len() >= 24 && dksh[..4] == *b"DKSH" {
            return Ok(dksh);
        }
    }
    let r = compile(stage, glsl);
    let _ = std::fs::create_dir_all(CACHE_DIR);
    match &r {
        Ok(dksh) => {
            let _ = std::fs::write(format!("{path}.dksh"), dksh);
            // Its source too, once: the whole set can be rechecked on the host (switch/shadercheck takes .glsl).
            let _ = std::fs::write(format!("{path}.glsl"), glsl);
        }
        Err(log) => {
            let _ = std::fs::write(format!("{path}.error.glsl"), format!("{glsl}\n/* uam:\n{log}\n*/\n"));
        }
    }
    r
}
