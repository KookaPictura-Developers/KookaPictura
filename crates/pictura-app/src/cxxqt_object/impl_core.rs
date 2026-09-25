use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use crate::history::{History, Snapshot};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString, QStringList};
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LockFlags, PsdRect,
};

/// Finish an imported raster document. When every decoded pixel is opaque the
/// single `from_rgba` layer becomes the locked `Background` and its redundant
/// alpha channel is dropped; otherwise the regular alpha layer named from the
/// file stem is left untouched.
pub(super) fn finalize_import(doc: &mut Document, rgba: &[u8]) {
    if !rgba.as_chunks::<4>().0.iter().all(|px| px[3] == 255) {
        return;
    }
    let Some(layer) = doc.layers.last_mut() else {
        return;
    };
    layer.name = "Background".to_string();
    layer.background = true;
    layer.lock = LockFlags::default()
        .with(LockFlags::TRANSPARENCY, true)
        .with(LockFlags::POSITION, true);
    layer.channels.retain(|channel| channel.id != -1);
}

/// A built-in working profile by the command's index: 0 sRGB, 1 Adobe RGB,
/// 2 Pro Photo RGB; `None` for any other index.
fn builtin_profile(index: i32) -> Option<pictura_codec::Profile> {
    match index {
        0 => Some(pictura_codec::Profile::srgb()),
        1 => Some(pictura_codec::Profile::adobe_rgb()),
        2 => Some(pictura_codec::Profile::pro_photo()),
        _ => None,
    }
}

impl qobject::PictureView {
    pub fn open(self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let policy = self.rust().color_policy;
        let mut loaded = std::fs::read(&path)
            .ok()
            .and_then(|bytes| pictura_codec::read_psd_with(&bytes, policy).ok());

        let ok = loaded.is_some();
        let gpu_compute = self.rust().gpu_compute;
        let rendered = loaded.as_ref().map(|doc| current_buffer(doc, gpu_compute));
        if let (Some(doc), Some(rendered)) = (loaded.as_mut(), rendered.as_ref()) {
            store_composite(doc, rendered);
        }
        let image = match (loaded.as_ref(), rendered.as_ref()) {
            (Some(doc), Some(rendered)) => {
                buffer_to_image(&pictura_codec::buffer_to_srgb(doc, rendered))
            }
            _ => test_image(),
        };
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = loaded;
        view.reset_edit_state();
        view.active_layer = view
            .doc
            .as_ref()
            .and_then(|doc| topmost_pixel_layer_index(doc).map(|i| i.to_string()));
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
        finalize_import(&mut doc, &rgba);
        let gpu_compute = self.rust().gpu_compute;
        let rendered = current_buffer(&doc, gpu_compute);
        store_composite(&mut doc, &rendered);
        let image = buffer_to_image(&rendered);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = Some(doc);
        view.reset_edit_state();
        view.active_layer = Some("0".to_string());
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
        view.active_layer = Some("0".to_string());
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
        view.active_layer = Some("0".to_string());
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

    pub fn save(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let dirty = self.rust().dirty;
        let bytes = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            if dirty {
                pictura_render::refresh_native_composite(doc);
            }
            let Ok(bytes) = pictura_codec::write_psd(doc) else {
                return false;
            };
            bytes
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

    pub fn mode_notice(&self) -> QString {
        let Some(doc) = self.rust().doc.as_ref() else {
            return QString::default();
        };
        let Some(mode) = doc.source_mode else {
            return QString::default();
        };
        let name = match mode {
            ColorMode::Bitmap => "Bitmap",
            ColorMode::Indexed => "Indexed",
            ColorMode::Cmyk => "CMYK",
            ColorMode::Lab => "Lab",
            _ => return QString::default(),
        };
        // An 8-bit Lab, CMYK, Indexed, or flat Bitmap source re-encodes as its
        // source mode on save, and so does a 16/32-bit Lab or CMYK source whose
        // depth was retained; every other converted mode writes RGB.
        // ponytail: the edit and the layer/extra-channel fallback are not
        // consulted, so an edited Indexed or a layered/edited Bitmap document
        // still claims its source mode though the writer then falls back to RGB;
        // calling the codec's unchanged predicate live would be the upgrade.
        let preserves_source = matches!(
            mode,
            ColorMode::Lab | ColorMode::Cmyk | ColorMode::Indexed | ColorMode::Bitmap
        ) && (doc.source_depth.is_none() || doc.retains_source_depth());
        let saved = if preserves_source { name } else { "RGB" };
        QString::from(format!("Converted from {name}; saved as {saved}"))
    }

    pub fn depth_notice(&self) -> QString {
        let Some(doc) = self.rust().doc.as_ref() else {
            return QString::default();
        };
        let Some(depth) = doc.source_depth else {
            return QString::default();
        };
        let name = match depth {
            BitDepth::Sixteen => "16-bit",
            BitDepth::ThirtyTwo => "32-bit",
            _ => return QString::default(),
        };
        // A retained Grayscale/RGB/Lab/CMYK read re-emits the source depth on
        // save; every other converted mode keeps the plain conversion notice
        // because its save is 8-bit.
        if doc.retains_source_depth() {
            QString::from(format!(
                "Converted from {name} to 8-bit for editing; saved at {name}"
            ))
        } else {
            QString::from(format!("Converted from {name}"))
        }
    }

    pub fn icc_notice(&self) -> QString {
        let Some(icc) = self
            .rust()
            .doc
            .as_ref()
            .and_then(|d| d.source_icc.as_deref())
        else {
            return QString::default();
        };
        match pictura_codec::profile_description(icc) {
            Some(name) => QString::from(format!("Converted from ICC profile {name}")),
            None => QString::from("Converted from embedded ICC profile"),
        }
    }

    pub fn exif_rows(&self) -> QStringList {
        let Some(doc) = self.rust().doc.as_ref() else {
            return QStringList::default();
        };
        pictura_codec::read_metadata(doc)
            .exif
            .entries()
            .iter()
            .map(|(tag, value)| {
                let label = pictura_codec::exif_tag_name(*tag)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("Tag 0x{tag:04X}"));
                QString::from(format!("{label}\t{}", value.display()))
            })
            .collect()
    }

    pub fn iptc_rows(&self) -> QStringList {
        let Some(doc) = self.rust().doc.as_ref() else {
            return QStringList::default();
        };
        pictura_codec::read_metadata(doc)
            .iptc
            .records()
            .iter()
            .map(|(record, dataset, value)| {
                let label = pictura_codec::iptc_field_name(*record, *dataset)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("{record}:{dataset}"));
                let text = String::from_utf8_lossy(value);
                QString::from(format!("{label}\t{text}"))
            })
            .collect()
    }

