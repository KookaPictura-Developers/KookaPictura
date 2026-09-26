# ADR: Provenance and specification/implementation separation

- **Status:** Accepted
- **Date:** 2026-09-26
- **Relates to:** `00-overview/licensing-and-provenance.md` (`OVR-004`).

## Context

The project reimplements the Adobe Photoshop CS6 feature set. The strongest
copyright posture is documented, independent creation: behaviour established
from public sources and lawful black-box observation, the specification written
without protected expression, and the implementation written from the
specification without Adobe source or decompilation.

A formal two-team independent-creation process with isolated specification and
implementation groups is hard to demonstrate for a small, AI-assisted project,
and overclaiming it would be worse than not claiming it.

## Decision

1. The project states its provenance **accurately**: documentation-first,
   independent creation from public sources; **not** a certified two-team
   independent-creation process.
2. It commits to the substantive rules: no Adobe source, no decompilation, no
   NDA/beta knowledge, no protected expression in specs.
3. Contributors attest to these rules when contributing
   (`CONTRIBUTING.md`).
4. Where practical, behaviour is specified (`docs/`) before it is implemented,
   and each doc cites only public sources in its `## Sources`.

## Consequences

- Claims stay truthful and auditable; `## Sources` sections and
  `assets/PROVENANCE.md` are the evidence trail.
- Independent creation is a defence against **copyright** only; patents and
  trade secrets are handled separately (`OVR-004`; patent FTO review).
- Contributors who cannot make the attestation must not contribute.
