#!/usr/bin/env bash
#
# verify-full.sh - full verification: verify-fast.sh plus the CMake/Qt build and
# the app self-tests. Slow; run verify-fast.sh during the edit loop.

set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/verify-fast.sh

echo "== cmake =="
[ -d build ] || cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld
cmake --build build --parallel
echo "cmake_build=ok"

echo "== self-tests =="
./build/pictura --headless --self-test
./build/pictura --headless --self-test crates/pictura-codec/tests/fixtures/two_layers.psd
echo "self_tests=ok"

echo "verify-full: OK"
