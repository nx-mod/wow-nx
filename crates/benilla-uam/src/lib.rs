//! wow-nx: uam, deko3d's shader compiler (GLSL 4.60 in, DKSH out), on the Switch.
//!
//! uam keeps global state and recurses deeply, so every compile runs on one thread of its own with an 8 MB stack;
//! [`compilar`] hands it the work and waits. [`compilar_con_cache`] keeps the results on the SD card
//! (`sdmc:/wow-nx/shaders/<hash>.dksh`), so a shader compiles once per console, and leaves the GLSL of a shader that
//! fails beside them (`<hash>.error.glsl`, with uam's log at the end).
#![cfg(target_os = "horizon")]

use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Etapa {
    Vertices = 0,
    Geometria = 3,
    Fragmentos = 4,
    Computo = 5,
}

extern "C" {
    fn benilla_uam_compilar(
        etapa: i32,
        glsl: *const core::ffi::c_char,
        dksh: *mut *mut u8,
        tamano: *mut usize,
        registro: *mut *mut core::ffi::c_char,
    ) -> i32;
    fn benilla_uam_liberar(p: *mut core::ffi::c_void);
}

type Tarea = (Etapa, std::ffi::CString, mpsc::Sender<Result<Vec<u8>, String>>);

fn hilo() -> &'static Mutex<mpsc::Sender<Tarea>> {
    static HILO: OnceLock<Mutex<mpsc::Sender<Tarea>>> = OnceLock::new();
    HILO.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Tarea>();
        std::thread::Builder::new()
            .name("uam".into())
            .stack_size(8 << 20)
            .spawn(move || {
                for (etapa, glsl, respuesta) in rx {
                    let _ = respuesta.send(unsafe { compilar_aqui(etapa, &glsl) });
                }
            })
            .expect("uam thread");
        Mutex::new(tx)
    })
}

unsafe fn compilar_aqui(etapa: Etapa, glsl: &std::ffi::CStr) -> Result<Vec<u8>, String> {
    let (mut dksh, mut tamano, mut registro) = (core::ptr::null_mut(), 0usize, core::ptr::null_mut());
    let ok = unsafe { benilla_uam_compilar(etapa as i32, glsl.as_ptr(), &mut dksh, &mut tamano, &mut registro) } != 0;
    let texto = if registro.is_null() {
        String::new()
    } else {
        let t = unsafe { std::ffi::CStr::from_ptr(registro) }.to_string_lossy().into_owned();
        unsafe { benilla_uam_liberar(registro.cast()) };
        t
    };
    if !ok || dksh.is_null() {
        return Err(texto);
    }
    let bytes = unsafe { std::slice::from_raw_parts(dksh, tamano) }.to_vec();
    unsafe { benilla_uam_liberar(dksh.cast()) };
    Ok(bytes)
}

/// Compiles one stage to a whole DKSH file (what the uam tool writes), or returns uam's log.
pub fn compilar(etapa: Etapa, glsl: &str) -> Result<Vec<u8>, String> {
    let glsl = std::ffi::CString::new(glsl).map_err(|_| String::from("NUL in GLSL"))?;
    let (tx, rx) = mpsc::channel();
    hilo().lock().unwrap().send((etapa, glsl, tx)).map_err(|_| String::from("uam thread gone"))?;
    rx.recv().map_err(|_| String::from("uam thread gone"))?
}

/// Where compiled shaders are kept.
pub const CARPETA_CACHE: &str = "sdmc:/wow-nx/shaders";

/// FNV-1a over the stage and the source: the cache key.
fn clave(etapa: Etapa, glsl: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ etapa as u64;
    for b in glsl.bytes() {
        h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// [`compilar`], through the SD card cache.
pub fn compilar_con_cache(etapa: Etapa, glsl: &str) -> Result<Vec<u8>, String> {
    let ruta = format!("{CARPETA_CACHE}/{:016x}", clave(etapa, glsl));
    if let Ok(dksh) = std::fs::read(format!("{ruta}.dksh")) {
        if dksh.len() >= 24 && dksh[..4] == *b"DKSH" {
            return Ok(dksh);
        }
    }
    let r = compilar(etapa, glsl);
    let _ = std::fs::create_dir_all(CARPETA_CACHE);
    match &r {
        Ok(dksh) => {
            let _ = std::fs::write(format!("{ruta}.dksh"), dksh);
        }
        Err(log) => {
            let _ = std::fs::write(format!("{ruta}.error.glsl"), format!("{glsl}\n/* uam:\n{log}\n*/\n"));
        }
    }
    r
}
