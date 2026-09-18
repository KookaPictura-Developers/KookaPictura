use super::helpers::*;
use super::helpers_composite::*;
use crate::history::{History, Snapshot};
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LayerMask, LockFlags,
    PixelBuffer, PsdRect,
};
use pictura_select::Selection;

pub(super) fn pixel_layer(name: &str, w: u32, h: u32, rgb: (u8, u8, u8)) -> Layer {
    let n = (w * h) as usize;
    Layer {
        name: name.into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: h as i32,
            right: w as i32,
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
        channels: vec![
            Channel {
                id: 0,
                data: vec![rgb.0; n],
            },
            Channel {
                id: 1,
                data: vec![rgb.1; n],
            },
            Channel {
                id: 2,
                data: vec![rgb.2; n],
            },
            Channel {
                id: -1,
                data: vec![255; n],
            },
        ],
        children: Vec::new(),
        is_group: false,
    }
}

#[test]
fn converts_planar_rgb_to_rgba() {
    let mut doc = Document::new(2, 1, ColorMode::Rgb, BitDepth::Eight);
    doc.composite.data = vec![10, 20, 30, 40, 50, 60];
    let image = document_to_image(&doc, false);
    assert_eq!(image.width(), 2);
    assert_eq!(image.height(), 1);
    assert_eq!(image.pixel_color(0, 0).red(), 10);
    assert_eq!(image.pixel_color(0, 0).green(), 30);
    assert_eq!(image.pixel_color(0, 0).blue(), 50);
    assert_eq!(image.pixel_color(0, 0).alpha(), 255);
}

#[test]
fn clamp_region_matches_document_bounds() {
    let rect = |t, l, b, r| PsdRect {
        top: t,
        left: l,
        bottom: b,
        right: r,
    };
    assert_eq!(clamp_region(rect(0, 0, 4, 4), 4, 4), Some((0, 0, 4, 4)));
    // Partially outside clamps to the intersection.
    assert_eq!(clamp_region(rect(-2, -2, 3, 3), 4, 4), Some((0, 0, 3, 3)));
    // Entirely outside, and zero-area rects, are empty.
    assert_eq!(clamp_region(rect(10, 10, 12, 12), 4, 4), None);
    assert_eq!(clamp_region(rect(0, 0, 0, 4), 4, 4), None);
}

#[test]
fn layered_document_composites_with_source_alpha() {
    let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![Layer {
        name: "red".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 1,
            right: 1,
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
        channels: vec![
            Channel {
                id: 0,
                data: vec![255],
            },
            Channel {
                id: 1,
                data: vec![0],
            },
            Channel {
                id: 2,
                data: vec![0],
            },
            Channel {
                id: -1,
                data: vec![255],
            },
        ],
        children: Vec::new(),
        is_group: false,
    }];

    let image = document_to_image(&doc, false);
    assert_eq!(image.pixel_color(0, 0).red(), 255);
    assert_eq!(image.pixel_color(0, 0).alpha(), 255);
    // Uncovered canvas stays transparent, not the embedded composite.
    assert_eq!(image.pixel_color(1, 1).alpha(), 0);
}

#[test]
fn invert_and_visibility_change_composite() {
    let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![Layer {
        name: "red".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 1,
            right: 1,
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
        channels: vec![
            Channel {
                id: 0,
                data: vec![255],
            },
            Channel {
                id: 1,
                data: vec![0],
            },
            Channel {
                id: 2,
                data: vec![0],
            },
            Channel {
                id: -1,
                data: vec![255],
            },
        ],
        children: Vec::new(),
        is_group: false,
    }];

    let before = document_to_image(&doc, false);
    assert_eq!(before.pixel_color(0, 0).red(), 255);

    doc.layers
        .push(adjustment_layer("invert", None).expect("known kind"));
    let after = document_to_image(&doc, false);
    assert_eq!(after.pixel_color(0, 0).red(), 0);
    assert_eq!(after.pixel_color(0, 0).green(), 255);
    assert_eq!(after.pixel_color(0, 0).blue(), 255);

    doc.layers[0].visible = false;
    let hidden = document_to_image(&doc, false);
    assert_eq!(hidden.pixel_color(0, 0).alpha(), 0);

    assert!(adjustment_layer("bogus", None).is_none());
}

