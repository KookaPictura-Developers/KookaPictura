#pragma once

#include "rust/cxx.h"

#include <cstdint>

// Render `text` with the system `family` font to tightly packed RGBA8888 of
// `width` × `height`, straight alpha. `justify` is 0 left / 1 right / 2 center.
// Returns an empty buffer for a non-positive size.
::rust::Vec<uint8_t> render_text_rgba(::rust::Str family, double pixel_size, ::rust::Str text,
                                      int32_t justify, uint8_t r, uint8_t g, uint8_t b, uint8_t a,
                                      int32_t width, int32_t height);
