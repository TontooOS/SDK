//! Foundation framework (dates, user defaults).

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "foundation";
const FREE: &[u8] = b"tontoo_foundation_string_free\0";

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_foundation_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Current time as a Unix timestamp in seconds.
pub fn date_now() -> Result<i64> {
    let f = sym::<unsafe extern "C" fn() -> i64>(
        load(NAME)?,
        b"tontoo_foundation_date_now\0",
    )?;
    Ok(unsafe { f() })
}

/// Shift a Unix timestamp by `days` days.
pub fn date_add_days(secs: i64, days: i64) -> Result<i64> {
    let f = sym::<unsafe extern "C" fn(i64, i64) -> i64>(
        load(NAME)?,
        b"tontoo_foundation_date_add_days\0",
    )?;
    Ok(unsafe { f(secs, days) })
}

/// Whether timestamp `a` is before timestamp `b`.
pub fn date_is_before(a: i64, b: i64) -> Result<bool> {
    let f = sym::<unsafe extern "C" fn(i64, i64) -> c_int>(
        load(NAME)?,
        b"tontoo_foundation_date_is_before\0",
    )?;
    Ok(unsafe { f(a, b) } != 0)
}

fn check(rc: c_int) -> Result<()> {
    if rc == 0 {
        Ok(())
    } else {
        Err(SdkError(format!("defaults write failed with code {rc}")))
    }
}

/// Read a string from the standard user defaults store.
pub fn defaults_get_string(key: &str) -> Option<String> {
    let lib = load(NAME).ok()?;
    let get =
        sym::<unsafe extern "C" fn(*const c_char) -> *mut c_char>(lib, b"tontoo_foundation_defaults_get_string\0").ok()?;
    let key = CString::new(key).ok()?;
    unsafe { take_string(lib, get(key.as_ptr()), FREE) }
}

/// Write a string to the standard user defaults store.
pub fn defaults_set_string(key: &str, value: &str) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn(*const c_char, *const c_char) -> c_int>(
        lib,
        b"tontoo_foundation_defaults_set_string\0",
    )?;
    let key = CString::new(key)?;
    let value = CString::new(value)?;
    check(unsafe { f(key.as_ptr(), value.as_ptr()) })
}

/// Read an integer from the standard user defaults store (0 when unset).
pub fn defaults_get_int(key: &str) -> Option<i64> {
    let lib = load(NAME).ok()?;
    let f = sym::<unsafe extern "C" fn(*const c_char) -> i64>(
        lib,
        b"tontoo_foundation_defaults_get_int\0",
    )
    .ok()?;
    let key = CString::new(key).ok()?;
    Some(unsafe { f(key.as_ptr()) })
}

/// Write an integer to the standard user defaults store.
pub fn defaults_set_int(key: &str, value: i64) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn(*const c_char, i64) -> c_int>(
        lib,
        b"tontoo_foundation_defaults_set_int\0",
    )?;
    let key = CString::new(key)?;
    check(unsafe { f(key.as_ptr(), value) })
}

/// Read a boolean from the standard user defaults store.
pub fn defaults_get_bool(key: &str) -> Option<bool> {
    let lib = load(NAME).ok()?;
    let f = sym::<unsafe extern "C" fn(*const c_char) -> c_int>(
        lib,
        b"tontoo_foundation_defaults_get_bool\0",
    )
    .ok()?;
    let key = CString::new(key).ok()?;
    Some(unsafe { f(key.as_ptr()) } != 0)
}

/// Write a boolean to the standard user defaults store.
pub fn defaults_set_bool(key: &str, value: bool) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn(*const c_char, c_int) -> c_int>(
        lib,
        b"tontoo_foundation_defaults_set_bool\0",
    )?;
    let key = CString::new(key)?;
    check(unsafe { f(key.as_ptr(), value as c_int) })
}
