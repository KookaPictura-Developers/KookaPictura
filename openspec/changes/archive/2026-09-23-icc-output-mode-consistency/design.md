## Context

`read_psd` normalizes a non-RGB mode to RGB and, for 16/32-bit CMYK/Lab, records
`source_mode`/`source_depth` but retains no planes. The embedded `1039` profile is
handled by `apply_icc` (Convert) and `apply_passthrough_policy` (Preserve/Off):
both leave the document byte-unchanged when the profile cannot build a transform
for the now-RGB pixels (`icc.rs:44-49`, `:88-94`). `write_psd` then picks the
output header mode (`write.rs:958-979`) and re-emits `doc.image_resources`
verbatim (`write.rs:1025-1026`). So a 16/32-bit CMYK source writes RGB bytes with
its CMYK profile still attached.

## Goals / Non-Goals

- **Goal:** the emitted `1039` always describes the emitted header color mode.
- **Non-goal:** re-deciding the read-time profile policy. `Preserve` still leaves
  an un-transformable profile in the in-memory document; the fix is at the single
  writer choke point, so it covers every read path and caller.
- **Non-goal:** finding a matching profile when one is absent — an untagged save
  stays untagged.

## Decisions

### Decide by the ICC data-space signature, not by lcms2

The four-byte data-space signature is at ICC header bytes `16..20`. Reading it
directly needs no profile parse and matches the spaces the header mode can have:

| Output mode | Expected data space |
|---|---|
| RGB | `RGB ` |
| Grayscale | `GRAY` |
| CMYK | `CMYK` |
| Lab | `Lab ` |

A profile shorter than 20 bytes carries no readable signature and is left
untouched, so byte preservation wins when the space cannot be proven mismatched.

### Filter at the writer, not the reader

An earlier candidate dropped the stale profile in `apply_icc` /
`apply_passthrough_policy`. That would change the deliberate `Preserve` contract
("leave the document byte-unchanged") and its test
(`policy_preserve_rejects_a_profile_the_pixels_cannot_transform`). The writer
guard leaves read semantics intact and fixes the save for every path that reaches
it, including a document constructed by an engine caller.

### Byte-identical when nothing mismatches

The helper returns `doc.image_resources` unchanged (a clone) unless a `1039`
mismatches, so `icc_profile.psd` (RGB profile on RGB) and every profile-free file
write byte-identically. Only the mismatched block is removed; the parsed tail the
decoder did not consume is re-appended, reusing the same decode/`encode`/tail
pattern `rebuild_resources` uses.

## Risks / Trade-offs

- A `Preserve`-mode RGB document carrying a stray CMYK profile now saves without
  that profile. That is the point: the emitted profile no longer matches the
  pixels, so keeping it is the bug. The in-memory `document_icc` display path is
  unchanged.
- A deliberately non-standard ICC data-space tag that matches no mode is dropped
  for an RGB/Gray/CMYK/Lab output. Acceptable: it described no real space.
