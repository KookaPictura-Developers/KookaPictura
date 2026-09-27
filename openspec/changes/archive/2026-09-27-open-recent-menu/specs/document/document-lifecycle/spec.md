# document-lifecycle Specification

## ADDED Requirements

### Requirement: Open Recent menu

File > Open Recent SHALL follow Open As Smart Object and SHALL be rebuilt each
time it opens, so a file opened in this session appears immediately. It SHALL
list the existing recent files most recent first, each labelled with its file
name and carrying its full path as a tooltip, followed by a separator and Clear
Recent File List; with no recent files it SHALL show only a disabled "No Recent
Files" row. Choosing an entry SHALL open it as File > Open would (PSD/PSB
through the codec, other images through the image importer), and a successful
image open SHALL also be recorded. Clear Recent File List SHALL empty and
persist the list.

#### Scenario: A file opened this session is offered at once

- **WHEN** the `recent_files` self-test opens an image and shows Open Recent
- **THEN** the image is the first row, labelled by name with its path as the tooltip, and Clear Recent File List is present

#### Scenario: Choosing a recent image reopens it

- **WHEN** that row is chosen
- **THEN** a new document opens

#### Scenario: Clearing empties the list

- **WHEN** Clear Recent File List is chosen
- **THEN** the list is empty and the submenu shows only "No Recent Files"
