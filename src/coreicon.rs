//! CoreIcon framework (colors, gradients, SF Symbols).

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::runtime::{load, read_static_string, sym, Result, SdkError};

const NAME: &str = "coreicon";

/// An RGBA color with float components in 0.0..=1.0.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
struct RawColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

type FromHexFn = unsafe extern "C" fn(*const c_char, *mut RawColor) -> c_int;

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f =
        sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"tontoo_coreicon_version\0")?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Parse `#rrggbb` or `#rrggbbaa` into a [`Color`].
pub fn color_from_hex(hex: &str) -> Result<Color> {
    let lib = load(NAME)?;
    let f = sym::<FromHexFn>(lib, b"tontoo_coreicon_color_from_hex\0")?;
    let hex = CString::new(hex)?;
    let mut raw = RawColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let rc = unsafe { f(hex.as_ptr(), &mut raw) };
    if rc == 0 {
        Ok(Color {
            r: raw.r,
            g: raw.g,
            b: raw.b,
            a: raw.a,
        })
    } else {
        Err(SdkError(format!("color_from_hex failed with code {rc}")))
    }
}
