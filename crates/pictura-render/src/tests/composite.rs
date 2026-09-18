use super::*;

// --- compositing pipeline ----------------------------------------------

#[test]
fn multi_layer_source_over_scene() {
    let d = doc(
        2,
        2,
        vec![
            solid(
                "white",
                full(2, 2),
                (255, 255, 255),
                255,
                BlendMode::Normal,
                255,
            ),
            solid(
                "red",
                full(2, 2),
                (255, 0, 0),
                255,
                BlendMode::Multiply,
                255,
            ),
            solid("blue", full(2, 2), (0, 0, 255), 255, BlendMode::Screen, 255),
        ],
    );
    let out = composite_rgba(&d);
    for y in 0..2 {
        for x in 0..2 {
            assert_eq!(px(&out, x, y), [255, 0, 255, 255], "at {x},{y}");
        }
    }
}

#[test]
fn blend_ignored_over_transparent_backdrop() {
    let d = doc(
        1,
        1,
        vec![solid(
            "top",
            full(1, 1),
            (255, 0, 0),
            255,
            BlendMode::Multiply,
            255,
        )],
    );
    assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 255]);
}

#[test]
fn layer_opacity_scales_over_backdrop() {
    // 1x1: base 200 opaque, top 0/200/0 at 50% opacity, Normal.
    let d = doc(
        1,
        1,
        vec![
            solid(
                "base",
                full(1, 1),
                (200, 100, 50),
                255,
                BlendMode::Normal,
                255,
            ),
            solid("top", full(1, 1), (0, 200, 0), 255, BlendMode::Normal, 128),
        ],
    );
    assert_eq!(px(&composite_rgba(&d), 0, 0), [100, 150, 25, 255]);
}

#[test]
fn fill_255_is_byte_identical_to_pre_change_composites() {
    // Each expected pixel is the pre-fill value: fill 255 is an exact 1.0
    // factor, so plain, masked, group and adjustment scenes must not move.

    // Plain: the multi-layer source-over scene.
    let plain = doc(
        2,
        2,
        vec![
            solid(
                "white",
                full(2, 2),
                (255, 255, 255),
                255,
                BlendMode::Normal,
                255,
            ),
            solid(
                "red",
                full(2, 2),
                (255, 0, 0),
                255,
                BlendMode::Multiply,
                255,
            ),
            solid("blue", full(2, 2), (0, 0, 255), 255, BlendMode::Screen, 255),
        ],
    );
    assert_eq!(px(&composite_rgba(&plain), 0, 0), [255, 0, 255, 255]);

    // Masked: mask zeroes the top half.
    let mut masked = solid(
        "masked",
        full(1, 2),
        (255, 0, 0),
        255,
        BlendMode::Normal,
        255,
    );
    masked.mask = Some(LayerMask {
        rect: full(1, 2),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(vec![0, 255]),
    });
    let masked_out = composite_rgba(&doc(1, 2, vec![masked]));
    assert_eq!(px(&masked_out, 0, 0), [0, 0, 0, 0]);
    assert_eq!(px(&masked_out, 0, 1), [255, 0, 0, 255]);

    // Group: fill lives on a group and is ignored; fill 0 must equal 255.
    let group_scene = |fill: u8| {
        let mut g = group(
            "group",
            BlendMode::Multiply,
            255,
            None,
            vec![solid(
                "child",
                full(1, 1),
                (50, 50, 50),
                255,
                BlendMode::Normal,
                255,
            )],
        );
        g.fill = fill;
        composite_rgba(&doc(
            1,
            1,
            vec![
                solid(
                    "backdrop",
                    full(1, 1),
                    (200, 200, 200),
                    255,
                    BlendMode::Normal,
                    255,
                ),
                g,
            ],
        ))
    };
    assert_eq!(px(&group_scene(255), 0, 0), [39, 39, 39, 255]);
    assert_eq!(
        group_scene(0).data,
        group_scene(255).data,
        "group fill is ignored"
    );

    // Adjustment: fill 255 leaves the invert gate at its pre-fill value.
    let base = solid(
        "base",
        full(2, 1),
        (100, 100, 100),
        255,
        BlendMode::Normal,
        255,
    );
    let mut adj = adjustment_layer("invert", *b"nvrt", Vec::new(), 255, None);
    adj.mask = Some(LayerMask {
        rect: full(2, 1),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(vec![255, 0]),
    });
    let adj_out = composite_rgba(&doc(2, 1, vec![base, adj]));
    assert_eq!(rgb(&adj_out, 0, 0), [155, 155, 155]);
    assert_eq!(rgb(&adj_out, 1, 0), [100, 100, 100]);
}

