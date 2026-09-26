# Crash Recovery and Autosave

- **Spec ID:** `XC-012`
- **Status:** `Draft`
- **Parity tier:** `Core` (Auto-Recovery and Background Save); the durable
  journaling design is a Kooka Pictura extension and is `Extended-only` parity.
- **New in CS6:** `Yes` — CS6 introduced **Background Save** (`File > Save`
  runs without blocking the UI) and **Auto Save / Auto-Recovery**
  (`Automatically Save Recovery Information Every`). CS6 also added the Auto Save
  File Path and Auto Save Format image resources to PSD/PSB. CS5 and earlier
  blocked the UI while saving and had no automatic recovery.
- **Depends on:** `ARCH-009` undo-history, `ARCH-008` document-model,
  `ARCH-006` gpu-rendering-pipeline, `ARCH-012` file-formats,
  `ARCH-002` preference-storage, `11-cross-cutting/update-and-versioning.md`,
  `10-workflow-io/save-and-save-as.md`, `10-workflow-io/scratch-disks-and-memory.md`.

> All module and widget names below are **design proposals**. The CS6-side
> behavior is sourced from community tutorials and an Adobe knowledge-base
> article (marked in `## Sources`); the exact set of bytes CS6 stores in a
> recovery file is not published and is marked *(inferred)*.

## CS6 behavior

Photoshop CS6 adds two related safeguards, both configured in
`Edit > Preferences > File Handling`:

- **Background Save.** `File > Save` no longer freezes the application. Save
  progress is shown as a percentage in the **document tab** and as a percentage
  plus a **progress bar in the bottom-left of the document window**. The user can
  keep editing the document being saved, or switch to and work on a different
  document, until the save completes. This is a workflow change from CS5 and
  earlier, which blocked the UI for the duration of a save.
- **Auto Save / Auto-Recovery.** With **Automatically Save Recovery
  Information Every** enabled, Photoshop writes a backup copy of the work at a
  user-selected interval. The interval choices are **5, 10, 15, 30 minutes, or
  1 hour**; the default is **10 minutes**. Auto Save does **not** overwrite the
  user's original file — the recovery information is kept in a separate backup
  file. If Photoshop crashes, relaunching it **automatically opens the most
  recently saved backup copy**, with **"Recovered"** added to the document's
  name in the tab. Adobe's knowledge-base article states: "Photoshop
  automatically stores crash recovery information at user-specified intervals.
  If you experience a crash, Photoshop recovers your work when you restart it,"
  and suggests 5 minutes as a more frequent option than the 10-minute default.

CS6 writes two PSD/PSB image resources for this feature: **Auto Save File Path**
(ID `1086`) and **Auto Save Format** (ID `1087`), both Unicode strings, with the
Adobe specification advising readers not to interpret them. This is direct
evidence that CS6's recovery snapshot is a document in the "Auto Save Format"
located at the "Auto Save File Path".

