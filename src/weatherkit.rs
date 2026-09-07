//! WeatherKit framework (current weather, forecasts).
//!
//! All functions are blocking network calls - call them from a worker
//! thread. JSON results are passed through unchanged.

use std::os::raw::{c_char, c_double};

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "weatherkit";
const FREE: &[u8] = b"tontoo_weatherkit_string_free\0";

type JsonFn = unsafe extern "C" fn(*mut *mut c_char) -> *mut c_char;
type TempFn = unsafe extern "C" fn(*mut *mut c_char) -> c_double;

fn call_json(symbol: &[u8]) -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<JsonFn>(lib, symbol)?;
    let mut error: *mut c_char = std::ptr::null_mut();
    unsafe { take_string(lib, f(&mut error), FREE) }.ok_or_else(|| {
        SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "weather unavailable".to_owned()),
        )
    })
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f =
        sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"tontoo_weatherkit_version\0")?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Current weather as a JSON object.
pub fn current_weather() -> Result<String> {
    call_json(b"tontoo_weatherkit_current_weather\0")
}

/// Current temperature in Celsius.
pub fn temperature() -> Result<f64> {
    let lib = load(NAME)?;
    let f = sym::<TempFn>(lib, b"tontoo_weatherkit_temperature\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let temp = unsafe { f(&mut error) };
    if temp.is_nan() {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "weather unavailable".to_owned()),
        ))
    } else {
        Ok(temp)
    }
}

/// Weekly forecast as a JSON array.
pub fn weekly_forecast() -> Result<String> {
    call_json(b"tontoo_weatherkit_weekly_forecast\0")
}