    pub fn xmp_packet(&self) -> QString {
        self.rust()
            .doc
            .as_ref()
            .map(|doc| QString::from(pictura_codec::read_metadata(doc).xmp))
            .unwrap_or_default()
    }

    /// The parsed XMP properties as `"Label\tValue"` rows, omitting absent ones.
    pub fn xmp_rows(&self) -> QStringList {
        let Some(doc) = self.rust().doc.as_ref() else {
            return QStringList::default();
        };
        let props = pictura_codec::xmp_properties(doc);
        let mut rows: Vec<(&str, String)> = Vec::new();
        if let Some(title) = &props.title {
            rows.push(("Title", title.clone()));
        }
        if !props.creator.is_empty() {
            rows.push(("Creator", props.creator.join(", ")));
        }
        if let Some(description) = &props.description {
            rows.push(("Description", description.clone()));
        }
        if !props.subject.is_empty() {
            rows.push(("Subject", props.subject.join(", ")));
        }
        if let Some(rights) = &props.rights {
            rows.push(("Rights", rights.clone()));
        }
        if let Some(credit) = &props.credit {
            rows.push(("Credit", credit.clone()));
        }
        if let Some(source) = &props.source {
            rows.push(("Source", source.clone()));
        }
        if let Some(headline) = &props.headline {
            rows.push(("Headline", headline.clone()));
        }
        if let Some(marked) = props.marked {
            rows.push(("Marked", if marked { "Yes" } else { "No" }.to_string()));
        }
        rows.into_iter()
            .map(|(label, value)| QString::from(format!("{label}\t{value}")))
            .collect()
    }

