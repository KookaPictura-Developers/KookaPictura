## Context

`LayersModel` is a `QAbstractItemModel` tree over the bridge's flat
topmost-first projection; each row carries `PathRole`, `KindRole`, `VisibleRole`,
`BlendRole`, `ColorRole`, `LockRole`, `ClippingRole`, `HasMaskRole`, etc.
(`layers_panel_internal.h`). The panel currently holds `LayersModel*` directly
and uses `model_->indexForPath` / `model_->pathForIndex` for selection and
expansion. This change interposes a `QSortFilterProxyModel` and moves those
lookups through it.

## Goals / Non-Goals

- **Goals:** the CS6 filter/search row; the six-dimension predicate; ancestor
  promotion; view-only and live.
- **Non-Goals:** the Effect dimension's criteria (disabled until layer styles);
  Type/Shape/Smart-Object kinds (no model field yet); persisting the filter;
  commands/Actions recording (filtering is inert).

## Decisions

### D1 — Proxy over the existing tree
`LayersFilterProxyModel : QSortFilterProxyModel` wraps `LayersModel`.
`filterAcceptsRow` accepts a row when the filter is disabled, when the row
itself matches, or when any descendant matches (ancestor promotion). Descendant
matching recurses the source model's `rowCount`/`data`. Highlighting is not
used; filtering only hides.

### D2 — `LayerFilter` value
```
struct LayerFilter {
    bool enabled = false;
    QString name;            // empty = inactive; case-insensitive substring
    QSet<QString> kinds;     // empty = inactive; active = any listed kind matches
    QString mode;            // empty = inactive; exact blend key
    int color = -1;          // -1 = inactive; 0..7 exact
    QString attribute;       // "" = inactive; "visible"|"hidden"|"locked"|"mask"|"clipped"
};
```
Active criteria are ANDed; Kind values OR within the set. Every value is a
structural key (`kind`, blend key, label index, attribute key), never a display
string, so localization cannot affect matching.

### D3 — Row widget
`LayerFilterBar` hosts a dimension `QComboBox` (Name, Kind, Effect, Mode,
Attribute, Color; default Kind), a `QStackedWidget` criteria stack, and a
checkable on/off `QToolButton`. It owns a `LayerFilter` and emits
`filterChanged(const LayerFilter&)` whenever the dimension, a criterion, or the
toggle changes. The Effect page is a disabled combo with a "not implemented
yet" tooltip; selecting it leaves the predicate inactive. `setFilter`/
`filter()` are also exposed for the panel and tests.

### D4 — Panel integration
The panel keeps `model_` as the source; `proxy_->setSourceModel(model_)` and
`tree_->setModel(proxy_)`. All index lookups go through helpers
`proxyIndexForPath(path)` (source `indexForPath` → `proxy_->mapFromSource`) and
`pathForProxyIndex(index)` (`proxy_->mapToSource` → source `pathForIndex`).
`selectedPaths`, `selectPaths`, `currentPath`, rename re-selection, and the
`expanded`/`collapsed` handlers use the proxy. On `filterChanged`, set the
proxy filter, and when the filter is active auto-expand each visible group
ancestor of a match (a collapsed group otherwise hides its matches); when the
filter is off, restore the session expansion set. Filtering never calls a
bridge mutator, so it adds no history state.

### D5 — Live re-evaluation
The source model resets on every `refresh()`; the proxy re-filters on
`modelReset`/`dataChanged` (connected in its constructor), so a rename or a
visibility/blend/color change is reflected while the filter is active.

### D6 — No persistence
The filter is transient. `setView` (document switch) clears it to the default
(Kind, off). The session store is untouched.

## Interface freeze

```
struct pictura::LayerFilter            // D2 fields above
class  pictura::LayersFilterProxyModel // setFilter(const LayerFilter&), filter()
class  pictura::LayerFilterBar         // LayerFilter filter() const,
                                       // void setFilter(const LayerFilter&),
                                       // signal filterChanged(const LayerFilter&)
LayersPanel                            // tree uses the proxy; helpers
                                       // proxyIndexForPath/pathForProxyIndex
```

## Risks / Trade-offs

- **Ancestor promotion recursion** is O(rows) per `filterAcceptsRow` call in the
  worst case; acceptable for panel-scale layer counts and marked with a
  `// ponytail:` note if it becomes hot.
- **Index mapping mistakes** are the main hazard when swapping the view model;
  the self-tests assert selection and auto-expand through the proxy.
- **Effect/Kind gaps** are visible in the UI (disabled entries) rather than
  silent, matching the existing "not implemented yet" convention.
