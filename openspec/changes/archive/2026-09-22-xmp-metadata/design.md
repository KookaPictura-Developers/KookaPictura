## Context

`metadata.rs` exposes XMP as a raw lossy string and deliberately does not parse
it (no XML dependency). Resource bytes round-trip verbatim through
`image_resources`, and `set_iptc_fields` established the pattern for an in-place
metadata edit: decode the section, refuse unless it re-encodes losslessly,
reframe one resource, preserve every other block's `raw`, and reassign
`Document.image_resources`.

The XMP packet is RDF/XML. `docs/01-architecture/file-formats.md` (ARCH-011)
names `quick-xml` as a candidate but no XML crate is a dependency, and AGENTS
rule 4 requires a stated reason for any new one. The unknown-namespace
preservation `WF-010` demands is only achievable through an edit if the writer
does not reserialise the whole document, so a span-patching writer is both the
no-dependency option and the more faithful one.

The six editable IPTC-Core fields already map one-to-one onto XMP properties:
Object Name→`dc:title`, By-line→`dc:creator`, Copyright Notice→`dc:rights`,
Caption/Abstract→`dc:description`, Credit→`photoshop:Credit`,
Source→`photoshop:Source`.

## Goals / Non-Goals

**Goals:**
- Extract the fixed XMP property set into typed values for display.
- Edit those properties by patching the packet in place, preserving every
  unmanaged byte (unknown namespaces, unknown properties, wrapper, comments).
- Never corrupt a packet: anything not recognised is a no-op.
- Keep XMP and IIM consistent for the shared six fields.
- One undo state per File Info accept; save round-trips both channels.

**Non-Goals:**
- Editing the raw packet text, EXIF editing (camera data is read-only per
  `WF-010`), or general RDF modelling (IPTC Extension, nested structures).
- Metadata templates and sidecars (separate changes).
- Resolving XML entities or fetching external DTDs (XXE stays disabled).
- A new XML/XMP dependency.

## Decisions

- **Property set is fixed and small.** The model holds the nine properties
  above; anything else in the packet is preserved but not modelled. This keeps
  the parser bounded and the patch surface reviewable.
- **Patch by byte spans, not reserialisation.** The parser locates each element
  or attribute that carries a managed property and records its byte range in the
  original packet; an edit replaces that range with a freshly serialised
  fragment. Unmanaged bytes are copied through untouched, which is the only way
  unknown namespaces survive an edit.
- **Recognise both XMP forms.** Simple properties may be attributes
  (`photoshop:Credit="…"`) or elements; `dc:title`/`dc:description`/`dc:rights`
  are `rdf:Alt` lang alternatives and `dc:creator`/`dc:subject` are
  `rdf:Seq`/`rdf:Bag`. The reader handles all of these; the writer emits the
  standard element form (`rdf:Alt` with `xml:lang="x-default"`, `rdf:Seq`/Bag
  with `rdf:li`) and the attribute form for the simple `photoshop:` properties
  when they were attributes.
- **Fail closed.** A packet that does not expose a recognised
  `rdf:Description`, or that uses a construct the patcher cannot safely rewrite,
  yields no change (the function returns false and the section is untouched),
  mirroring `set_iptc_fields`'s lossless-decode guard.
- **Create a minimal packet when absent.** Setting an XMP property on a document
  with no resource 1060 frames a minimal well-formed packet rather than silently
  doing nothing.
- **One codec entry point for the dialog.** `set_file_info_fields` writes the
  shared six fields to both XMP and IIM; the app bridge calls it and records one
  history state, so the two channels cannot diverge.

## Risks / Trade-offs

- **A hand-rolled tokenizer is the risk.** Mitigations: it never resolves
  entities, is bounded (no recursion, capped element/attribute counts and data
  size), fails closed on anything unexpected, and carries fuzz-style unit tests
  for malformed input. The parse and patch are unit-tested independently of the
  exiftool oracle, which CI does not install.
- **Only the fixed set is editable.** A user cannot edit an arbitrary XMP
  property or the raw packet this cycle; the raw packet stays visible read-only.
- **Namespace prefixes are taken as written.** The writer reuses the prefix that
  appears in the packet for a property it edits, and emits a declared prefix for
  one it inserts; it does not rewrite prefixes or namespace declarations.
- **Duplicate properties:** if an unmanaged block precedes a 1060, two packets
  can exist in theory (shared with the ICC path); the first is edited and the
  ceiling is noted, matching the existing resource behaviour.

## Migration Plan

Additive: a new module and new functions; `read_metadata`'s raw XMP string is
unchanged, so existing tests and behaviour are unaffected. The dialog change is
additive (a new category; the existing IPTC page keeps working and now syncs).
No data migration.

## Open Questions

- The exact IIM↔XMP synced subset is a `WF-010` open question; this change syncs
  the six fields the dialog already edits and leaves the rest to a later policy.
