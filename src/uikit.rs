//! UIKit framework (global CSS, widget helpers).

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

use crate::runtime::{load, read_static_string, sym, Result, SdkError};

const NAME: &str = "uikit";

type LoadCssFn = unsafe extern "C" fn(*const c_char) -> c_int;
type ApplyFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f =
        sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"tontoo_uikit_version\0")?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Load global CSS that applies to all windows of the process.
pub fn load_css(css: &str) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<LoadCssFn>(lib, b"tontoo_uikit_load_css\0")?;
    let css = CString::new(css)?;
    let rc = unsafe { f(css.as_ptr()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(SdkError(format!("load_css failed with code {rc} (GTK running?)")))
    }
}

/// Apply CSS to a single GTK widget pointer.
pub fn apply_widget_css(widget: *mut c_void, css: &str) -> Result<()> {
    let lib = load(NAME)?;
    let f = sym::<ApplyFn>(lib, b"tontoo_uikit_widget_apply_css\0")?;
    let css = CString::new(css)?;
    let rc = unsafe { f(widget, css.as_ptr()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(SdkError(format!("apply_css failed with code {rc}")))
    }
}
