# M1 — Document Model + PSD Layers

Goal: the data spine. A real layer/channel/mask model and PSD read/write that
round-trips documents with layers, verified against an independent oracle.

## Scope

In:
- Layer tree: pixel layers + groups, in PSD z-order (bottom-first on disk).
- Layer properties: name, bounds (top/left/bottom/right), opacity, blend mode,
  clipping, visible, protected flags.
- Channels per layer: color channels (`0,1,2…`), transparency mask (`-1`); layer
  mask channel (`-2`).
- Raster layer mask (`top/left/bottom/right`, default color, disabled flag, data).
- `Document` gains a `layers` tree; the M0 `composite` stays.
- PSD layer info section: `'Layr'` (8-bit), layer records, channel image data
  (raw + PackBits RLE), global layer mask info, and the `'luni'` additional-layer
  key for Unicode names.
- Round-trip harness: model → write → read → model equality.

Out (later milestones):
- Layer styles/fx, adjustment layers, smart objects, text, vector masks.
- 16/32-bit layer data (`Lr16`/`Lr32`), ZIP/ZIP-prediction channel compression.
- Blend-mode *math* (M1 only round-trips the mode enum, not compositing).
- True PSD fidelity with real-world files beyond the supported subset.

## Task DAG

| ID | Task | Owner | Owns | Acceptance |
|---|---|---|---|---|
| M1-A | Core layer/channel/mask model | agent | `crates/pictura-core` | Types + helpers + unit tests; `BlendMode` covers all 27 PSD keys |
| M1-C | Oracle fixtures + differential tests | agent | `scripts/**`, `crates/pictura-codec/tests/**`, fixtures | psd-tools generates layered fixtures; tests read them and validate codec output once available |
| M1-B | PSD layer read/write | agent | `crates/pictura-codec/src` | Reads/writes layered PSDs; round-trip equality; malformed input errors, never panics |
| M1-D | Integrate + full harness | orchestrator | — | workspace green; round-trip + oracle tests pass |

## Model contract (guidance — M1-A is authoritative)

- All data is **big-endian** on disk; bounds are signed and may place a layer
  outside the canvas.
- `BlendMode` must map to the 4-byte PSD key (`'norm'`, `'mul '`, `'scrn'`, …).
- Channel data is **planar per layer**, compressed per the channel-info header
  (2 bytes for PSD: compression + optional RLE counts).
- Layer order on disk is bottom-to-top; the model must not lose that ordering.
- Group layers in PSD have a `'lsct'` marker; handle at least open/closed groups.

## Exit gate

- `cargo test --workspace` green, including round-trip and oracle tests.
- psd-tools can open a PSD written by `pictura-codec` and sees the expected
  layers; `pictura-codec` reads a psd-tools-authored PSD and sees the same.
- Malformed/truncated layer sections return `PsdError`, never panic.
