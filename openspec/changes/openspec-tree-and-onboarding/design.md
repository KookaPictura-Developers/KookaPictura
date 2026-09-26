# Design

## Context

See `proposal.md` — Why. Current state that shapes the approach:

- `openspec/specs/` is flat: 110 capability folders. The CLI is **1.13.2**
  locally; CI pins **1.3.1** (`.github/workflows/guards.yml:19`). Nested
  discovery requires ≥1.7.0, so the pin must move for the tree to be readable.
- Under 1.13.2, `openspec validate --all --strict` reports **0 passed / 110
  failed** because every main spec carries the archive-generated Purpose
  placeholder. Non-strict validation is green. `scripts/verify-fast.sh` uses
  `set -euo pipefail`, so both its `--strict` steps currently fail.
- `openspec update` has already regenerated `.opencode/skills/openspec-*` and
  `.opencode/commands/opsx-*` to 1.13.2 (uncommitted, plus the new
  `openspec-sync-specs` / `opsx-sync`). The regenerated tooling already speaks
  `<capability-path>` (`user-auth` or `identity/user-auth`).
- 1.13.2's `config.yaml` schema is `schema`, `context` (a single string),
  `rules` (map of artifact id → string[]), and `operations` guidance. Valid
  rule artifact ids are `proposal`, `specs`, `design`, `tasks`.
- Collateral flat-path references live in hand-written files only:
  `docs/GLOSSARY.md` (99), root `AGENTS.md`, `DEVELOPING.md`,
  `docs/dev/STATE.md`, `.serena/memories/`. No script globs `specs/*/`.

## Goals / Non-Goals

**Goals:**

- A two-level `specs/{domain}/{capability}/` tree whose ids are the relative path.
- Strict validation green on the pinned 1.13.2, locally and in CI.
- The taxonomy and delta-path rule discoverable at propose time.
- `DEVELOPING.md` reproduces the OpenSpec + Serena + agent toolchain.

**Non-Goals:**

- Three-level domains (deferred until a domain outgrows two levels).
- Editing spec requirements; only Purpose overviews and paths change.
- Migrating archived change deltas (historical; archive never re-reads them).
- Pinning a single shared version constant across files; the pin is documented
  in two places that must agree.

## Decisions

### D1. Domains are functional areas, not a crate mirror

The ten domains are call-graph/area seams, not the eight engine crates:
`document`, `codec`, `color`, `compositing`, `imaging`, `tools`, `ui`,
`interop`, `verification`, and `meta` (repository and development-process
contracts). `compositing` spans `pictura-render`, `imaging`
spans `pictura-adjust` + `pictura-filters`, `tools` spans `pictura-paint` +
`pictura-select`. A crate-mirror was rejected because the crates do not match
the concepts a reader groups by. The full mapping (110 capabilities, verified
against the corpus with no drift):

| domain | n | capabilities |
|---|---|---|
| `document/` | 13 | document-model, document-lifecycle, document-tabs, workspace-persistence, edit-history, command-registry, free-transform, document-resize, image-resize, document-orientation, image-orientation, canvas-operations, canvas-scrollbars |
| `codec/` | 16 | psd-codec, psd-layer-io, psd-bit-depth, psd-color-modes, psd-image-resources, psd-file-info, psd-iptc-write, psd-xmp-metadata, psd-icc-conversion, psd-advanced-blending, psd-opaque-preservation, psd-smart-objects, psd-smart-filters, psd-type-tool, pictura-raw, metadata-templates |
| `color/` | 6 | color-management, color-profile-assignment, color-settings, bit-depth-sample-model, hdr-conversion, hdr-toning |
| `compositing/` | 18 | layer-compositing, blend-modes, composite-view, knockout-compositing, gpu-compositing, gpu-compute-backend, gpu-filter-acceleration, layer-management, layer-locks, layer-effects, layers-filtering-search, smart-object-layer-actions, smart-object-rendering, native-depth-composite, native-depth-masks, native-depth-layer-content, native-depth-save, native-depth-edit-preserve |
| `imaging/` | 20 | adjustment-layers, adjustment-layer-rendering, adjustment-ui, image-adjustments, native-depth-adjustments, filter-application, filter-app-ui, render-filters, artistic-filters, blur-filters, brush-stroke-filters, distort-filters, noise-filters, pixelate-filters, sharpen-filters, sketch-filters, stylize-filters, texture-filters, other-filters, oil-paint-filter |
| `tools/` | 19 | tool-framework, tool-hint-bar, paint-engine, brush-tools, canvas-tools, selection-model, selection-tools, selection-channels, selection-content-move, selection-masked-edits, shape-selection-tools, select-menu, text-rasterize-bundled, text-render-seam, text-shaping, text-subpixel-positioning, type-engine-data, type-layer-kind, type-live-composite |
| `ui/` | 13 | application-shell, document-canvas, panel-rail, panel-column, layers-panel, history-panel, navigator-panel, info-histogram-panel, color-swatches-panel, image-ops-app-ui, icon-assets, svg-cursors, numeric-fields |
| `interop/` | 3 | agentic-control, file-drop-routing, image-import |
| `verification/` | 2 | test-reporting, verification-harness |

