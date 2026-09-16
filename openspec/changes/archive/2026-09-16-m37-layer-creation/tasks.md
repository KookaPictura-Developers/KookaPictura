## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Revise `docs/dev/layers-panel-program.md`: insert **M37 — layer
  creation and grouping** and shift the remaining stages (panel anatomy → M38,
  filtering/search → M39, remaining management ops → M40, styles/effects → M41,
  smart objects/vector masks/artboards/comps → M42), with the rationale that
  creation is the most basic panel action and was missing
- [x] 1.2 Update `docs/dev/STATE.md`'s program section (M36 done, M37 next) and
  the milestone numbering
- [x] 1.3 Write `proposal.md`, `design.md`, this `tasks.md`, and the
  `specs/layers-panel/spec.md` delta
- [x] 1.4 Freeze in `design.md`: the five op signatures and `next_layer_name`;
  the insertion-index rule (`above + 1`, out-of-range/negative → top); the
  empty-layer representation (`0/1/2/-1` zero channels, Normal/255/255/visible,
  inert composite) and the empty-rect ceiling; the naming rule; the
  composite-then-record undo convention and labels; the bridge signatures; the
  panel buttons and `LayersPanel::currentLayer()`; the menu ids and handlers

## 2. pictura-render: `document_ops::layer_ops`

- [x] 2.1 Add `crates/pictura-render/src/document_ops/layer_ops.rs` with
  `add_layer`, `add_group`, `duplicate_layer`, `group_layer`, `ungroup_layer`,
  and `next_layer_name`, plus the `// ponytail:` empty-rect note
- [x] 2.2 Re-export the ops from `document_ops/mod.rs` and the crate root
- [x] 2.3 Unit tests: insertion index/order (including no-selection top); the
  transparent layer's channels and metadata; deep duplicate preserving children,
  mask, adjustment and attributes with `"<name> copy"`; group wraps in place;
  ungroup splices in order and refuses a non-group; `next_layer_name` highest
  suffix; a transparent layer leaves `composite_rgba(doc)` unchanged
- [x] 2.4 `cargo test -p pictura-render`; `cargo fmt`/`clippy` clean

## 3. Bridge

- [x] 3.1 Add the five `#[qinvokable]` declarations and implementations with
  the frozen signatures
- [x] 3.2 Each successful op calls `recomposite()` then `record(<label>)` with
  `New Layer` / `New Group` / `Duplicate Layer` / `Group Layers` /
  `Ungroup Layers`; a failed op records nothing
- [x] 3.3 `cargo check -p pictura_app`

## 4. Panel and menu

- [x] 4.1 Freeze the five ids in `crates/pictura-app/cpp/commands.h` and make
  the five `Layer` leaves implemented commands in `command_tree.cpp` (no other
  id renamed)
- [x] 4.2 Add `LayersPanel::currentLayer()`/`selectLayer(int)` and the New
  Group / New Layer buttons before Delete; wire them to the bridge and select
  the returned index
- [x] 4.3 Register the five `Layer` handlers in `frame.cpp` against the active
  view's current layer, with the `// ponytail:` M38 multi-selection note
- [x] 4.4 `cmake --build build`

## 5. Self-test

- [x] 5.1 Add the `m37_create` block to `main.cpp`: count growth, unchanged
  composite on layer creation, a `" copy"` duplicate name, wrap/unwrap, five
  history steps, and undo; print
  `pictura self-test: m37_create new=1 group=1 duplicate=1 ungroup=1 undo=1`
  and fail with exit codes 90–94
- [x] 5.2 `xvfb-run -a ./build/pictura --self-test` and the `two_layers.psd`
  variant

## 6. Verification and evidence

- [x] 6.1 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo test --workspace`
- [x] 6.2 `cmake -S . -B build && cmake --build build`
- [x] 6.3 `xvfb-run -a ./build/pictura --self-test` (no argument and
  `crates/pictura-codec/tests/fixtures/two_layers.psd`)
- [x] 6.4 `openspec validate m37-layer-creation --strict`;
  `openspec validate --all --strict`

## 7. Close-out

- [x] 7.1 Record the M37 result in `docs/dev/STATE.md`
- [ ] 7.2 Archive the change (`openspec archive m37-layer-creation`) and commit
  — deferred; this task explicitly does not commit
