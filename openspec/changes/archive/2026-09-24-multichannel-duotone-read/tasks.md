# Tasks: multichannel-duotone-read

## 1. Read path

- [x] 1.1 `color_mode_from_code`: accept `MODE_MULTICHANNEL` and `MODE_DUOTONE`.
- [x] 1.2 Multichannel channel-count gate: after header, if Multichannel and `channels ∉ {1, 3}` → `Unsupported`; Multichannel depth must be 8.
- [x] 1.3 Duotone: `retain_planes` for depth 8; normalize like grayscale (`gray_to_rgb`); `source_mode = Duotone`; keep `color_mode_data`.
- [x] 1.4 Multichannel: split_planes / composite use `header_channels` as color count; N=1 → `gray_to_rgb`; N=3 → `cmy_to_rgb` (`255-x`); `source_mode = Multichannel`; retain plates.
- [x] 1.5 `convert_pixels` / `convert_layer_color_channels` handle Multichannel/Duotone (layer Multichannel stays a no-op because `color_channels()==0` — ceiling documented in design D3).

## 2. Write path

- [x] 2.1 Write mode code 7 or 8 when `source_mode` matches and retained plates are unchanged (flat document); else working RGB.
- [x] 2.2 Re-emit `color_mode_data` as today (Duotone spec preserved).

## 3. Tests and gates

- [x] 3.1 Unit: Duotone open → RGB + `source_mode` + `color_mode_data` preserved.
- [x] 3.2 Unit: 1-ch and 3-ch Multichannel open; other N → Unsupported.
- [x] 3.3 Unit: unchanged write-back mode 7/8; edited → RGB.
- [x] 3.4 Update `unsupported_depths_and_color_modes_are_rejected` (drop modes 7/8 from the always-fail list; keep bad depth cases).
- [x] 3.5 `cargo nextest run -p pictura-codec`, fmt, clippy, `openspec validate multichannel-duotone-read --strict`.