#[test]
fn row_thumbnails_respect_contents_and_masks() {
    let mut layer = pixel_layer("red", 2, 2, (255, 0, 0));
    layer.rect = PsdRect {
        top: 2,
        left: 2,
        bottom: 4,
        right: 4,
    };

    let positioned = layer_thumbnail_positioned(&layer, 4, 4, 4).expect("entire-document square");
    assert_eq!((positioned.width(), positioned.height()), (4, 4));
    assert_eq!(positioned.pixel_color(2, 2).red(), 255);
    assert_eq!(positioned.pixel_color(2, 2).alpha(), 255);
    assert_eq!(
        positioned.pixel_color(0, 0).alpha(),
        0,
        "document is empty outside the layer"
    );

    let bounds = layer_thumbnail_image(&layer, 4).expect("layer-bounds thumbnail");
    assert_eq!(
        (bounds.width(), bounds.height()),
        (4, 4),
        "layer bounds fill the square"
    );
    assert_eq!(bounds.pixel_color(0, 0).red(), 255);

    let mask = LayerMask {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 2,
            right: 2,
        },
        default_color: 255,
        disabled: false,
        flags: 0,
        data: Some(vec![0, 128, 255, 64]),
    };
    let mask_image = mask_thumbnail_image(&mask, 4).expect("mask thumbnail");
    assert_eq!((mask_image.width(), mask_image.height()), (4, 4));
    assert_eq!(mask_image.pixel_color(0, 0).red(), 0);
    assert_eq!(mask_image.pixel_color(3, 3).red(), 64);
}

#[test]
fn move_preview_base_equals_full_composite_with_layer_hidden() {
    let mut doc = Document::new(64, 64, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        pixel_layer("base", 64, 64, (30, 60, 90)),
        pixel_layer("top", 32, 32, (200, 100, 50)),
    ];
    let index = topmost_pixel_layer_index(&doc).expect("pixel layer");
    let rect = doc.layers[index].rect;
    assert_eq!(
        (rect.left, rect.top, rect.right, rect.bottom),
        (0, 0, 32, 32)
    );

    // The authoritative composite holds the layer visible, as M34 keeps it.
    let rendered = current_buffer(&doc, false);
    store_composite(&mut doc, &rendered);

    // (a) Full composite with the topmost layer hidden.
    doc.layers[index].visible = false;
    let hidden = document_to_image(&doc, false);
    // (b) With the layer still hidden, region-composite its rect.
    let (buffer, _) = pictura_render::composite_region_active(&doc, rect, false);
    doc.layers[index].visible = true;
    let (x0, y0, w, h) = clamp_region(rect, doc.width, doc.height).expect("in-bounds");
    assert_eq!((buffer.width, buffer.height), (w, h));

    // Patch that region into a planar clone of the composite: the bridge path.
    let mut base_buffer = doc.composite.clone();
    patch_buffer_region(&mut base_buffer, &buffer, x0, y0);
    let base = buffer_to_image(&base_buffer);

    assert_eq!(
        (base.width(), base.height()),
        (hidden.width(), hidden.height())
    );
    for y in 0..hidden.height() {
        for x in 0..hidden.width() {
            assert_eq!(
                base.pixel_color(x, y),
                hidden.pixel_color(x, y),
                "pixel ({x},{y}) differs"
            );
        }
    }
}

