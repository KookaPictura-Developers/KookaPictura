# Proposal: drop-dco-signoff

## Why

The repository required a Developer Certificate of Origin `Signed-off-by:` trailer
on every commit and enforced it with a CI `dco` job. The DCO is a project policy,
not a GPL-3.0 condition, and it is the weakest form of contributor attestation:
it does not bind employer IP the way a CLA does, and the "signature" is an
unsigned git trailer. For a project whose contributors are not yet corporate, the
per-commit requirement is friction without the legal coverage it appears to
provide, so it is removed.

## What Changes

- **Remove the `dco` CI job** from `.github/workflows/guards.yml` and its
  `fetch-depth: 0` checkout.
- **Drop the sign-off requirement** from `CONTRIBUTING.md`. The licence terms and
  the provenance attestation stay; the attestation is no longer tied to a
  `Signed-off-by:` trailer and becomes a condition of contributing.
- **Update the pointers** in `AGENTS.md` and `DEVELOPING.md`, and the four
  `docs/` references that name the DCO (this PR carries `TASK-ALLOWS-DOCS`).
- **No replacement mechanism.** No CLA is introduced.
- **BREAKING**: none for the code or the build.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `verification-harness`: the "DCO sign-off checked on push and pull request"
  requirement is removed.

## Impact

- `.github/workflows/guards.yml`: the `dco` job is deleted; the `guards` job is
  unchanged.
- `CONTRIBUTING.md`, `AGENTS.md`, `DEVELOPING.md`: sign-off wording removed.
- `docs/00-overview/licensing-and-provenance.md`,
  `docs/11-cross-cutting/adr-project-license.md`,
  `docs/11-cross-cutting/adr-provenance-separation.md`,
  `docs/dev/legal-hold-and-counsel-handoff.md`: DCO references retargeted.
- Existing commits keep their `Signed-off-by:` trailers; history is not rewritten.
- No runtime dependency, no code, no PSD change.
