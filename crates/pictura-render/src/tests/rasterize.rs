use super::*;

#[test]
fn solid_fill_composites_over_transparency() {
    let mut d = doc(2, 2, Vec::new());
    let path = add_solid_fill(&mut d, "", [10, 200, 30, 255]);
    assert_eq!(path, "0");
    assert!(is_fill_content_layer(resolve_path(&d, &path).unwrap()));

    let out = composite_rgba(&d);
    assert_eq!(rgb(&out, 0, 0), [10, 200, 30]);
    assert_eq!(rgb(&out, 1, 1), [10, 200, 30]);
    assert_eq!(px(&out, 1, 1)[3], 255);
}

#[test]
fn authored_solid_fill_is_version16_descriptor() {
    let mut d = doc(2, 2, Vec::new());
    let path = add_solid_fill(&mut d, "", [10, 200, 30, 255]);
    let layer = resolve_path(&d, &path).unwrap();
    let data = layer.adjustment.as_ref().expect("SoCo block");
    assert_eq!(data.key, *b"SoCo");
    assert_eq!(&data.data[0..4], &[0, 0, 0, 16], "version-16 header");
    assert_eq!(
        decode_adjustment(data),
        Some(Adjustment::SolidFill([10, 200, 30, 255]))
    );

    let out = composite_rgba(&d);
    assert_eq!(rgb(&out, 0, 0), [10, 200, 30]);
    assert_eq!(px(&out, 0, 0)[3], 255);
}

#[test]
fn solid_fill_respects_opacity_and_mask() {
    let mut d = doc(2, 1, Vec::new());
    let path = add_solid_fill(&mut d, "", [50, 100, 150, 255]);
    {
        let layer = resolve_path_mut(&mut d, &path).unwrap();
        layer.opacity = 128;
        // 2x1 mask reveals x=0, hides x=1.
        layer.mask = Some(LayerMask {
            rect: full(2, 1),
            default_color: 0,
            disabled: false,
            flags: 0,
            data: Some(vec![255, 0]),
            ..Default::default()
        });
    }
    let out = composite_rgba(&d);
    assert_eq!(rgb(&out, 0, 0), [50, 100, 150]);
    assert!(
        (px(&out, 0, 0)[3] as i16 - 128).abs() <= 1,
        "50% opacity should halve the alpha, got {}",
        px(&out, 0, 0)[3]
    );
    assert_eq!(px(&out, 1, 0)[3], 0, "masked-out pixel stays transparent");
}

#[test]
fn rasterize_bakes_color_and_clears_fill() {
    let mut d = doc(2, 2, Vec::new());
    let path = add_solid_fill(&mut d, "", [1, 2, 3, 255]);
    let name = resolve_path(&d, &path).unwrap().name.clone();

    assert!(rasterize_fill_content(&mut d, &path));
    let layer = resolve_path(&d, &path).unwrap();
    assert!(layer.adjustment.is_none(), "fill data is cleared");
    assert_eq!(layer.name, name, "name is kept");
    assert_eq!(crate::channel(layer, 0).unwrap(), &[1, 1, 1, 1]);
    assert_eq!(crate::channel(layer, 1).unwrap(), &[2, 2, 2, 2]);
    assert_eq!(crate::channel(layer, 2).unwrap(), &[3, 3, 3, 3]);
    assert_eq!(crate::channel(layer, -1).unwrap(), &[255, 255, 255, 255]);

    // The baked pixels composite to the same color the generative fill showed.
    let out = composite_rgba(&d);
    assert_eq!(rgb(&out, 1, 1), [1, 2, 3]);
    assert_eq!(px(&out, 1, 1)[3], 255);

    // A second run has no fill data left, so it refuses.
    assert!(!rasterize_fill_content(&mut d, &path));
}

