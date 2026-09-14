# application-shell Specification

## Purpose
TBD - created by archiving change m0-walking-skeleton. Update Purpose after archive.
## Requirements
### Requirement: CXX-Qt bridge exposes a Rust QObject to C++
The system SHALL define a Rust `QObject` through a `#[cxx_qt::bridge]` module and
SHALL expose it to the C++ shell using `cxx-qt` 0.10 and `cxx-qt-build`, rather
than hand-written `extern "C"` glue.

#### Scenario: C++ constructs the Rust bridge object
- **WHEN** the C++ shell constructs the Rust-defined `PictureView` object
- **THEN** the generated `QObject` is available to C++ and its methods are callable

#### Scenario: Rust owns the bridge state
- **WHEN** the bridged object is destroyed by C++
- **THEN** its Rust-side state is released without a double free

### Requirement: CMake builds the bridge and the C++ shell
The system SHALL build the application from a top-level `CMakeLists.txt` using
C++17, `find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets)`, and
`cxx_qt_import_crate` (Corrosion) to import the `pictura_app` static library and
link it into the `pictura` executable.

#### Scenario: Configure and build with CMake
- **WHEN** a developer runs `cmake -S . -B build && cmake --build build` with Qt 6 present
- **THEN** CMake imports `pictura_app` and links the `pictura` executable

### Requirement: Qt 6 is discovered through qmake6
The system SHALL locate a Qt 6 `qmake` by preferring `qmake6` over `qmake`,
verifying that `qmake -query QT_VERSION` reports a major version of 6, and SHALL
fail configuration with an error that names the missing package when no Qt 6
`qmake` is found.

#### Scenario: Qt 6 qmake is found
- **WHEN** `qmake6` reports a version starting with `6`
- **THEN** CMake uses it as the Qt discovery executable

#### Scenario: No Qt 6 qmake is available
- **WHEN** neither `qmake6` nor a Qt 6 `qmake` exists on `PATH`
- **THEN** CMake configuration fails with an error naming the Qt 6 development package

### Requirement: Headless window shell
The system SHALL open a top-level Qt Widgets window with a central image view and
SHALL run under a headless display server (Xvfb or the offscreen QPA plugin)
without requiring a GPU.

#### Scenario: Run under Xvfb
- **WHEN** the executable is started under `xvfb-run` or with `QT_QPA_PLATFORM=offscreen`
- **THEN** it opens its window and exits without error

### Requirement: Bridge self-test mode
The system SHALL provide a `--self-test` mode that exercises the Rust↔Qt bridge,
writes a summary to stderr, and exits non-zero when the bridged image is missing.

#### Scenario: Self-test succeeds
- **WHEN** the executable is started with `--self-test` and the bridge returns an image
- **THEN** it prints a self-test summary and exits 0

#### Scenario: Self-test fails on a null image
- **WHEN** the bridge returns no image
- **THEN** the self-test prints a failure and exits with a non-zero status