#[test]
fn move_preview_region_guards_empty_and_missing_composite() {
    let rect = |t, l, b, r| PsdRect {
        top: t,
        left: l,
        bottom: b,
        right: r,
    };
    // An in-bounds rect takes the region path.
    assert_eq!(
        move_preview_region(rect(0, 0, 64, 64), 64, 64, true),
        Some((0, 0, 64, 64))
    );
    // A rect larger than the old 1 MP budget no longer falls back.
    assert_eq!(
        move_preview_region(rect(0, 0, 2000, 2000), 4000, 4000, true),
        Some((0, 0, 2000, 2000))
    );
    // A missing/mismatched composite falls back.
    assert_eq!(move_preview_region(rect(0, 0, 64, 64), 64, 64, false), None);
    // An empty rect (here, zero area after clamping) falls back.
    assert_eq!(move_preview_region(rect(0, 0, 0, 64), 64, 64, true), None);
}

#[test]
fn layer_visibility_region_bounds_raster_and_bounded_adjustments() {
    // A raster layer is bounded by its own rect (clamped by refresh_region).
    let raster = pixel_layer("raster", 8, 4, (10, 20, 30));
    assert_eq!(layer_visibility_region(&raster), Some(raster.rect));

    let mask = |t, l, b, r, data: Option<Vec<u8>>, default_color, disabled| LayerMask {
        rect: PsdRect {
            top: t,
            left: l,
            bottom: b,
            right: r,
        },
        default_color,
        disabled,
        flags: 0,
        data,
    };

    // An adjustment with an enabled, data-carrying, zero-default mask is
    // bounded by its mask rect.
    let bounded = adjustment_layer(
        "invert",
        Some(mask(2, 3, 6, 9, Some(vec![128; 4 * 6]), 0, false)),
    )
    .expect("known kind");
    assert_eq!(
        layer_visibility_region(&bounded),
        Some(PsdRect {
            top: 2,
            left: 3,
            bottom: 6,
            right: 9,
        })
    );

    // No mask, a disabled mask, missing mask data, and a non-zero default
    // are all unbounded: full recomposite.
    let unmasked = adjustment_layer("invert", None).expect("known kind");
    assert_eq!(layer_visibility_region(&unmasked), None);
    let disabled = adjustment_layer(
        "invert",
        Some(mask(0, 0, 4, 4, Some(vec![128; 16]), 0, true)),
    )
    .expect("known kind");
    assert_eq!(layer_visibility_region(&disabled), None);
    let no_data =
        adjustment_layer("invert", Some(mask(0, 0, 4, 4, None, 0, false))).expect("known kind");
    assert_eq!(layer_visibility_region(&no_data), None);
    let non_zero = adjustment_layer(
        "invert",
        Some(mask(0, 0, 4, 4, Some(vec![128; 16]), 255, false)),
    )
    .expect("known kind");
    assert_eq!(layer_visibility_region(&non_zero), None);

    // A group is unbounded.
    let mut group = pixel_layer("group", 4, 4, (0, 0, 0));
    group.is_group = true;
    assert_eq!(layer_visibility_region(&group), None);
}

