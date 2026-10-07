# Tasks

## 1. Filter job

- [x] 1.1 Add a `changes` job to `.github/workflows/ci.yml` that runs `dorny/paths-filter` pinned to a full commit SHA (with `pull-requests: read` permission) and exposes the changed file list; verify the workflow still parses (`python3 -c "import yaml, sys; yaml.safe_load(open('.github/workflows/ci.yml'))"`) and the commit message states the new-dependency justification
- [x] 1.2 Add a shell step that computes `cpp_only` (true only when the file list is non-empty and every path is under `crates/pictura-app/cpp/`); verify by running the snippet locally against four fixture lists — cpp-only, single `.rs` file, mixed cpp+`.rs`, `.github/workflows/ci.yml` — and observing `cpp_only=true,false,false,false`

## 2. Job gates

- [x] 2.1 Gate `lint`, `test`, and `oracles` with a single-line `if: needs.changes.outputs.cpp_only != 'true'` and leave `qt-headless` ungated; verify by grep that exactly three single-line `if:` gates exist, none spans multiple lines, and `qt-headless` has no gate
- [x] 2.2 Confirm scope: `git diff --stat` shows `.github/workflows/ci.yml` is the only workflow file changed (the `paths-ignore` block and `guards.yml` untouched); verify `git diff .github/workflows/guards.yml` is empty and the `paths-ignore` lines still match the spec'd docs set

## 3. Acceptance scenarios

- [ ] 3.1 Open a PR touching only `crates/pictura-app/cpp/` (spec scenario "C++-only change"); verify `qt-headless` and `guards` run while `lint`, `test`, `oracles` report skipped-success
- [ ] 3.2 Open a PR touching a `.rs` file (scenarios "Rust change runs everything" / "Mixed change runs everything"); verify all four build jobs run
- [ ] 3.3 Open a PR touching `.github/workflows/ci.yml` or a root-level config path (scenarios "Workflow edit re-runs the suite" / "Unrecognized path runs everything"); verify all four build jobs run despite no Rust/C++ diff
- [ ] 3.4 Push a docs-only change to a branch with an open PR (existing scenario "Docs-only change"); verify the CI workflow does not start and the guards workflow still runs

## 4. Spec and repo gates

- [ ] 4.1 Run `openspec validate --all --strict`; verify it exits 0 with the `ci-conditional-jobs` delta present
- [ ] 4.2 Run `bash scripts/guard.sh`; verify it passes, and commit the change referencing #199
