# Design: edit-clipboard-interop

## Context

The bridge clipboard (`Clip`) keeps a copy's document position and selection
coverage; a system-clipboard image carries only pixels. photorust uses the
system clipboard as the only store and remembers the copy origin in the shell,
dropping it when `QClipboard::dataChanged` reports another owner.

## Decisions

**Two stores, one mirror.** The bridge `Clip` stays authoritative for our own
copies, so position and coverage survive. After every successful Copy / Cut /
Copy Merged the shell writes `masked_rgba` to the system clipboard as a
`Format_RGBA8888` image and marks the two as mirrored. A `dataChanged` that did
not come from our own write clears the mark. A paste imports the system image
(`Clip::from_rgba`, origin `(0, 0)`) only when the mark is clear and the system
clipboard holds an image; otherwise the bridge copy is pasted.

**Paste in Place.** Places the clip at `clip.rect`'s top-left: the source
position for our own copy, the canvas origin for an imported image (photorust's
rule). History label "Paste", as for a plain paste.

**Shortcut.** CS6 and photorust bind `Shift+Ctrl+V` to Paste in Place and
`Alt+Shift+Ctrl+V` to Paste Into, but `docs/02-ui-ux/menus.md` gives
`Shift+Ctrl+V` to Paste Into and does not list Paste in Place. The documented
binding is kept, and Paste in Place ships without a default shortcut.

**Purge.** Drops the bridge copy, and clears the system clipboard only while it
still holds our export.

## Risks / Trade-offs

- `ponytail:` a platform that emits our own write's `dataChanged`
  asynchronously clears the mirror; the next paste then re-imports our own
  export, so pixels survive but Paste in Place lands at the origin.
- The OS clipboard carries no selection coverage beyond alpha, and no
  "Export Clipboard on quit" preference is implemented.
