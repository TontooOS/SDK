//! MapsKit framework (2D map views, 3D globe, geocoding, routing).
//!
//! Loaded from `/Library/System/mapskit.library`. Service calls
//! ([`search`], [`reverse_geocode`], [`route`]) are blocking and belong on a
//! worker thread.

use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_int, c_void};

use crate::runtime::{load, read_static_string, sym, take_string, Result, SdkError};

const NAME: &str = "mapskit";
const FREE: &[u8] = b"tontoo_mapskit_string_free\0";

#[repr(C)]
struct TontooMapView {
    _opaque: [u8; 0],
}

#[repr(C)]
struct TontooGlobe {
    _opaque: [u8; 0],
}

type SearchFn = unsafe extern "C" fn(
    query: *const c_char,
    ref_lat: c_double,
    ref_lon: c_double,
    has_reference: c_int,
    limit: c_int,
    error_out: *mut *mut c_char,
) -> *mut c_char;

type ReverseGeocodeFn =
    unsafe extern "C" fn(lat: c_double, lon: c_double, error_out: *mut *mut c_char) -> *mut c_char;

type RouteFn = unsafe extern "C" fn(
    from_lat: c_double,
    from_lon: c_double,
    to_lat: c_double,
    to_lon: c_double,
    mode: c_int,
    error_out: *mut *mut c_char,
) -> *mut c_char;

/// Travel mode for [`route`].
#[derive(Clone, Copy, Debug)]
pub enum TravelMode {
    Driving = 0,
    Walking = 1,
    Cycling = 2,
}

impl TravelMode {
    fn as_int(self) -> c_int {
        self as c_int
    }
}

fn json_call<T>(
    symbol: &[u8],
    build: impl FnOnce(&T, *mut *mut c_char) -> *mut c_char,
) -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<T>(lib, symbol)?;
    let mut error: *mut c_char = std::ptr::null_mut();
    let result = build(&f, &mut error);
    unsafe { take_string(lib, result, FREE) }.ok_or_else(|| {
        SdkError(
            unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "maps service call failed".to_owned()),
        )
    })
}

/// Framework version string.
pub fn version() -> Result<String> {
    let lib = load(NAME)?;
    let f = sym::<unsafe extern "C" fn() -> *const c_char>(
        lib,
        b"tontoo_mapskit_version\0",
    )?;
    Ok(unsafe { read_static_string(f()) }.unwrap_or_default())
}

/// Search for places. Returns a JSON array of results.
pub fn search(query: &str, near: Option<(f64, f64)>, limit: i32) -> Result<String> {
    let query = CString::new(query)?;
    json_call::<SearchFn>(b"tontoo_mapskit_search\0", |f, err| unsafe {
        f(
            query.as_ptr(),
            near.map_or(0.0, |(lat, _)| lat),
            near.map_or(0.0, |(_, lon)| lon),
            near.is_some() as c_int,
            limit.clamp(1, 50),
            err,
        )
    })
}

/// Reverse geocode a coordinate. Returns a JSON address object.
pub fn reverse_geocode(lat: f64, lon: f64) -> Result<String> {
    json_call::<ReverseGeocodeFn>(b"tontoo_mapskit_reverse_geocode\0", |f, err| unsafe {
        f(lat, lon, err)
    })
}

/// Compute a route. Returns a JSON route object.
pub fn route(from: (f64, f64), to: (f64, f64), mode: TravelMode) -> Result<String> {
    json_call::<RouteFn>(b"tontoo_mapskit_route\0", |f, err| unsafe {
        f(from.0, from.1, to.0, to.1, mode.as_int(), err)
    })
}

/// A 2D map view backed by the system MapsKit.
pub struct MapView {
    raw: *mut TontooMapView,
}

unsafe impl Send for MapView {}