*(inferred, community/observational)* The recovery backup is a full-document
snapshot in a Photoshop-readable format, not a delta journal: reopening restores
the document, but the **History panel is not restored** — recovery reopens the
document state, not the undo list. Auto Save does not run for an unmodified
document, and (per the Auto Save resource being written into the file) the path
and format travel with the saved document.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Edit > Preferences > File Handling` | Preference pane | `Ctrl/Cmd+K` then pane | `Save in Background`; `Automatically Save Recovery Information Every` + interval. |
| Document tab | Progress | — | Save percentage during Background Save. |
| Document window bottom-left | Progress | — | Percentage + progress bar during save/autosave (CS6). |
| Status bar | Readout | — | Save/autosave state; scratch/efficiency (`ARCH-003`). |
| Document tab name | Text | — | Recovered documents append **"Recovered"**. |
| Startup recovery dialog | Dialog | — | Proposed: lists recoverable documents; Recover / Discard / Open original. *(CS6 reportedly auto-opens the latest backup; whether it shows a chooser is unverified.)* |
| `File > Save` | Command | `Ctrl/Cmd+S` | May run in background; progress indicators appear. |
| `File > Open Recent` / File menu | Menu | — | Recovered documents are transient until saved under a new name. |
| Crash-report dialog | Dialog | — | Proposed: next-launch notice that a crash was detected, with dump path. |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Save in Background | bool | On | on / off | CS6 File Handling. |
| Automatically Save Recovery Information | bool | On | on / off | CS6 File Handling. |
| Auto Save interval | enum | 10 min | 5 / 10 / 15 / 30 / 60 min | CS6 choices; default reported as 10. |
| Recovery snapshot format | enum | PSD/PSB | matching document | Mirrors the Auto Save Format resource (1087). |
| Recovery file path | path | XDG state dir | any writable path | Mirrors the Auto Save File Path resource (1086). |
| Journal fsync policy | enum | `interval` | per-command / interval / off | Kooka Pictura extension; durability vs speed. |
| Checkpoint interval | duration | 5 min | 1 min…1 h | Journal compaction cadence. |
| Journal segment size | bytes | 64 MiB | 1 MiB…1 GiB | Rotation threshold. |
| Max recovery age | duration | 7 days | 0…30 days | Cleanup of stale recovery files. |
| Autosave dirty threshold | int commands | 0 | 0 = time only | Optional "snapshot after N edits". |
| Crash-dump capture | bool | On | on / off | Backtrace/minidump on fatal signal. |
| Recovery directory | path | `$XDG_STATE_HOME/kooka-pictura/recovery` | — | Per `ARCH-004` sandbox rules. |

## Algorithms & pipeline

### What is saved, and when

- **Target:** every open document that is **modified since its last checkpoint**
  (a dirty bit driven by the document revision counter from `ARCH-009`).
- **Content:** a PSD/PSB snapshot written with the normal serializer
  (`ARCH-012`), plus the Auto Save resources (`Auto Save File Path`,
  `Auto Save Format`) so the snapshot names its own origin and format, as CS6
  does. Document pixels and structure are complete; History is not.
- **When:** at the Auto Save interval (a timer per document, staggered so all
  documents do not fire at once); optionally after `N` committed commands; and
  on a few coarse events (a large committed command, entering a long filter).
  Save runs off the UI thread.
- **Never overwrite the original:** the snapshot goes to the recovery directory,
  not to the user's file. A successful user `Save` promotes the current state to
  a checkpoint and may delete the now-redundant snapshot.

### Background Save

```text
File > Save (async)
  snapshot   = document.freeze()          # cheap, copy-on-write tile refs
  worker: bytes = psd::serialize(snapshot) # normal serializer
  worker: write temp in target dir; fsync; atomic rename (QSaveFile::commit)
  publish progress in tab + status bar (percentage)
  on success: clear dirty, promote checkpoint, keep UI responsive throughout
  on failure/cancel: discard temp; original untouched; surface error
```

The freeze step must not block on pixel work: it captures immutable references
to the current tile versions (`ARCH-006`/`ARCH-009`), so concurrent edits create
new tiles rather than mutating the ones being serialized. Writing the file is
atomic: a temporary sibling is `fsync`ed and renamed into place; a crash mid-save
leaves the previous file intact.

### Durable journal (Kooka Pictura extension)

Snapshots bound data loss to the interval (up to 10 minutes by default). To
narrow that, every committed command also appends to a per-document journal:

```text
Command committed (from the ARCH-009 command bus)
  record = { seq, doc_id, label, command_kind, payload, parent_seq, crc32 }
  append to journal segment; flush; fsync per policy
  every checkpoint_interval or segment-size: write a full PSD checkpoint,
  truncate/rotate the journal, keep the previous checkpoint until commit
Recovery:
  load latest valid checkpoint
  replay journal records with seq > checkpoint.seq, in order
  stop at the first bad CRC (truncate to last valid record)
