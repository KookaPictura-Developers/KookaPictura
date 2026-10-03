# Design

## Context

See `proposal.md` for motivation. Current state that shapes the design:

- `pictura::icon(id)` builds `QIcon(path)` for `:/icons/<id>.svg`, or `.png`
  for the `app` mark. There is no recolour step; every SVG hard-codes
  `stroke="#c8c8c8"`.
- `Theme` has four brightness levels and says "No widget may hard-code a frame
  colour"; `Theme::apply` is called on demand from Preferences. So icons must
  be able to change colour at runtime.
- The 176 ids are referenced by id across the C++ app, the self-test, and Qt
  Test; keeping ids stable avoids a 176-file call-site churn and leaves
  `assets/pictura.qrc` untouched.
- Qt 6.11 `QIconEngine` exposes `paint`, `pixmap`, `scaledPixmap(size, mode,
  state, scale)`, `key`, and `clone`.

## Goals / Non-Goals

**Goals:**
- One SVG per id, tinted at render time from the palette or an explicit colour,
  crisp at any size/DPR, disabled-aware, theme-change-aware.
- Licensed, provenance-mapped asset set with a reproducible vendoring step.
- Zero call-site churn: ids and qrc unchanged.

**Non-Goals:**
- Cursors (`assets/cursors/`) — separate follow-up.
- Multi-tone/accent rendering: the set is monochrome; internal shading uses
  alpha only.
- Theming beyond the existing four foreground levels.
- 3D camera/object tool glyphs, which stay as the existing artwork.

## Decisions

### Recolour by alpha in a `QIconEngine`

A `SvgIconEngine` renders the SVG into a transparent pixmap with
`QSvgRenderer`, then fills it with the chosen colour in
`QPainter::CompositionMode_SourceIn`. `SourceIn` keeps the rendered alpha and
ignores the source RGB, so the vendored SVGs can stay verbatim
(`stroke="currentColor"`), and any alpha gradient becomes a tint-coloured
gradient — which is how Photoshop's monochrome-with-shading icons read.

- Colour selection: an explicit colour wins; otherwise `QIcon::Disabled` uses
  the palette's disabled button text, and the normal state uses the palette's
  button/window text.
- Stretch `scaledPixmap` to render at `size * scale` and set the pixmap's
  device pixel ratio; `pixmap` delegates with `scale = 1`. `paint` mirrors it
  for immediate-mode draws.
- *Alternatives considered:* a custom `QIconEnginePlugin` (unnecessary — we
  construct the engine directly), and pre-tinting at vendor time (rejected:
  cannot follow the theme at runtime, the whole point of choice B).

### Cache-bust on theme change

`QIcon` caches engine pixmaps. `SvgIconEngine::key()` includes a global
palette/theme generation counter, so a brightness change yields a new key and
a fresh render. `Theme::apply` bumps the counter and calls
`QPixmapCache::clear()`. The presence of both is belt-and-braces; the Qt Test
`lia_theme_change` scenario is the arbiter.

### Vendoring: ids stable, bodies replaced

`scripts/sync-lucide-icons.py` reads `assets/icons/lucide-map.json` and, for
each `lucide` entry, copies the pinned release's SVGs to
`assets/icons/<id>.svg`. Custom and out-of-scope entries are left untouched.
Because ids do not change, the qrc is never regenerated. The release tag and
commit hash are recorded in the map header and `docs/dev/icon-provenance.md`.

### Mapping source of truth

`assets/icons/lucide-map.json` is authoritative: `{id: {kind: lucide|custom|
note|out-of-scope, slug?, match?, note?}}`. The prose map in
`docs/dev/icon-provenance.md` and the review-only
`lucide-icon-comparison.html` are generated from it. A guard asserts the file
set and the map keys are identical, so a new icon cannot ship unmapped.

### Monochrome, alpha for shading

Lucide is single-colour; Photoshop's icons are grey with alpha gradients. We
drop the old `#3d6f99` accent rather than emulate two-tone (a string-substitution
two-pass engine would mix two visual languages for no functional gain).
Custom icons that need internal shading use `stroke-opacity` or a
white→transparent gradient. A custom icon keeps its named Lucide base at the
original size and position (no scale or crop) and draws any small added symbol
(a plus, a magnet, a scissors) at `stroke-width="1"`, since the 2px weight
swallows the detail of a small mark.

## Risks / Trade-offs

- **[Stale icon colours after a brightness change]** -> `key()` folds in the
  theme generation and `Theme::apply` clears `QPixmapCache`; `lia_theme_change`
  guards it.
- **[Custom icons drifting from the Lucide grid]** -> `lia_*` and the spec's
  attribute check enforce the 24×24/`currentColor`/width-2 contract; the sync
  script is a no-op for customs.
- **[Vendored release renamed/removed a slug]** -> the map pins a release and
  the sync script fails loudly on a missing slug; the initial set was verified
  against `lucide-static` 1.50.0 (all 174 slugs resolve).
- **[Out-of-scope tool glyphs stay the old 1.5-width artwork]** -> acceptable;
  they are disabled non-scope tools and are listed as such in the provenance
  map.

## Migration Plan

Artwork and loader land together in one change. Rollback is reverting the
commit; ids and qrc are unchanged so there is no data migration.
