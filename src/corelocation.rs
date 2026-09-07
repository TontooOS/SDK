//! CoreLocation framework (device location).
//!
//! Service calls are blocking - call them from a worker thread.

use std::os::raw::{c_char, c_int};

use crate::runtime::{load, read_static_string, sym, take_string, Result};

const NAME: &str = "corelocation";
const FREE: &[u8] = b"tontoo_corelocation_string_free\0";

type LocationFn = unsafe extern "C" fn(*mut *mut c_char) -> *mut c_char;
type LocationFromFn = unsafe extern "C" fn(c_int, *mut *mut c_char) -> *mut c_char;

fn call_location(symbol: &[u8], source: Option<c_int>) -> Result<String> {
    let lib = load(NAME)?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let raw = match source {
        Some(source) => {
            let f = sym::<LocationFromFn>(lib, symbol)?;
            unsafe { f(source, &mut error) }
        }
        None => {
            let f = sym::<LocationFn>(lib, symbol)?;
            unsafe { f(&mut error) }
        }
    };
    unsafe { take_string(lib, raw, FREE) }.ok_or_else(|| {
        crate::runtime::SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "location unavailable".to_owned()),
        )
    })
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_corelocation_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Resolve the current location. Returns a JSON object with `latitude`,
/// `longitude`, `accuracy`, `source`, `city`, `country`, `region`.
pub fn get_location() -> Result<String> {
    call_location(b"tontoo_corelocation_get_location\0", None)
}

/// Resolve the current location from a specific source:
/// 0 GPS, 1 WiFi, 2 IP, 3 timezone, 4 manual.
pub fn get_location_from(source: c_int) -> Result<String> {
    call_location(b"tontoo_corelocation_get_location_from\0", Some(source))
}
