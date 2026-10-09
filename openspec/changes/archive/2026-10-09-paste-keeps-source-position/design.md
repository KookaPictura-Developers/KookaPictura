# Design: paste-keeps-source-position

## Context

The bridge clipboard keeps a copy's document position in `Clip::rect`, but
plain Paste ignored it and centred the clip on the canvas widget's centre; only
Paste in Place used `clip.rect`. Issue #99: a copy/paste should land where the
source pixels sat, and a plain paste should drop the marquee the way the
Into/Outside paths already do.

## Decisions

**Plain Paste adopts the clip origin.** `clipboard_paste` places the Plain and
InPlace kinds at `(clip.rect.left, clip.rect.top)`; the view centre stays only
for Paste Outside (and as the Into fallback). The stored rect was already
correct — `copy_layer` clips the selection bounds to the layer and the canvas —
so the bridge's origin computation was the part to fix, not the copy. A clip
imported from the system clipboard has no source position, so its rect is the
canvas origin (`Clip::from_rgba`), and a plain paste of it lands there, as Paste
in Place already did.

**Every paste deselects.** After a successful paste the active selection moves
into `deselected_selection` (the store Select ▸ Reselect reads), for every kind,
not just Into/Outside. A paste with no marquee leaves any earlier deselected
selection intact, matching `deselect()`.

**Paste in Place stays.** With the placement change it coincides with Paste for
app clips; it remains as the explicit command and the imported-image rule (both
land at the canvas origin). Removing it is out of scope.

## Risks / Trade-offs

- The view-centre origin is gone for a plain paste: an image imported from the
  system clipboard now lands at the canvas origin rather than centred. Its
  source position is unknown by construction (`Clip::from_rgba`), and Paste in
  Place already documented the origin rule.
- No engine change, so `tests_clipboard.rs` is untouched; the runnable check is
  the Qt Test `tst_edit_clipboard` (`cpp/tests/`), per the GUI-test migration
  guard.
