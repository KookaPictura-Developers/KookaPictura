# Refactor — code splitting (LOC guardrail)

- **Status:** done. Tracks 0–5 landed as the commits in §5; Track 6 closed with
  one deliberate exception (`selftest.cpp`).
- **Type:** pure-mechanical refactor program. Moves only; no behavior change, no
  spec deltas, no new dependency.
- **Contract:** the moves keep every public symbol, signature, registration, and
  self-test exit code identical. `scripts/check-file-size.sh` is the guard; the
  full gate list in §6 is the proof.
- **Non-goals:** renaming, re-designing interfaces, changing any algorithm,
  golden baseline, `openspec/specs/` requirement, or on-disk/PSD format, and any
  commit that mixes a split with a behavior fix.

## 1. Goal

Bring the codebase under an **800-LOC preference / 1000-LOC hard cap** by pure
mechanical moves along existing class/concern seams. When a file approaches the
cap it is split; nothing else about it changes.

## 2. Why

At kick-off, sixteen files exceeded the 1000-LOC hard cap:

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

Those sixteen were recorded at kick-off in `scripts/file-size-allowlist.txt` at
their then-current LOC. The list only shrank as files were split and never grew:
an entry is a ceiling, and `check-file-size.sh` fails if a listed file exceeds
its recorded count.

## 3. Contract

- **Moves only.** Cut/paste declarations and definitions; keep names, ordering
  of registration, and behavior. Where a helper is translation-unit-private,
  move it to a `*_internal.h` shared by the split `.cpp` files rather than
  duplicating it.
- **No renames.** A split commit touches no public name.
- **One commit per split.** Each step is independently green and revertible.
- **Proof per step** is §6: build, both headless self-tests with the same exit
  codes, the Rust suite, the spec validation, and the size guard.

## 4. Tracks

### Track 0 — guardrail (done)

`scripts/check-file-size.sh`, `scripts/file-size-allowlist.txt`, the fast-gate
invocation, the `AGENTS.md` rule, and this brief. No source file changes.

### Track 1 — C++ app splits (done)

`panels/panel_group.cpp` (1095), `panels/layers_panel.cpp` (1511),
`panels/panel_column.cpp` (2828), and `frame.cpp` (2245) split into
concern-based sibling `.cpp` files plus a `*_internal.h` for the anon-namespace
helpers the siblings share. New translation units are added to the `add_executable`
source list in `CMakeLists.txt`; headers stay where they are unless a
translation-unit split requires otherwise. Final layouts in §5.

### Track 2 — `main.cpp` (done)

The whole `--self-test` block moved into `selftest.cpp` / `selftest.h` as **one
function**, gated on byte-identical stderr and the same exit codes. Subdividing
that function follows only after the single extraction is proven green, one
self-test group at a time — the deliberate later step recorded in §5.

### Track 3 — `cxxqt_object.rs` (done)

The bridge keeps the cxx-qt macro root at `src/cxxqt_object.rs` and moves the
concern bodies under `src/cxxqt_object/`; the root + directory layout preserved
the generated-header path and the 14 C++ includes. Final layout in §5.

### Track 4 — Rust engine crates (done)

`pictura-render` (`gpu.rs` 1722, `gpu_filter.rs` 1284,
`document_ops/layer_ops.rs` 1315, `lib.rs` 1793), `pictura-adjust` (`lib.rs`
1825), `pictura-codec` (`lib.rs` 1646), and `pictura-filters`
(`artistic/filters.rs` 2322, `lib.rs` 1121) were split into submodule
directories. `mod` re-exports preserve every existing path, so call sites did not
move. Final layouts in §5.

### Track 5 — integration-test files (done)

`pictura-filters/tests/oracle.rs` (1540) and
`pictura-render/tests/gpu_parity.rs` (1016) split into helper modules under the
test directory, keeping the test function set and asserting count identical.

### Track 6 — close-out (done, one exception)

Record the final layout in `docs/dev/STATE.md`. The allowlist did not empty:
`crates/pictura-app/cpp/selftest.cpp` is the single remaining entry, kept whole
to protect the self-test oracle; subdividing it by section is a deliberate later
step.

## 5. Outcome

All tracks landed as pure moves, each commit independently green (build + both
headless self-tests with byte-identical stderr + the Rust suite + `openspec
validate --all --strict` + the size guard).

