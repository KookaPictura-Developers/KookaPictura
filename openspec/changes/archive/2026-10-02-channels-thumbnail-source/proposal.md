# Proposal: channels-thumbnail-source

## Why

The Channels panel builds its 24 px thumbnails from the coarsest display
pyramid level whose longest side is at least 64 px, falling back to
`view_->image()` only for single-level or tiny documents, so a panel refresh
never copies the full-resolution composite. The archived capability still says
every thumbnail comes from `view_->image()`, which the code deliberately and
correctly does not do.

## What Changes

- The `Channel thumbnails` requirement names the pyramid source and its
  full-resolution fallback, matching `channels_panel.cpp`.

## Non-Goals

- No change to the thumbnail size, channel extraction, the panel code, or the
  per-channel visibility model.