#[test]
fn selection_becomes_full_frame_mask_that_confines_adjustment() {
    let mut doc = Document::new(4, 1, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![pixel_layer("base", 4, 1, (255, 0, 0))];

    let selection = Selection {
        width: 4,
        height: 1,
        data: vec![255, 255, 0, 0],
    };
    let mask = selection_to_mask(&selection, &doc);
    assert_eq!(mask.rect.right, 4);
    assert_eq!(mask.rect.bottom, 1);
    assert_eq!(mask.data.as_deref(), Some(&[255u8, 255, 0, 0][..]));

    doc.layers
        .push(adjustment_layer("invert", Some(mask)).expect("known kind"));
    let image = document_to_image(&doc, false);
    // Selected half inverts red -> cyan; unselected half is untouched.
    assert_eq!(image.pixel_color(0, 0).red(), 0);
    assert_eq!(image.pixel_color(0, 0).green(), 255);
    assert_eq!(image.pixel_color(0, 0).blue(), 255);
    assert_eq!(image.pixel_color(3, 0).red(), 255);
    assert_eq!(image.pixel_color(3, 0).blue(), 0);
}

#[test]
fn test_image_is_not_null() {
    assert!(!test_image().is_null());
}

#[test]
fn store_composite_stores_rgba_for_rgb_and_keeps_grayscale_plane() {
    let mut rgb = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
    rgb.layers = vec![pixel_layer("base", 8, 8, (30, 60, 90))];
    let rendered = current_buffer(&rgb, false);
    assert_eq!(rendered.channels, 4);
    store_composite(&mut rgb, &rendered);
    assert_eq!(rgb.composite.channels, 4);
    assert_eq!(rgb.composite.data, rendered.data);

    let mut gray = Document::new(8, 8, ColorMode::Grayscale, BitDepth::Eight);
    gray.layers = vec![Layer {
        name: "gray".into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 8,
            right: 8,
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
        channels: vec![
            Channel {
                id: 0,
                data: vec![120; 64],
            },
            Channel {
                id: -1,
                data: vec![255; 64],
            },
        ],
        children: Vec::new(),
        is_group: false,
    }];
    let rendered = current_buffer(&gray, false);
    assert_eq!(rendered.channels, 4);
    store_composite(&mut gray, &rendered);
    assert_eq!(gray.composite.channels, 1, "grayscale stays one plane");
    assert_eq!(
        gray.composite.data,
        rendered.data[..64],
        "stored grey plane"
    );
}

#[test]
fn undo_display_from_snapshot_composite_equals_full_recomposite() {
    let mut doc = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![
        pixel_layer("base", 16, 16, (30, 60, 90)),
        pixel_layer("top", 6, 6, (200, 100, 50)),
    ];
    let rendered = current_buffer(&doc, false);
    store_composite(&mut doc, &rendered);
    let open = Snapshot {
        doc: doc.clone(),
        selection: None,
    };
    let mut history = History::default();
    history.capture(open, "Open");

    // A full-composite mutation, captured after the store.
    assert!(pictura_render::translate_layer(&mut doc, 3, 2));
    let rendered = current_buffer(&doc, false);
    store_composite(&mut doc, &rendered);
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Move Layer",
    );

    // A region-refresh mutation (`refresh_region`'s non-painting body).
    let before = doc.layers.last().unwrap().rect;
    assert!(pictura_render::translate_layer_rect(&mut doc, 2, 1));
    let after = doc.layers.last().unwrap().rect;
    let dirty = union_rect(before, after);
    let (x0, y0, w, h) = clamp_region(dirty, doc.width, doc.height).expect("in-bounds");
    assert_eq!((w, h), (dirty.width() as u32, dirty.height() as u32));
    let (buffer, _) = pictura_render::composite_region_active(&doc, dirty, false);
    assert_eq!((buffer.width, buffer.height), (w, h));
    patch_composite_region(&mut doc, &buffer, x0, y0);
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Move Layer",
    );

    // Walk the undo stack; every restored display must be the snapshot
    // composite and equal a full recomposite of the restored document.
    let mut undone = 0;
    while let Some(snapshot) = history.undo() {
        let display = buffer_to_image(&snapshot.doc.composite);
        let full = document_to_image(&snapshot.doc, false);
        assert_eq!(
            display, full,
            "undo display differs from a full recomposite"
        );
        undone += 1;
    }
    assert_eq!(undone, 2);
    // And back up the redo stack.
    let mut redone = 0;
    while let Some(snapshot) = history.redo() {
        let display = buffer_to_image(&snapshot.doc.composite);
        let full = document_to_image(&snapshot.doc, false);
        assert_eq!(
            display, full,
            "redo display differs from a full recomposite"
        );
        redone += 1;
    }
    assert_eq!(redone, 2);
}