#[test]
fn fill_below_255_scales_contribution() {
    // Over a transparent backdrop: source alpha is scaled to 128/255.
    let over_transparent = doc(
        1,
        1,
        vec![{
            let mut l = solid("top", full(1, 1), (10, 20, 30), 255, BlendMode::Normal, 255);
            l.fill = 128;
            l
        }],
    );
    assert_eq!(
        px(&composite_rgba(&over_transparent), 0, 0),
        [10, 20, 30, 128]
    );

    // Over an opaque backdrop: the same factor lerps toward the source.
    let over_opaque = doc(
        1,
        1,
        vec![
            solid(
                "base",
                full(1, 1),
                (100, 100, 100),
                255,
                BlendMode::Normal,
                255,
            ),
            {
                let mut l = solid("top", full(1, 1), (200, 0, 0), 255, BlendMode::Normal, 255);
                l.fill = 128;
                l
            },
        ],
    );
    assert_eq!(px(&composite_rgba(&over_opaque), 0, 0), [150, 50, 50, 255]);
}

#[test]
fn masked_layer_zeroes_masked_alpha() {
    let lw = 1u32;
    let mut top = solid(
        "masked",
        full(1, 2),
        (255, 0, 0),
        255,
        BlendMode::Normal,
        255,
    );
    top.mask = Some(LayerMask {
        rect: full(1, 2),
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(vec![0, 255]),
    });
    let _ = lw;
    let d = doc(1, 2, vec![top]);
    let out = composite_rgba(&d);
    assert_eq!(px(&out, 0, 0), [0, 0, 0, 0]);
    assert_eq!(px(&out, 0, 1), [255, 0, 0, 255]);
}

#[test]
fn disabled_mask_is_ignored() {
    let mut top = solid(
        "masked",
        full(1, 1),
        (255, 0, 0),
        255,
        BlendMode::Normal,
        255,
    );
    top.mask = Some(LayerMask {
        rect: full(1, 1),
        default_color: 0,
        disabled: true,
        flags: 0x02,
        data: Some(vec![0]),
    });
    let d = doc(1, 1, vec![top]);
    assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 255]);
}

#[test]
fn isolated_group_blends_as_single_layer() {
    // Backdrop 200; group Multiply containing an opaque 50 child. Isolation
    // means the child is Normal inside the group, then Multiply applies:
    // 200/255 * 50/255 = 0.1537 -> 39.
    let d = doc(
        1,
        1,
        vec![
            solid(
                "backdrop",
                full(1, 1),
                (200, 200, 200),
                255,
                BlendMode::Normal,
                255,
            ),
            group(
                "group",
                BlendMode::Multiply,
                255,
                None,
                vec![solid(
                    "child",
                    full(1, 1),
                    (50, 50, 50),
                    255,
                    BlendMode::Normal,
                    255,
                )],
            ),
        ],
    );
    assert_eq!(px(&composite_rgba(&d), 0, 0), [39, 39, 39, 255]);
}

#[test]
fn group_opacity_scales_isolated_result() {
    let d = doc(
        1,
        1,
        vec![group(
            "group",
            BlendMode::Normal,
            128,
            None,
            vec![solid(
                "child",
                full(1, 1),
                (255, 0, 0),
                255,
                BlendMode::Normal,
                255,
            )],
        )],
    );
    assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 128]);
}

#[test]
fn pass_through_group_equals_ungrouped_stack() {
    let backdrop = solid(
        "backdrop",
        full(2, 2),
        (200, 120, 40),
        255,
        BlendMode::Normal,
        255,
    );
    let child = solid(
        "child",
        full(2, 2),
        (50, 180, 90),
        255,
        BlendMode::Multiply,
        255,
    );
    let grouped = doc(
        2,
        2,
        vec![
            backdrop.clone(),
            group(
                "group",
                BlendMode::PassThrough,
                255,
                None,
                vec![child.clone()],
            ),
        ],
    );
    let ungrouped = doc(2, 2, vec![backdrop, child]);
    assert_eq!(
        composite_rgba(&grouped).data,
        composite_rgba(&ungrouped).data,
        "pass-through group must equal the flattened stack"
    );
}

