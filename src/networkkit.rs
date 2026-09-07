//! NetworkKit framework (WiFi, Bluetooth, local network).
//!
//! All functions are blocking - call them from a worker thread. JSON
//! results are passed through unchanged.

use std::os::raw::c_char;

use libloading::Library;

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "networkkit";
const FREE: &[u8] = b"tontoo_networkkit_string_free\0";

type JsonFn = unsafe extern "C" fn(*mut *mut c_char) -> *mut c_char;
type DiscoverFn = unsafe extern "C" fn(u64, *mut *mut c_char) -> *mut c_char;

fn finish(
    lib: &'static Library,
    result: *mut c_char,
    error: *mut c_char,
) -> Result<Option<String>> {
    match unsafe { take_string(lib, result, FREE) } {
        Some(json) => Ok(Some(json)),
        None => match unsafe { take_string(lib, error, FREE) } {
            Some(message) => Err(SdkError(message)),
            None => Ok(None),
        },
    }
}

fn call_json(symbol: &[u8]) -> Result<Option<String>> {
    let lib = load(NAME)?;
    let f = sym::<JsonFn>(lib, symbol)?;
    let mut error: *mut c_char = std::ptr::null_mut();
    finish(lib, unsafe { f(&mut error) }, error)
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_networkkit_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Scan for WiFi networks. Returns a JSON array.
pub fn scan_wifi() -> Result<String> {
    call_json(b"tontoo_networkkit_scan_wifi\0")?.ok_or_else(|| SdkError("no data".into()))
}

/// Current WiFi status as a JSON object, or `None` when not connected.
pub fn wifi_status() -> Result<Option<String>> {
    call_json(b"tontoo_networkkit_wifi_status\0")
}

/// Local network interfaces as a JSON array.
pub fn local_interfaces() -> Result<String> {
    call_json(b"tontoo_networkkit_local_interfaces\0")?
        .ok_or_else(|| SdkError("no data".into()))
}

/// IP neighbors as a JSON array.
pub fn neighbors() -> Result<String> {
    call_json(b"tontoo_networkkit_neighbors\0")?.ok_or_else(|| SdkError("no data".into()))
}

/// Discover Bluetooth devices for `timeout_secs` seconds. Returns a JSON
/// array.
pub fn discover_bluetooth(timeout_secs: u64) -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<DiscoverFn>(lib, b"tontoo_networkkit_discover_bluetooth\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    unsafe { take_string(lib, f(timeout_secs, &mut error), FREE) }.ok_or_else(|| {
        SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "bluetooth discovery failed".to_owned()),
        )
    })
}

/// Paired Bluetooth devices as a JSON array.
pub fn paired_bluetooth_devices() -> Result<String> {
    call_json(b"tontoo_networkkit_paired_bluetooth_devices\0")?
        .ok_or_else(|| SdkError("no data".into()))
}
