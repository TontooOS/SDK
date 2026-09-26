//! MediaKit framework (video playback, capture, metadata).
//!
//! Playback is stateful - `open` returns a player id, transport calls
//! mutate it. JSON results are passed through unchanged.

use std::ffi::CString;
use std::os::raw::c_char;

use libloading::Library;

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "mediakit";
const FREE: &[u8] = b"tontoo_mediakit_string_free\0";

type JsonFn = unsafe extern "C" fn(*mut *mut c_char) -> *mut c_char;
type OpenFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> u64;
type TransportFn = unsafe extern "C" fn(u64, *mut *mut c_char) -> i32;
type PathJsonFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> *mut c_char;

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

fn call_transport(symbol: &[u8], id: u64) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<TransportFn>(lib, symbol)?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "mediakit transport call failed".to_owned()),
        ))
    }
}

fn call_path_json(symbol: &[u8], path: &str) -> Result<Option<String>> {
    let lib = load(NAME)?;
    let f = sym::<PathJsonFn>(lib, symbol)?;
    let c_path = CString::new(path).map_err(|_| SdkError("invalid path".into()))?;
    let mut error: *mut c_char = std::ptr::null_mut();
    finish(lib, unsafe { f(c_path.as_ptr(), &mut error) }, error)
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_mediakit_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Opens a video file. Returns the player id.
pub fn open(path: &str) -> Result<u64> {
    let lib = load(NAME)?;
    let f = sym::<OpenFn>(lib, b"tontoo_mediakit_open\0")?;
    let c_path = CString::new(path).map_err(|_| SdkError("invalid path".into()))?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let id = unsafe { f(c_path.as_ptr(), &mut error) };
    if id == 0 {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "mediakit open failed".to_owned()),
        ))
    } else {
        Ok(id)
    }
}

/// Starts playback of a player.
pub fn play(id: u64) -> Result<()> {
    call_transport(b"tontoo_mediakit_play\0", id)
}

/// Pauses playback of a player.
pub fn pause(id: u64) -> Result<()> {
    call_transport(b"tontoo_mediakit_pause\0", id)
}

/// Stops playback of a player and resets its position.
pub fn stop(id: u64) -> Result<()> {
    call_transport(b"tontoo_mediakit_stop\0", id)
}

/// Removes a player.
pub fn close(id: u64) -> Result<()> {
    let lib = load(NAME)?;
    type CloseFn = unsafe extern "C" fn(u64) -> i32;
    let f = sym::<CloseFn>(lib, b"tontoo_mediakit_close\0")?;
    if unsafe { f(id) } == 0 {
        Ok(())
    } else {
        Err(SdkError("mediakit close failed".into()))
    }
}

/// Seeks a player to `position_secs`.
pub fn seek(id: u64, position_secs: f64) -> Result<()> {
    let lib = load(NAME)?;
    type SeekFn = unsafe extern "C" fn(u64, f64, *mut *mut c_char) -> i32;
    let f = sym::<SeekFn>(lib, b"tontoo_mediakit_seek\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, position_secs, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "mediakit seek failed".to_owned()),
        ))
    }
}

/// Sets playback speed (0.5-2.0).
pub fn set_speed(id: u64, speed: f32) -> Result<()> {
    let lib = load(NAME)?;
    type SpeedFn = unsafe extern "C" fn(u64, f32, *mut *mut c_char) -> i32;
    let f = sym::<SpeedFn>(lib, b"tontoo_mediakit_set_speed\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, speed, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "mediakit speed failed".to_owned()),
        ))
    }
}

/// Player state: 0 stopped, 1 playing, 2 paused, 3 finished, -1 unknown.
pub fn state(id: u64) -> Result<i32> {
    let lib = load(NAME)?;
    type StateFn = unsafe extern "C" fn(u64) -> i32;
    let f = sym::<StateFn>(lib, b"tontoo_mediakit_state\0")?;
    Ok(unsafe { f(id) })
}

/// Video metadata as a JSON object.
pub fn metadata(path: &str) -> Result<String> {
    call_path_json(b"tontoo_mediakit_metadata\0", path)?
        .ok_or_else(|| SdkError("no data".into()))
}

/// Camera list as a JSON array.
pub fn list_cameras() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<JsonFn>(lib, b"tontoo_mediakit_list_cameras\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    finish(lib, unsafe { f(&mut error) }, error)?.ok_or_else(|| SdkError("no data".into()))
}

/// Loads subtitles (SRT/VTT) into a player.
pub fn load_subtitles(id: u64, subtitle_path: &str) -> Result<()> {
    let lib = load(NAME)?;
    type SubFn = unsafe extern "C" fn(u64, *const c_char, *mut *mut c_char) -> i32;
    let f = sym::<SubFn>(lib, b"tontoo_mediakit_load_subtitles\0")?;
    let c_path =
        CString::new(subtitle_path).map_err(|_| SdkError("invalid path".into()))?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, c_path.as_ptr(), &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "mediakit subtitles failed".to_owned()),
        ))
    }
}

/// Chapter list of a file as a JSON array.
pub fn chapters(path: &str) -> Result<String> {
    call_path_json(b"tontoo_mediakit_chapters\0", path)?
        .ok_or_else(|| SdkError("no data".into()))
}

/// System volume in percent, or `None` when unavailable.
pub fn system_volume_get() -> Result<Option<u32>> {
    let lib = load(NAME)?;
    type GetFn = unsafe extern "C" fn() -> i32;
    let f = sym::<GetFn>(lib, b"tontoo_mediakit_system_volume_get\0")?;
    let value = unsafe { f() };
    if value < 0 {
        Ok(None)
    } else {
        Ok(Some(value as u32))
    }
}
