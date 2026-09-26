//! Dynamic loader for TontooOS system frameworks.
//!
//! Every framework lives as a single binary at `<lib-dir>/<name>.library`
//! (default `/Library/System`). Libraries are opened once per process and
//! kept alive until exit; symbols are resolved lazily on first call.

use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use libloading::{Library, Symbol};

const DEFAULT_LIB_DIR: &str = "/Library/System";
const WSL_LIB_DIR: &str = "/mnt/c/Users/arlo1/Documents/TontooLibs";

static REGISTRY: OnceLock<Mutex<HashMap<String, &'static Library>>> = OnceLock::new();

fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|s| {
            let lower = s.to_ascii_lowercase();
            lower.contains("microsoft") || lower.contains("wsl")
        })
        .unwrap_or(false)
}

fn pascal_dir(framework: &str) -> Option<&'static str> {
    match framework {
        "accessibility" => Some("Accessibility"),
        "audiokit" => Some("AudioKit"),
        "coredata" => Some("CoreData"),
        "coreicon" => Some("CoreIcon"),
        "coreimage" => Some("CoreImage"),
        "corelocation" => Some("CoreLocation"),
        "fishfile" => Some("FishFile"),
        "foundation" => Some("Foundation"),
        "mapskit" => Some("MapsKit"),
        "networkkit" => Some("NetworkKit"),
        "tontooui" => Some("TontooUI"),
        "weatherkit" => Some("WeatherKit"),
        "webkit" => Some("WebKit"),
        _ => None,
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;

#[derive(Debug)]
pub struct SdkError(pub String);

impl std::fmt::Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SdkError {}

impl From<std::ffi::NulError> for SdkError {
    fn from(e: std::ffi::NulError) -> Self {
        SdkError(format!("string contains interior NUL byte: {e}"))
    }
}

/// Directory the SDK loads `.library` files from.
///
/// Defaults to `/Library/System`; override with the `TONTOO_LIB_DIR`
/// environment variable (useful on build machines without a full install).
/// On WSL automatically uses `/mnt/c/Users/arlo1/Documents/TontooLibs`.
pub fn lib_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("TONTOO_LIB_DIR") {
        return PathBuf::from(dir);
    }
    if is_wsl() {
        return PathBuf::from(WSL_LIB_DIR);
    }
    PathBuf::from(DEFAULT_LIB_DIR)
}

/// Load a framework by name, opening it only once per process.
///
/// On WSL tries PascalCase per-lib paths first:
/// `<WSL_LIB_DIR>/<PascalDir>/target/release/<name>.library`
/// and `<WSL_LIB_DIR>/<PascalDir>/<name>.library` before falling back
/// to `<lib_dir>/<name>.library`.
pub fn load(framework: &str) -> Result<&'static Library> {
    let registry = REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = registry.lock().expect("sdk library registry poisoned");
    if let Some(lib) = guard.get(framework) {
        return Ok(lib);
    }
    let candidates: Vec<PathBuf> = if std::env::var_os("TONTOO_LIB_DIR").is_none() && is_wsl() {
        let mut v = Vec::new();
        if let Some(dir) = pascal_dir(framework) {
            let base = PathBuf::from(WSL_LIB_DIR);
            v.push(base.join(dir).join("target").join("release").join(format!("{framework}.library")));
            v.push(base.join(dir).join(format!("{framework}.library")));
        }
        v.push(lib_dir().join(format!("{framework}.library")));
        v.push(PathBuf::from(DEFAULT_LIB_DIR).join(format!("{framework}.library")));
        v
    } else {
        vec![lib_dir().join(format!("{framework}.library"))]
    };

    let mut last_err = String::new();
    for path in &candidates {
        match unsafe { Library::new(path) } {
            Ok(l) => {
                let lib: &'static Library = Box::leak(Box::new(l));
                guard.insert(framework.to_owned(), lib);
                return Ok(lib);
            }
            Err(e) => {
                last_err = format!("failed to load {}: {e}", path.display());
            }
        }
    }
    Err(SdkError(last_err))
}

/// Resolve a symbol from an already loaded framework.
pub(crate) fn sym<'lib, T>(lib: &'lib Library, name: &[u8]) -> Result<Symbol<'lib, T>> {
    unsafe { lib.get::<T>(name) }
        .map_err(|e| SdkError(format!("missing symbol {}: {e}", String::from_utf8_lossy(name))))
}

/// Read and free a string owned by a framework.
///
/// # Safety
///
/// `ptr` must have been returned by the framework and must still be valid;
/// `free_symbol` must be the framework's matching free function.
pub(crate) unsafe fn take_string(
    lib: &'static Library,
    ptr: *mut c_char,
    free_symbol: &[u8],
) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    if let Ok(free) = sym::<unsafe extern "C" fn(*mut c_char)>(lib, free_symbol) {
        free(ptr);
    }
    Some(s)
}

/// Read a static (non-owned) C string.
///
/// # Safety
///
/// `ptr` must point to a valid static NUL-terminated string or be null.
pub(crate) unsafe fn read_static_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
}

/// Ask any framework for its version via its `*_version` symbol.
///
/// Tries `tontoo_<framework>_version` first, then `<framework>_version`.
pub fn version_of(framework: &str) -> Option<String> {
    let lib = load(framework).ok()?;
    for name in [
        format!("tontoo_{framework}_version\0"),
        format!("{framework}_version\0"),
    ] {
        if let Ok(f) =
            sym::<unsafe extern "C" fn() -> *const c_char>(lib, name.as_bytes())
        {
            if let Some(v) = unsafe { read_static_string(f()) } {
                return Some(v);
            }
        }
    }
    None
}