    /// The six editable IPTC core fields as `"record:dataset\tLabel\tValue"`.
    pub fn iptc_edit_fields(&self) -> QStringList {
        const EDITABLE: &[(u8, u8)] = &[(2, 5), (2, 80), (2, 116), (2, 120), (2, 110), (2, 115)];
        let Some(doc) = self.rust().doc.as_ref() else {
            return QStringList::default();
        };
        let iptc = pictura_codec::read_metadata(doc).iptc;
        EDITABLE
            .iter()
            .map(|(record, dataset)| {
                let label = pictura_codec::iptc_field_name(*record, *dataset).unwrap_or("IPTC");
                let value = iptc.text(*record, *dataset).unwrap_or_default();
                QString::from(format!("{record}:{dataset}\t{label}\t{value}"))
            })
            .collect()
    }

    /// Apply `"record:dataset\tValue"` edits to the document as one undo state,
    /// writing the shared IPTC-Core fields to both XMP and IIM. Returns whether
    /// anything changed.
    pub fn apply_metadata_edits(mut self: Pin<&mut Self>, edits: &QStringList) -> bool {
        let mut fields = Vec::new();
        for row in edits.iter() {
            let row = row.to_string();
            let Some((id, value)) = row.split_once('\t') else {
                continue;
            };
            let Some((record, dataset)) = id.split_once(':') else {
                continue;
            };
            let (Ok(record), Ok(dataset)) = (record.parse::<u8>(), dataset.parse::<u8>()) else {
                continue;
            };
            fields.push((record, dataset, value.to_string()));
        }
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_codec::set_file_info_fields(doc, &fields),
            None => false,
        };
        if changed {
            self.as_mut().record("File Info");
            self.changed();
        }
        changed
    }

    /// Export the active document's managed XMP properties to `dest` as a
    /// standalone template. Read-only: no mutation and no history state; false
    /// without a document or on a write error.
    pub fn export_metadata_template(&self, dest: &QString) -> bool {
        let Some(doc) = self.rust().doc.as_ref() else {
            return false;
        };
        std::fs::write(dest.to_string(), pictura_codec::export_template(doc)).is_ok()
    }

    /// Apply the XMP template at `path` to the active document with `mode`
    /// (0 Append, 1 Replace, 2 KeepOriginalReplaceMatching): one
    /// "Metadata Template" state when anything changes. False without a
    /// document, on a read error, or when nothing changed.
    pub fn apply_metadata_template(mut self: Pin<&mut Self>, path: &QString, mode: i32) -> bool {
        use pictura_codec::MergeMode;
        let mode = match mode {
            0 => MergeMode::Append,
            1 => MergeMode::Replace,
            2 => MergeMode::KeepOriginalReplaceMatching,
            _ => return false,
        };
        let Ok(text) = std::fs::read_to_string(path.to_string()) else {
            return false;
        };
        let template = pictura_codec::parse_xmp(&text);
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_codec::apply_template(doc, &template, mode),
            None => false,
        };
        if changed {
            self.as_mut().record("Metadata Template");
            self.changed();
        }
        changed
    }

    /// Assign a built-in working profile to the active document (retag only,
    /// pixels untouched): 0 sRGB, 1 Adobe RGB, 2 Pro Photo RGB. Recomposites and
    /// records one "Assign Profile" state; false without a document or on a bad
    /// index.
    pub fn assign_profile(mut self: Pin<&mut Self>, profile_index: i32) -> bool {
        let Some(target) = builtin_profile(profile_index) else {
            return false;
        };
        {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            if doc.mode != ColorMode::Rgb {
                return false;
            }
            pictura_codec::assign_document_profile(doc, &target);
        }
        self.as_mut().recomposite();
        self.as_mut().record("Assign Profile");
        true
    }

    /// Convert the active document's composite and layer color channels to a
    /// built-in destination: 0 sRGB, 1 Adobe RGB, 2 Pro Photo RGB. Recomposites
    /// and records one "Convert to Profile" state; false without a document, on
    /// a bad index, or when the current profile cannot be parsed.
    pub fn convert_profile(mut self: Pin<&mut Self>, profile_index: i32) -> bool {
        let Some(dst) = builtin_profile(profile_index) else {
            return false;
        };
        {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            if doc.mode != ColorMode::Rgb {
                return false;
            }
            if !pictura_codec::convert_document(doc, &dst) {
                return false;
            }
        }
        self.as_mut().recomposite();
        self.as_mut().record("Convert to Profile");
        true
    }

    pub fn document_mode(&self) -> QString {
        match self.rust().doc.as_ref().map(|d| d.mode) {
            Some(ColorMode::Grayscale) => QString::from("grayscale"),
            Some(_) => QString::from("rgb"),
            None => QString::default(),
        }
    }

    pub fn document_depth_bits(&self) -> i32 {
        match self.rust().doc.as_ref().map(|d| d.depth) {
            Some(BitDepth::Eight) => 8,
            Some(BitDepth::Sixteen) => 16,
            Some(BitDepth::ThirtyTwo) => 32,
            Some(BitDepth::One) => 1,
            None => 0,
        }
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

    /// Record the panel's selection as the active layer. An empty `path` means
    /// no single selection, disabling tool edits until one layer is selected.
    pub fn set_active_layer(mut self: Pin<&mut Self>, path: &QString) {
        let path = path.to_string();
        self.as_mut().rust_mut().active_layer = (!path.is_empty()).then_some(path);
    }

    /// The panel path of the active layer, or empty when none is active.
    pub fn active_layer_path(&self) -> QString {
        self.rust()
            .active_layer
            .as_deref()
            .map(QString::from)
            .unwrap_or_default()
    }

    /// Whether the single active layer a tool edit may target is visible; true
    /// when there is no single editable active layer (nothing to refuse).
    pub fn active_layer_visible(&self) -> bool {
        let rust = self.rust();
        rust.doc
            .as_ref()
            .is_none_or(|doc| active_layer_visible(doc, rust.active_layer.as_deref()))
    }

    pub fn set_gpu_compute(mut self: Pin<&mut Self>, enabled: bool) {
        self.as_mut().rust_mut().gpu_compute = enabled;
        self.as_mut().recomposite();
    }

    pub fn gpu_compute(&self) -> bool {
        self.rust().gpu_compute
    }

    pub fn set_color_policy(mut self: Pin<&mut Self>, code: i32) {
        if let Some(policy) = pictura_codec::Policy::from_code(code) {
            self.as_mut().rust_mut().color_policy = policy;
        }
    }

    pub fn color_policy(&self) -> i32 {
        self.rust().color_policy.to_code()
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

    /// Brush size/hardness step for a `[`/`]` key event, or 0. See
    /// [`brush_shortcut_delta`].
    pub fn brush_shortcut_delta(&self, key: i32, scan: u32, shift: bool, paint: bool) -> i32 {
        brush_shortcut_delta(key, scan, shift, paint)
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
        self.active_layer = None;
        self.move_base = None;
        self.move_layer = None;
        self.move_x = 0;
        self.move_y = 0;
        self.move_opacity = 0;
        self.transform_session = None;
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
            let (buffer, image) = {
                let source: &Document = if painting {
                    rust.stroke.as_ref().unwrap().document()
                } else if let Some(doc) = rust.doc.as_ref() {
                    doc
                } else {
                    return;
                };
                let buffer = pictura_render::composite_region_active(source, rect, gpu_compute).0;
                let image = (buffer.width != 0 && buffer.height != 0)
                    .then(|| buffer_to_image(&pictura_codec::buffer_to_srgb(source, &buffer)));
                (buffer, image)
            };
            let Some(image) = image else {
                return;
            };
            if !painting {
                if let Some(doc) = rust.doc.as_mut() {
                    patch_composite_region(doc, &buffer, x0, y0);
                }
            }
            rust.display_dirty = true;
            image
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
            rust.image = match rust.doc.as_ref() {
                Some(doc) => buffer_to_image(&pictura_codec::buffer_to_srgb(doc, &rendered)),
                None => buffer_to_image(&rendered),
            };
            rust.display_dirty = false;
        }
        self.changed();
    }
}
