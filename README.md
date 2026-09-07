# Tontoo SDK

Thin system SDK for TontooOS. Provides safe Rust bindings for every
framework installed under `/Library/System` as a `.library` binary.
Framework code is **not** compiled into your app - it is loaded at runtime,
so an app always uses whichever framework version is installed on the
running system.

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["Foundation", "WebKit"] }
```

Then at the crate root declare the shims (PascalCase):

```rust
sdk::preinclude!();

use Foundation::date_now;
use WebKit::WebView;

fn main() {
    println!("{}", Foundation::date_now().unwrap());
}
```

## License

TCL v26.1
