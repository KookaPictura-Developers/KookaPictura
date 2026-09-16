## Context

The M19 asset pass gave the app 40 icons (8 `tool.*`) and 8 `tool.*` cursors.
M23 rebuilt the Tools panel as a two-column icon grid of the 10 `ToolId` values.
`icons.cpp` hardcodes the cursor hotspot (`eyedropper ? (2,22) : (12,12)`), the
rail renders two-letter text glyphs (`frame.cpp`'s `addRailPanel` with the
`// ponytail:` note), and the Layers/History action buttons are text-only. The
canonical `tool-framework` spec still says "two-column grid"; `svg-cursors` says
only the eyedropper is off-centre.

`docs/02-ui-ux/toolbox-and-options-bar.md` is the authoritative CS6 tool-group
table (group order, shortcuts, flyout behaviour, the unlettered Blur slot, the
Extended-only 3D and Count tools). `docs/dev/m38-icon-cursor-library.md` is the
frozen contract this design implements: the 71-row catalogue, the style guides,
the hotspot rules, the panel inventory, and the verification plan.

Constraints: docs/proposal only in this change; the Rust bridge, document model,
compositor, and PSD I/O are untouched; no new dependency; the 10 existing tool
asset ids and the 10 `ToolId` values are frozen; Qt only.

## Goals / Non-Goals

**Goals:**

- Freeze the 71-tool catalogue (id, label, shortcut, group/slot, implemented,
  `Qt::CursorShape`, hotspot) and the 23-slot order, once, in `tools.{h,cpp}`.
- Draw the full asset set in the M19 style: 61 new tool icons, 63 new cursors,
  ~28 panel icons, 7 Layers-strip icons, 1 History snapshot icon.
- Replace the hardcoded cursor hotspot with a catalogue-driven lookup.
- Rebuild the toolbox as a CS6 single-column flyout grid, with unimplemented
  tools shown disabled.
- Wire panel icons into the rail, dock tabs, Layers strip, and History button.
- Ship one runnable check (`m38_icons`) that fails on any missing asset.

**Non-Goals:**

- Tool engines. No unimplemented tool gains behaviour; no canvas routing or
  options-bar change.
- The 1-column/2-column double-arrow toggle (single column stays).
- The CS6 hover tooltip *name below the pointer* overlay (Qt's own tooltip is
  used), `Show Tool Tips`, and temporary spring-loaded tool switching.
- Redrawing the 10 existing tool icons/cursors.
- Edition gating (Standard vs Extended) beyond leaving a documented hook: the
  catalogue marks Extended tools; the slot list may hide groups 19–20 and Count
  when an edition flag exists.
- The Layers-panel buttons that the strip icons anticipate (link, fx, mask);
  they land in M39/M41/M42.

## Decisions

### 1. One catalogue table, 71 rows, frozen

`tools.h` gains a `ToolCatalogueEntry` (id, label, shortcut, group, slot,
implemented, `Qt::CursorShape`, hotspot x/y) and `allCatalogueEntries()`; the
existing `ToolInfo`/`allToolIds`/`toolInfo` stay for the 10 implemented tools so
`ToolController` and the options bar are untouched. The catalogue is indexed by
`ToolId` for the 10 implemented entries and by catalogue index for the rest.

*Alternative considered:* one table replacing `ToolInfo`. Rejected: `ToolInfo`
is consumed by `ToolController`, `options_bar.cpp`, and the toolbox; keeping it
avoids a wide, risky C++ refactor.

*Alternative considered:* generate the catalogue from a data file at build time.
Rejected as over-engineering: 71 literal rows compile fine and are greppable.

### 2. Asset ids and the 10 frozen names

Tools are `tool.<name>` with a **kebab-free lowercase** name (task contract);
the 10 existing ids are unchanged (`move`, `marquee`, `lasso`, `quickselection`,
`crop`, `eyedropper`, `hand`, `zoom`, `brush`, `pencil`). New ids concatenate
the lowercase words (`polygonallasso`, `contentawaremove`, `objectrotate`).
Panel icons reuse the frozen `window.panels.<name>` command-id namespace so the
tab, rail, and menu resolve one asset; the Layers strip uses `layers.<action>`
and History uses `history.snapshot`.

### 3. Toolbox = single-column flyout slots

`toolbox.cpp` builds one `QToolButton` per slot (23), in catalogue order. A slot
with >1 member uses `QToolButton::MenuButtonPopup` and a corner triangle; its
`QMenu` lists every member, disabled iff unimplemented. The visible icon is the
slot's last-used member (a `QMap<int, ToolId>` in session state; default = first
implemented member). Clicking an enabled member selects it; `Alt`-click and
`Shift`+shortcut cycle the **enabled** members only.

*Alternative considered:* stack the 71 tools as 71 buttons. Rejected: it is not
CS6 and would make the toolbox unusable.

### 4. Disabled unimplemented tools with a fixed tooltip

Any catalogue entry with `implemented == false` produces a disabled button/menu
entry. The tooltip text is exactly `<label> — not implemented yet` (U+2014 em
dash, one space each side). A fully-unimplemented slot's button is disabled and
carries its default member's tooltip. This is what lets the whole 71-tool
catalogue be visible without pretending it works.

