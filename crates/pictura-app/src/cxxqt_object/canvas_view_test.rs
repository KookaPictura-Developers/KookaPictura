use super::helpers_composite::*;
use super::PictureViewRust;
use pictura_core::{BitDepth, ColorMode, Document, PixelBuffer, PsdRect};
use pictura_render::{CanvasDamage, ViewPyramid};

/// A 4-plane RGBA buffer with variation in every channel, so a level mismatch
/// cannot hide behind uniform pixels.
fn varied_rgba(width: u32, height: u32, seed: u32) -> PixelBuffer {
    let n = (width * height) as usize;
    let mut data = vec![0u8; n * 4];
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            let v = ((x * 7 + y * 13 + seed) % 251) as u8;
            data[i] = v;
            data[n + i] = v.wrapping_mul(3);
            data[2 * n + i] = 255 - v;
            data[3 * n + i] = (60 + (x * 3 + y) % 196) as u8;
        }
    }
    PixelBuffer {
        width,
        height,
        channels: 4,
        data,
    }
}

/// Copy `rect` of `src` into `dst`, plane by plane.
fn blit_region(dst: &mut PixelBuffer, src: &PixelBuffer, rect: PsdRect) {
    let (dplane, splane) = (dst.pixel_count(), src.pixel_count());
    for y in rect.top..rect.bottom {
        for x in rect.left..rect.right {
            let di = y as usize * dst.width as usize + x as usize;
            let si = y as usize * src.width as usize + x as usize;
            for c in 0..4 {
                dst.data[c * dplane + di] = src.data[c * splane + si];
            }
        }
    }
}

/// Copy `rect` of `src` into a new 4-plane buffer.
fn crop_buffer(src: &PixelBuffer, rect: PsdRect) -> PixelBuffer {
    let (w, h) = (rect.width() as usize, rect.height() as usize);
    let splane = src.pixel_count();
    let dplane = w * h;
    let mut data = vec![0u8; dplane * 4];
    for y in 0..h {
        for x in 0..w {
            let si = (rect.top as usize + y) * src.width as usize + rect.left as usize + x;
            let di = y * w + x;
            for c in 0..4 {
                data[c * dplane + di] = src.data[c * splane + si];
            }
        }
    }
    PixelBuffer {
        width: w as u32,
        height: h as u32,
        channels: 4,
        data,
    }
}

fn whole_level(pyramid: &ViewPyramid, level: usize) -> PsdRect {
    let (w, h) = pyramid.level_size(level);
    PsdRect {
        top: 0,
        left: 0,
        bottom: h as i32,
        right: w as i32,
    }
}

fn assert_pyramids_equal(a: &ViewPyramid, b: &ViewPyramid, source: &PixelBuffer) {
    assert_eq!(a.level_count(), b.level_count(), "level count");
    let planes = planes_of(source).expect("RGBA source");
    for level in 0..a.level_count() {
        let rect = whole_level(a, level);
        assert_eq!(
            a.crop(planes, level, rect).data(),
            b.crop(planes, level, rect).data(),
            "level {level}"
        );
    }
}

#[test]
fn display_conversion_premultiplies_and_straight_helpers_do_not() {
    let buffer = PixelBuffer {
        width: 2,
        height: 1,
        channels: 4,
        data: vec![200, 255, 100, 255, 50, 255, 128, 0],
    };
    assert_eq!(
        buffer_to_rgba_bytes(&buffer),
        vec![200, 100, 50, 128, 255, 255, 255, 0],
        "straight helper is unchanged"
    );
    assert_eq!(
        display_rgba_bytes(&buffer),
        vec![100, 50, 25, 128, 0, 0, 0, 0],
        "display pixel is round(c*a/255) and alpha is kept"
    );
    let image = premultiplied_display_image(&buffer);
    assert_eq!(
        image.format(),
        cxx_qt_lib::QImageFormat::Format_RGBA8888_Premultiplied
    );
    assert_eq!(image.pixel_color(0, 0).alpha(), 128);
    assert_eq!(image.pixel_color(1, 0).alpha(), 0);
}

