# Refactor — code splitting (LOC guardrail)

- **Status:** planned. Track 0 (the guardrail) lands with this brief; Tracks 1–6
  are not implemented.
- **Type:** pure-mechanical refactor program. Moves only; no behavior change, no
  spec deltas, no new dependency.
- **Contract:** the moves keep every public symbol, signature, registration, and
  self-test exit code identical. `scripts/check-file-size.sh` is the guard; the
  full gate list in §5 is the proof.
- **Non-goals:** renaming, re-designing interfaces, changing any algorithm,
  golden baseline, `openspec/specs/` requirement, or on-disk/PSD format, and any
  commit that mixes a split with a behavior fix.

## 1. Goal

Bring the codebase under an **800-LOC preference / 1000-LOC hard cap** by pure
mechanical moves along existing class/concern seams. When a file approaches the
cap it is split; nothing else about it changes.

## 2. Why

Sixteen files exceed the 1000-LOC hard cap today:

| File | LOC |
|---|---:|
| `crates/pictura-app/cpp/main.cpp` | 7320 |
| `crates/pictura-app/src/cxxqt_object.rs` | 5217 |
| `crates/pictura-app/cpp/panels/panel_column.cpp` | 2828 |
| `crates/pictura-filters/src/artistic/filters.rs` | 2322 |
| `crates/pictura-app/cpp/frame.cpp` | 2245 |
| `crates/pictura-adjust/src/lib.rs` | 1825 |
| `crates/pictura-render/src/lib.rs` | 1793 |
| `crates/pictura-render/src/gpu.rs` | 1722 |
| `crates/pictura-codec/src/lib.rs` | 1646 |
| `crates/pictura-filters/tests/oracle.rs` | 1540 |
| `crates/pictura-app/cpp/panels/layers_panel.cpp` | 1511 |
| `crates/pictura-render/src/document_ops/layer_ops.rs` | 1315 |
| `crates/pictura-render/src/gpu_filter.rs` | 1284 |
| `crates/pictura-filters/src/lib.rs` | 1121 |
| `crates/pictura-app/cpp/panels/panel_group.cpp` | 1095 |
| `crates/pictura-render/tests/gpu_parity.rs` | 1016 |

These sixteen are recorded in `scripts/file-size-allowlist.txt` at their current
LOC. The list shrinks as files are split and never grows: an entry is a ceiling,
and `check-file-size.sh` fails if a listed file exceeds its recorded count.

## 3. Contract

- **Moves only.** Cut/paste declarations and definitions; keep names, ordering
  of registration, and behavior. Where a helper is translation-unit-private,
  move it to a `*_internal.h` shared by the split `.cpp` files rather than
  duplicating it.
- **No renames.** A split commit touches no public name.
- **One commit per split.** Each step is independently green and revertible.
- **Proof per step** is §5: build, both headless self-tests with the same exit
  codes, the Rust suite, the spec validation, and the size guard.

## 4. Tracks

### Track 0 — guardrail (this document)

`scripts/check-file-size.sh`, `scripts/file-size-allowlist.txt`, the fast-gate
invocation, the `AGENTS.md` rule, and this brief. No source file changes.

### Track 1 — C++ app splits

`panels/panel_group.cpp` (1095), `panels/layers_panel.cpp` (1511),
`panels/panel_column.cpp` (2828), and `frame.cpp` (2245) split into
concern-based sibling `.cpp` files plus a `*_internal.h` for the anon-namespace
helpers the siblings share. New translation units are added to the `add_executable`
source list in `CMakeLists.txt`; headers stay where they are unless a
translation-unit split requires otherwise.

### Track 2 — `main.cpp` (7320)

The whole `--self-test` block moves into `selftest.cpp` / `selftest.h` as **one
function**, gated on byte-identical stderr and the same exit codes. Subdividing
that function follows only after the single extraction is proven green, one
self-test group at a time.

### Track 3 — `cxxqt_object.rs` (5217)

Convert the file to a directory module: bridge `mod.rs`, `helpers.rs`,
`tests.rs`. Then spike an impl split within the bridge to see whether the
cxx-qt macro tolerates it; if it does not, the directory module is the ceiling
and is recorded as such.

### Track 4 — Rust engine crates

Split `pictura-render` (`gpu.rs` 1722, `gpu_filter.rs` 1284,
`document_ops/layer_ops.rs` 1315, `lib.rs` 1793), `pictura-adjust` (`lib.rs`
1825), `pictura-codec` (`lib.rs` 1646), and `pictura-filters`
(`artistic/filters.rs` 2322, `lib.rs` 1121) into submodule directories. `mod`
re-exports preserve every existing path, so call sites do not move.

### Track 5 — integration-test files

`pictura-filters/tests/oracle.rs` (1540) and
`pictura-render/tests/gpu_parity.rs` (1016) split into helper modules under the
test directory, keeping the test function set and asserting count identical.

### Track 6 — close-out

Empty `scripts/file-size-allowlist.txt` once every file is under the cap, and
record the final layout in `docs/dev/STATE.md`.

## 5. Per-step gate

Run after every split commit, from the repo root:

```bash
cmake --build build
./build/pictura --headless --self-test
./build/pictura --headless --self-test crates/pictura-codec/tests/fixtures/two_layers.psd
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
openspec validate --all --strict
bash scripts/check-file-size.sh
```

A step is done when the build succeeds, both self-tests exit 0 with the same
codes, the 589 Rust tests pass, the 60 specs validate, and the guard passes.

## 6. Deferred cleanup

M45 left `PanelGroup::detachPanel` / `attachPanel` / `detached_` and a dead
`panelFlyoutHeader` QSS selector behind. Removing them is an optional, separate
dead-code commit, not part of any split above; it must not be folded into a
Track 1 move.
