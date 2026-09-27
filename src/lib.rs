//! TontooOS system SDK.
//!
//! Thin, safe bindings for the frameworks installed under
//! `/Library/System` as `<name>.library` binaries. Each framework is
//! opened once per process with `dlopen`; apps therefore always use the
//! version installed on the running system instead of compiling framework
//! code into the binary.
//!
//! Full crate re-exports are available via PascalCase modules when the
//! corresponding feature is enabled (e.g. `features = ["TontooUI"]`).
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! sdk = { path = "/Library/System/sdk", features = ["TontooUI", "UIKit"] }
//! ```
//!
//! ```rust
//! // main.rs – one line at the top creates PascalCase shims:
//! sdk::preinclude!();
//!
//! use TontooUI::Button;
//!
//! fn main() {}
//! ```
//!
//! Legacy lowercase `sdk::frameworks!()` still works.

pub mod accessibility;
pub mod audiokit;
pub mod coreicon;
pub mod coreimage;
pub mod corelocation;
pub mod coretext;
pub mod corewindows;
pub mod foundation;
pub mod mapskit;
pub mod mediakit;
pub mod networkkit;
pub mod runtime;
pub mod tontooui;
pub mod weatherkit;
pub mod webkit;

// ── PascalCase re-exports (full crate when feature enabled, else FFI fallback) ──

#[cfg(feature = "accessibility")]
#[allow(non_snake_case)]
pub mod Accessibility {
    pub use ::accessibility::*;
}
#[cfg(not(feature = "accessibility"))]
#[allow(non_snake_case)]
pub mod Accessibility {
    pub use crate::accessibility::*;
}

#[cfg(feature = "audiokit")]
#[allow(non_snake_case)]
pub mod AudioKit {
    pub use ::audiokit::*;
}
#[cfg(not(feature = "audiokit"))]
#[allow(non_snake_case)]
pub mod AudioKit {
    pub use crate::audiokit::*;
}

#[cfg(feature = "coreicon")]
#[allow(non_snake_case)]
pub mod CoreIcon {
    pub use ::coreicon::*;
}
#[cfg(not(feature = "coreicon"))]
#[allow(non_snake_case)]
pub mod CoreIcon {
    pub use crate::coreicon::*;
}

#[cfg(feature = "coreimage")]
#[allow(non_snake_case)]
pub mod CoreImage {
    pub use ::coreimage::*;
}
#[cfg(not(feature = "coreimage"))]
#[allow(non_snake_case)]
pub mod CoreImage {
    pub use crate::coreimage::*;
}

#[cfg(feature = "coretext")]
#[allow(non_snake_case)]
pub mod CoreText {
    pub use ::coretext::*;
}
#[cfg(not(feature = "coretext"))]
#[allow(non_snake_case)]
pub mod CoreText {
    pub use crate::coretext::*;
}

#[cfg(feature = "corelocation")]
#[allow(non_snake_case)]
pub mod CoreLocation {
    pub use ::corelocation::*;
}
#[cfg(not(feature = "corelocation"))]
#[allow(non_snake_case)]
pub mod CoreLocation {
    pub use crate::corelocation::*;
}

#[cfg(feature = "foundation")]
#[allow(non_snake_case)]
pub mod Foundation {
    pub use ::foundation::*;
}
#[cfg(not(feature = "foundation"))]
#[allow(non_snake_case)]
pub mod Foundation {
    pub use crate::foundation::*;
}

#[cfg(feature = "mapskit")]
#[allow(non_snake_case)]
pub mod MapsKit {
    pub use ::mapskit::*;
}
#[cfg(not(feature = "mapskit"))]
#[allow(non_snake_case)]
pub mod MapsKit {
    pub use crate::mapskit::*;
}

#[cfg(feature = "mediakit")]
#[allow(non_snake_case)]
pub mod MediaKit {
    pub use ::mediakit::*;
}
#[cfg(not(feature = "mediakit"))]
#[allow(non_snake_case)]
pub mod MediaKit {
    pub use crate::mediakit::*;
}

#[cfg(feature = "networkkit")]
#[allow(non_snake_case)]
pub mod NetworkKit {
    pub use ::networkkit::*;
}
#[cfg(not(feature = "networkkit"))]
#[allow(non_snake_case)]
pub mod NetworkKit {
    pub use crate::networkkit::*;
}

#[cfg(feature = "pdfkit")]
#[allow(non_snake_case)]
pub mod PDFKit {
    pub use ::pdfkit::*;
}
#[cfg(not(feature = "pdfkit"))]
#[allow(non_snake_case)]
pub mod PDFKit {}

#[cfg(feature = "tontooui")]
#[allow(non_snake_case)]
pub mod TontooUI {
    pub use ::tontooui::*;
}
#[cfg(not(feature = "tontooui"))]
#[allow(non_snake_case)]
pub mod TontooUI {
    pub use crate::tontooui::*;
}

#[cfg(feature = "weatherkit")]
#[allow(non_snake_case)]
pub mod WeatherKit {
    pub use ::weatherkit::*;
}
#[cfg(not(feature = "weatherkit"))]
#[allow(non_snake_case)]
pub mod WeatherKit {
    pub use crate::weatherkit::*;
}

