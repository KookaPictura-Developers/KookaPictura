use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use crate::history::{History, Snapshot};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString};
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LockFlags, PsdRect,
};

impl qobject::PictureView {
    pub fn open(self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let mut loaded = std::fs::read(&path)
            .ok()
            .and_then(|bytes| pictura_codec::read_psd(&bytes).ok());

        let ok = loaded.is_some();
        let gpu_compute = self.rust().gpu_compute;
        let rendered = loaded.as_ref().map(|doc| current_buffer(doc, gpu_compute));
        if let (Some(doc), Some(rendered)) = (loaded.as_mut(), rendered.as_ref()) {
            store_composite(doc, rendered);
        }
        let image = rendered.as_ref().map_or_else(test_image, buffer_to_image);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = loaded;
        view.reset_edit_state();
        let initial = view.doc.as_ref().map(|doc| Snapshot {
            doc: doc.clone(),
            selection: None,
        });
        if let Some(snapshot) = initial {
            view.history.capture(snapshot, "Open");
        }
        view.path = if ok { Some(path) } else { None };
        view.dirty = false;
        ok
    }

    /// `File > Open` for a raster image: read `path`, probe and decode it with
    /// Qt, and replace the view with an untitled RGB/8-bit document holding the
    /// decoded pixels. Records one "Open" state, leaves the view's path empty,
    /// and marks it unmodified; `false` without mutating on any refusal.
    pub fn open_image(self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let name = std::path::Path::new(&path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some((rgba, width, height)) =
            std::fs::read(&path).ok().as_deref().and_then(decode_import)
        else {
            return false;
        };
        let mut doc = Document::from_rgba(&name, width, height, &rgba);
        let gpu_compute = self.rust().gpu_compute;
        let rendered = current_buffer(&doc, gpu_compute);
        store_composite(&mut doc, &rendered);
        let image = buffer_to_image(&rendered);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = Some(doc);
        view.reset_edit_state();
        let initial = view.doc.as_ref().map(|doc| Snapshot {
            doc: doc.clone(),
            selection: None,
        });
        if let Some(snapshot) = initial {
            view.history.capture(snapshot, "Open");
        }
        view.path = None;
        view.dirty = false;
        true
    }

    /// `File > Open As Smart Object…`: read `path`, decode it as a PSD/PSB
    /// source, and replace the view with a new untitled document whose sole
    /// layer is that source as an embedded smart object. Records one
    /// "Open As Smart Object" state on success; `false` without mutating when
    /// the file is missing, unreadable, or not a PSD/PSB document.
    pub fn open_as_smart_object(self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let name = std::path::Path::new(&path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut doc = match std::fs::read(&path)
            .ok()
            .and_then(|bytes| pictura_render::open_as_smart_object(&name, &bytes))
        {
            Some(doc) => doc,
            None => return false,
        };
        let gpu_compute = self.rust().gpu_compute;
        let rendered = current_buffer(&doc, gpu_compute);
        store_composite(&mut doc, &rendered);
        let image = buffer_to_image(&rendered);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = Some(doc);
        view.reset_edit_state();
        let initial = view.doc.as_ref().map(|doc| Snapshot {
            doc: doc.clone(),
            selection: None,
        });
        if let Some(snapshot) = initial {
            view.history.capture(snapshot, "Open As Smart Object");
        }
        view.path = None;
        view.dirty = false;
        true
    }

    pub fn new_document(
        self: Pin<&mut Self>,
        width: i32,
        height: i32,
        mode: &QString,
        depth: i32,
        background: &QString,
    ) -> bool {
        if width < 1 || height < 1 || depth != 8 {
            return false;
        }
        let mode = match mode.to_string().as_str() {
            "rgb" => ColorMode::Rgb,
            "grayscale" => ColorMode::Grayscale,
            _ => return false,
        };
        let white = match background.to_string().as_str() {
            "white" => true,
            "transparent" => false,
            _ => return false,
        };
        let mut doc = Document::new(width as u32, height as u32, mode, BitDepth::Eight);
        let fill = if white { 255 } else { 0 };
        if white {
            doc.composite.data.fill(255);
        }
        let pixels = width as usize * height as usize;
        let mut channels: Vec<Channel> = (0..mode.color_channels())
            .map(|id| Channel {
                id: id as i16,
                data: vec![fill; pixels],
            })
            .collect();
        channels.push(Channel {
            id: -1,
            data: vec![fill; pixels],
        });
        doc.layers.push(Layer {
            name: "Layer 0".to_string(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: height,
                right: width,
            },
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels,
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        });
        let gpu_compute = self.rust().gpu_compute;
        let rendered = current_buffer(&doc, gpu_compute);
        store_composite(&mut doc, &rendered);
        let image = buffer_to_image(&rendered);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = Some(doc);
        view.reset_edit_state();
        let initial = view.doc.as_ref().map(|doc| Snapshot {
            doc: doc.clone(),
            selection: None,
        });
        if let Some(snapshot) = initial {
            view.history.capture(snapshot, "New");
        }
        view.path = None;
        view.dirty = false;
        true
    }

    pub fn save(self: Pin<&mut Self>, path: &QString) -> bool {
        let Some(bytes) = self
            .rust()
            .doc
            .as_ref()
            .and_then(|doc| pictura_codec::write_psd(doc).ok())
        else {
            return false;
        };
        let path = path.to_string();
        let tmp = format!("{path}.tmp");
        if std::fs::write(&tmp, &bytes).is_err() {
            return false;
        }
        if std::fs::rename(&tmp, &path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return false;
        }
        let mut view = self.rust_mut();
        view.path = Some(path);
        view.dirty = false;
        true
    }

    pub fn is_dirty(&self) -> bool {
        self.rust().dirty
    }

    pub fn file_path(&self) -> QString {
        self.rust()
            .path
            .as_deref()
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn image(mut self: Pin<&mut Self>) -> QImage {
        let mut rust = self.as_mut().rust_mut();
        if !rust.display_dirty {
            return rust.image.clone();
        }
        let rebuilt = rebuild_display(&rust.doc, rust.stroke.as_ref(), rust.gpu_compute);
        if let Some(image) = rebuilt {
            rust.image = image;
            rust.display_dirty = false;
        }
        rust.image.clone()
    }

    pub fn has_document(&self) -> bool {
        self.rust().doc.is_some()
    }

    pub fn document_width(&self) -> i32 {
        self.rust()
            .doc
            .as_ref()
            .map(|d| d.width as i32)
            .unwrap_or(0)
    }

    pub fn document_height(&self) -> i32 {
        self.rust()
            .doc
            .as_ref()
            .map(|d| d.height as i32)
            .unwrap_or(0)
    }

    pub fn sample_argb(&self, x: i32, y: i32) -> u32 {
        let rust = self.rust();
        let Some(doc) = rust.doc.as_ref() else {
            return 0;
        };
        // Read the authoritative planar composite directly; never build a full
        // image for one pixel.
        sample_planar_argb(&doc.composite, x, y)
    }

    pub fn composite_argb(&self, x: i32, y: i32) -> u32 {
        self.sample_argb(x, y)
    }

    /// M0.5 GPU smoke probe: offscreen-render the demo gradient and report
    /// whether it is non-blank (0 unavailable, 1 non-blank, 2 blank).
    ///
    /// It must not write `image`/`doc`: a document's display image is always
    /// derived from its composite, so a probe that stored the demo gradient in
    /// `image` presented garbage on the first paint (E1) until the next
    /// recomposite rebuilt the image from the white document.
    pub fn render_gpu(self: Pin<&mut Self>) -> i32 {
        let (width, height) = (512u32, 512u32);
        match crate::gpu::render_gradient(width, height) {
            crate::gpu::GpuRender::Unavailable => 0,
            crate::gpu::GpuRender::Rendered { distinct, .. } => {
                if distinct >= 2 {
                    1
                } else {
                    2
                }
            }
        }
    }

    pub fn gpu_interop_prepare(self: Pin<&mut Self>) -> bool {
        let state = crate::gpu::create_interop_state();
        let ok = state.is_some();
        self.rust_mut().interop = state;
        ok
    }

    pub fn gpu_vk_instance(&self) -> u64 {
        self.rust()
            .interop
            .as_ref()
            .map_or(0, |s| s.handles.instance)
    }

    pub fn gpu_vk_physical_device(&self) -> u64 {
        self.rust()
            .interop
            .as_ref()
            .map_or(0, |s| s.handles.physical_device)
    }

    pub fn gpu_vk_device(&self) -> u64 {
        self.rust().interop.as_ref().map_or(0, |s| s.handles.device)
    }

    pub fn gpu_vk_queue_family(&self) -> u32 {
        self.rust()
            .interop
            .as_ref()
            .map_or(0, |s| s.handles.queue_family)
    }

    pub fn gpu_vk_image(&self) -> u64 {
        self.rust().interop.as_ref().map_or(0, |s| s.image)
    }

    pub fn gpu_image_width(&self) -> u32 {
        self.rust().interop.as_ref().map_or(0, |s| s.width)
    }

    pub fn gpu_image_height(&self) -> u32 {
        self.rust().interop.as_ref().map_or(0, |s| s.height)
    }

    pub fn set_gpu_compute(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().gpu_compute = enabled;
        self.as_mut().recomposite();
    }

    pub fn gpu_compute(&self) -> bool {
        self.rust().gpu_compute
    }

    pub fn gpu_available(&self) -> bool {
        pictura_render::gpu_available()
    }

    pub fn active_backend(&self) -> QString {
        if !pictura_render::gpu_available() {
            QString::from("CPU (no GPU)")
        } else if self.rust().gpu_compute {
            QString::from("GPU")
        } else {
            QString::from("CPU")
        }
    }
}

impl super::PictureViewRust {
    /// Reset the per-edit transient state shared by `open` and `new_document`:
    /// drop the selection, history, in-progress stroke, and move-preview drag.
    /// The caller assigns `image`/`doc` before calling.
    fn reset_edit_state(&mut self) {
        self.selection = None;
        self.history = History::default();
        self.stroke = None;
        self.stroke_label.clear();
        self.move_base = None;
        self.move_layer = None;
        self.move_x = 0;
        self.move_y = 0;
        self.move_opacity = 0;
        self.display_dirty = false;
        self.link_sets.clear();
    }
}

impl qobject::PictureView {
    /// Snapshot the current state, capture it under `label`, and mark dirty.
    ///
    /// Callers must run their `recomposite`/`refresh_region` first, so the
    /// captured document's composite is the current rendered image.
    pub(super) fn record(mut self: Pin<&mut Self>, label: &str) {
        if let Some(snapshot) = self.snapshot() {
            let mut rust = self.as_mut().rust_mut();
            rust.history.capture(snapshot, label);
            rust.dirty = true;
            rust.content_revision = rust.content_revision.wrapping_add(1);
        }
    }

    /// As [`record`], but without bumping `content_revision`.
    ///
    /// Only for moves of the topmost pixel layer, which leave the composite
    /// below that layer unchanged, so a cached move-preview base stays valid.
    pub(super) fn record_move(mut self: Pin<&mut Self>, label: &str) {
        if let Some(snapshot) = self.snapshot() {
            let mut rust = self.as_mut().rust_mut();
            rust.history.capture(snapshot, label);
            rust.dirty = true;
        }
    }

    pub(super) fn snapshot(&self) -> Option<Snapshot> {
        let rust = self.rust();
        let doc = rust.doc.clone()?;
        Some(Snapshot {
            doc,
            selection: rust.selection.clone(),
        })
    }

    /// Composite only `rect`, patch the authoritative `doc.composite`, and emit
    /// [`region_blitted`] with a rectangle-sized image.
    ///
    /// The source is the active stroke's working document while painting, else
    /// the app document. While painting `doc.composite` is the pre-stroke base
    /// and is left alone (refreshed by `end_paint`'s full recomposite). An empty
    /// clamped rect is a no-op; the region path emits no `changed` and never
    /// rebuilds the full image.
    pub(super) fn refresh_region(mut self: Pin<&mut Self>, rect: PsdRect) {
        let gpu_compute = self.rust().gpu_compute;
        let (region, painting) = {
            let rust = self.rust();
            let painting = rust.stroke.is_some();
            let dims = rust
                .stroke
                .as_ref()
                .map(|stroke| (stroke.document().width, stroke.document().height))
                .or_else(|| rust.doc.as_ref().map(|doc| (doc.width, doc.height)));
            (
                dims.and_then(|(width, height)| clamp_region(rect, width, height)),
                painting,
            )
        };
        let Some((x0, y0, ..)) = region else {
            return;
        };
        let region_image = {
            let mut rust = self.as_mut().rust_mut();
            let rust = &mut *rust;
            let buffer = {
                let source: &Document = if painting {
                    rust.stroke.as_ref().unwrap().document()
                } else if let Some(doc) = rust.doc.as_ref() {
                    doc
                } else {
                    return;
                };
                pictura_render::composite_region_active(source, rect, gpu_compute).0
            };
            if buffer.width == 0 || buffer.height == 0 {
                return;
            }
            if !painting {
                if let Some(doc) = rust.doc.as_mut() {
                    patch_composite_region(doc, &buffer, x0, y0);
                }
            }
            rust.display_dirty = true;
            buffer_to_image(&buffer)
        };
        self.region_blitted(region_image, x0, y0);
    }

    /// Refresh `image` from the current document and emit [`changed`].
    ///
    /// The full-document path for every mutation that does not report a dirty
    /// rectangle; [`refresh_region`] is the incremental extension point.
    pub(super) fn recomposite(mut self: Pin<&mut Self>) {
        let gpu_compute = self.rust().gpu_compute;
        let rendered = self
            .rust()
            .doc
            .as_ref()
            .map(|doc| current_buffer(doc, gpu_compute));
        let Some(rendered) = rendered else {
            self.changed();
            return;
        };
        {
            let mut rust = self.as_mut().rust_mut();
            if let Some(doc) = rust.doc.as_mut() {
                store_composite(doc, &rendered);
            }
            rust.image = buffer_to_image(&rendered);
            rust.display_dirty = false;
        }
        self.changed();
    }
}