#[test]
fn level0_region_update_matches_a_rebuild() {
    let mut doc = Document::new(600, 400, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = varied_rgba(600, 400, 0);
    let mut rust = PictureViewRust {
        doc: Some(doc.clone()),
        ..Default::default()
    };
    rust.reset_pyramid();

    let reference = {
        let level0 = rust.level0.as_ref().expect("sRGB level-0 frame");
        ViewPyramid::rebuild(planes_of(level0).unwrap())
    };
    let level0 = rust.level0.as_ref().expect("sRGB level-0 frame");
    assert_pyramids_equal(&rust.pyramid, &reference, level0);

    // A described region change, applied to the app's own composite and folded
    // into the cached level-0 frame the way `refresh_region` does.
    let rect = PsdRect {
        top: 100,
        left: 120,
        bottom: 180,
        right: 260,
    };
    let changed = varied_rgba(600, 400, 77);
    blit_region(&mut doc.composite, &changed, rect);
    rust.doc = Some(doc.clone());
    rust.refresh_level0_region(crop_buffer(&changed, rect), rect.left, rect.top);
    rust.update_pyramid(rect);

    let level0 = rust.level0.as_ref().expect("sRGB level-0 frame");
    let reference = ViewPyramid::rebuild(planes_of(level0).unwrap());
    assert_pyramids_equal(&rust.pyramid, &reference, level0);
}

#[test]
fn level0_carries_the_document_profile_conversion() {
    let mut doc = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = varied_rgba(16, 16, 4);
    pictura_codec::assign_document_profile(&mut doc, &pictura_codec::Profile::adobe_rgb());

    let expected = level0_buffer(&doc);
    let raw = rgba_frame(&doc.composite);
    assert_ne!(
        expected.data, raw.data,
        "a non-identity profile must convert the frame"
    );

    let mut rust = PictureViewRust {
        doc: Some(doc),
        ..Default::default()
    };
    rust.reset_pyramid();
    let level0 = rust.level0.as_ref().expect("sRGB level-0 frame");
    assert_eq!(
        level0.data, expected.data,
        "the crop source is color managed"
    );
}

#[test]
fn region_refresh_reuses_the_level0_frame_when_it_matches() {
    let mut doc = Document::new(400, 300, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = varied_rgba(400, 300, 0);
    let mut rust = PictureViewRust {
        doc: Some(doc.clone()),
        ..Default::default()
    };
    rust.reset_pyramid();
    let before = rust
        .level0
        .as_ref()
        .expect("sRGB level-0 frame")
        .data
        .as_ptr();

    let rect = PsdRect {
        top: 20,
        left: 30,
        bottom: 80,
        right: 120,
    };
    let changed = varied_rgba(400, 300, 9);
    blit_region(&mut doc.composite, &changed, rect);
    rust.doc = Some(doc.clone());
    rust.refresh_level0_region(crop_buffer(&changed, rect), rect.left, rect.top);

    let level0 = rust.level0.as_ref().expect("sRGB level-0 frame");
    assert_eq!(
        level0.data.as_ptr(),
        before,
        "the frame is patched in place"
    );
    assert_eq!(level0.data, level0_buffer(&doc).data, "and stays in sync");
}

#[test]
fn image_after_region_refresh_matches_a_full_recomposite() {
    let mut doc = Document::new(64, 48, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = varied_rgba(64, 48, 1);
    let mut rust = PictureViewRust {
        doc: Some(doc.clone()),
        ..Default::default()
    };
    rust.reset_pyramid();
    let revision = rust.canvas_revision;

    // Replay `refresh_region`'s non-painting body, with no paint or `take`
    // between the refresh and the dirty rebuild `image()` performs.
    let rect = PsdRect {
        top: 8,
        left: 8,
        bottom: 24,
        right: 24,
    };
    let changed = varied_rgba(64, 48, 21);
    blit_region(&mut doc.composite, &changed, rect);
    rust.doc = Some(doc.clone());
    rust.refresh_level0_region(crop_buffer(&changed, rect), rect.left, rect.top);
    rust.damage.mark(rect);
    rust.update_pyramid(rect);

    assert!(
        rust.canvas_revision > revision,
        "refresh bumps the revision"
    );
    assert_eq!(
        rust.display_rect_damage(),
        Some(rect),
        "damage survives for image()"
    );

    let rebuilt = rebuild_display(&rust.doc, rust.stroke.as_ref(), false).expect("document");
    let full = premultiplied_display_image(&level0_buffer(&doc));
    assert_eq!(rebuilt, full, "image() equals a full recomposite");
}

#[test]
fn a_level0_crop_outside_the_level_is_null_and_a_partial_one_is_not() {
    let mut doc = Document::new(100, 100, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = varied_rgba(100, 100, 2);
    let mut rust = PictureViewRust {
        doc: Some(doc),
        ..Default::default()
    };
    rust.reset_pyramid();

    let outside = PsdRect {
        top: 2000,
        left: 2000,
        bottom: 2010,
        right: 2010,
    };
    assert!(rust.display_crop(0, outside).is_null());
    let partial = PsdRect {
        top: 90,
        left: 90,
        bottom: 110,
        right: 110,
    };
    let crop = rust.display_crop(0, partial);
    assert!(!crop.is_null());
    assert_eq!((crop.width(), crop.height()), (20, 20));
}

#[test]
fn reset_pyramid_rebuilds_and_clears_damage() {
    // `recomposite`, `undo`, `redo`, `history_jump`, `history_restore_snapshot`,
    // and the move-preview path all route through `reset_pyramid`.
    let first = varied_rgba(600, 400, 0);
    let mut doc = Document::new(600, 400, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = first.clone();
    let mut rust = PictureViewRust {
        doc: Some(doc),
        ..Default::default()
    };
    rust.reset_pyramid();
    assert_eq!(rust.damage, CanvasDamage::default());
    assert_pyramids_equal(
        &rust.pyramid,
        &ViewPyramid::rebuild(planes_of(&first).unwrap()),
        &first,
    );

    rust.damage.mark(PsdRect {
        top: 1,
        left: 1,
        bottom: 5,
        right: 5,
    });
    let second = varied_rgba(600, 400, 31);
    let mut restored = Document::new(600, 400, ColorMode::Rgb, BitDepth::Eight);
    restored.composite = second.clone();
    rust.doc = Some(restored);
    rust.reset_pyramid();
    assert_eq!(rust.damage, CanvasDamage::default(), "reset clears damage");
    assert_pyramids_equal(
        &rust.pyramid,
        &ViewPyramid::rebuild(planes_of(&second).unwrap()),
        &second,
    );
}

#[test]
fn display_accessors_report_levels_and_damage() {
    let mut doc = Document::new(1500, 700, ColorMode::Rgb, BitDepth::Eight);
    doc.composite = varied_rgba(1500, 700, 5);
    let mut rust = PictureViewRust {
        doc: Some(doc),
        ..Default::default()
    };
    rust.reset_pyramid();

    assert_eq!(rust.display_level_count(), 3);
    assert_eq!(rust.display_level_size(0), Some((1500, 700)));
    assert_eq!(rust.display_level_size(1), Some((750, 350)));
    assert_eq!(rust.display_level_size(2), Some((375, 175)));
    assert_eq!(rust.display_level_size(3), None);
    assert_eq!(rust.display_level_size(-1), None);

    // A level crop carries that level's size; out-of-range and empty rects are
    // null, not a panic.
    let crop = rust.display_crop(
        1,
        PsdRect {
            top: 10,
            left: 10,
            bottom: 26,
            right: 42,
        },
    );
    assert_eq!((crop.width(), crop.height()), (32, 16));
    assert!(!crop.is_null());
    assert!(rust
        .display_crop(3, whole_level(&rust.pyramid, 0))
        .is_null());
    assert!(rust
        .display_crop(-1, whole_level(&rust.pyramid, 0))
        .is_null());
    let empty = PsdRect {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    assert!(rust.display_crop(0, empty).is_null());

    assert_eq!(rust.display_rect_damage(), None);
    let dirty = PsdRect {
        top: 4,
        left: 5,
        bottom: 12,
        right: 20,
    };
    rust.damage.mark(dirty);
    assert_eq!(rust.display_rect_damage(), Some(dirty));
    assert_eq!(rust.display_rect_damage(), None, "take starts afresh");
}
