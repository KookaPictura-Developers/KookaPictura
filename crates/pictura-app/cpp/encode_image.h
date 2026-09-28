#pragma once

#include "rust/cxx.h"

#include <cstdint>

// Encode tightly packed RGBA8888 of `width` × `height` to `path` with the named
// Qt image writer (`PNG`, `JPG`, `TIF`, `WEBP`, `BMP`). `scale` is a percentage
// (100 = unchanged); `quality` is 0–100 for lossy formats and ignored otherwise.
// Formats without alpha (JPEG) are flattened onto opaque white. Returns false
// and writes nothing when the buffer size, path, format, or writer is unusable.
// Defined at global scope because the cxx bridge declares no namespace.
bool encode_image_rgba(::rust::Slice<const uint8_t> rgba, int32_t width, int32_t height,
                       ::rust::Str path, ::rust::Str format, int32_t quality, int32_t scale);