#[cfg(feature = "webkit")]
#[allow(non_snake_case)]
pub mod WebKit {
    pub use ::webkit::*;
}
#[cfg(not(feature = "webkit"))]
#[allow(non_snake_case)]
pub mod WebKit {
    pub use crate::webkit::*;
}

// CoreData / FishFile / CoreSettings have no thin FFI shim – only full crate
#[cfg(feature = "corewindows")]
#[allow(non_snake_case)]
pub mod CoreWindows {
  pub use ::corewindows::*;
}
#[cfg(not(feature = "corewindows"))]
#[allow(non_snake_case)]
pub mod CoreWindows {
  pub use crate::corewindows::*;
}

#[cfg(feature = "coredata")]
#[allow(non_snake_case)]
pub mod CoreData {
    pub use ::coredata::*;
}
#[cfg(not(feature = "coredata"))]
#[allow(non_snake_case)]
pub mod CoreData {}

#[cfg(feature = "coresettings")]
#[allow(non_snake_case)]
pub mod CoreSettings {
    pub use ::coresettings::*;
}
#[cfg(not(feature = "coresettings"))]
#[allow(non_snake_case)]
pub mod CoreSettings {}

#[cfg(feature = "fishfile")]
#[allow(non_snake_case)]
pub mod FishFile {
    pub use ::fishfile::*;
}
#[cfg(not(feature = "fishfile"))]
#[allow(non_snake_case)]
pub mod FishFile {}

// LaunchPad has no thin FFI shim (the lib exposes a Rust client API) –
// full crate re-export only, like CoreData / FishFile / CoreSettings.
#[cfg(feature = "launchpad")]
#[allow(non_snake_case)]
pub mod LaunchPad {
    pub use ::launchpad::*;
}
#[cfg(not(feature = "launchpad"))]
#[allow(non_snake_case)]
pub mod LaunchPad {}

/// Preferred entry point: creates PascalCase shims at the crate root
/// so `use TontooUI::...`, `use UIKit::...` etc. work.
///
/// Also creates lowercase shims for backward compatibility
/// (`use accessibility::...`).
#[macro_export]
macro_rules! preinclude {
    () => {
        #[allow(non_snake_case, unused_imports)]
        mod Accessibility {
            pub use ::sdk::Accessibility::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod AudioKit {
            pub use ::sdk::AudioKit::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreData {
            pub use ::sdk::CoreData::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreIcon {
            pub use ::sdk::CoreIcon::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreImage {
            pub use ::sdk::CoreImage::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreLocation {
            pub use ::sdk::CoreLocation::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreText {
            pub use ::sdk::CoreText::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreWindows {
            pub use ::sdk::CoreWindows::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod FishFile {
            pub use ::sdk::FishFile::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod LaunchPad {
            pub use ::sdk::LaunchPad::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod Foundation {
            pub use ::sdk::Foundation::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod MapsKit {
            pub use ::sdk::MapsKit::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod MediaKit {
            pub use ::sdk::MediaKit::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod NetworkKit {
            pub use ::sdk::NetworkKit::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod PDFKit {
            pub use ::sdk::PDFKit::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod CoreSettings {
            pub use ::sdk::CoreSettings::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod TontooUI {
            pub use ::sdk::TontooUI::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod WeatherKit {
            pub use ::sdk::WeatherKit::*;
        }
        #[allow(non_snake_case, unused_imports)]
        mod WebKit {
            pub use ::sdk::WebKit::*;
        }
        // lowercase backward-compat
        #[allow(unused_imports)]
        mod accessibility {
            pub use ::sdk::accessibility::*;
        }
        #[allow(unused_imports)]
        mod audiokit {
            pub use ::sdk::audiokit::*;
        }
        #[allow(unused_imports)]
        mod coreicon {
            pub use ::sdk::coreicon::*;
        }
        #[allow(unused_imports)]
        mod coreimage {
            pub use ::sdk::coreimage::*;
        }
        #[allow(unused_imports)]
        mod corelocation {
            pub use ::sdk::corelocation::*;
        }
        #[allow(unused_imports)]
        mod coretext {
            pub use ::sdk::coretext::*;
        }
        #[allow(unused_imports)]
        mod corewindows {
            pub use ::sdk::corewindows::*;
        }
        #[allow(unused_imports)]
        mod foundation {
            pub use ::sdk::foundation::*;
        }
        #[allow(unused_imports)]
        mod mapskit {
            pub use ::sdk::mapskit::*;
        }
        #[allow(unused_imports)]
        mod mediakit {
            pub use ::sdk::mediakit::*;
        }
        #[allow(unused_imports)]
        mod networkkit {
            pub use ::sdk::networkkit::*;
        }
        #[allow(unused_imports)]
        mod tontooui {
            pub use ::sdk::tontooui::*;
        }
        #[allow(unused_imports)]
        mod weatherkit {
            pub use ::sdk::weatherkit::*;
        }
        #[allow(unused_imports)]
        mod webkit {
            pub use ::sdk::webkit::*;
        }
    };
}

/// Legacy name – same as `preinclude!()`.
#[macro_export]
macro_rules! frameworks {
    () => {
        $crate::preinclude!();
    };
}