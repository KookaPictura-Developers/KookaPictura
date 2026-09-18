# Agent guide — Kooka Pictura

Documentation-first project reimplementing **Adobe Photoshop CS6** (v13) in
**Rust + Qt6** on Linux. Specifications live in `docs/`; code lives in `crates/`.

## Layout

```
docs/            specification corpus (source of truth for behavior)
openspec/        OpenSpec change proposals + capability specs
crates/
  pictura-core/     document/pixel types
  pictura-codec/     image codecs (PSD/PSB first)
  pictura-testkit/   verification harness (golden images, hashing)
  pictura-app/       Qt shell / cxx-qt bridge
```

## Commands

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo nextest run --workspace          # preferred; cargo test --workspace is the fallback
cargo test --workspace --doc           # doctests (nextest does not run them)
bash scripts/test-report.sh            # unified report: nextest + doctests + app self-test
bash scripts/verify-fast.sh            # fmt, clippy, test-report, file-size, guard, openspec
bash scripts/verify-full.sh            # CMake app build first, then verify-fast (runs self-test)
cargo deny check                       # licenses/advisories/bans
cmake -S . -B build -G Ninja -DCMAKE_EXE_LINKER_FLAGS=-fuse-ld=lld && cmake --build build --parallel
./build/pictura --headless --self-test         # headless Qt app self-test (also: with a .psd argument)
openspec validate --all --strict       # validate change proposals + specs
```

Toolchain is pinned by `rust-toolchain.toml` (1.98). Qt is the system Qt 6
(`qmake6 -query QT_VERSION`). Cargo links with `lld` (`.cargo/config.toml`),
`[profile.test]` compiles dependencies at `opt-level = 0`, and CMake builds with
Ninja and `--parallel`.

## Testing

- Suite: **595 tests, 8 ignored** (`cargo nextest run --workspace` preferred;
  `cargo test --workspace` is the fallback). Doctests run separately with
  `cargo test --workspace --doc` — nextest does not run them. nextest writes a
  JUnit report to `target/nextest/default/junit.xml`.
- External oracle tests (ImageMagick / `psd-tools`) self-skip when the tool is
  absent; CI has a dedicated `oracles` job that installs both so they run for
  real.
- `scripts/test-report.sh [auto|always|never]` runs nextest, doctests, and both
  app self-test invocations, then prints one unified report via
  `scripts/report_tests.py` (python3 stdlib; `python3 scripts/report_tests.py
  --self-check` is the parser's runnable check).
- Local gates: `scripts/verify-fast.sh` (fmt, clippy, test-report, file-size,
  guard, openspec) and `scripts/verify-full.sh` (adds the CMake app build so the
  headless self-test runs).

## Headless mode

`./build/pictura --headless` selects the offscreen QPA plugin before
`QApplication` and implies `--self-test` when no document is given, so it never
blocks. The self-test asserts the platform is `offscreen`. Explicit
`QT_QPA_PLATFORM=offscreen` and `xvfb-run` remain valid. `--interop-probe`
requires a real platform Vulkan instance and is **not** offscreen-compatible.

## Spec workflow (OpenSpec)

This project uses [OpenSpec](https://github.com/Fission-AI/OpenSpec) to turn work
into reviewable change proposals before code. `docs/` remains the long-form
contract; OpenSpec carries the per-change *requirements* and their task list.

- A change lives in `openspec/changes/<kebab-name>/`:
  `proposal.md` (why / what / capabilities), `design.md` (how),
  `specs/<capability>/spec.md` (ADDED / MODIFIED / REMOVED requirement deltas),
  `tasks.md` (implementation checklist).
- Capabilities are kebab-case names; each becomes `openspec/specs/<capability>/spec.md`
  once the change is archived. Prefer new capability names over `MODIFIED` unless the
  requirement itself changes.
- Commands: `/opsx-explore`, `/opsx-propose`, `/opsx-apply`, `/opsx-archive`
  (skills in `.opencode/skills/openspec-*`). Artifacts are generated from
  `openspec instructions <artifact> --change <name> --json`; validate before
  committing with `openspec validate --all --strict`.
- Format is strict: `### Requirement:` then `#### Scenario:` (exactly four `#`),
  normative SHALL/MUST wording, at least one scenario per requirement.
- Archive a completed change with `openspec archive <name>` to merge its deltas
  into `openspec/specs/`.
- The M0–M5 work was built before this workflow was adopted; it is documented
  retroactively as the `openspec/changes/m0-*` … `m5-*` proposals.

## Rules

1. **Specs are the contract.** Do not change files under `docs/` unless the task
   explicitly says so. Read the matching spec before implementing a behavior.
2. **Definition of done.** Non-trivial logic (a branch, loop, parser, or
   money/security path) ships with one runnable check: a unit test, a `demo()`
   self-check, or an integration test. Trivial one-liners need none.
3. **No unrequested abstractions.** No factory for one product, no trait with one
   implementation, no config for a constant. Delete over add.
4. **No new dependencies** without stating why in the commit/task result. Prefer
   std and already-present crates.
5. **Behavioral parity only where the oracle exists.** Adobe's closed algorithms
   are approximated and marked as such; never claim verified parity without a
   test that proves it.
6. **Determinism.** Golden comparisons run on CPU; GPU is an accelerator, not an
   oracle. Fix seeds for anything random.
7. **No comments explaining what the code says.** Comment only non-obvious
   intent. Mark deliberate shortcuts with a `ponytail:` comment naming the
   ceiling.
8. **Escalate, don't guess.** If a spec is ambiguous or a dependency/toolchain
   decision is needed, stop and report the blocker rather than inventing scope.
9. **File size.** Target under **800 LOC**, hard cap **1000**, enforced by
   `scripts/check-file-size.sh`. When a file approaches the cap, split it along
   class/concern seams with pure moves (no behavior change): C++ classes may
   span several `.cpp` translation units, Rust modules become submodule
   directories.

## Anti-patterns

- Weakening or deleting a test to make CI pass. Changing a golden baseline
  requires an explicit note in the task result.
- Committing build artifacts (`target/`, `build/`, `.opencode/node_modules/`).
- Touching unrelated files. Keep diffs scoped to the task.
