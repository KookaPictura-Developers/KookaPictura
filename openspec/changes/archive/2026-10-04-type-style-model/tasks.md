# Tasks

## 1. Character and paragraph model in `pictura-core`

- [x] 1.1 Add `AntiAlias`, `KerningMode`, `Leading`, `Justify`, `Composer`, and `Direction` enums plus `CharacterAttrs`, `ParagraphAttrs`, and `TextStyleSheet` in `crates/pictura-core/src/type_tool.rs`, deriving `Default` from the CS6 defaults; verify a unit test asserts each default (scales 100, kerning Metrics, anti-aliasing Sharp, toggles off, indents/spacing 0, hyphenation off, composer single-line, `Basic Paragraph` present)
- [x] 1.2 Give `TypeSpec` `character` and `paragraph` fields and a `TypeSpec::new` constructor, and re-express `TextStyle` over the same structs; verify a round-trip test sets every field to a non-default and reads it back equal
- [x] 1.3 Replace `TypeSpec.antialias: bool` with `AntiAlias` and migrate every in-tree Rust consumer — the six struct literals (`cxxqt_object/type_tools.rs:130`, `pictura-render/src/type_layer.rs:256/326/416`, `type_caret.rs:78`, `type_write.rs:442`) and the renderer switch in `type_layer.rs:242`; verify `cargo nextest run -p pictura-core -p pictura-render -p pictura-codec` builds and passes, with Crisp/Strong/Smooth marked `ponytail:` as approximated
- [x] 1.4 Migrate the C++ anti-aliasing sites from `bool` to the enum: `cpp/tool_context.h` (`TypeOptions`), `cpp/options_bar_type.cpp` (five-item combo), `cpp/tool_type.cpp`; verify the app builds and `cargo nextest run -p pictura-app` passes

## 2. EngineData encoder in `pictura-codec`

- [x] 2.1 Implement `encode_engine_data(&EngineValue) -> Vec<u8>` beside `parse_engine_data`; verify a parse -> encode -> parse test on a nested dict/list fixture yields an equal tree
- [x] 2.2 Round-trip the value kinds the type engine uses (UTF-16BE strings, MacRoman keys, doubles, integers, booleans) and document the lossy edges (unpaired surrogates, no MacRoman reverse table, number/key-order drift); verify a unit test covers each kind and the doc comment lists the edges
- [x] 2.3 Split the encoder into its own module if the change would push `engine_data.rs` past the `scripts/file-size-allowlist.txt` ceiling; verify the file-size gate passes

## 3. Authoring and preservation in `type_write.rs`

- [x] 3.1 Drive `style_sheet` and `paragraph_sheet` from the `TypeSpec` attributes instead of constants; verify a test authors non-default tracking, scale, leading, first-line indent, and space-after, then re-reads them
- [x] 3.2 Author the full modelled character and paragraph key set from the mapping in `design.md` (font index, toggles, colour, hyphenation, composer, etc.), with `/AutoHyphenate` defaulting to **false**; verify a per-key author/re-read test covers each modelled attribute
- [x] 3.3 Author the anti-aliasing method to the text descriptor's `AntA` enum and the EngineData `/AntiAlias` flag consistently; verify each of the five methods encodes and reads back as the same method
- [x] 3.4 Give `author_type_tool` an existing-descriptor parameter and thread the layer's current `text_desc` through `type_content` and `replace_type_layer` in `pictura-render/src/type_layer.rs`; verify a re-set of a layer with a known descriptor runs through the merge path
- [x] 3.5 Merge over the parsed tree: parse the existing EngineData, overwrite only the single run's and paragraph sheet's modelled keys, encode; verify a test that an unmodeled key present in the input is still present and valid after a re-set
- [x] 3.6 Handle the no-existing-EngineData path with the complete skeleton and the multi-run path with a documented single-run collapse; verify a test for each

## 4. Full read-side decode and read-back

- [x] 4.1 Extend the character extraction to the full model and add paragraph extraction, resolving a run's omission against the style-sheet and paragraph-sheet defaults; verify tests for an omitted tracking resolving to the sheet value and an omitted `SpaceAfter` resolving to the paragraph value
- [x] 4.2 Decode the anti-aliasing method from the descriptor `AntA`, falling back to the EngineData `/AntiAlias` flag; verify a test for each path
- [x] 4.3 Extend `pictura-render::type_layer::type_layer_spec` to read the full `TextStyle` back into a `TypeSpec` instead of font/size/colour/justification; verify a test that a read-then-authored layer keeps every modelled attribute
- [x] 4.4 Verify the modified `TypeTool exposes the font set and first-run style` scenarios: real font/size, sheet-default resolution, paragraph-default resolution, descriptor anti-aliasing, flag fallback, missing EngineData, malformed EngineData

