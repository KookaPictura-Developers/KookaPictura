# Kooka Pictura — M0 walking skeleton (M0-A).
#
# Rust owns the QObject and the image; CMake builds the Rust crate as a static
# library through Corrosion (via CXX-Qt-CMake) and links it into the C++ shell.

cmake_minimum_required(VERSION 3.24)

project(kooka_pictura LANGUAGES CXX)

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE RelWithDebInfo CACHE STRING "Build type" FORCE)
    message(STATUS "No CMAKE_BUILD_TYPE set; defaulting to RelWithDebInfo")
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets Svg)

find_package(CxxQt QUIET)
if(NOT CxxQt_FOUND)
    include(FetchContent)
    FetchContent_Declare(
        CxxQt
        GIT_REPOSITORY https://github.com/kdab/cxx-qt-cmake.git
        GIT_TAG 0.10.0
    )
    FetchContent_MakeAvailable(CxxQt)
endif()

# cxx-qt reaches Qt through qmake. Debian/Ubuntu ship qmake6 for Qt 6; the bare
# `qmake` is Qt 5, so pin the executable explicitly.
find_program(QMAKE_EXECUTABLE NAMES qmake6 qmake)
if(NOT QMAKE_EXECUTABLE)
    message(FATAL_ERROR "qmake not found — install qt6-base-dev")
endif()

# The RHI headers live under the versioned QtGui include dir, which the plain
# Qt6::Gui target does not export on Debian (no Qt6::GuiPrivate package).
execute_process(COMMAND "${QMAKE_EXECUTABLE}" -query QT_INSTALL_HEADERS
    OUTPUT_VARIABLE QT_HEADERS_DIR OUTPUT_STRIP_TRAILING_WHITESPACE)
execute_process(COMMAND "${QMAKE_EXECUTABLE}" -query QT_VERSION
    OUTPUT_VARIABLE QT_VERSION_STR OUTPUT_STRIP_TRAILING_WHITESPACE)

cxx_qt_import_crate(
    MANIFEST_PATH crates/pictura-app/Cargo.toml
    CRATES pictura_app
    QT_MODULES Qt::Core Qt::Gui Qt::Widgets
    QMAKE "${QMAKE_EXECUTABLE}"
)

