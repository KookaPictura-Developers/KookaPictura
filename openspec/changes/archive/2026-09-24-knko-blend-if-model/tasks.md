# Tasks: knko-blend-if-model

## 1. Model

- [x] 1.1 Add `pictura_core::Knockout { None, Shallow, Deep }` with `from_byte` / `to_byte` (0/1/2; other → `None`).
- [x] 1.2 Add `pictura_core::BlendIf` with `composite_source: (u16,u16)`, `composite_dest: (u16,u16)`, `channel_ranges: Vec<((u16,u16),(u16,u16))>` — 8-byte composite (source + dest pairs) then 8-byte channel groups, matching design.md and psd-tools.
- [x] 1.3 Add `Layer` fields: `knockout: Knockout`, `blend_clipping: bool`, `blend_interior: bool`, `blend_if: Option<BlendIf>`; update `Default` (`None`, `true`, `true`, `None`).

## 2. Codec read

- [x] 2.1 In `read_layer_record`, consume `b"knko"` / `b"clbl"` / `b"infx"` into the new fields (empty payload → default; do not push to `extra_blocks`).
- [x] 2.2 After `blending_ranges` is filled, parse into `blend_if` (empty or malformed → view `None`, raw kept; well-formed non-empty → ranges).

## 3. Codec write

- [x] 3.1 In `write_extra`, emit `knko` only when `knockout != None`, `clbl` only when `blend_clipping == false`, `infx` only when `blend_interior == false`; payload `[value,0,0,0]`.
- [x] 3.2 Add `encode_blend_if(&BlendIf) -> Vec<u8>` and use it when an edited `blend_if` is written back (replace `blending_ranges`); unmodified path still writes the raw field.

## 4. Tests and gates

- [x] 4.1 Hand-built `tagged_layer_psd` with `knko=1`/`2`, `clbl=0`, `infx=0` decodes to the typed fields.
- [x] 4.2 Open→save preserves non-default `knko`/`clbl`/`infx` bytes and omits them at defaults (existing fixtures unchanged).
- [x] 4.3 Non-empty `blending_ranges` decodes composite + channel ranges; empty → defaults; truncated → view `None`, document still reads.
- [x] 4.4 `encode_blend_if` → parse round-trips ranges.
- [x] 4.5 `cargo nextest run -p pictura-core -p pictura-codec`, `cargo fmt --all --check`, `cargo clippy -p pictura-core -p pictura-codec --all-targets -- -D warnings`, `openspec validate knko-blend-if-model --strict`.
