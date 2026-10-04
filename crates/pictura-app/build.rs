use std::path::PathBuf;

use cxx_qt_build::CxxQtBuilder;

/// Locate a Qt 6 `qmake`.
///
/// `qt-build-utils` drives Qt discovery through `qmake`. On Debian/Ubuntu the
/// bare `qmake` may be Qt 5, so prefer `qmake6`.
fn find_qmake6() -> Option<PathBuf> {
    for dir in std::env::split_paths(&std::env::var_os("PATH")?) {
        for name in ["qmake6", "qmake"] {
            let candidate = dir.join(name);
            if !candidate.is_file() {
                continue;
            }
            let output = std::process::Command::new(&candidate)
                .args(["-query", "QT_VERSION"])
                .output();
            if let Ok(output) = output {
                if output.status.success() && output.stdout.starts_with(b"6") {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn main() {
    // CMake's cxx_qt_import_crate already exports QMAKE; only fill it in for
    // standalone `cargo build`.
    if std::env::var_os("QMAKE").is_none() {
        if let Some(qmake) = find_qmake6() {
            std::env::set_var("QMAKE", qmake);
        }
    }

    // `cpp/` holds hand-written headers (e.g. `decode_image.h`) included by the
    // cxx bridge, so the generated C++ glue can find them. The `.cpp` files
    // themselves are compiled and linked by CMake, not here.
    unsafe {
        CxxQtBuilder::new()
            .qt_module("Gui")
            .files([
                "src/cxxqt_object.rs",
                "src/decode_bridge.rs",
                "src/cxxqt_object/adjustment_edit.rs",
                "src/cxxqt_object/align.rs",
                "src/cxxqt_object/annotations.rs",
                "src/cxxqt_object/channels.rs",
                "src/cxxqt_object/clipping.rs",
                "src/cxxqt_object/color_range.rs",
                "src/cxxqt_object/clipboard.rs",
                "src/cxxqt_object/crop_group.rs",
                "src/cxxqt_object/export.rs",
                "src/cxxqt_object/filter_tools.rs",
                "src/cxxqt_object/healing.rs",
                "src/cxxqt_object/image_adjust.rs",
                "src/cxxqt_object/magnetic.rs",
                "src/cxxqt_object/paint_tools.rs",
                "src/cxxqt_object/paint_tools/fills.rs",
                "src/cxxqt_object/paths.rs",
                "src/cxxqt_object/shapes.rs",
                "src/cxxqt_object/type_tools.rs",
                "src/cxxqt_object/web_export.rs",
            ])
            .cc_builder(|cc| {
                cc.include("cpp");
            })
            .build();
    }
}
