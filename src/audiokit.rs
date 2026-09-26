//! AudioKit framework (playback, recording, devices).
//!
//! Playback is stateful - the first play call opens the system default
//! output. JSON results are passed through unchanged.

use std::ffi::CString;
use std::os::raw::c_char;

use libloading::Library;

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "audiokit";
const FREE: &[u8] = b"tontoo_audiokit_string_free\0";

type JsonFn = unsafe extern "C" fn(*mut *mut c_char) -> *mut c_char;
type PlayFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> u64;
type StreamFn = unsafe extern "C" fn(u64, *mut *mut c_char) -> i32;

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

fn call_stream(symbol: &[u8], id: u64) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<StreamFn>(lib, symbol)?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { f(id, &mut error) };
    if rc == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit stream call failed".to_owned()),
        ))
    }
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_audiokit_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Output devices as a JSON array.
pub fn list_output_devices() -> Result<String> {
    call_json(b"tontoo_audiokit_list_output_devices\0")?
        .ok_or_else(|| SdkError("no data".into()))
}

/// Input devices as a JSON array.
pub fn list_input_devices() -> Result<String> {
    call_json(b"tontoo_audiokit_list_input_devices\0")?
        .ok_or_else(|| SdkError("no data".into()))
}

/// Default output device as a JSON object, or `None` when headless.
pub fn default_output() -> Result<Option<String>> {
    call_json(b"tontoo_audiokit_default_output\0")
}

/// Plays a file on the system default output. Returns the stream id.
pub fn play_file(path: &str) -> Result<u64> {
    let lib = load(NAME)?;
    let f = sym::<PlayFn>(lib, b"tontoo_audiokit_play_file\0")?;
    let c_path = CString::new(path).map_err(|_| SdkError("invalid path".into()))?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let id = unsafe { f(c_path.as_ptr(), &mut error) };
    if id == 0 {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit play failed".to_owned()),
        ))
    } else {
        Ok(id)
    }
}

/// Streams a file with bounded RAM. Returns the stream id.
pub fn play_stream(path: &str) -> Result<u64> {
    let lib = load(NAME)?;
    let f = sym::<PlayFn>(lib, b"tontoo_audiokit_play_stream\0")?;
    let c_path = CString::new(path).map_err(|_| SdkError("invalid path".into()))?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let id = unsafe { f(c_path.as_ptr(), &mut error) };
    if id == 0 {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit stream failed".to_owned()),
        ))
    } else {
        Ok(id)
    }
}

/// Resumes playback of a stream.
pub fn stream_play(id: u64) -> Result<()> {
    call_stream(b"tontoo_audiokit_stream_play\0", id)
}

/// Pauses playback of a stream.
pub fn stream_pause(id: u64) -> Result<()> {
    call_stream(b"tontoo_audiokit_stream_pause\0", id)
}

/// Stops playback of a stream and resets its position.
pub fn stream_stop(id: u64) -> Result<()> {
    call_stream(b"tontoo_audiokit_stream_stop\0", id)
}

/// Removes a stream from the engine.
pub fn stream_remove(id: u64) -> Result<()> {
    call_stream(b"tontoo_audiokit_stream_remove\0", id)
}

/// Seeks a stream to `position_ms`.
pub fn stream_seek_ms(id: u64, position_ms: u64) -> Result<()> {
    let lib = load(NAME)?;
    type SeekFn = unsafe extern "C" fn(u64, u64, *mut *mut c_char) -> i32;
    let f = sym::<SeekFn>(lib, b"tontoo_audiokit_stream_seek_ms\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, position_ms, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit seek failed".to_owned()),
        ))
    }
}

/// Sets the per-stream volume (0.0-1.0).
pub fn stream_set_volume(id: u64, volume: f32) -> Result<()> {
    let lib = load(NAME)?;
    type VolumeFn = unsafe extern "C" fn(u64, f32, *mut *mut c_char) -> i32;
    let f = sym::<VolumeFn>(lib, b"tontoo_audiokit_stream_set_volume\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, volume, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit volume failed".to_owned()),
        ))
    }
}

/// Stream state: 0 stopped, 1 playing, 2 paused, 3 finished, -1 unknown.
pub fn stream_state(id: u64) -> Result<i32> {
    let lib = load(NAME)?;
    type StateFn = unsafe extern "C" fn(u64) -> i32;
    let f = sym::<StateFn>(lib, b"tontoo_audiokit_stream_state\0")?;
    Ok(unsafe { f(id) })
}

/// System volume in percent, or `None` when unavailable.
pub fn system_volume_get() -> Result<Option<u32>> {
    let lib = load(NAME)?;
    type GetFn = unsafe extern "C" fn() -> i32;
    let f = sym::<GetFn>(lib, b"tontoo_audiokit_system_volume_get\0")?;
    let value = unsafe { f() };
    if value < 0 {
        Ok(None)
    } else {
        Ok(Some(value as u32))
    }
}

/// Sets the system volume (0-150).
pub fn system_volume_set(percent: u32) -> Result<()> {
    let lib = load(NAME)?;
    type SetFn = unsafe extern "C" fn(u32, *mut *mut c_char) -> i32;
    let f = sym::<SetFn>(lib, b"tontoo_audiokit_system_volume_set\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(percent, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit system volume failed".to_owned()),
        ))
    }
}

/// Sets a stereo echo on a stream.
pub fn stream_set_echo(id: u64, delay_ms: u32, feedback: f32, mix: f32) -> Result<()> {
    let lib = load(NAME)?;
    type EchoFn = unsafe extern "C" fn(u64, u32, f32, f32, *mut *mut c_char) -> i32;
    let f = sym::<EchoFn>(lib, b"tontoo_audiokit_stream_set_echo\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, delay_ms, feedback, mix, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit echo failed".to_owned()),
        ))
    }
}

/// Sets a room reverb on a stream.
pub fn stream_set_reverb(id: u64, mix: f32, decay: f32) -> Result<()> {
    let lib = load(NAME)?;
    type ReverbFn = unsafe extern "C" fn(u64, f32, f32, *mut *mut c_char) -> i32;
    let f = sym::<ReverbFn>(lib, b"tontoo_audiokit_stream_set_reverb\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, mix, decay, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit reverb failed".to_owned()),
        ))
    }
}

/// Sets a 3-band equalizer on a stream (dB per band).
pub fn stream_set_eq(id: u64, low_db: f32, mid_db: f32, high_db: f32) -> Result<()> {
    let lib = load(NAME)?;
    type EqFn = unsafe extern "C" fn(u64, f32, f32, f32, *mut *mut c_char) -> i32;
    let f = sym::<EqFn>(lib, b"tontoo_audiokit_stream_set_eq\0")?;
    let mut error: *mut c_char = std::ptr::null_mut();
    if unsafe { f(id, low_db, mid_db, high_db, &mut error) } == 0 {
        Ok(())
    } else {
        Err(SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "audiokit eq failed".to_owned()),
        ))
    }
}

/// Removes all effects from a stream.
pub fn stream_clear_effects(id: u64) -> Result<()> {
    call_stream(b"tontoo_audiokit_stream_clear_effects\0", id)
}
