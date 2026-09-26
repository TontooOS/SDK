# System bindings

## Minimal Example

```rust
sdk::frameworks!();

use accessibility::translate;

fn main() {
    println!("{}", translate("en", "greeting.hello"));
}
```

## What it does

The SDK gives apps access to TontooOS system frameworks without compiling
framework code into the binary. Each framework is loaded at runtime from
`/Library/System/<name>.library` with `dlopen`, so an app always uses the
version installed on the running system.

## Feature Index

| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Wiki authoring rules |
| Dynamic loading | [#dynamic-loading](#dynamic-loading) | Library loader, `TONTOO_LIB_DIR` override, version queries |
| Frameworks macro | [#frameworks-macro](#frameworks-macro) | `sdk::frameworks!()` shim modules and import rules |
| Bound frameworks | [#bound-frameworks](#bound-frameworks) | accessibility, coreicon, coreimage, corelocation, coretext, corewindows, foundation, mapskit, networkkit, tontooui, weatherkit, webkit APIs |

## Dynamic loading

`sdk::runtime::load("name")` opens `/Library/System/name.library` once per
process and keeps it alive until exit. The directory defaults to
`/Library/System` and can be overridden with the `TONTOO_LIB_DIR`
environment variable.

`sdk::runtime::version_of("name")` asks any framework for its version via
its `tontoo_<name>_version` or `<name>_version` symbol.

```rust
let v = sdk::runtime::version_of("webkit").unwrap();
```

## Frameworks macro

Invoke once at the crate root of every binary:

```rust
sdk::frameworks!();
```

This declares root-level shim modules so top-level imports work exactly
like normal crates:

```rust
use accessibility::translate;
use webkit::WebView;
```

Inside nested modules the shim lives at the crate root, so use the
`crate::` prefix there:

```rust
mod inner {
    pub fn demo() -> String {
        use crate::accessibility::translate;
        translate("en", "greeting.hello").to_uppercase()
    }
}
```

## Bound frameworks

### accessibility

| Function | Purpose |
| --- | --- |
| `init(fallback: &str)` | Initialize translation store with fallback language |
| `translate(lang: &str, key: &str) -> String` | Translate a key, falls back to the key itself |
| `langs() -> Vec<String>` | All loaded language codes |
| `lang_count() -> Result<i32>` | Number of loaded languages |
| `version() -> Result<String>` | Framework version |

### coreicon

| Function / Type | Purpose |
| --- | --- |
| `Color { r, g, b, a }` | RGBA color with float components |
| `color_from_hex(&str) -> Result<Color>` | Parse `#rrggbb` / `#rrggbbaa` |

### corelocation

Blocking - worker thread.

| Function | Purpose |
| --- | --- |
| `get_location() -> Result<String>` | JSON location object (blocking) |
| `get_location_from(source) -> Result<String>` | 0 GPS, 1 WiFi, 2 IP, 3 timezone, 4 manual |

### corewindows

| Function | Purpose |
| --- | --- |
| `ping() -> Result<bool>` | Ping the window daemon (default socket) |
| `ping_at(socket) -> Result<bool>` | Ping the window daemon at an explicit socket |
| `list_windows() -> Result<String>` | Open windows as a JSON array of `WindowInfo` |
| `list_windows_at(socket) -> Result<String>` | Same via an explicit socket |
| `minimize_window(id) -> Result<()>` | Minimize (iconify) a window |
| `set_fullscreen(id, fullscreen) -> Result<()>` | Fullscreen / unfullscreen a window |
| `close_window(id) -> Result<()>` | Graceful close (app may show a save dialog) |
| `force_quit_window(id) -> Result<()>` | Force quit the window owner (`SIGKILL`) |
| `force_quit_pid(pid) -> Result<()>` | Force quit a process id (`SIGKILL`) |
| `list_programs() -> Result<String>` | Installed programs as a JSON array of `AppEntry` |

### foundation

| Function | Purpose |
| --- | --- |
| `date_now() -> Result<i64>` | Unix timestamp in seconds |
| `date_add_days(secs, days)` / `date_is_before(a, b)` | Date math |
| `defaults_get_string / set_string` | Standard user defaults store |
| `defaults_get_int / set_int`, `defaults_get_bool / set_bool` | Typed defaults access |

### networkkit

Blocking - worker thread. All results are JSON strings.

| Function | Purpose |
| --- | --- |
| `scan_wifi() -> Result<String>` | WiFi networks as JSON array |
| `wifi_status() -> Result<Option<String>>` | Current connection or None |
| `local_interfaces()` / `neighbors()` | Interface and neighbor listings |
| `discover_bluetooth(timeout_secs)` / `paired_bluetooth_devices()` | Bluetooth devices |

### mapskit

Blocking service calls - worker thread.

| Function / Type | Purpose |
| --- | --- |
| `search(query, near, limit) -> Result<String>` | Place search, returns JSON |
| `reverse_geocode(lat, lon) -> Result<String>` | Address lookup, returns JSON |
| `route(from, to, mode) -> Result<String>` | Routing, returns JSON |
| `MapView` | 2D map view: `set_center`, `add_annotation`, `display_route`, `show_user_location`, `widget` |
| `Globe` | 3D globe: `set_center`, `add_marker`, `set_auto_rotate`, `widget` |

### tontooui

Handle-based components; destroyed automatically on drop.

| Function / Type | Purpose |
| --- | --- |
| `ProgressView::new()` + `.widget()` | Spinner progress view |
| `TextInput::new(placeholder)` + `.text()` + `.widget()` | Text input field |

### coreimage

| Function | Purpose |
| --- | --- |
| `version() -> Result<String>` | Framework version string |
| `dimensions(path) -> Result<(u32, u32)>` | Image size without full processing |
| `blur_to_file(input, output, sigma) -> Result<()>` | Gaussian blur into a PNG file |

With `features = ["CoreImage"]` the full crate is re-exported as
`sdk::CoreImage` instead of this thin binding.

### coretext

| Function | Purpose |
| --- | --- |
| `version() -> Result<String>` | Framework version string |
| `measure(text, size, scale) -> Result<(f32, f32)>` | Logical text size via the system library |

With `features = ["CoreText"]` the full crate is re-exported as
`sdk::CoreText` instead of this thin binding.

### weatherkit

Blocking network calls - worker thread.

| Function | Purpose |
| --- | --- |
| `current_weather() -> Result<String>` | JSON weather object |
| `temperature() -> Result<f64>` | Temperature in Celsius |
| `weekly_forecast() -> Result<String>` | JSON forecast array |

### webkit

| Function / Type | Purpose |
| --- | --- |
| `WebView::new(start_url)` / `with_config_json(&str)` | Create a browser view |
| `load_url`, `load_html`, `go_back`, `go_forward`, `reload`, `stop_loading` | Navigation |
| `url()`, `title()`, `is_loading()`, `progress()` | Load state |
| `evaluate_javascript(&str) -> Result<String>` | Run JS, returns JSON result text |
| `widget() -> *mut c_void` | GTK4 widget pointer for embedding |

All view and component handles destroy their system counterpart
automatically on drop.
