# select-menu Specification

## MODIFIED Requirements

### Requirement: Select menu command surface

The system SHALL implement the CS6 Select menu rows Reselect, Inverse, the
Modify submenu (Border, Smooth, Expand, Contract, Feather), Grow, Similar, Save
Selection, Load Selection, All Layers, Deselect Layers, and Similar Layers
through stable command ids and bridge operations. Each row SHALL keep its
documented menu path and shortcut. The rows Color Range, Refine Edge, and
Transform Selection SHALL remain present but disabled with a documented reason.

#### Scenario: Implemented rows are enabled with a document

- **WHEN** a document is open with an active selection
- **THEN** Reselect, Inverse, Modify, Grow, Similar, and Save Selection are enabled

#### Scenario: Deferred rows stay disabled

- **WHEN** the Select menu is opened
- **THEN** Color Range, Refine Edge, and Transform Selection are visible and disabled
