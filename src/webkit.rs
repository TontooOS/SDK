//! WebKit framework (web views).
//!
//! Loaded from `/Library/System/webkit.library`. Handles are owned by the
//! SDK wrapper and destroyed automatically on drop.

use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_void};

use crate::runtime::{load, read_static_string, sym, take_string, Result};

const NAME: &str = "webkit";
const FREE: &[u8] = b"tontoo_webkit_string_free\0";

#[repr(C)]
struct TontooWebView {
    _opaque: [u8; 0],
}

type ViewNew = unsafe extern "C" fn(
    config_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut TontooWebView;
type ViewLoadUrl = unsafe extern "C" fn(
    view: *mut TontooWebView,
    url: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int;
type ViewLoadHtml = unsafe extern "C" fn(
    view: *mut TontooWebView,
    html: *const c_char,
    base_uri: *const c_char,
);
type ViewVoid = unsafe extern "C" fn(view: *mut TontooWebView);
type ViewBool = unsafe extern "C" fn(view: *mut TontooWebView) -> c_int;
type ViewString = unsafe extern "C" fn(view: *mut TontooWebView) -> *mut c_char;
type ViewProgress = unsafe extern "C" fn(view: *mut TontooWebView) -> c_double;
type Evaluate = unsafe extern "C" fn(
    view: *mut TontooWebView,
    script: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char;

/// An embedded web browser view backed by the system WebKit.
pub struct WebView {
    raw: *mut TontooWebView,
}

// The handle is only ever used behind `&self` FFI calls that are internally
// synchronized by WebKitGTK's main-loop model.
unsafe impl Send for WebView {}

impl WebView {
    /// Create a view with a plain start URL.
    pub fn new(start_url: &str) -> Result<Self> {
        let config = format!(r#"{{ "start_url": "{}" }}"#, start_url.replace('"', "\\\""));
        Self::with_config_json(&config)
    }

    /// Create a view from raw configuration JSON.
    ///
    /// Keys mirror the framework config: `start_url`, `settings`,
    /// `user_scripts`, `message_handlers`, `data_store`.
    pub fn with_config_json(config_json: &str) -> Result<Self> {
        let lib = load(NAME)?;
        let new_view = sym::<ViewNew>(lib, b"tontoo_webkit_view_new\0")?;
        let config = CString::new(config_json)?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let raw = unsafe { new_view(config.as_ptr(), &mut error) };
        if raw.is_null() {
            let message = unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "view creation failed".to_owned());
            return Err(crate::runtime::SdkError(message));
        }
        Ok(Self { raw })
    }

    fn check(&self) -> Result<()> {
        if self.raw.is_null() {
            Err(crate::runtime::SdkError("web view already freed".into()))
        } else {
            Ok(())
        }
    }

    /// The underlying GTK4 widget pointer (borrowed; do not free).
    pub fn widget(&self) -> Result<*mut c_void> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooWebView) -> *mut c_void>(
            load(NAME)?,
            b"tontoo_webkit_view_widget\0",
        )?;
        Ok(unsafe { f(self.raw) })
    }

    /// Load a URL.
    pub fn load_url(&self, url: &str) -> Result<()> {
        self.check()?;
        let lib = load(NAME)?;
        let f = sym::<ViewLoadUrl>(lib, b"tontoo_webkit_view_load_url\0")?;
        let url = CString::new(url)?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { f(self.raw, url.as_ptr(), &mut error) };
        if rc == 0 {
            Ok(())
        } else {
            let message = unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "load failed".to_owned());
            Err(crate::runtime::SdkError(message))
        }
    }

    /// Load raw HTML, optionally with a base URI.
    pub fn load_html(&self, html: &str, base_uri: Option<&str>) -> Result<()> {
        self.check()?;
        let f = sym::<ViewLoadHtml>(load(NAME)?, b"tontoo_webkit_view_load_html\0")?;
        let html = CString::new(html)?;
        let base = base_uri.map(CString::new).transpose()?;
        unsafe { f(self.raw, html.as_ptr(), base.as_ref().map_or(std::ptr::null(), |b| b.as_ptr())) }
        Ok(())
    }

    pub fn go_back(&self) -> Result<()> {
        self.call_void(b"tontoo_webkit_view_go_back\0")
    }

    pub fn go_forward(&self) -> Result<()> {
        self.call_void(b"tontoo_webkit_view_go_forward\0")
    }

    pub fn reload(&self) -> Result<()> {
        self.call_void(b"tontoo_webkit_view_reload\0")
    }

    pub fn stop_loading(&self) -> Result<()> {
        self.call_void(b"tontoo_webkit_view_stop_loading\0")
    }

    pub fn can_go_back(&self) -> Result<bool> {
        Ok(self.call_bool(b"tontoo_webkit_view_can_go_back\0")? != 0)
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        Ok(self.call_bool(b"tontoo_webkit_view_can_go_forward\0")? != 0)
    }

    pub fn is_loading(&self) -> Result<bool> {
        Ok(self.call_bool(b"tontoo_webkit_view_is_loading\0")? != 0)
    }

    /// Current URL, if any.
    pub fn url(&self) -> Result<Option<String>> {
        self.check()?;
        let f = sym::<ViewString>(load(NAME)?, b"tontoo_webkit_view_get_url\0")?;
        Ok(unsafe { take_string(load(NAME)?, f(self.raw), FREE) })
    }

    /// Current page title, if any.
    pub fn title(&self) -> Result<Option<String>> {
        self.check()?;
        let f = sym::<ViewString>(load(NAME)?, b"tontoo_webkit_view_get_title\0")?;
        Ok(unsafe { take_string(load(NAME)?, f(self.raw), FREE) })
    }

    /// Estimated load progress between 0.0 and 1.0.
    pub fn progress(&self) -> Result<f64> {
        self.check()?;
        let f =
            sym::<ViewProgress>(load(NAME)?, b"tontoo_webkit_view_get_progress\0")?;
        Ok(unsafe { f(self.raw) })
    }

    /// Evaluate JavaScript; returns the result as JSON text.
    pub fn evaluate_javascript(&self, script: &str) -> Result<String> {
        self.check()?;
        let lib = load(NAME)?;
        let f = sym::<Evaluate>(lib, b"tontoo_webkit_view_evaluate_javascript\0")?;
        let script = CString::new(script)?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let result = unsafe { f(self.raw, script.as_ptr(), &mut error) };
        unsafe { take_string(lib, result, FREE) }
            .ok_or_else(|| crate::runtime::SdkError(
                unsafe { take_string(lib, error, FREE) }
                    .unwrap_or_else(|| "javascript evaluation failed".to_owned()),
            ))
    }

    /// Framework version string.
    pub fn version() -> Result<String> {
        let lib = load(NAME)?;
        let f = sym::<unsafe extern "C" fn() -> *const c_char>(
            lib,
            b"tontoo_webkit_version\0",
        )?;
        Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
    }

    fn call_void(&self, symbol: &[u8]) -> Result<()> {
        self.check()?;
        let f = sym::<ViewVoid>(load(NAME)?, symbol)?;
        unsafe { f(self.raw) };
        Ok(())
    }

    fn call_bool(&self, symbol: &[u8]) -> Result<c_int> {
        self.check()?;
        let f = sym::<ViewBool>(load(NAME)?, symbol)?;
        Ok(unsafe { f(self.raw) })
    }
}

impl Drop for WebView {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if let Ok(lib) = load(NAME) {
                if let Ok(free) =
                    sym::<unsafe extern "C" fn(*mut TontooWebView)>(
                        lib,
                        b"tontoo_webkit_view_free\0",
                    )
                {
                    unsafe { free(self.raw) };
                }
            }
            self.raw = std::ptr::null_mut();
        }
    }
}
