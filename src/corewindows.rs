//! CoreWindows framework (currently open windows with toolkit type and app icons).

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "corewindows";
const FREE: &[u8] = b"tontoo_corewindows_string_free\0";

/// Framework version string.
pub fn version() -> Result<String> {
  let lib = load(NAME)?;
  let f =
    sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"tontoo_corewindows_version\0")?;
  Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Ping the window daemon: `true` on pong.
///
/// Uses the default socket (`WINDOWS_SOCKET` or `/run/tontoo-windows.sock`).
pub fn ping() -> Result<bool> {
  ping_at_impl(None)
}

/// Ping the window daemon at an explicit socket path.
pub fn ping_at(socket_path: &str) -> Result<bool> {
  ping_at_impl(Some(socket_path))
}

fn ping_at_impl(socket_path: Option<&str>) -> Result<bool> {
  let lib = load(NAME)?;
  let f = sym::<unsafe extern "C" fn(*const c_char) -> c_int>(
    lib,
    b"tontoo_corewindows_ping\0",
  )?;
  let arg = socket_path
    .map(CString::new)
    .transpose()
    .map_err(|e| SdkError(e.to_string()))?;
  let ptr = arg.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
  Ok(unsafe { f(ptr) } != 0)
}

/// Currently open windows as a JSON array of `WindowInfo` objects
/// (see the CoreWindows wiki: id, app_id, title, pid, window_type,
/// bundle_id, app_name, icon).
///
/// Uses the default socket (`WINDOWS_SOCKET` or `/run/tontoo-windows.sock`).
pub fn list_windows() -> Result<String> {
  list_windows_at_impl(None)
}

/// Currently open windows as a JSON array, via an explicit socket path.
pub fn list_windows_at(socket_path: &str) -> Result<String> {
  list_windows_at_impl(Some(socket_path))
}

fn list_windows_at_impl(socket_path: Option<&str>) -> Result<String> {
  let lib = load(NAME)?;
  let f = sym::<unsafe extern "C" fn(*const c_char) -> *mut c_char>(
    lib,
    b"tontoo_corewindows_list_windows\0",
  )?;
  let arg = socket_path
    .map(CString::new)
    .transpose()
    .map_err(|e| SdkError(e.to_string()))?;
  let ptr = arg.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
  unsafe { take_string(lib, f(ptr), FREE) }
    .ok_or_else(|| SdkError("window daemon unreachable".to_owned()))
}

fn action0(symbol: &[u8], socket_path: Option<&str>, id: u64) -> Result<()> {
  let lib = load(NAME)?;
  let f =
    sym::<unsafe extern "C" fn(*const c_char, u64) -> c_int>(lib, symbol)?;
  let arg = socket_path
    .map(CString::new)
    .transpose()
    .map_err(|e| SdkError(e.to_string()))?;
  let ptr = arg.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
  check(unsafe { f(ptr, id) }, "window action failed")
}

fn check(rc: c_int, what: &str) -> Result<()> {
  if rc == 0 {
    Ok(())
  } else {
    Err(SdkError(format!("{what} (code {rc})")))
  }
}

/// Minimize a window (iconify) via the default socket.
pub fn minimize_window(id: u64) -> Result<()> {
  action0(b"tontoo_corewindows_minimize_window\0", None, id)
}

/// Set fullscreen state of a window via the default socket
/// (`true` = fullscreen like the green UIKit traffic light).
pub fn set_fullscreen(id: u64, fullscreen: bool) -> Result<()> {
  let lib = load(NAME)?;
  let f = sym::<unsafe extern "C" fn(*const c_char, u64, c_int) -> c_int>(
    lib,
    b"tontoo_corewindows_set_fullscreen\0",
  )?;
  check(unsafe { f(std::ptr::null(), id, fullscreen as c_int) }, "fullscreen failed")
}

/// Gracefully close a window via the default socket: the app is asked to
/// close and may show a save dialog. Nothing is killed.
pub fn close_window(id: u64) -> Result<()> {
  action0(b"tontoo_corewindows_close_window\0", None, id)
}

/// Force quit the owner of a window (`SIGKILL`, no save dialog).
pub fn force_quit_window(id: u64) -> Result<()> {
  action0(b"tontoo_corewindows_force_quit_window\0", None, id)
}

/// Force quit a process id (`SIGKILL`, no save dialog).
pub fn force_quit_pid(pid: i32) -> Result<()> {
  let lib = load(NAME)?;
  let f = sym::<unsafe extern "C" fn(i32) -> c_int>(
    lib,
    b"tontoo_corewindows_force_quit_pid\0",
  )?;
  check(unsafe { f(pid) }, "force quit failed")
}

/// Installed programs (`~/Applications` and `/Applications`) as a JSON
/// array of `AppEntry` objects (bundle id, all names, display name,
/// bundle path, source, icon).
pub fn list_programs() -> Result<String> {
  let lib = load(NAME)?;
  let f = sym::<unsafe extern "C" fn() -> *mut c_char>(
    lib,
    b"tontoo_corewindows_list_programs\0",
  )?;
  unsafe { take_string(lib, f(), FREE) }
    .ok_or_else(|| SdkError("program listing failed".to_owned()))
}