add_executable(pictura
    crates/pictura-app/cpp/main.cpp
    crates/pictura-app/cpp/decode_image.cpp
    crates/pictura-app/cpp/decode_image.h
    crates/pictura-app/cpp/selftest.cpp
    crates/pictura-app/cpp/selftest_report.cpp
    crates/pictura-app/cpp/selftest_layers_controls.cpp
    crates/pictura-app/cpp/selftest_layers_smart_object.cpp
    crates/pictura-app/cpp/selftest_layers_filter.cpp
    crates/pictura-app/cpp/selftest_tools_selection.cpp
    crates/pictura-app/cpp/selftest_layers_adjustments.cpp
    crates/pictura-app/cpp/selftest_canvas.cpp
    crates/pictura-app/cpp/selftest_numeric.cpp
    crates/pictura-app/cpp/selftest_layers_drag.cpp
    crates/pictura-app/cpp/interop.cpp
    crates/pictura-app/cpp/commands.cpp
    crates/pictura-app/cpp/command_tree.cpp
    crates/pictura-app/cpp/theme.cpp
    crates/pictura-app/cpp/session.cpp
    crates/pictura-app/cpp/preferences_dialog.cpp
    crates/pictura-app/cpp/image_view.cpp
    crates/pictura-app/cpp/icons.cpp
    crates/pictura-app/cpp/file_drop_router.cpp
    crates/pictura-app/cpp/frame.cpp
    crates/pictura-app/cpp/frame_columns.cpp
    crates/pictura-app/cpp/frame_session.cpp
    crates/pictura-app/cpp/frame_menus.cpp
    crates/pictura-app/cpp/frame_menus_select.cpp
    crates/pictura-app/cpp/frame_test.cpp
    crates/pictura-app/cpp/frame_build.cpp
    crates/pictura-app/cpp/panels/layers_panel.cpp
    crates/pictura-app/cpp/panels/layers_filter_proxy.cpp
    crates/pictura-app/cpp/panels/layers_filter_bar.cpp
    crates/pictura-app/cpp/panels/layers_panel_actions.cpp
    crates/pictura-app/cpp/panels/layers_panel_menu.cpp
    crates/pictura-app/cpp/panels/layers_panel_test.cpp
    crates/pictura-app/cpp/panels/percent_field.cpp
    crates/pictura-app/cpp/panels/numeric_field.cpp
    crates/pictura-app/cpp/panels/history_panel.cpp
    crates/pictura-app/cpp/panels/navigator_panel.cpp
    crates/pictura-app/cpp/panels/color_panel.cpp
    crates/pictura-app/cpp/panels/swatches_panel.cpp
    crates/pictura-app/cpp/panels/info_panel.cpp
    crates/pictura-app/cpp/panels/histogram_panel.cpp
    crates/pictura-app/cpp/panels/placeholder_panel.cpp
    crates/pictura-app/cpp/panels/panel_group.cpp
    crates/pictura-app/cpp/panels/panel_group_menu.cpp
    crates/pictura-app/cpp/panels/panel_group_test.cpp
    crates/pictura-app/cpp/panels/panel_column.cpp
    crates/pictura-app/cpp/panels/panel_float.cpp
    crates/pictura-app/cpp/panels/panel_column_iconic.cpp
    crates/pictura-app/cpp/panels/panel_column_menu.cpp
    crates/pictura-app/cpp/panels/panel_column_drag.cpp
    crates/pictura-app/cpp/panels/panel_column_test.cpp
    crates/pictura-app/cpp/tools.cpp
    crates/pictura-app/cpp/tools_marquee.cpp
    crates/pictura-app/cpp/tools_selection_move.cpp
    crates/pictura-app/cpp/toolbox.cpp
    crates/pictura-app/cpp/options_bar.cpp
    crates/pictura-app/cpp/new_document_dialog.cpp
    crates/pictura-app/cpp/layer_new_dialog.cpp
    crates/pictura-app/cpp/dialogs.cpp
    crates/pictura-app/cpp/commands.h
    crates/pictura-app/cpp/theme.h
    crates/pictura-app/cpp/session.h
    crates/pictura-app/cpp/preferences_dialog.h
    crates/pictura-app/cpp/image_view.h
    crates/pictura-app/cpp/icons.h
    crates/pictura-app/cpp/file_drop_router.h
    crates/pictura-app/cpp/frame.h
    crates/pictura-app/cpp/selftest_layers_controls.h
    crates/pictura-app/cpp/selftest_layers_smart_object.h
    crates/pictura-app/cpp/selftest_layers_filter.h
    crates/pictura-app/cpp/selftest_tools_selection.h
    crates/pictura-app/cpp/selftest_layers_adjustments.h
    crates/pictura-app/cpp/selftest_canvas.h
    crates/pictura-app/cpp/selftest_numeric.h
    crates/pictura-app/cpp/selftest_layers_drag.h
    crates/pictura-app/cpp/panels/layers_panel.h
    crates/pictura-app/cpp/panels/layers_filter_proxy.h
    crates/pictura-app/cpp/panels/layers_filter_bar.h
    crates/pictura-app/cpp/panels/percent_field.h
    crates/pictura-app/cpp/panels/numeric_field.h
    crates/pictura-app/cpp/panels/jump_slider.h
    crates/pictura-app/cpp/panels/history_panel.h
    crates/pictura-app/cpp/panels/navigator_panel.h
    crates/pictura-app/cpp/panels/color_panel.h
    crates/pictura-app/cpp/panels/swatches_panel.h
    crates/pictura-app/cpp/panels/info_panel.h
    crates/pictura-app/cpp/panels/histogram_panel.h
    crates/pictura-app/cpp/panels/placeholder_panel.h
    crates/pictura-app/cpp/panels/panel_group.h
    crates/pictura-app/cpp/panels/panel_column.h
    crates/pictura-app/cpp/tools.h
    crates/pictura-app/cpp/toolbox.h
    crates/pictura-app/cpp/options_bar.h
    crates/pictura-app/cpp/new_document_dialog.h
    crates/pictura-app/cpp/layer_new_dialog.h
    crates/pictura-app/cpp/dialogs.h
    assets/pictura.qrc
)
target_include_directories(pictura PRIVATE
    crates/pictura-app/cpp
    "${QT_HEADERS_DIR}/QtGui/${QT_VERSION_STR}/QtGui"
)
target_link_libraries(pictura PRIVATE pictura_app Qt6::Svg)
