# channels-panel Specification

## Purpose

The thumbnail source the Channels panel actually uses: the display pyramid's
coarse level rather than the full-resolution composite.

## MODIFIED Requirements

### Requirement: Channel thumbnails

The `ChannelsPanel` SHALL build a grayscale thumbnail for each row from the
coarsest display-pyramid level whose longest side is at least 64 px, falling
back to `view_->image()` when no such level exists (a single-level or tiny
document): the composite row SHALL use luminance (`qGray`), and the colour rows
SHALL use `qRed`, `qGreen`, and `qBlue` respectively, each scaled to a small
pixmap. With no document, the thumbnails SHALL be empty.

#### Scenario: Every row has a thumbnail [cp_thumbnail_present]

- **WHEN** an RGB document is open and the panel is refreshed
- **THEN** every row's thumbnail pixmap is non-null
