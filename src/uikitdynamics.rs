//! UIKitDynamics framework (springs, easings, tweens).

use std::os::raw::{c_char, c_int};

use crate::runtime::{load, read_static_string, sym, Result};

const NAME: &str = "uikitdynamics";

type EasingFn = unsafe extern "C" fn(c_int, f32) -> f32;
type SpringAdvanceFn = unsafe extern "C" fn(c_int, f32, f32, *mut f32, f32) -> f32;
type SpringAtRestFn = unsafe extern "C" fn(c_int, f32, f32, f32) -> c_int;

#[repr(C)]
struct RawTween {
    _opaque: [u8; 0],
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_uikitdynamics_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Apply an easing curve to a progress value `t` (0.0..=1.0).
///
/// `kind`: 0 linear, 1 quad-in, 2 quad-out, 3 quad-in-out, 4 cubic-in,
/// 5 cubic-out, 6 cubic-in-out, 7 sine-in, 8 sine-out, 9 sine-in-out,
/// 10 back-out, 11 back-in, 12 bounce-out.
pub fn easing_apply(kind: c_int, t: f32) -> Result<f32> {
    let f = sym::<EasingFn>(load(NAME)?, b"tontoo_uikitdynamics_easing_apply\0")?;
    Ok(unsafe { f(kind, t) })
}

/// Advance a spring one step. `preset`: 0 default, 1 snappy, 2 bouncy,
/// 3 soft. Returns the new value and updates `velocity`.
pub fn spring_advance(
    preset: c_int,
    current: f32,
    target: f32,
    velocity: &mut f32,
    dt: f32,
) -> Result<f32> {
    let f = sym::<SpringAdvanceFn>(load(NAME)?, b"tontoo_uikitdynamics_spring_advance\0")?;
    Ok(unsafe { f(preset, current, target, velocity as *mut f32, dt) })
}

/// Whether a spring has come to rest.
pub fn spring_at_rest(preset: c_int, current: f32, target: f32, velocity: f32) -> Result<bool> {
    let f = sym::<SpringAtRestFn>(load(NAME)?, b"tontoo_uikitdynamics_spring_at_rest\0")?;
    Ok(unsafe { f(preset, current, target, velocity) } != 0)
}

/// A tween animation handle. Destroyed automatically on drop.
pub struct Tween {
    raw: *mut RawTween,
}

unsafe impl Send for Tween {}

impl Tween {
    /// Create a tween from `from` to `to` over `duration` seconds.
    ///
    /// See [`easing_apply`] for `kind` values.
    pub fn new(from: f32, to: f32, duration: f32, kind: c_int) -> Result<Self> {
        let f = sym::<unsafe extern "C" fn(f32, f32, f32, c_int) -> *mut RawTween>(
            load(NAME)?,
            b"tontoo_uikitdynamics_tween_new\0",
        )?;
        let raw = unsafe { f(from, to, duration, kind) };
        if raw.is_null() {
            return Err(crate::runtime::SdkError("tween creation failed".into()));
        }
        Ok(Self { raw })
    }

    fn check(&self) -> Result<()> {
        if self.raw.is_null() {
            Err(crate::runtime::SdkError("tween already freed".into()))
        } else {
            Ok(())
        }
    }

    fn call_f32(&self, symbol: &[u8]) -> Result<f32> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut RawTween) -> f32>(load(NAME)?, symbol)?;
        Ok(unsafe { f(self.raw) })
    }

    /// The tween's current interpolated value.
    pub fn value(&self) -> Result<f32> {
        self.call_f32(b"tontoo_uikitdynamics_tween_value\0")
    }

    /// The tween's progress between 0.0 and 1.0.
    pub fn progress(&self) -> Result<f32> {
        self.call_f32(b"tontoo_uikitdynamics_tween_progress\0")
    }

    /// Advance by `dt` seconds. Returns true when finished.
    pub fn advance(&mut self, dt: f32) -> Result<bool> {
        self.check()?;
        let f =
            sym::<unsafe extern "C" fn(*mut RawTween, f32) -> c_int>(load(NAME)?, b"tontoo_uikitdynamics_tween_advance\0")?;
        Ok(unsafe { f(self.raw, dt) } != 0)
    }
}

impl Drop for Tween {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if let Ok(lib) = load(NAME) {
                if let Ok(free) =
                    sym::<unsafe extern "C" fn(*mut RawTween)>(lib, b"tontoo_uikitdynamics_tween_free\0")
                {
                    unsafe { free(self.raw) };
                }
            }
            self.raw = std::ptr::null_mut();
        }
    }
}
