//! TontooUI framework (declarative UI components).

use std::ffi::CString;
use std::os::raw::{c_char, c_void};

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "tontooui";
const FREE: &[u8] = b"tontooui_string_free\0";

#[repr(C)]
struct RawProgressView {
    _opaque: [u8; 0],
}

#[repr(C)]
struct RawTextInput {
    _opaque: [u8; 0],
}

type NewFn<T> = unsafe extern "C" fn() -> *mut T;
type NewStrFn<T> = unsafe extern "C" fn(*const c_char) -> *mut T;
type WidgetFn<T> = unsafe extern "C" fn(*mut T) -> *mut c_void;
type TextFn<T> = unsafe extern "C" fn(*mut T) -> *mut c_char;
type FreeFn<T> = unsafe extern "C" fn(*mut T);

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(lib, b"tontooui_version\0")?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// A spinner-style progress view. Destroyed automatically on drop.
pub struct ProgressView {
    raw: *mut RawProgressView,
}

unsafe impl Send for ProgressView {}

impl ProgressView {
    /// Create a new progress view.
    pub fn new() -> Result<Self> {
        let f = sym::<NewFn<RawProgressView>>(load(NAME)?, b"tontooui_progress_view_new\0")?;
        let raw = unsafe { f() };
        if raw.is_null() {
            return Err(SdkError("progress view creation failed".into()));
        }
        Ok(Self { raw })
    }

    /// The underlying GTK4 widget pointer (borrowed; do not free).
    pub fn widget(&self) -> Result<*mut c_void> {
        if self.raw.is_null() {
            return Err(SdkError("progress view already freed".into()));
        }
        let f = sym::<WidgetFn<RawProgressView>>(load(NAME)?, b"tontooui_progress_view_widget\0")?;
        Ok(unsafe { f(self.raw) })
    }
}

impl Drop for ProgressView {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if let Ok(lib) = load(NAME) {
                if let Ok(free) =
                    sym::<FreeFn<RawProgressView>>(lib, b"tontooui_progress_view_free\0")
                {
                    unsafe { free(self.raw) };
                }
            }
            self.raw = std::ptr::null_mut();
        }
    }
}

/// A text input field. Destroyed automatically on drop.
pub struct TextInput {
    raw: *mut RawTextInput,
}

unsafe impl Send for TextInput {}

impl TextInput {
    /// Create a text input with a placeholder.
    pub fn new(placeholder: &str) -> Result<Self> {
        let lib = load(NAME)?;
        let f = sym::<NewStrFn<RawTextInput>>(lib, b"tontoo_tontooui_text_input_new\0")?;
        let placeholder = CString::new(placeholder)?;
        let raw = unsafe { f(placeholder.as_ptr()) };
        if raw.is_null() {
            return Err(SdkError("text input creation failed".into()));
        }
        Ok(Self { raw })
    }

    /// The current input text.
    pub fn text(&self) -> Result<String> {
        if self.raw.is_null() {
            return Err(SdkError("text input already freed".into()));
        }
        let lib = load(NAME)?;
        let f = sym::<TextFn<RawTextInput>>(lib, b"tontoo_tontooui_text_input_text\0")?;
        unsafe { take_string(lib, f(self.raw), FREE) }
            .ok_or_else(|| SdkError("no text returned".into()))
    }

    /// The underlying GTK4 widget pointer (borrowed; do not free).
    pub fn widget(&self) -> Result<*mut c_void> {
        if self.raw.is_null() {
            return Err(SdkError("text input already freed".into()));
        }
        let f = sym::<WidgetFn<RawTextInput>>(load(NAME)?, b"tontooui_text_input_widget\0")?;
        Ok(unsafe { f(self.raw) })
    }
}

impl Drop for TextInput {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if let Ok(lib) = load(NAME) {
                if let Ok(free) =
                    sym::<FreeFn<RawTextInput>>(lib, b"tontoo_tontooui_text_input_free\0")
                {
                    unsafe { free(self.raw) };
                }
            }
            self.raw = std::ptr::null_mut();
        }
    }
}