/// The region path patches `doc.composite` in place (no FFI) and the image
/// rebuilt from it equals a full recomposite, across several refreshes.
#[test]
fn region_refresh_keeps_composite_and_rebuilt_image_equal_to_full() {
    let mut doc = Document::new(32, 32, ColorMode::Rgb, BitDepth::Eight);
    doc.composite.data.fill(255);
    doc.layers = vec![
        pixel_layer("base", 32, 32, (30, 60, 90)),
        pixel_layer("top", 8, 8, (200, 100, 50)),
    ];
    let rendered = current_buffer(&doc, false);
    store_composite(&mut doc, &rendered);

    // Replay `refresh_region`'s non-painting body for a sequence of moves:
    // patch the composite, then rebuild the display image from it as
    // `image()` does when dirty.
    for (dx, dy) in [(2, 1), (1, 3), (0, 2), (3, 0)] {
        let before = doc.layers.last().unwrap().rect;
        assert!(pictura_render::translate_layer_rect(&mut doc, dx, dy));
        let after = doc.layers.last().unwrap().rect;
        let dirty = union_rect(before, after);
        let (x0, y0, w, h) = clamp_region(dirty, doc.width, doc.height).expect("in-bounds");
        let (buffer, _) = pictura_render::composite_region_active(&doc, dirty, false);
        assert_eq!((buffer.width, buffer.height), (w, h));
        patch_composite_region(&mut doc, &buffer, x0, y0);
        let image = buffer_to_image(&doc.composite);
        let full = document_to_image(&doc, false);
        assert_eq!(
            image, full,
            "region-refreshed image differs after move ({dx},{dy})"
        );
    }
}

#[test]
fn sample_planar_argb_matches_buffer_to_image_for_every_plane_count() {
    for channels in 1..=4u8 {
        let mut buffer = PixelBuffer::new(3, 2, channels);
        let plane = 3 * 2;
        for c in 0..channels as usize {
            for i in 0..plane {
                buffer.data[c * plane + i] = ((c * 37 + i * 11 + 5) % 256) as u8;
            }
        }
        let image = buffer_to_image(&buffer);
        for y in 0..2 {
            for x in 0..3 {
                let color = image.pixel_color(x, y);
                let expect = ((color.alpha() as u32) << 24)
                    | ((color.red() as u32) << 16)
                    | ((color.green() as u32) << 8)
                    | (color.blue() as u32);
                assert_eq!(
                    sample_planar_argb(&buffer, x, y),
                    expect,
                    "channels={channels} pixel ({x},{y})"
                );
            }
        }
        assert_eq!(sample_planar_argb(&buffer, -1, 0), 0);
        assert_eq!(sample_planar_argb(&buffer, 0, 2), 0);
    }
}

#[test]
fn save_after_edit_serializes_the_current_composite() {
    let mut doc = Document::new(8, 8, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![pixel_layer("base", 8, 8, (40, 40, 40))];
    let rendered = current_buffer(&doc, false);
    store_composite(&mut doc, &rendered);
    let plane = 64usize;
    let before = doc.composite.data[..3 * plane].to_vec();

    // Edit every colour channel, then make the composite current (the app's
    // `recomposite`). The pre-edit merged image is no longer the composite.
    let layer = doc.layers.last_mut().unwrap();
    for ch in layer.channels.iter_mut().filter(|c| c.id >= 0) {
        for byte in ch.data.iter_mut() {
            *byte = byte.wrapping_add(100);
        }
    }
    let rendered = current_buffer(&doc, false);
    store_composite(&mut doc, &rendered);
    assert_eq!(doc.composite.channels, 4);
    let edited = doc.composite.data[..3 * plane].to_vec();
    assert_ne!(edited, before, "the edit must change the composite");

    let bytes = pictura_codec::write_psd(&doc).expect("write psd");
    let back = pictura_codec::read_psd(&bytes).expect("read psd");
    assert_eq!(
        back.composite.channels, 3,
        "RGB read-back is the mode's planes"
    );
    assert_eq!(
        back.composite.data, edited,
        "reopened composite is the edited render, not the pre-edit one"
    );
}
