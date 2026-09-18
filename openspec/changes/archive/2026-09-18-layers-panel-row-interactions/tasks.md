## 1. Engine — reparent

- [x] 1.1 `move_path_to(doc, path, target, mode)` in `layer_ops/properties.rs` (0 above, 1 below, 2 into); refusal rules; re-export.
- [x] 1.2 Unit test: above/below/into, same-container index adjustment, self-drop and descendant-drop refusal, locked/Background refusal.
- [x] 1.3 Bridge `move_layer_to(path, target, mode)` — recomposite + one `"Move Layer"` record.

## 2. Widgets

- [x] 2.1 `PercentField(label)`: owns the label and a `%` suffix; label and suffix scrub like the field; `labelText()`.
- [x] 2.2 Lock icons semantic (only `lockAll` a padlock); eye icons `layers.eyeOn/Off`; disclosure `layers.disclosureRight/Down`; Kind icons `layers.kind{Pixel,Adjustment,Group,Background}`; add all to qrc.
- [x] 2.3 Panel: construct `PercentField` with the label; draw the eye/chevron assets at the inset; set Kind button icons.

## 3. Drag and drop

- [x] 3.1 `LayersTreeView`: enable drag/drop, `startDrag` MIME, enter/move/drop overrides computing target + mode; drag does not extend the selection.
- [x] 3.2 Panel wires the drop handler to `move_layer_to`; strip buttons accept the MIME and run delete/duplicate/group; mask/link/fx inert.
- [x] 3.3 Self-test `lpr_*`: label scrub, `%`, semantic lock icons, disclosure icon, drag reorder, drop-on-button.

## 4. Verification

- [x] 4.1 Build; `./build/pictura --headless --self-test` and the `.psd` run exit 0; screenshot check.
- [x] 4.2 `TASK_ALLOWS_DOCS=1 bash scripts/verify-full.sh`; `openspec validate --all --strict`.
