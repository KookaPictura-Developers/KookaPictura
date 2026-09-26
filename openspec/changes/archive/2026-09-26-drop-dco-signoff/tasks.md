## 1. Remove enforcement

- [x] 1.1 Delete the `dco` job from `.github/workflows/guards.yml` (and the `fetch-depth: 0` it needed)

## 2. Update contributing docs

- [x] 2.1 Rewrite the sign-off section of `CONTRIBUTING.md`: keep the licence and provenance attestation, drop the DCO requirement
- [x] 2.2 Remove the sign-off convention from `AGENTS.md` and the DCO pointer in `DEVELOPING.md`
- [x] 2.3 Retarget the DCO references in `docs/00-overview/licensing-and-provenance.md`, `docs/11-cross-cutting/adr-project-license.md`, `docs/11-cross-cutting/adr-provenance-separation.md`, and `docs/dev/legal-hold-and-counsel-handoff.md` (commit with `TASK-ALLOWS-DOCS`)

## 3. Verify

- [x] 3.1 `openspec validate --all --strict` passes
- [ ] 3.2 The guards workflow still runs, with no `dco` job, and CI stays green
