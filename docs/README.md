# Kooka Pictura — Photoshop CS6 parity specification

Documentation-only specification for remaking **Adobe Photoshop CS6 (v13, 2012)**
as a native Linux application using **Rust** (core) and **Qt6** (UI), targeting
behavioral parity with CS6.

> This repository contains **no code, no build files, and no dependencies.**
> It is a specification corpus. Every `## Rust module mapping` / `## Qt6
> component mapping` section is a design proposal, not an implementation.

## Scope

- **Parity target:** Photoshop CS6, Standard + Extended editions.
- **Platform:** Linux (native), X11 + Wayland.
- **Stack:** Rust core, Qt6 UI (Widgets and/or QML), GPU compositing.
- **Out of scope for parity:** OS-specific integration with no Linux equivalent
  (e.g. some print paths), 3D and video where the cost is prohibitive, and Adobe's
  proprietary cloud services. Each exclusion is documented with rationale rather
  than silently dropped.

## How to read this

1. [`00-overview/`](00-overview/) — what we are building and why, feasibility, non-goals, legal.
2. [`01-architecture/`](01-architecture/) — the constraints that shape everything else. Read first.
3. [`02-ui-ux/`](02-ui-ux/) — application frame, panels, preferences, shortcuts.
4. `03-`…`10-` — the feature domains (tools, image ops, layers, filters, color, selection, automation, workflow).
5. [`11-cross-cutting/`](11-cross-cutting/) — concerns that cut across domains.
6. [`TRACEABILITY.md`](TRACEABILITY.md) — CS6 feature → spec → Rust module → Qt component.
7. [`SPEC_TEMPLATE.md`](SPEC_TEMPLATE.md) — the canonical structure every spec follows.
8. [`GLOSSARY.md`](GLOSSARY.md) — terms and CS6-specific vocabulary.

## Status legend

| Status | Meaning |
|---|---|
| Stub | Heading exists, content not yet written. |
| Draft | Written from sources but not cross-reviewed. |
| Spec'd | Complete, sourced, cross-reviewed against the template. |
| Verified | Sources re-checked; parity criteria executable in principle. |

## Source policy

- Primary: Adobe's official **Photoshop CS6 Help reference PDF**
  (`help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf`), Adobe
  file-format and SDK documentation.
- Secondary: reputable books/tutorials, standards (ICC, TIFF, PNG, JPEG), and
  the Qt6 / Rust ecosystem docs for the technology mapping.
- Unverified claims live under `## Open questions`, never asserted as fact.
- `helpx.adobe.com` returns HTTP 403; use the archived PDF or `support` mirrors.

## Index

See [`INDEX.md`](INDEX.md) for the full file-by-file listing and current status.

## Legal

Adobe, Photoshop, and related marks are trademarks of Adobe Inc. This is a
documentation-first behavioral specification produced without Adobe source code, binaries,
or assets. See [`00-overview/licensing-and-provenance.md`](00-overview/licensing-and-provenance.md).