## 5. Style sheet and hierarchy resolution

- [x] 5.1 Add `TextStyleSheet` to `Document` (defaulted to `Basic Paragraph`) and a pure `resolve` over manual override > character style > paragraph style; verify unit tests for the precedence order and a paragraph style's character attributes as the character fallback
- [x] 5.2 Implement create/edit/delete/rename/apply of styles with `Basic Paragraph` refusing rename and delete but accepting edits; verify a unit test for the refusal and the accepted edit
- [x] 5.3 Verify applying a style keeps an existing manual override on one of its attributes
- [x] 5.4 Verify editing a named style changes the resolved value for every run or paragraph applying it, and that each style create/edit/delete/apply is one undoable state

## 6. App bridge and type tools

- [x] 6.1 Keep `TypeSetting` for placement (`x, y, xx, xy, yx, yy`, `vertical`) and add `CharacterSetting` + `ParagraphSetting` cxx-qt structs (enums as `i32`, toggles as `bool`); update `type_preview_rect`, `type_caret_stops`, `type_preview_rgba`, `type_commit_layer`, `type_commit_mask`, `type_layer_setting`, `type_commit_edit`, `type_update_layer`; verify the app builds
- [x] 6.2 Wire `tool_type.cpp`, `options_bar_type.cpp`, and `image_view_type.cpp` to read and write the new settings; verify `ctest --test-dir build -R '^tst_' --output-on-failure` passes
- [x] 6.3 Verify that an attribute edit records exactly one `"Edit Type Layer"` state and one undo restores the previous value, and a cancel records none and leaves the layer bit-identical (extend `tst_type_tools` or a new suite)
- [x] 6.4 Verify a committed type layer with a non-default attribute reopens with that attribute intact

## 7. Integration and validation

- [x] 7.1 Validate the inferred `AntA` spellings for Crisp/Strong/Smooth and the `/AntiAlias` integer width against a CS6-authored PSD or a grounded fixture; verify the test asserts the confirmed mapping and self-skips when the fixture is absent, and correct the codec if it differs
- [x] 7.2 Add a PSD round-trip test that authors a type layer with modelled and unmodeled EngineData keys, writes it, and re-reads it through `psd-tools`; verify it passes where the oracle is installed and self-skips otherwise
- [x] 7.3 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `bash scripts/verify-fast.sh`; verify all pass
- [x] 7.4 Confirm no new crate or dependency was introduced and no file exceeds its size cap

## 8. Post-review remediation

Review of the implementation found genuine defects; these fixed them.

- [x] 8.1 Stop the merge clobbering unmodeled `/LeadingType` (preserve it), and add a test that `/LeadingType 2` survives a merge
- [x] 8.2 Correct the paragraph word-spacing default to CS6's `[0.8 1.0 1.33]` in the model and the read fallback
- [x] 8.3 Decode the `/AntiAlias` fallback through `as_bool` so a boolean flag yields None/Sharp; add tests
- [x] 8.4 Remove the ungroundable `KerningMode::Optical` and the dead `Direction` field; extend `Justify` to the 0–6 `/Justification` values with no silent coercion
- [x] 8.5 Add an encoder depth guard and document the programmatic-input limits
- [x] 8.6 Preserve the reopened layer's anti-aliasing method in `tool_type.cpp` (it was reset to Sharp) with a Qt test
- [x] 8.7 Fix `/Font` index/FontSet resolution, append `/FontFamily`/`/FontStyle`, and thread the PostScript name so a re-author does not duplicate or mis-target a font entry
- [x] 8.8 Carry `font_style` across the bridge instead of resetting it
- [x] 8.9 Render the character attributes (fixed leading, tracking/manual kerning, H/V scale, baseline shift, alignment) and record the paragraph-layout ceiling
- [x] 8.10 Make the cancel test assert bit-identity (pixel hash + full settings) and fix overclaiming `every modelled attribute` tests
- [x] 8.11 Make styles sparse, persist applied names + manual overrides on the layer, keep overrides on apply, re-resolve applied layers on style edit, and record manual overrides from real edits
- [x] 8.12 Add bridge create/edit/delete style operations that each record one undoable history state
- [x] 8.13 Keep the anti-aliasing `AntA` grounding test self-skipping and record that it cannot be validated without a CS6-authored fixture; do not claim parity
