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
    let path = add_solid_fill(&mut d, "", [1, 2, 3, 200]);
    let name = resolve_path(&d, &path).unwrap().name.clone();

    assert!(rasterize_fill_content(&mut d, &path));
    let layer = resolve_path(&d, &path).unwrap();
    assert!(layer.adjustment.is_none(), "fill data is cleared");
    assert_eq!(layer.name, name, "name is kept");
    assert_eq!(crate::channel(layer, 0).unwrap(), &[1, 1, 1, 1]);
    assert_eq!(crate::channel(layer, 1).unwrap(), &[2, 2, 2, 2]);
    assert_eq!(crate::channel(layer, 2).unwrap(), &[3, 3, 3, 3]);
    assert_eq!(crate::channel(layer, -1).unwrap(), &[200, 200, 200, 200]);

    // The baked pixels composite to the same color the generative fill showed.
    let out = composite_rgba(&d);
    assert_eq!(rgb(&out, 1, 1), [1, 2, 3]);
    assert_eq!(px(&out, 1, 1)[3], 200);

    // A second run has no fill data left, so it refuses.
    assert!(!rasterize_fill_content(&mut d, &path));
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

    // A descriptor-shaped SoCo (not the 4-byte subset) refuses.
    let mut descriptor = doc(
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
    assert!(!is_fill_content_layer(
        resolve_path(&descriptor, "0").unwrap()
    ));
    assert!(!rasterize_fill_content(&mut descriptor, "0"));

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