### 5. Hotspots are data, not code

`icons.cpp` replaces the eyedropper special case with a lookup into the
catalogue's hotspot pair. Default `(12,12)` for ids not in the catalogue
(guidance follows `AGENTS.md`: a table, not a chain of `if`s). The four rules
(centre, lower-left tip, upper-left nib, pointing finger) are in §3 of the
contract; the per-row value in §2.2 is authoritative.

### 6. Style guides match the existing art

Icons: 24×24 `viewBox`, `fill="none"`, `stroke="#c8c8c8"`, `stroke-width="1.5"`,
round caps/joins, at most one element in the `#3d6f99` accent (the accent already
used by `tool.eyedropper.svg` and the two panel icons). Cursors: two stacked art
copies — a `stroke-width="4"` white halo under a white-fill/`#202020`-stroke
body — monochrome, hotspot per the catalogue. Full rules in §4 and §5 of the
contract.

### 7. qrc regeneration

`assets/pictura.qrc` stays checked in (CMake is not touched). A new
`scripts/gen_qrc.sh` globs `assets/icons/*.svg` and `assets/cursors/*.svg`,
sorts, and rewrites the `<qresource>` block; the asset tasks run it once. The
`m38_icons` self-test (below) cross-checks every catalogue id, so a stale qrc
fails the build's self-test.

*Alternative considered:* CMake `file(GLOB CONFIGURE_DEPENDS)` + a generated
qrc. Rejected: it changes the build system, which this change must not, and the
checked-in qrc is reviewable.

### 8. Panel wiring

- `frame.cpp`'s `addRailPanel` passes `icon(commandId)` instead of `QIcon()` and
  drops the glyph-text workaround; the rail is icon-only
  (`Qt::ToolButtonIconOnly`). The five rail ids already are
  `window.panels.<name>`, so no id change.
- `registerPanel` sets the dock's window icon from the same `window.panels.<name>`
  asset so the tab strip shows it.
- `layers_panel.cpp` sets each strip button's icon (`layers.group`,
  `layers.newLayer`, `layers.delete`, `layers.fillAdjustment`; the deferred
  link/fx/mask buttons get their assets ready); `history_panel.cpp` sets
  `history.snapshot` on the snapshot button. The text labels stay (icons do not
  replace them), so no behaviour changes.

### 9. Verification is one self-test step

`main.cpp` gains `m38_icons`: catalogue completeness (non-null icon, non-null
cursor, hotspot in range, exactly 71 entries / 10 implemented), every qrc SVG
parses via `QSvgRenderer`, the toolbox exposes 23 slots and 71 menu entries,
implemented enabled / unimplemented disabled with the exact tooltip, and every
panel/strip/snapshot icon resolves. It prints a fixed summary line and uses
fresh exit codes. Rust tests are untouched because no Rust file changes.

## Risks / Trade-offs

- **71 cursors is real art work.** → The asset section is split per tool group
  so sub-agents can draw groups in parallel; the style guide is mechanical, and
  `m38_icons` rejects a missing or malformed file.
- **A high-DPI cursor with a badly placed hotspot is visible only interactively.**
  → The hotspot is data in the catalogue and the self-test asserts it is in
  range; the art is drawn to the hotspot, and a visual pass can adjust the data
  without code.
- **`QToolButton::MenuButtonPopup` is awkward to hold-and-reveal.** → `MenuButtonPopup`
  shows the triangle and opens the menu on the arrow; press-and-hold uses
  `DelayedPopup`. The design allows either; CS6 parity is "hold reveals", which
  is `DelayedPopup`. Decide at implementation with a quick GUI check.
- **The canonical `tool-framework` and `svg-cursors` specs contradict the
  change.** → This change carries MODIFIED deltas for both so `openspec/specs/`
  is consistent after archive.
- **Unimplemented tools visible but inert could confuse.** → The tooltip is
  explicit and the slot/button is disabled; the contract fixes the exact string.
- **Screen-reader names for icon-only rail buttons.** → The rail action keeps its
  tooltip/accessible name; icons do not remove it.

## Migration Plan

Additive and app-local. Rollback: restore the two-column toolbox, the glyph rail,
the hardcoded hotspot, and delete the added SVGs and qrc lines; no data, PSD, or
bridge change is involved. The numbered-layer stages are unaffected.

## Open Questions

- **Blur/Sharpen/Smudge key.** The CS6 table leaves the slot unlettered; the
  catalogue follows it. If a reliable CS6 source names a key, add it to group 13.
- **Zoom hotspot.** This design follows the task rule (pointing finger `(9,2)`);
  the art may read better at the lens centre, in which case only the per-row
  value changes.
- **Extended gating.** How the app learns Standard vs Extended (a build flag or
  a preference) is unresolved; the catalogue is ready, the slot filter is not.
- **Panel icon namespace.** `window.panels.<name>` is reused for tab/rail
  assets; if a future change wants a distinct `panel.*` namespace, this is the
  point to switch.
