# Kooka Pictura — M0 walking skeleton (M0-A).
#
# Rust owns the QObject and the image; CMake builds the Rust crate as a static
# library through Corrosion (via CXX-Qt-CMake) and links it into the C++ shell.

cmake_minimum_required(VERSION 3.24)

project(kooka_pictura LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets)

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

cxx_qt_import_crate(
    MANIFEST_PATH crates/pictura-app/Cargo.toml
    CRATES pictura_app
    QT_MODULES Qt::Core Qt::Gui Qt::Widgets
    QMAKE "${QMAKE_EXECUTABLE}"
)

add_executable(pictura crates/pictura-app/cpp/main.cpp)
target_link_libraries(pictura PRIVATE pictura_app)
