# Building Kooka Pictura on Fedora 43

The Fedora package names for the prerequisites in
[`DEVELOPING.md`](DEVELOPING.md), which uses Debian names. Every package here
was checked on a Fedora 43 machine that builds the project and passes the full
self-test.

## 1. System packages (required)

```bash
sudo dnf install gcc-c++ cmake ninja-build lld git \
    qt6-qtbase-devel qt6-qtbase-private-devel qt6-qtsvg-devel \
    lcms2-devel
```

- `qt6-qtbase-devel` provides the Core, Gui, Widgets, and Network modules and
  `qmake6`, which cxx-qt uses to locate Qt.
- `qt6-qtsvg-devel` provides the Svg module.
- `qt6-qtbase-private-devel` carries Qt's private headers; the build includes
  the RHI headers from the versioned QtGui include directory.
- `lcms2-devel` is Little CMS 2. The C++ binary links `-llcms2` directly,
  because a Rust `staticlib` does not pass its native link dependencies on.

## 2. Rust toolchain

The toolchain is pinned to 1.98 by `rust-toolchain.toml`. Install it with
rustup, not Fedora's `rust` package:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The first `cargo` or `cmake` build downloads 1.98. The test commands also use
nextest:

```bash
cargo install cargo-nextest --locked
```

## 3. Optional

```bash
# The --interop-probe diagnostic (it compiles to a stub without vulkan.h)
sudo dnf install vulkan-headers vulkan-loader-devel

# Oracle tests; each suite skips itself when its tool is missing
sudo dnf install ImageMagick python3-pip nodejs-npm
pip install --user "psd-tools>=1.19"

# OpenSpec CLI, used by scripts/verify-fast.sh
sudo npm i -g @fission-ai/openspec@1.13.2
```

- The oracle tests call `magick`, so they need ImageMagick 7. Fedora 43 ships
  7.1.x.
- On Fedora, npm is packaged as `nodejs-npm`.

## 4. Build and run

```bash
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld
cmake --build build --parallel
./build/pictura
```

The first configure needs network access: it fetches cxx-qt-cmake 0.10.0 and
the Rust crates.

To check the build without a display:

```bash
./build/pictura --headless --self-test
```

## Qt version

`docs/dev/STATE.md` records Qt 6.11.1, the version CI installs. Fedora 43
ships Qt 6.10.3, which builds and passes the whole C++ self-test, so the
distribution Qt works.