impl MapView {
    /// Create a map view. `config_json` may be `None` for defaults.
    ///
    /// Config keys: `style` (`light`/`dark`/`standard`),
    /// `shows_points_of_interest`, `cache_directory`, `user_agent`.
    pub fn new(config_json: Option<&str>) -> Result<Self> {
        let lib = load(NAME)?;
        let new_view =
            sym::<unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> *mut TontooMapView>(
                lib,
                b"tontoo_mapskit_view_new\0",
            )?;
        let config = config_json.map(CString::new).transpose()?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let raw = unsafe {
            new_view(config.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()), &mut error)
        };
        if raw.is_null() {
            let message = unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "view creation failed".to_owned());
            return Err(SdkError(message));
        }
        Ok(Self { raw })
    }

    fn check(&self) -> Result<()> {
        if self.raw.is_null() {
            Err(SdkError("map view already freed".into()))
        } else {
            Ok(())
        }
    }

    /// The underlying GTK4 widget pointer (borrowed; do not free).
    pub fn widget(&self) -> Result<*mut c_void> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooMapView) -> *mut c_void>(
            load(NAME)?,
            b"tontoo_mapskit_view_widget\0",
        )?;
        Ok(unsafe { f(self.raw) })
    }

    /// Set the camera center and zoom level.
    pub fn set_center(&self, lat: f64, lon: f64, zoom: f64) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooMapView, c_double, c_double, c_double)>(
            load(NAME)?,
            b"tontoo_mapskit_view_set_center\0",
        )?;
        unsafe { f(self.raw, lat, lon, zoom) };
        Ok(())
    }

    /// Add an annotation pin. JSON keys: `latitude`, `longitude`, `title`,
    /// optional `subtitle`.
    pub fn add_annotation(&self, annotation_json: &str) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooMapView, *const c_char)>(
            load(NAME)?,
            b"tontoo_mapskit_view_add_annotation\0",
        )?;
        let annotation = CString::new(annotation_json)?;
        unsafe { f(self.raw, annotation.as_ptr()) };
        Ok(())
    }

    /// Display a previously computed route JSON object.
    pub fn display_route(&self, route_json: &str) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooMapView, *const c_char)>(
            load(NAME)?,
            b"tontoo_mapskit_view_display_route\0",
        )?;
        let route = CString::new(route_json)?;
        unsafe { f(self.raw, route.as_ptr()) };
        Ok(())
    }

    /// Show the blue user location dot using CoreLocation.
    pub fn show_user_location(&self) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooMapView)>(
            load(NAME)?,
            b"tontoo_mapskit_view_show_user_location\0",
        )?;
        unsafe { f(self.raw) };
        Ok(())
    }
}

impl Drop for MapView {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if let Ok(lib) = load(NAME) {
                if let Ok(free) = sym::<unsafe extern "C" fn(*mut TontooMapView)>(
                    lib,
                    b"tontoo_mapskit_view_free\0",
                ) {
                    unsafe { free(self.raw) };
                }
            }
            self.raw = std::ptr::null_mut();
        }
    }
}

/// A 3D globe view backed by the system MapsKit.
pub struct Globe {
    raw: *mut TontooGlobe,
}

unsafe impl Send for Globe {}

impl Globe {
    /// Create a 3D globe. `config_json` may be `None` for defaults.
    pub fn new(config_json: Option<&str>) -> Result<Self> {
        let lib = load(NAME)?;
        let new_globe =
            sym::<unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> *mut TontooGlobe>(
                lib,
                b"tontoo_mapskit_globe_new\0",
            )?;
        let config = config_json.map(CString::new).transpose()?;
        let mut error: *mut c_char = std::ptr::null_mut();
        let raw = unsafe {
            new_globe(config.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()), &mut error)
        };
        if raw.is_null() {
            let message = unsafe { take_string(lib, error, FREE) }
                .unwrap_or_else(|| "globe creation failed".to_owned());
            return Err(SdkError(message));
        }
        Ok(Self { raw })
    }

    fn check(&self) -> Result<()> {
        if self.raw.is_null() {
            Err(SdkError("globe already freed".into()))
        } else {
            Ok(())
        }
    }

    /// The underlying GTK4 widget pointer (borrowed; do not free).
    pub fn widget(&self) -> Result<*mut c_void> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooGlobe) -> *mut c_void>(
            load(NAME)?,
            b"tontoo_mapskit_globe_widget\0",
        )?;
        Ok(unsafe { f(self.raw) })
    }

    /// Center the globe on a coordinate.
    pub fn set_center(&self, lat: f64, lon: f64) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooGlobe, c_double, c_double)>(
            load(NAME)?,
            b"tontoo_mapskit_globe_set_center\0",
        )?;
        unsafe { f(self.raw, lat, lon) };
        Ok(())
    }

    /// Add an accent marker dot with an optional label.
    pub fn add_marker(&self, lat: f64, lon: f64, label: Option<&str>) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooGlobe, c_double, c_double, *const c_char)>(
            load(NAME)?,
            b"tontoo_mapskit_globe_add_marker\0",
        )?;
        let label = label.map(CString::new).transpose()?;
        unsafe {
            f(self.raw, lat, lon, label.as_ref().map_or(std::ptr::null(), |l| l.as_ptr()))
        };
        Ok(())
    }

    /// Enable or disable idle rotation.
    pub fn set_auto_rotate(&self, enabled: bool) -> Result<()> {
        self.check()?;
        let f = sym::<unsafe extern "C" fn(*mut TontooGlobe, c_int)>(
            load(NAME)?,
            b"tontoo_mapskit_globe_set_auto_rotate\0",
        )?;
        unsafe { f(self.raw, enabled as c_int) };
        Ok(())
    }
}

impl Drop for Globe {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            if let Ok(lib) = load(NAME) {
                if let Ok(free) =
                    sym::<unsafe extern "C" fn(*mut TontooGlobe)>(
                        lib,
                        b"tontoo_mapskit_globe_free\0",
                    )
                {
                    unsafe { free(self.raw) };
                }
            }
            self.raw = std::ptr::null_mut();
        }
    }
}
