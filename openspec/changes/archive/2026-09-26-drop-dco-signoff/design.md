## Context

`CONTRIBUTING.md` required a `Signed-off-by:` trailer on every commit, and a
`dco` job in `.github/workflows/guards.yml` checked each non-merge commit in the
push or pull request range. The trailer is the Developer Certificate of Origin
1.1, a project convention that attests the contributor has the right to submit
the change. It is separate from the GPL-3.0-or-later licence and from a CLA.

The provenance attestation (the contribution is not derived from Adobe source,
decompilation, NDA material, or non-public employee knowledge) is a separate,
load-bearing rule in this project and is worth keeping.

## Goals / Non-Goals

**Goals:**

- Remove the per-commit DCO requirement and its CI enforcement.
- Keep the provenance attestation and the licence terms intact.
- Remove every now-false reference to the DCO so the contract is consistent.

**Non-Goals:**

- Introduce a CLA or any other attestation mechanism.
- Rewrite git history to strip existing trailers.

## Decisions

### D1. Delete the enforcement, keep the provenance attestation

The `dco` job is removed. `CONTRIBUTING.md` keeps the requirement that a
contribution was not derived from the listed sources, reworded as a condition of
contributing rather than something a trailer certifies. The provenance rule is
the part with substance; the trailer was only its carrier.

### D2. No replacement

No CLA is added. If corporate contributions or a firmer IP position are needed
later, that is a separate decision; reintroducing the DCO is not the fix, since it
never covered employer IP.

## Risks / Trade-offs

- [Weaker provenance trail: no per-commit dated attestation] → The rule remains in
  `CONTRIBUTING.md`; the project can revisit if it takes on outside contributors.
- [A future contributor adds `-s` out of habit] → Harmless; the trailer is simply
  no longer checked.
- [Docs still describe the DCO] → This change updates all four references.

## Migration Plan

Single PR on the current CI branch. No runtime migration. Existing commits are
untouched. Verify by confirming `guards.yml` has no `dco` job, `openspec validate
--all --strict` passes, and a push with an unsigned commit still runs the remaining
jobs.