```

The journal is a durable subset of the already-existing command stream; it adds
no new mutation path. It is what makes sub-interval recovery possible; it does
**not** make History persistent (see below).

### Temp-document snapshots and the manifest

Recovery lives in `$XDG_STATE_HOME/kooka-pictura/recovery/` (Flatpak/Snap-safe;
`ARCH-004`):

```text
recovery/
  <doc-id>/
    manifest.json         # schema, original path, display name, timestamps,
                          # app + format versions, dirty flag, active doc
    checkpoint.psd        # latest full snapshot (or .psb)
    journal/0001.bin ...  # append-only segments
    lock                  # PID + start time of the owning instance
```

A per-instance lock file records the owning PID. On clean exit the directory is
removed. On startup, an existing directory whose PID is dead (or whose lock is
missing) marks an **unclean shutdown**.

### Crash handler

- **Rust panics:** install `std::panic::set_hook` to write a panic report
  (thread, message, backtrace via the `backtrace` crate) into the recovery
  directory and flush the journal. A panic in the main process then unwinds or
  aborts per build policy.
- **Fatal POSIX signals:** use `signal-hook` (or `signal_hook_registry`) to
  catch `SIGSEGV`, `SIGABRT`, `SIGBUS`, `SIGILL`, and termination signals. The
  handler must be **async-signal-safe**: no allocation, no locks, no Rust
  `String`/`Vec` work. Capture only a fixed buffer / write a preformatted marker,
  then defer the real work to a separate crash-helper process (fork/exec) or to
  the next launch.
- **Caveat:** `signal-hook` deliberately forbids registering a handler for
  `SIGSEGV` by default (it panics); raw registration is possible through its
  low-level backend but is documented as not done lightly. The proposed design
  therefore treats signal-based dumps as best-effort and relies on the
  lock-file/manifest detection as the reliable crash signal. A dedicated
  minidump/crashpad-style out-of-process writer is the robust alternative
  (Open questions).
- **No recovery data is written inside the signal handler.** The journal is
  already durable (fsynced at commit time); the handler only needs to leave the
  lock file and a crash marker, so the next launch knows recovery is required.

### Recovery dialog

On startup, if an unclean shutdown is detected:

1. Scan `recovery/` for manifests with a dead owner.
2. Present a `RecoveryDialog` listing documents (display name, original path,
   snapshot time, journal depth).
3. User choices: **Recover** (open checkpoint + replay journal), **Recover
   Snapshot Only** (skip journal), **Discard**, **Open Original** (the on-disk
   file, if it exists).
4. Recovered documents open with **"Recovered"** appended to the tab name (CS6
   naming parity) and are **not** associated with the original path until the
   user saves; this prevents an accidental overwrite of the original.
5. After a successful user save (or explicit discard), the recovery directory is
   cleared and the normal autosave timer resumes.

### Integration with `ARCH-009` (undo history)

- History is in-memory and is **not** serialized (`ARCH-009`). Recovery restores
  the document, not the History panel: after recovery, the History panel starts
  from a single "Recovered" state, exactly as reopening a PSD does. The journal
  replays **commands**, so it can optionally reconstruct a **History Log**
  (`ARCH-009`'s text log) but not the full image-state list.
- Snapshots reuse the tile store's copy-on-write references; background saving
  must invalidate GPU tile references consistently (`ARCH-006`).
- The document revision counter is the dirty signal shared with undo
  dirty-tracking.

## Rust module mapping

- `pictura_recovery::autosave` — per-document timer, dirty check, snapshot
  scheduling, and the "never overwrite original" policy.
- `pictura_recovery::background_save` — `SaveJob`: freeze → serialize → atomic
  write on a worker; progress/cancel channel; integrate with the command bus.
- `pictura_recovery::journal` — append-only segments, CRC, rotation, replay,
  and truncate-to-last-valid; `JournalRecord`.
- `pictura_recovery::checkpoint` — full PSD/PSB checkpoint via `pictura_io::psd`.
- `pictura_recovery::manifest` — `RecoveryManifest` (serde JSON), lock/PID file,
  unclean-shutdown detection.
- `pictura_recovery::crash` — `panic::set_hook`, signal registration
  (`signal-hook`), fixed-buffer crash marker, best-effort backtrace.
- `pictura_recovery::session` — startup scan, `RecoveryCandidate`, and the
  recover/discard/open-original operations.
- `pictura_recovery::cleanup` — age-out and post-save cleanup, with a safety
  margin that never deletes a recovery file the user has not resolved.

Types crossing the Rust↔Qt boundary: `RecoveryCandidate`, `RecoveryManifest`,
`SaveProgress`, `CrashNotice`. Qt performs discovery, presentation, and user
consent; all file I/O and serialization stay in Rust.

## Qt6 component mapping

| Proposal | Base | Responsibility |
|---|---|---|
| `FileHandlingPreferences` | `QWidget` | `Save in Background` toggle; Auto Save toggle + interval combo. |
| `RecoveryDialog` | `QDialog` | List recoverable documents; Recover / Snapshot-only / Discard / Open Original. |
| `RecoveryListModel` | `QAbstractListModel` | Candidates with roles: name, path, timestamp, journal depth. |
| `BackgroundSaveController` (`QObject`) | cxx-qt | Bridges `SaveJob` progress to the tab/status-bar indicators; cancel. |
| `SaveProgressIndicator` | `QWidget` | Tab percentage and bottom-left progress bar (CS6 parity). |
| `CrashReportDialog` | `QDialog` | Next-launch notice; shows the panic/backtrace location. |
| `qInstallMessageHandler` hook | Qt Core | Fold Qt warnings/critical messages into the crash report (best-effort). |

Widgets (not QML): these are modal/desktop dialogs and docked progress surfaces
that must match the CS6 shell and be screen-reader accessible.

## Data-model impact

- **PSD/PSB image resources written:** `Auto Save File Path` (ID `1086`) and
  `Auto Save Format` (ID `1087`), both Unicode strings, matching CS6. The Adobe
  spec discourages interpreting them; we read them for recovery discovery and
  preserve them verbatim on round-trip (`AIR-010`-style opaque preservation).
  **Version Info** (`1057`) provides writer/reader/file version for the
  recovered document.
- **Runtime structures (not serialized into a user document):**
  `RecoveryManifest { schema, doc_id, original_path, display_name, saved_at,
  app_version, format_version, dirty, active }`; `JournalRecord { seq, doc_id,
  label, kind, payload, parent_seq, crc32 }`; `Checkpoint { seq, path, sha256 }`.
- **Dirty tracking:** the per-node/tile revision counter from `ARCH-009` is the
  authoritative dirty flag; a document with no committed commands is not
  snapshotted.
- **Undo granularity:** autosave and journaling do not create History states.
  They observe committed commands; they never add an undo entry. A user `Save`
  does not create an undo entry either (CS6 behavior).
- **Recovered document naming:** the in-memory display name gains the
  "Recovered" suffix; the document's `original_path` is retained in the manifest
  so the user can save over it deliberately, but the default is save-as-new.

## Edge cases

- **Crash during autosave** — the atomic rename means the previous checkpoint
  remains valid; the new temp is discarded. Journal replay proceeds from the old
  checkpoint.
- **Crash during a normal user save** — the original file is untouched (temp +
  rename); recovery still has the last checkpoint.
- **Disk full / read-only recovery dir** — autosave fails loudly once and
  disables itself for the session (CS6-style warning); the document is never
  silently left without recovery, and the failure is reported in the status bar.
- **Multiple instances / same document** — separate recovery directories keyed
  by `doc_id` + PID; a second instance of the same file must not adopt another's
  journal. Cross-instance editing of one file is out of scope and warned.
- **Stale recovery data** — age-out after the configured maximum, but never
  delete an entry the user has chosen to keep; deletion is explicit.
- **New, unsaved (`Untitled`) document** — has no original path; recovery
  restores it as "Recovered" and forces save-as-new on first save.
- **Huge/PSB documents** — autosave cost scales with document size; the interval
  snapshot may be deferred if a save is already in flight. The journal keeps
  edits durable in the meantime. PSB 8-byte lengths are exercised.
- **32-bit float documents** — checkpoint size is ~4× the 8-bit case; scheduling
  and cleanup must account for it.
- **Journal corruption** — CRC failure truncates to the last valid record; a
  corrupt first segment falls back to snapshot-only recovery.
- **Recovery file references a missing original** — offer save-as-new; never
  fail the whole recovery because one original moved.
- **Autosave disabled** — no snapshots; crash loses work (documented, matches
  CS6 when the option is off).
- **Power loss** — fsync policy decides the durability window; the journal
  records are fsynced per policy, the checkpoint on commit.
- **Flatpak/Snap sandbox** — recovery path must be inside the granted XDG state
  directory (`ARCH-004`); portal-based fallback otherwise.
- **Signal handler re-entrancy** — crash marker writes are fixed-size and
  allocation-free; a second fatal signal is not handled recursively.
- **Crash while a modal dialog is open** — recovery restores the document, not
  the dialog; unsaved dialog state is lost (acceptable; note it).
- **Clock skew / timestamp** — manifest timestamps are informational; ordering
  uses the monotonic journal `seq`.

## Parity acceptance criteria

1. Given `Automatically Save Recovery Information` on with interval 10 minutes
   and a modified document, a recovery snapshot appears within 10 minutes ± one
   scheduling tick, and the original file's bytes are unchanged.
2. Given the interval options, the UI offers exactly 5, 10, 15, 30, and 60
   minutes with 10 selected by default.
3. Given `File > Save` on a large document with `Save in Background` on, the UI
   remains interactive, the tab shows a percentage, and the bottom-left shows a
   percentage plus progress bar until completion.
4. Given a simulated crash (SIGKILL) after at least one snapshot, relaunching
   offers recovery; recovering produces a document whose pixels/structure match
   the last checkpoint (plus replayed journal within the same tolerances as a
   normal open) and whose tab name contains **"Recovered"**.
5. Given a crash, the original file on disk is byte-identical to its pre-crash
   state.
6. Given `Automatically Save Recovery Information` off, a crash produces no
   recovery snapshot and no recovery prompt.
7. Given an unclean shutdown with a valid journal and a checkpoint, replay
   yields a document whose structural hash equals the expected replayed state;
   a corrupted journal record truncates to the last valid state without panic.
8. Given a crash during autosave, the previous checkpoint remains recoverable
   and the partially written temp file is never presented as valid.
9. Given an `Untitled` recovered document, the first save is forced to
   save-as-new and does not silently overwrite any file.
10. Given recovery files older than the maximum age, they are removed; a
    recovery entry the user chose to keep is not removed.
11. Given multiple simultaneous instances, each recovery directory is isolated
    by PID and the second instance never adopts the first's journal.
12. Given recovery is complete and the user saves, the recovery directory for
    that document is cleared and the normal autosave timer resumes.

## Sources

- `https://www.photoshopessentials.com/basics/background-auto-save-cs6/` —
  CS6 Background Save (UI stays responsive; progress in tab and bottom-left;
  switch documents while saving) and Auto Save (`Automatically Save Recovery
  Information Every`, default 10 minutes with 5/15/30/60 options; separate
  backup file; does not save over the original; after a crash Photoshop opens
  the most recent backup with "Recovered" in the tab name). Secondary tutorial,
  CS6-specific.
