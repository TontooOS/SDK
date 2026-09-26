//! CoreImage framework (image loading, filters, composite).
//!
//! Thin dynamic binding to the `coreimage.library` binary. When the
//! `coreimage` feature is enabled the full crate is re-exported as
//! `sdk::CoreImage` instead.

use std::os::raw::{c_char, c_float, c_int};

use crate::runtime::{load, read_static_string, sym, take_string, Result};

const NAME: &str = "coreimage";

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f =
        sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"coreimage_version\0")?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Probe image dimensions without full processing.
pub fn dimensions(path: &str) -> Result<(u32, u32)> {
    use std::ffi::CString;
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn(*const c_char, *mut u32, *mut u32) -> c_int>(
        lib,
        b"coreimage_dimensions\0",
    )?;
    let owned = CString::new(path)?;
    let (mut w, mut h) = (0u32, 0u32);
    let ok = unsafe { f(owned.as_ptr(), &mut w, &mut h) };
    if ok == 0 {
        return Err(crate::runtime::SdkError("dimensions failed".into()));
    }
    Ok((w, h))
}

/// Blur `input` into `output` as a PNG file.
pub fn blur_to_file(input: &str, output: &str, sigma: f32) -> Result<()> {
    use std::ffi::CString;
    let lib = load(NAME)?;
    let f = sym::<
        unsafe extern "C" fn(*const c_char, *const c_char, c_float, *mut *mut c_char) -> c_int,
    >(lib, b"coreimage_blur_to_file\0")?;
    let inp = CString::new(input)?;
    let outp = CString::new(output)?;
    let mut err: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { f(inp.as_ptr(), outp.as_ptr(), sigma, &mut err) };
    if rc != 0 {
        let msg = unsafe { take_string(lib, err, b"coreimage_string_free\0") }
            .unwrap_or_else(|| "blur failed".to_string());
        return Err(crate::runtime::SdkError(msg));
    }
    Ok(())
}