#[test]
fn pass_through_group_with_opacity_falls_back_to_isolated() {
    // Approximation: pass-through + opacity is isolated with mode Normal.
    let d = doc(
        1,
        1,
        vec![group(
            "group",
            BlendMode::PassThrough,
            128,
            None,
            vec![solid(
                "child",
                full(1, 1),
                (255, 0, 0),
                255,
                BlendMode::Normal,
                255,
            )],
        )],
    );
    assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 128]);
}

#[test]
fn bounds_and_negative_rect_clipping() {
    let mut d = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
    d.layers = vec![
        // Covers only canvas (0,0): local (1,1) of a 2x2 rect anchored at -1,-1.
        solid(
            "neg",
            rect(-1, -1, 1, 1),
            (255, 0, 0),
            255,
            BlendMode::Normal,
            255,
        ),
        // Fully off-canvas, contributes nothing and must not panic.
        solid(
            "off",
            rect(-9, -9, -1, -1),
            (0, 255, 0),
            255,
            BlendMode::Normal,
            255,
        ),
        // Zero-area layer.
        solid(
            "empty",
            rect(1, 1, 1, 1),
            (0, 0, 255),
            255,
            BlendMode::Normal,
            255,
        ),
    ];
    let out = composite_rgba(&d);
    assert_eq!(px(&out, 0, 0), [255, 0, 0, 255]);
    assert_eq!(px(&out, 1, 0), [0, 0, 0, 0]);
    assert_eq!(px(&out, 0, 1), [0, 0, 0, 0]);
    assert_eq!(px(&out, 1, 1), [0, 0, 0, 0]);
}

#[test]
fn positive_offset_rect_lands_in_canvas() {
    let d = doc(
        3,
        2,
        vec![solid(
            "offset",
            rect(1, 1, 2, 2),
            (10, 20, 30),
            255,
            BlendMode::Normal,
            255,
        )],
    );
    let out = composite_rgba(&d);
    assert_eq!(px(&out, 1, 1), [10, 20, 30, 255]);
    assert_eq!(px(&out, 0, 0), [0, 0, 0, 0]);
    assert_eq!(px(&out, 2, 1), [0, 0, 0, 0]);
}

#[test]
fn empty_layer_stack_is_transparent() {
    let out = composite_rgba(&doc(2, 2, Vec::new()));
    assert_eq!(out.channels, 4);
    assert!(out.data.iter().all(|&b| b == 0));
}

#[test]
fn dissolve_is_binary_and_deterministic() {
    let d = doc(
        8,
        8,
        vec![solid(
            "noise",
            full(8, 8),
            (255, 0, 0),
            255,
            BlendMode::Dissolve,
            128,
        )],
    );
    let a = composite_rgba(&d);
    let b = composite_rgba(&d);
    assert_eq!(a.data, b.data, "dissolve must be deterministic");
    let mut sources = 0;
    for y in 0..8 {
        for x in 0..8 {
            let p = px(&a, x, y);
            assert!(
                p == [255, 0, 0, 255] || p == [0, 0, 0, 0],
                "dissolve pixel must be binary, got {p:?}"
            );
            if p == [255, 0, 0, 255] {
                sources += 1;
            }
        }
    }
    assert!(sources > 0 && sources < 64, "expected a mix, got {sources}");
}

#[test]
fn grayscale_layer_replicates_channel() {
    let mut d = Document::new(1, 1, ColorMode::Grayscale, BitDepth::Eight);
    d.layers = vec![Layer {
        name: "gray".into(),
        rect: full(1, 1),
        blend: BlendMode::Normal,
        opacity: 255,
        fill: 255,
        lock: LockFlags::default(),
        color: ColorLabel::None,
        clipping: false,
        visible: true,
        mask: None,
        adjustment: None,
        channels: vec![Channel {
            id: 0,
            data: vec![120],
        }],
        children: Vec::new(),
        is_group: false,
        background: false,
    }];
    assert_eq!(px(&composite_rgba(&d), 0, 0), [120, 120, 120, 255]);
}
