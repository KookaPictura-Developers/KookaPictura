# Tasks: adopt-release-please

## 1. Configuration

- [x] 1.1 `release-please-config.json` (`simple`, `initial-version: 0.1.0`, generic `extra-files` for `Cargo.toml` + `CMakeLists.txt`).
- [x] 1.2 `.release-please-manifest.json` bootstrapped at `".": "0.0.0"`.
- [x] 1.3 `version.txt` seeded at `0.0.0`.
- [x] 1.4 Version annotations: `Cargo.toml` `[workspace.package] version`; `CMakeLists.txt` `project(... VERSION ...)`.
- [x] 1.5 Enable "Allow GitHub Actions to create and approve pull requests" at the organization and repository, so `GITHUB_TOKEN` can open the Release PR.

## 2. Workflow

- [x] 2.1 `.github/workflows/release-please.yml` on `master`, `contents: write` / `issues: write` / `pull-requests: write`, `RELEASE_PLEASE_TOKEN || GITHUB_TOKEN`.

## 3. Documentation

- [x] 3.1 `DEVELOPING.md` release section: Release PR lifecycle, semver from Conventional Commits, `release-as` escape hatch, token caveat.

## 4. Verification

- [x] 4.1 `openspec validate --all --strict`.
- [x] 4.2 `bash scripts/guard.sh` and JSON/YAML parse of the new config.
