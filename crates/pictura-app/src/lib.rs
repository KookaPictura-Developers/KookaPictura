//! M0 application shell. This crate is the Rust↔Qt bridge spike.
//!
//! ## M0 contract (owned by task M0-A)
//!
//! Prove that a Qt 6 window can be driven from Rust via **cxx-qt**:
//!
//! 1. A `QObject` subclass defined in Rust, exposed to C++/QML.
//! 2. A top-level window that opens.
//! 3. It can load a PSD through `pictura-codec` and show the composite image.
//! 4. It builds via CMake against the system Qt 6.11.
//!
//! If cxx-qt cannot build against Qt 6.11.1, document the exact failure and the
//! fallback (Qt 6.8 LTS via aqtinstall, or `qmetaobject-rs`). Do not paper over
//! a version incompatibility.

pub fn hello() -> &'static str {
    "pictura-app"
}
