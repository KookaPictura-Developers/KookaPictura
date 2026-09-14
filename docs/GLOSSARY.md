# Glossary

CS6-specific and image-processing vocabulary used across the specs. Alphabetical.
CS6-defined terms are marked **(CS6)**.

| Term | Definition |
|---|---|
| **ACR** | Adobe Camera Raw. The RAW-processing engine, version 7 in CS6. Also surfaced as a filter (`Filter > Camera Raw Filter`) in CS6. |
| **Adjustment layer** | A non-destructive layer that applies an adjustment to layers below it; carries a layer mask. |
| **Alpha channel** | An extra channel storing transparency or a saved selection; up to 56 channels total with color channels. |
| **Artboard** | A document-level container for multiple design regions on one canvas. **Not a CS6 feature** — artboards were added in Photoshop CC 2015; CS6 parity therefore excludes them (`05-layers/artboards.md` is a non-goal note). |
| **Blend mode** | A formula for compositing a layer's pixels with those below. CS6 has 27 modes. |
| **Clipping mask** | A layer whose pixels are constrained to the opaque area of the layer below. |
| **Color mode** | Per-document color model: Bitmap, Grayscale, Indexed, RGB, CMYK, Multichannel, Duotone, Lab. |
| **Document model** | The in-memory representation of a document: layers, channels, paths, metadata, history. |
| **Higher bit depth** | 8, 16, or 32 bits per channel; 32-bit is floating-point (HDR). |
| **History state / snapshot** | A recorded reversible step in the History panel; snapshots are named copies of a state. |
| **Layer comp** | A saved combination of layer visibility, position, and style settings. |
| **Layer style (fx)** | Effects attached to a layer: drop shadow, glows, bevel/emboss, satin, overlays, stroke. |
| **Mercury Graphics Engine (CS6)** | Adobe's GPU-accelerated rendering pipeline, greatly expanded in CS6 for interactive edits. |
| **Non-destructive** | Editing that preserves original pixel data (smart objects, smart filters, adjustment layers). |
| **PSB** | "Large Document Format". PSD variant supporting up to 300,000 px per dimension. Version field `2`. |
| **PSD** | Adobe Photoshop native document format; max 30,000 px per dimension. Version field `1`. |
| **Refine Edge (CS6)** | Selection-edge refinement controls (feather, contrast, shift edge, decontaminate colors). |
| **Smart Object** | A container layer holding embedded/linked source content; supports non-destructive smart filters. |
| **Smart Filter** | A filter applied to a smart object non-destructively, with an editable mask. |
| **Snapshot** | See history state. |
| **Vector mask** | A resolution-independent path-based mask on a layer or group. |
| **Quick Mask** | Temporary mask mode (`Q`) for editing a selection as a paintable channel. |
| **QRhi** | Qt Rendering Hardware Interface — Qt6's GPU abstraction; a candidate interop layer with Rust `wgpu`. |
| **cxx-qt** | Rust↔Qt bridge built on `cxx`; generates C++/Rust bindings for Qt types. |
| **qmetaobject-rs** | Rust crate exposing Qt QML via `QObject` macros without C++. |
