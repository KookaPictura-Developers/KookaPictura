# Legal hold and counsel hand-off

- **Status:** working note. **Not legal advice.**
- **Posture:** no actual or anticipated dispute at the time of writing. If that
  changes, **stop further history rewriting / deletions** and treat the copies
  below as under legal hold.

## Where the history and copies live

| Artefact | Location | Note |
|---|---|---|
| Clean (scrubbed) repository | `github.com/KookaPictura-Developers/KookaPictura` (private) | Rewritten history: no Adobe-authored blobs, no old brand/fixture-derivation terms. |
| Pre-scrub repository | a private remote (`legacy`) | Retained, **not** deleted; still holds the original history. Keep for now. |
| Pre-scrub mirror | `/tmp/opencode/kooka-pictura-prerewrite.git` | Offline backup of the pre-rewrite history. |
| Pre-history-scrub mirror | `/tmp/opencode/kooka-pictura-prehistory-scrub.git` | Offline backup taken before the verbatim-expression history rewrite. |
| Legacy working copy | a local pre-rename working copy | Lightweight source + full original `.git`. |

History rewriting removed the encumbered blobs from the new repo, but **does not
erase copies elsewhere** (this list, any clone/fork, CI caches). If a dispute is
possible, do not delete these; they are the record.

## Evidence pack for counsel

1. [`00-overview/licensing-and-provenance.md`](../00-overview/licensing-and-provenance.md)
   (`OVR-004`) — legal posture, provenance method, open questions.
2. [`assets/PROVENANCE.md`](../../assets/PROVENANCE.md) — per-asset origin/license.
3. [`patent-freedom-to-operate-notes.md`](patent-freedom-to-operate-notes.md) —
   algorithm inventory and high-risk families for an FTO review.
4. [`trademark-clearance-brief.md`](trademark-clearance-brief.md) — mark audit,
   nominative-use classification, clearance request.
5. [`verbatim-expression-audit.md`](verbatim-expression-audit.md) (+ findings) —
   Help-prose reproduction and remediation plan.
6. [`../11-cross-cutting/adr-project-license.md`](../11-cross-cutting/adr-project-license.md)
   and [`adr-provenance-separation.md`](../11-cross-cutting/adr-provenance-separation.md).
7. [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — provenance attestation.
8. `LICENSE`, `LICENSES/`, `THIRD-PARTY-LICENSES`, `deny.toml` — distribution
   license and dependency notices.

## Consolidated requests for counsel

1. **Trademark** — clear/register "Kooka Pictura" and the org name (classes
   9/42); confirm the nominative-use posture for "Photoshop CS6"; decide the
   feature-label policy (may we keep coined names such as Smart Object / Refine
   Edge / Puppet Warp?).
2. **Patent** — freedom-to-operate review scoped to the families in the FTO
   notes (resampling, distortion/warp, lighting/flare, content-aware, matting,
   blind deconvolution, raw pipeline).
3. **Copyright** — fair-use position on short attributed Help excerpts; whether
   all Help-derived text must be paraphrased (recommended); the status of the
   rolling/CC-contaminated CS6 Help PDF as a source.
4. **Process** — is the provenance attestation adequate, or is a formal CLA /
   contributor separation evidence required?

## Housekeeping

- Do not rewrite history further without counsel's sign-off.
- Keep `legacy` and the mirror until counsel says otherwise.