- `https://photoshopguides.github.io/File%20Handling` — File Handling
  preferences: `Save in Background`, `Automatically Save Recovery Information
  Every: 10 Minutes` with options 5/10/15/30/1 hour; describes it as crash
  protection creating a temporary backup at the interval. Secondary/community,
  cross-version.
- `https://web.archive.org/web/20240303004655/https://helpx.adobe.com/photoshop/kb/file-recovery-photoshop.html`
  — Adobe KB "Troubleshoot file recovery in Photoshop": "Photoshop automatically
  stores crash recovery information at user-specified intervals. If you
  experience a crash, Photoshop recovers your work when you restart it"; default
  10 minutes, can be set to 5; guidance to enable file recovery in preferences
  and the `Maximize PSD and PSB file compatibility` interaction for
  composite-only recovery.
- `https://web.archive.org/web/2023id_/https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/`
  — CS6 Auto Save image resources 1086 `Auto Save File Path` and 1087
  `Auto Save Format` (Unicode strings, "recommended that you do not interpret or
  use this data"); Version Info resource 1057; PSD/PSB structure and compression
  codes used by the checkpoint serializer.
- `https://doc.qt.io/qt-6/qsavefile.html` — atomic document save: temp file +
  `commit()` rename, `cancelWriting()`, automatic error detection, and the
  direct-write fallback caveat (non-atomic when the directory forbids temp
  files). Used for both user saves and recovery writes.
- `https://docs.rs/signal-hook/latest/signal_hook/` — safe Unix signal handling:
  async-signal-safety constraints (no allocation/mutexes in handlers),
  `SIGSEGV`/`SIGBUS` registration is *forbidden* by default and panics, second
  Ctrl-C shutdown pattern, and the recommendation to register early before other
  threads start. Used for the crash-handler design and its caveats.
- `https://docs.rs/backtrace/latest/backtrace/` — runtime backtrace acquisition,
  `trace`/`resolve_frame`, and the platform/unwind accuracy caveats; used for
  panic and crash reports.
- Internal: `docs/01-architecture/undo-history.md` (in-memory history, not
  serialized; tile-diffusion model; crash recovery must use autosave),
  `docs/01-architecture/file-formats.md` (PSD serializer, 8/16/32-bit, PSB),
  `docs/02-ui-ux/preferences.md` (File Handling defaults, `Save in Background`,
  Auto Save), `docs/01-architecture/build-and-packaging.md` (XDG/Flatpak state
  paths).

Whether CS6 shows a recovery chooser or silently opens the latest backup, and
the exact recovery-file format/location, are *(inferred/unverified)* — see
Open questions.

## Open questions

- **Recovery dialog vs silent open.** The community sources say Photoshop
  *automatically opens* the most recent backup; it is not confirmed whether CS6
  also presents a chooser when several documents were open. *Resolves with:* a
  CS6 crash experiment or the CS6 Help File Handling page.
- **Exact recovery file format and path.** The Auto Save resources (1086/1087)
  name a path and a format, but the format encoding and default directory are
  unpublished. *Resolves with:* inspecting a CS6 recovery file on disk (e.g.
  after a forced crash) and reading resource 1087's value.
- **Whether History survives recovery.** Assumed not, consistent with
  `ARCH-009`. *Resolves with:* a CS6 crash-then-recover test checking the
  History panel.
- **Snapshot cadence semantics.** Whether CS6's timer is wall-clock, per
  document, or per session, and whether it resets on save/undo. *Resolves with:*
  observing autosave file mtimes across edit patterns.
- **Durability/perf trade-off.** Journal fsync policy (per command vs interval)
  and its cost on large documents. *Resolves with:* a journaling benchmark.
- **Crash-dump mechanism.** Whether to rely on `signal-hook` + best-effort
  backtrace or integrate an out-of-process minidump/crashpad-style writer.
  *Resolves with:* a crash-capture spike and the platform matrix.
- **Recovery encryption.** Whether snapshots of sensitive documents need
  at-rest encryption. *Resolves with:*
  `11-cross-cutting/security-and-sandboxing.md`.
- **Interaction with auto-save resources on foreign files.** Whether to write
  1086/1087 into kooka-pictura-saved files for parity, or only into recovery
  snapshots. *Resolves with:* the PSD compatibility policy in `ARCH-007`.
- **Multi-instance same-file policy.** Detect-and-warn vs lock. *Resolves with:*
  a cross-instance test.
- **Recovery of plugin/script-created documents.** Whether non-UI sessions
  (batch/actions) participate in autosave. *Resolves with:* the automation spec
  and a decision.
- **Stale recovery cleanup UX.** Automatic age-out vs a manual "Recovery Bin".
  *Resolves with:* a product decision.
