# shape-selection-tools Specification

## REMOVED Requirements

### Requirement: Deferred selection tools stay visible and disabled

**Reason**: The Magnetic Lasso, the only deferred member of the Lasso group, is
now implemented (`tools/magnetic-lasso`).

**Migration**: The general rule that an unimplemented tool is shown disabled and
cannot be activated remains in `tools/tool-framework` and
`ui/application-shell`; the self-test guard (code 98) now probes Perspective
Crop.
