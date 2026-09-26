//! CoreText framework (crisp text stack).
//!
//! Thin dynamic binding to the `coretext.library` binary. When the
//! `coretext` feature is enabled the full crate is re-exported as
//! `sdk::CoreText` instead.

use std::os::raw::c_char;

use crate::runtime::{load, read_static_string, sym, Result};

const NAME: &str = "coretext";

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f =
        sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"coretext_version\0")?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Measure `text` at `size` logical px on a `scale` display.
/// Returns logical (width, height).
pub fn measure(text: &str, size: f32, scale: f32) -> Result<(f32, f32)> {
    use std::ffi::CString;
    let lib = load(NAME)?;
    let f = sym::<
        unsafe extern "C" fn(*const c_char, f32, f32, *mut f32, *mut f32) -> i32,
    >(lib, b"coretext_measure\0")?;
    let owned = CString::new(text)?;
    let (mut w, mut h) = (0.0f32, 0.0f32);
    let ok = unsafe { f(owned.as_ptr(), size, scale, &mut w, &mut h) };
    if ok == 0 {
        return Err(crate::runtime::SdkError("measure failed".into()));
    }
    Ok((w, h))
}