| Track | Commit(s) | Split |
|---|---|---|
| 0 — guardrail | `57d9bf0` | `scripts/check-file-size.sh`, `scripts/file-size-allowlist.txt`, the `scripts/verify-fast.sh` invocation, the `AGENTS.md` <800/1000 rule, this brief. |
| 1 — C++ app | `f706370`, `7f6661b`, `a3b2faa`, `23f1d08` | `panel_group.cpp` → `panel_group{,_menu,_test}.cpp`; `layers_panel.cpp` → `layers_panel{,_actions,_menu,_test}.cpp` + `layers_panel_internal.h`; `panel_column.cpp` → `panel_column{,_drag,_iconic,_menu,_test}.cpp` + `panel_float.cpp` + `panel_column_internal.h`; `frame.cpp` → `frame{,_columns,_session,_menus,_build,_test}.cpp`. New TUs registered in `CMakeLists.txt`. |
| 2 — self-test | `cc745cc` | The `--self-test` block moved out of `main.cpp` into `selftest.cpp` / `selftest.h` (`int runSelfTest(QApplication&, bool, const QString&, PicturaMainWindow&, PictureView*, const QImage&, bool, int)`); `main.cpp` 7320 → 180 LOC; self-test stderr byte-identical. |
| 3 — cxx-qt bridge | `2baecaf` | `cxxqt_object.rs` (5217) → Rust-2018 root `src/cxxqt_object.rs` + `src/cxxqt_object/{impl_core,impl_layers,impl_selection,impl_transform,impl_paint,impl_history,impl_filters,helpers,helpers_composite,tests,tests_impl}.rs`. The root + directory layout keeps the generated-header path and the 14 C++ includes untouched. |
| 4 — engine crates | `c129640` | `pictura-filters`, `pictura-render`, `pictura-adjust`, `pictura-codec` — layouts below. |
| 5 — integration tests | `aca8792` | `pictura-filters/tests/oracle.rs` → `tests/oracle/`; `pictura-render/tests/gpu_parity.rs` → `tests/gpu_parity/`. |

### Final module layouts

- **C++ app** (`crates/pictura-app/cpp/`): the Track 1 siblings above, plus
  `selftest.{h,cpp}` for the extracted harness.
- **cxx-qt bridge** (`crates/pictura-app/src/`): `cxxqt_object.rs` (root — the
  cxx-qt bridge and the shared private helpers) and the `cxxqt_object/`
  submodules listed in Track 3.
- **`pictura-filters`**: `artistic/filters.rs` →
  `artistic/filters/{effects,brush,common,tests}.rs` behind a `filters.rs`
  facade; `lib.rs` → `filter.rs`.
- **`pictura-render`**: `lib.rs` → `composite.rs` + `tests/`; `gpu.rs` →
  `gpu/{mod,backend,shader}.rs`; `gpu_filter.rs` →
  `gpu_filter/{mod,plan,resources}.rs`; `document_ops/layer_ops.rs` →
  `document_ops/layer_ops/{mod,create,paths,properties,tests}.rs`.
- **`pictura-adjust`**: `lib.rs` → `types.rs` / `common.rs` / `apply.rs` /
  `tonal.rs` / `color.rs` / `auto.rs` / `tests.rs`.
- **`pictura-codec`**: `lib.rs` → `error.rs` / `common.rs` / `read.rs` /
  `write.rs` / `tests.rs`.
- **Integration tests**: `tests/oracle/` and `tests/gpu_parity/` (Track 5).

### Remaining allowlist

`scripts/file-size-allowlist.txt` now has **one** entry —
`crates/pictura-app/cpp/selftest.cpp` (7212 LOC) — down from sixteen. Every other
source file is under the 1000-LOC hard cap. The self-test was extracted whole
first (Track 2) so the verification oracle stayed intact; subdividing it by
section is a deliberate later step, kept out of this program.

## 6. Per-step gate

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

## 7. Deferred cleanup

M45 left `PanelGroup::detachPanel` / `attachPanel` / `detached_` and a dead
`panelFlyoutHeader` QSS selector behind. Removing them is an optional, separate
dead-code commit, not part of any split above; it must not be folded into a
Track 1 move.