#[test]
fn rasterize_bakes_non_opaque_in_house_fill() {
    // The 4-byte in-house tuple carries alpha; the descriptor form does not.
    let mut d = doc(
        2,
        2,
        vec![adjustment_layer(
            "soco",
            *b"SoCo",
            vec![10, 20, 30, 200],
            255,
            None,
        )],
    );
    resolve_path_mut(&mut d, "0").unwrap().rect = full(2, 2);

    assert!(rasterize_fill_content(&mut d, "0"));
    let layer = resolve_path(&d, "0").unwrap();
    assert!(layer.adjustment.is_none(), "fill data is cleared");
    assert_eq!(crate::channel(layer, 0).unwrap(), &[10, 10, 10, 10]);
    assert_eq!(crate::channel(layer, 1).unwrap(), &[20, 20, 20, 20]);
    assert_eq!(crate::channel(layer, 2).unwrap(), &[30, 30, 30, 30]);
    assert_eq!(
        crate::channel(layer, -1).unwrap(),
        &[200, 200, 200, 200],
        "baked alpha is the tuple's alpha"
    );
}

#[test]
fn rasterize_refuses_group_without_mutating() {
    let mut d = doc(2, 2, vec![group("g", BlendMode::Normal, 255, None, vec![])]);
    let before = d.clone();
    assert!(!rasterize_fill_content(&mut d, "0"));
    assert_eq!(d, before, "refusal leaves the document unchanged");
}

#[test]
fn rasterize_refuses_non_fill_targets() {
    let mut pixel_doc = doc(
        2,
        2,
        vec![solid(
            "pix",
            full(2, 2),
            (9, 9, 9),
            255,
            BlendMode::Normal,
            255,
        )],
    );
    assert!(!rasterize_fill_content(&mut pixel_doc, "0"));
    assert!(!rasterize_fill_content(&mut pixel_doc, "99"));

    // An ordinary adjustment layer is not fill content.
    let mut adj_doc = doc(
        2,
        2,
        vec![adjustment_layer("inv", *b"nvrt", Vec::new(), 255, None)],
    );
    assert!(!rasterize_fill_content(&mut adj_doc, "0"));

    // A 6-byte non-descriptor SoCo is not decodable.
    let mut garbage = doc(
        2,
        2,
        vec![adjustment_layer(
            "soco",
            *b"SoCo",
            vec![0, 1, 2, 3, 4, 5],
            255,
            None,
        )],
    );
    assert!(!is_fill_content_layer(resolve_path(&garbage, "0").unwrap()));
    assert!(!rasterize_fill_content(&mut garbage, "0"));

    // A valid descriptor-form SoCo is fill content and rasterizes.
    let mut fill = adjustment_layer(
        "soco",
        *b"SoCo",
        encode_solid_color_fill([4, 5, 6]).data,
        255,
        None,
    );
    fill.rect = full(2, 2);
    let mut descriptor = doc(2, 2, vec![fill]);
    assert!(is_fill_content_layer(
        resolve_path(&descriptor, "0").unwrap()
    ));
    assert!(rasterize_fill_content(&mut descriptor, "0"));
    let layer = resolve_path(&descriptor, "0").unwrap();
    assert_eq!(crate::channel(layer, 0).unwrap(), &[4, 4, 4, 4]);
    assert_eq!(crate::channel(layer, -1).unwrap(), &[255, 255, 255, 255]);

    // A 3-byte SoCo is not decodable either.
    let mut short = doc(
        2,
        2,
        vec![adjustment_layer("soco", *b"SoCo", vec![1, 2, 3], 255, None)],
    );
    assert!(!rasterize_fill_content(&mut short, "0"));
}

#[test]
fn rasterize_all_counts_fill_layers() {
    let mut d = doc(
        2,
        2,
        vec![solid(
            "base",
            full(2, 2),
            (0, 0, 0),
            255,
            BlendMode::Normal,
            255,
        )],
    );
    let first = add_solid_fill(&mut d, "0", [1, 1, 1, 255]);
    let second = add_solid_fill(&mut d, &first, [2, 2, 2, 255]);
    assert_eq!(rasterize_all_fill_content(&mut d), 2);
    assert!(resolve_path(&d, &first).unwrap().adjustment.is_none());
    assert!(resolve_path(&d, &second).unwrap().adjustment.is_none());
    assert_eq!(rasterize_all_fill_content(&mut d), 0, "already rasterized");
}