The table sums to **110**, the corpus as it stands, and every name is a pure
move. This change also introduces a tenth domain, `meta/`, for
repository-level contracts: `meta/openspec-layout` and `meta/agent-onboarding`
are the two new capabilities it adds, so they join the corpus only when the
change is archived and are deliberately absent from the mapping above.

### D2. The move is `git mv` only; ids derive from the path

Capability ids become the relative path (`compositing/layer-compositing`).
1.13.2 derives the id from the directory, so no spec text changes for the move.
H1 headings (`# layer-compositing Specification`) are left as-is; updating them
to the path id is cosmetic and out of scope.

### D3. Changes stay flat; only their `specs/` subtree nests

`changes/<name>/` is always a direct child of `changes/` (1.13.2 rejects a
namespace level). The delta inside a change mirrors the main path:
`changes/<name>/specs/{domain}/{capability}/spec.md`. `specs-apply.js` walks
deltas recursively and merges to the same relative path.

### D4. Purposes are edited directly in the main specs, not via deltas

The 110 placeholders are overview text, not requirements. The validator's own
guidance is to edit the main spec directly; a delta `## Purpose` is read only
when a capability is created. So the Purpose work is direct edits to
`openspec/specs/**/spec.md`, and needs no spec delta.

### D5. Strict-green is a prerequisite of the move, folded into this change

Per the user's direction, the pin bump, the Purpose edits, the tree move, and
the onboarding docs ship in one branch/change. The ordering inside the change
still matters: land Purposes + pin first (strict goes green), then move, so a
bisect can tell "content fix" from "pure move". Alternatives considered: a
separate predecessor change (cleaner bisect, more overhead) and relaxing the
gate to non-strict (rejected — weakens the contract and hides 110 warnings).

### D6. Taxonomy encoded as config `context` + `rules`

`openspec/config.yaml` is `schema: spec-driven` plus a `context` string naming
the ten domains and a `rules` map whose `proposal` and `specs` entries state
the `{domain}/{capability}` path and the mirrored delta path. 1.13.2 supports no
structured domain list, so `context` is one freeform string. Encoding the domain
in the capability name (`codec-psd-rle`) was rejected: zero churn but no visible
grouping.

### D7. Archived deltas are not migrated

Archived changes are history; `validate --all` ignores them and archive never
re-applies them. Moving them would be churn with no reader.

## Risks / Trade-offs

- **110 Purposes are real content, not a mechanical rename.** Mitigation: one
  or two sentences each, derived from the capability's existing requirements;
  review the batch. A weak-but-real Purpose still clears `--strict`.
- **BREAKING id change.** Anything keying on a flat id breaks. Mitigation: the
  only in-repo consumers are docs and memories (grepped and rewritten); no
  script or code reads spec ids.
- **Two version sources (CI pin + `DEVELOPING.md`) can drift.** Mitigation: a
  scenario in `agent-onboarding` requires they match; add them in the same commit.
- **`openspec update` output depends on each machine's global profile** (this
  one: `delivery: both`, workflows propose/explore/apply/archive). Mitigation:
  commit the generated `.opencode/` files so the repo, not the machine, is the
  source of truth; review the diff before committing.
- **`docs/GLOSSARY.md` is under `docs/`.** Mitigation: its commit carries
  `TASK-ALLOWS-DOCS` (or is committed with the other docs changes).

## Migration Plan

1. Fill the 110 Purposes in place; bump the CI pin to 1.13.2. Verify
   `openspec validate --all --strict` exits zero.
2. Add the taxonomy to `openspec/config.yaml`; track the file.
3. `git mv` the 110 folders into the tree (D1). Verify `openspec list --specs`
   shows nested ids and `openspec show compositing/layer-compositing` resolves.
4. Rewrite the flat paths in `docs/GLOSSARY.md`, `AGENTS.md`, `DEVELOPING.md`,
   `docs/dev/STATE.md`, `.serena/memories/`; record the new version/layout.
5. Add the `DEVELOPING.md` onboarding sections.
6. Commit the regenerated `.opencode/` tooling (including `sync-specs`).

Rollback: steps 2–6 are `git revert` of a pure-move commit plus doc edits; step
1 is independent content that can stay.
