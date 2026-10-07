# Spec Delta

## Purpose

Defines how the repository produces versioned releases: a release-please Release
PR on `master` that bumps versions, maintains the changelog, and creates the
tag that triggers packaging.

## ADDED Requirements

### Requirement: Release PR is maintained on master

The repository SHALL run release-please on pushes to `master`, and it SHALL
open and keep up to date a single Release PR that, when merged, creates the
version tag, the GitHub Release, and the `CHANGELOG.md` update for everything
merged since the previous release.

#### Scenario: A releasable commit reaches master

- **WHEN** a Conventional Commit of type `feat` is merged to `master`
- **THEN** release-please opens or updates a Release PR proposing a minor
  version bump and lists that commit in the proposed changelog entry

#### Scenario: No releasable change reaches master

- **WHEN** only commits with non-releasable types (for example `ci` or
  `chore` without a breaking marker) are merged to `master`
- **THEN** release-please does not open a Release PR

### Requirement: The first release is v0.1.0

The version manifest SHALL be bootstrapped at `0.0.0` and the release-please
configuration SHALL set `initial-version` to `0.1.0`, so the first merged Release
PR MUST create tag `v0.1.0` with a `0.1.0` `CHANGELOG.md` section.

#### Scenario: The initial Release PR is merged

- **WHEN** the first Release PR is merged with the manifest at `0.0.0`
- **THEN** the manifest reads `0.1.0`, the tag created is `v0.1.0`, and
  `CHANGELOG.md` is created with a `0.1.0` section

#### Scenario: The configured initial version is used

- **WHEN** release-please bootstraps from a `0.0.0` manifest
- **THEN** the proposed first version equals the `initial-version` in
  `release-please-config.json`, not a value derived from the commit types

### Requirement: Version files stay in sync

Every release SHALL update all tracked version files in one Release PR: the
release strategy's `version.txt`, the Cargo workspace package version
(`[workspace.package].version` in `Cargo.toml`), and the CMake `project()`
version. No release SHALL be created that leaves them disagreeing.

#### Scenario: A Release PR is prepared

- **WHEN** release-please prepares a Release PR proposing version `X`
- **THEN** the PR's diff sets `version.txt` to `X`, `version = "X"` in
  `Cargo.toml`, and the CMake version field to `X` in the same commit

#### Scenario: Version drift is impossible in a release commit

- **WHEN** any tagged release commit is checked out
- **THEN** `version.txt`, `Cargo.toml`, and `CMakeLists.txt` report the same
  version as the tag (modulo the leading `v`)

### Requirement: The tag is the packaging trigger

The GitHub Release created by merging the Release PR SHALL be the documented
trigger point for packaging workflows (AppImage, deb, snap, flatpak, DMG,
Windows). Packaging automation MUST key off the release event or tag, not off
merges to `master`.

#### Scenario: A release is cut

- **WHEN** the Release PR for version `X` is merged
- **THEN** tag `vX` and a GitHub Release exist, and the release notes derive
  from the `CHANGELOG.md` section for `X`

#### Scenario: A packaging workflow listens for releases

- **WHEN** a workflow triggered by release/tag events runs
- **THEN** it can resolve the version to build from the release tag alone

### Requirement: Release automation documentation exists

`DEVELOPING.md` SHALL document the release flow: how the Release PR is
produced, that versions and the changelog are updated by merging it, how to
force an exact version, the repository and organization setting that lets the
default `GITHUB_TOKEN` open the Release PR, and that release-triggered workflows
do not run for events created by the default `GITHUB_TOKEN` (so packaging that
depends on them requires a PAT).

#### Scenario: A maintainer prepares a release

- **WHEN** a maintainer reads the release section of `DEVELOPING.md`
- **THEN** they find the Release PR workflow, the version-bump semantics
  (Conventional Commits → semver), the exact-version escape hatch, and the
  `GITHUB_TOKEN` setting/PAT caveats for opening the PR and triggering
  packaging
