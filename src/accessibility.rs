//! Accessibility framework (i18n / translations).
//!
//! Loaded from `/Library/System/accessibility.library`.

use std::ffi::CString;
use std::os::raw::c_char;

use crate::runtime::{read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "accessibility";
const FREE: &[u8] = b"accessibility_free_string\0";

/// Initialize the translation store with a fallback language.
pub fn init(fallback: &str) -> Result<()> {
    let lib = crate::runtime::load(NAME)?;
    let f = sym::<unsafe extern "C" fn(*const c_char) -> i32>(
        lib,
        b"accessibility_init\0",
    )?;
    let fallback = CString::new(fallback)?;
    let rc = unsafe { f(fallback.as_ptr()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(SdkError(format!("accessibility_init failed with code {rc}")))
    }
}

/// Translate `key` for `lang`; falls back to the key itself on any error.
pub fn translate(lang: &str, key: &str) -> String {
    translate_inner(lang, key).unwrap_or_else(|| key.to_string())
}

fn translate_inner(lang: &str, key: &str) -> Option<String> {
    let lib = crate::runtime::load(NAME).ok()?;
    let f = sym::<unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char>(
        lib,
        b"accessibility_translate\0",
    )
    .ok()?;
    let lang = CString::new(lang).ok()?;
    let key = CString::new(key).ok()?;
    let raw = unsafe { f(lang.as_ptr(), key.as_ptr()) };
    unsafe { take_string(lib, raw, FREE) }
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = crate::runtime::load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"accessibility_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Number of loaded languages.
pub fn lang_count() -> Result<i32> {
    let lib = crate::runtime::load(NAME)?;
    let f =
        sym::<unsafe extern "C" fn() -> i32>(lib, b"accessibility_lang_count\0")?;
    Ok(unsafe { f() })
}

/// All loaded language codes.
pub fn langs() -> Vec<String> {
    let mut langs = Vec::new();
    for index in 0.. {
        match lang_at(index) {
            Some(code) => langs.push(code),
            None => break,
        }
        if index > 4096 {
            break;
        }
    }
    langs
}

fn lang_at(index: i32) -> Option<String> {
    let lib = crate::runtime::load(NAME).ok()?;
    let f = sym::<unsafe extern "C" fn(i32) -> *const c_char>(
        lib,
        b"accessibility_lang_at\0",
    )
    .ok()?;
    unsafe { read_static_string(f(index)) }
}
