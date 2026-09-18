use super::create::empty_group;
use super::*;
use pictura_core::{
    BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LayerMask, LockFlags,
    PsdRect,
};

fn rect(w: i32, h: i32) -> PsdRect {
    PsdRect {
        top: 0,
        left: 0,
        bottom: h,
        right: w,
    }
}

fn pixel_layer(name: &str, w: u32, h: u32, value: u8) -> Layer {
    let n = w as usize * h as usize;
    Layer {
        name: name.into(),
        rect: rect(w as i32, h as i32),
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
                data: vec![value; n],
            },
            Channel {
                id: 1,
                data: vec![value; n],
            },
            Channel {
                id: 2,
                data: vec![value; n],
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

fn doc_with(layers: Vec<Layer>) -> Document {
    let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = layers;
    doc
}

#[test]
fn add_layer_inserts_above_and_is_transparent() {
    let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
    let index = add_layer(&mut doc, 0, "Layer 1");

    assert_eq!(index, 1);
    assert_eq!(doc.layers.len(), 2);
    let added = &doc.layers[1];
    assert_eq!(added.name, "Layer 1");
    assert_eq!(added.rect, rect(4, 4));
    assert_eq!(added.blend, BlendMode::Normal);
    assert_eq!((added.opacity, added.fill), (255, 255));
    assert!(added.visible && !added.is_group);
    assert_eq!(added.channels.len(), 4);
    for (id, channel) in [(0i16, 0), (1, 1), (2, 2), (-1, 3)] {
        assert_eq!(added.channels[channel].id, id);
        assert_eq!(added.channels[channel].data, vec![0u8; 16]);
    }
}

#[test]
fn add_layer_without_selection_goes_to_top() {
    let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1), pixel_layer("b", 4, 4, 2)]);
    assert_eq!(
        add_layer(&mut doc, -1, "top"),
        2,
        "no selection inserts on top"
    );
    assert_eq!(doc.layers[2].name, "top");

    let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1)]);
    assert_eq!(
        add_layer(&mut doc, 99, "top"),
        1,
        "out-of-range inserts on top"
    );
}

#[test]
fn add_group_inserts_empty_group() {
    let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
    let index = add_group(&mut doc, 0, "Group 1");

    assert_eq!(index, 1);
    let group = &doc.layers[1];
    assert!(group.is_group);
    assert!(group.children.is_empty());
    assert_eq!(group.rect, rect(0, 0));
    assert_eq!(group.blend, BlendMode::Normal);
}

#[test]
fn duplicate_layer_deep_copies_above_the_source() {
    let mask = LayerMask {
        rect: rect(4, 4),
        default_color: 255,
        disabled: false,
        flags: 0,
        data: Some(vec![128; 16]),
    };
    let mut original = pixel_layer("base", 4, 4, 40);
    original.mask = Some(mask);
    original.color = ColorLabel::Red;
    original.opacity = 200;
    let mut doc = doc_with(vec![original, pixel_layer("top", 4, 4, 80)]);

    let index = duplicate_layer(&mut doc, 0);
    assert_eq!(index, 1);
    assert_eq!(doc.layers.len(), 3);
    assert_eq!(doc.layers[1].name, "base copy");
    assert_eq!(doc.layers[1].mask, doc.layers[0].mask);
    assert_eq!(doc.layers[1].color, ColorLabel::Red);
    assert_eq!(doc.layers[1].opacity, 200);

    doc.layers[1].channels[0].data[0] = 7;
    assert_eq!(doc.layers[0].channels[0].data[0], 40, "deep copy");
}

#[test]
fn duplicate_group_copies_children() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("child", 4, 4, 30));
    let mut doc = doc_with(vec![group]);

    assert_eq!(duplicate_layer(&mut doc, 0), 1);
    assert_eq!(doc.layers[1].name, "Group 1 copy");
    assert!(doc.layers[1].is_group);
    assert_eq!(doc.layers[1].children.len(), 1);
    assert_eq!(doc.layers[1].children[0].name, "child");
}

#[test]
fn duplicate_rejects_out_of_range() {
    let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 1)]);
    assert_eq!(duplicate_layer(&mut doc, -1), -1);
    assert_eq!(duplicate_layer(&mut doc, 9), -1);
    assert_eq!(doc.layers.len(), 1);
}

#[test]
fn group_layer_wraps_in_place() {
    let mut doc = doc_with(vec![
        pixel_layer("a", 4, 4, 1),
        pixel_layer("b", 4, 4, 2),
        pixel_layer("c", 4, 4, 3),
    ]);
    let index = group_layer(&mut doc, 1);

    assert_eq!(index, 1);
    assert_eq!(doc.layers.len(), 3);
    let group = &doc.layers[1];
    assert!(group.is_group);
    assert_eq!(group.name, "Group 1");
    assert_eq!(group.children.len(), 1);
    assert_eq!(group.children[0].name, "b");
    assert_eq!(doc.layers[0].name, "a");
    assert_eq!(doc.layers[2].name, "c");
}

#[test]
fn group_layer_names_after_existing_groups() {
    let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1)]);
    group_layer(&mut doc, 0);
    // Wrap the new group again: the next name must skip the existing one.
    assert_eq!(doc.layers[0].name, "Group 1");
    doc.layers.push(pixel_layer("b", 4, 4, 2));
    group_layer(&mut doc, 1);
    assert_eq!(doc.layers[1].name, "Group 2");
}

#[test]
fn ungroup_splices_children_in_order() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("a", 4, 4, 1));
    group.children.push(pixel_layer("b", 4, 4, 2));
    let mut doc = doc_with(vec![
        pixel_layer("bottom", 4, 4, 0),
        group,
        pixel_layer("top", 4, 4, 3),
    ]);

    assert!(ungroup_layer(&mut doc, 1));
    assert_eq!(doc.layers.len(), 4);
    let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["bottom", "a", "b", "top"]);
}

#[test]
fn ungroup_rejects_non_group() {
    let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 1)]);
    assert!(!ungroup_layer(&mut doc, 0));
    assert!(!ungroup_layer(&mut doc, -1));
    assert_eq!(doc.layers.len(), 1);
}

#[test]
fn next_layer_name_uses_highest_suffix() {
    let mut doc = doc_with(vec![
        pixel_layer("Layer 1", 4, 4, 1),
        pixel_layer("Layer 5", 4, 4, 2),
        pixel_layer("Group 2", 4, 4, 3),
    ]);
    doc.layers[1].children.push(pixel_layer("Layer 9", 4, 4, 4));
    assert_eq!(next_layer_name(&doc, "Layer"), "Layer 10");
    assert_eq!(next_layer_name(&doc, "Group"), "Group 3");

    let empty = doc_with(Vec::new());
    assert_eq!(next_layer_name(&empty, "Layer"), "Layer 1");
}

#[test]
fn transparent_layer_does_not_change_the_composite() {
    let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
    let before = crate::composite_rgba(&doc);

    let index = add_layer(&mut doc, 0, "Layer 1");
    assert!(index >= 0);
    let after = crate::composite_rgba(&doc);

    assert_eq!(before, after, "a fully transparent layer must be inert");
    assert!(after.data.iter().any(|&b| b != 0), "base layer is visible");
}

fn locked_layer(name: &str) -> Layer {
    let mut layer = pixel_layer(name, 4, 4, 1);
    layer.lock = LockFlags::all();
    layer
}

fn sample_doc() -> Document {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("child", 4, 4, 2));
    doc_with(vec![
        pixel_layer("Background", 4, 4, 0),
        locked_layer("locked"),
        group,
        pixel_layer("normal", 4, 4, 3),
    ])
}

fn visible_names(doc: &Document) -> Vec<String> {
    fn walk(layers: &[Layer], out: &mut Vec<String>) {
        for layer in layers {
            if layer.visible {
                out.push(layer.name.clone());
            }
            walk(&layer.children, out);
        }
    }
    let mut out = Vec::new();
    walk(&doc.layers, &mut out);
    out
}

#[test]
fn path_resolve_nested() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("a", 4, 4, 1));
    group.children.push(pixel_layer("b", 4, 4, 2));
    let mut doc = doc_with(vec![pixel_layer("bottom", 4, 4, 0), group]);

    assert_eq!(resolve_path(&doc, "0").unwrap().name, "bottom");
    assert_eq!(resolve_path(&doc, "1/0").unwrap().name, "a");
    assert_eq!(resolve_path(&doc, "1/1").unwrap().name, "b");
    assert!(resolve_path(&doc, "1/2").is_none(), "out of range child");
    assert_eq!(parent_path("1/1"), Some("1"));
    assert_eq!(parent_path("0"), None);

    resolve_path_mut(&mut doc, "1/0").unwrap().opacity = 100;
    assert_eq!(doc.layers[1].children[0].opacity, 100);
}

#[test]
fn path_resolve_rejects() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("a", 4, 4, 1));
    let mut doc = doc_with(vec![pixel_layer("bottom", 4, 4, 0), group]);

    for bad in [
        "", "01", "-1", "1/", "/1", "1//2", "1/a", "+1", " 1", "1/999", "999",
    ] {
        assert!(resolve_path(&doc, bad).is_none(), "resolve {bad:?}");
        assert!(
            resolve_path_mut(&mut doc, bad).is_none(),
            "resolve_mut {bad:?}"
        );
    }
    assert_eq!(parent_path(""), None);
    assert_eq!(parent_path("01"), None);
}

#[test]
fn flatten_rows_is_topmost_first() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("c0", 4, 4, 1));
    group.children.push(pixel_layer("c1", 4, 4, 2));
    let doc = doc_with(vec![
        pixel_layer("bottom", 4, 4, 0),
        group,
        pixel_layer("top", 4, 4, 3),
    ]);

    let rows = flatten_rows(&doc);
    let expected: Vec<(String, u32)> = vec![
        ("2".into(), 0),
        ("1".into(), 0),
        ("1/1".into(), 1),
        ("1/0".into(), 1),
        ("0".into(), 0),
    ];
    assert_eq!(rows, expected);
}

#[test]
fn is_background_uses_default_heuristic() {
    let doc = doc_with(vec![
        pixel_layer("Background", 4, 4, 0),
        pixel_layer("Layer 1", 4, 4, 1),
    ]);
    assert!(is_background(&doc, "0"));
    assert!(!is_background(&doc, "1"));
    assert!(!is_background(&doc, "9"), "missing path");

    let named_group = doc_with(vec![empty_group("Background")]);
    assert!(
        !is_background(&named_group, "0"),
        "a group is not Background"
    );

    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("Background", 4, 4, 0));
    let nested = doc_with(vec![group]);
    assert!(!is_background(&nested, "0/0"), "nested is not top-level 0");

    let mut adjusted = pixel_layer("Background", 4, 4, 0);
    adjusted.adjustment = Some(crate::encode_invert());
    let adjusted = doc_with(vec![adjusted]);
    assert!(
        !is_background(&adjusted, "0"),
        "an adjustment is not Background"
    );
}

#[test]
fn visibility_applies_everywhere() {
    let mut doc = sample_doc();
    assert_eq!(set_visible_paths(&mut doc, &["0", "1", "2", "3"], false), 4);
    assert!(doc.layers.iter().all(|layer| !layer.visible));
    assert_eq!(set_visible_paths(&mut doc, &["0", "1", "2", "3"], false), 0);
}

#[test]
fn blend_and_opacity_skip_background_and_locked() {
    let mut doc = sample_doc();
    assert_eq!(
        set_blend_paths(&mut doc, &["0", "1", "2", "3"], BlendMode::Multiply),
        2
    );
    assert_eq!(doc.layers[0].blend, BlendMode::Normal, "Background skipped");
    assert_eq!(doc.layers[1].blend, BlendMode::Normal, "locked skipped");
    assert_eq!(doc.layers[2].blend, BlendMode::Multiply, "group applies");
    assert_eq!(doc.layers[3].blend, BlendMode::Multiply);

    let mut doc = sample_doc();
    assert_eq!(set_opacity_paths(&mut doc, &["0", "1", "2", "3"], 100), 2);
    assert_eq!(doc.layers[0].opacity, 255);
    assert_eq!(doc.layers[1].opacity, 255);
    assert_eq!(doc.layers[2].opacity, 100);
    assert_eq!(doc.layers[3].opacity, 100);
}

#[test]
fn fill_skips_groups() {
    let mut doc = sample_doc();
    assert_eq!(set_fill_paths(&mut doc, &["0", "1", "2", "3"], 100), 1);
    assert_eq!(doc.layers[0].fill, 255, "Background skipped");
    assert_eq!(doc.layers[1].fill, 255, "locked skipped");
    assert_eq!(doc.layers[2].fill, 255, "group has no Fill");
    assert_eq!(doc.layers[3].fill, 100);
}

#[test]
fn lock_skips_background_but_applies_to_locked() {
    let mut doc = sample_doc();
    assert_eq!(
        set_lock_paths(&mut doc, &["0", "1"], LockFlags::PIXELS, false),
        1
    );
    assert_eq!(doc.layers[0].lock.bits(), 0, "Background lock untouched");
    assert!(!doc.layers[1].lock.contains(LockFlags::PIXELS));
}

#[test]
fn color_skips_background_applies_to_locked() {
    let mut doc = sample_doc();
    assert_eq!(
        set_color_paths(&mut doc, &["0", "1", "3"], ColorLabel::Red),
        2
    );
    assert_eq!(doc.layers[0].color, ColorLabel::None, "Background skipped");
    assert_eq!(doc.layers[1].color, ColorLabel::Red, "locked applies");
    assert_eq!(doc.layers[3].color, ColorLabel::Red);
}

#[test]
fn delete_skips_background_and_locked() {
    let mut doc = sample_doc();
    assert_eq!(delete_paths(&mut doc, &["0", "1"]), 0);
    assert_eq!(doc.layers.len(), 4);
    assert_eq!(delete_paths(&mut doc, &["2", "3"]), 2);
    assert_eq!(doc.layers.len(), 2);
}

#[test]
fn duplicate_applies_everywhere() {
    let mut doc = sample_doc();
    let created = duplicate_paths(&mut doc, &["0", "1"]);
    assert_eq!(created.len(), 2, "Background and locked both duplicate");
    let names: Vec<&str> = doc.layers.iter().map(|layer| layer.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Background",
            "Background copy",
            "locked",
            "locked copy",
            "Group 1",
            "normal"
        ]
    );
}

#[test]
fn ungroup_skips_non_groups() {
    let mut doc = sample_doc();
    assert_eq!(ungroup_paths(&mut doc, &["0", "3"]), 0);
    assert_eq!(ungroup_paths(&mut doc, &["2"]), 1);
    assert_eq!(doc.layers.len(), 4);
    assert_eq!(doc.layers[2].name, "child");
}

#[test]
fn group_paths_refuses_cross_container_and_succeeds_within_one() {
    let mut doc = sample_doc();
    assert_eq!(
        group_paths(&mut doc, &["0", "2/0"]),
        None,
        "cross container"
    );
    assert_eq!(
        group_paths(&mut doc, &["0", "3"]),
        None,
        "Background refuses"
    );
    assert_eq!(group_paths(&mut doc, &["1", "3"]), None, "locked refuses");

    let mut doc = doc_with(vec![
        pixel_layer("a", 4, 4, 0),
        pixel_layer("b", 4, 4, 1),
        pixel_layer("c", 4, 4, 2),
    ]);
    let path = group_paths(&mut doc, &["0", "2"]).unwrap();
    assert_eq!(path, "1");
    assert_eq!(doc.layers.len(), 2);
    assert!(doc.layers[1].is_group);
    let children: Vec<&str> = doc.layers[1]
        .children
        .iter()
        .map(|layer| layer.name.as_str())
        .collect();
    assert_eq!(children, ["a", "c"]);
    assert_eq!(doc.layers[0].name, "b");
}

#[test]
fn apply_visibility_is_solo() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("c", 4, 4, 1));
    group.children.push(pixel_layer("d", 4, 4, 2));
    let mut doc = doc_with(vec![
        pixel_layer("a", 4, 4, 0),
        pixel_layer("b", 4, 4, 3),
        group,
    ]);

    assert_eq!(apply_visibility(&mut doc, &["1"]), 4);
    assert_eq!(visible_names(&doc), vec!["b"]);
    assert!(!doc.layers[2].visible);
    assert!(doc.layers[2].children.iter().all(|child| !child.visible));

    assert_eq!(apply_visibility(&mut doc, &["2", "2/0"]), 3);
    assert_eq!(visible_names(&doc), vec!["Group 1", "c"]);
}

#[test]
fn delete_paths_nested_pair_deletes_once() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("child", 4, 4, 1));
    let mut doc = doc_with(vec![group]);

    assert_eq!(delete_paths(&mut doc, &["0", "0/0"]), 1);
    assert!(doc.layers.is_empty());
}

#[test]
fn duplicate_paths_returns_new_paths() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("child", 4, 4, 1));
    let mut doc = doc_with(vec![pixel_layer("bottom", 4, 4, 0), group]);

    let created = duplicate_paths(&mut doc, &["0", "1"]);
    assert_eq!(created, vec!["2".to_string(), "1".to_string()]);
    let names: Vec<&str> = doc.layers.iter().map(|layer| layer.name.as_str()).collect();
    assert_eq!(names, ["bottom", "bottom copy", "Group 1", "Group 1 copy"]);
    assert_eq!(doc.layers[3].children.len(), 1, "deep copy");
}

#[test]
fn add_layer_in_inside_group_and_on_top() {
    let mut group = empty_group("Group 1");
    group.children.push(pixel_layer("child", 4, 4, 1));
    let mut doc = doc_with(vec![group]);

    let path = add_layer_in(&mut doc, "0", "Inside");
    assert_eq!(path, "0/1");
    assert_eq!(doc.layers[0].children.len(), 2);
    assert_eq!(doc.layers[0].children[1].name, "Inside");

    let path = add_group_in(&mut doc, "", "New Group");
    assert_eq!(path, "1", "empty selection lands on top");
    assert!(doc.layers[1].is_group);

    let path = add_layer_in(&mut doc, "99", "Top");
    assert_eq!(path, "2", "missing selection lands on top");
    assert_eq!(doc.layers[2].name, "Top");
}

#[test]
fn move_path_clamps_and_rename_resolves() {
    let mut doc = doc_with(vec![pixel_layer("a", 4, 4, 0), pixel_layer("b", 4, 4, 1)]);
    assert!(!move_path(&mut doc, "0", -1), "boundary");
    assert!(move_path(&mut doc, "0", 1));
    assert_eq!(doc.layers[0].name, "b");
    assert_eq!(doc.layers[1].name, "a");
    assert!(!move_path(&mut doc, "1", 5), "clamped to itself");
    assert!(!move_path(&mut doc, "9", 1), "missing path");

    assert!(rename_path(&mut doc, "1", "renamed"));
    assert_eq!(doc.layers[1].name, "renamed");
    assert!(!rename_path(&mut doc, "9", "x"));
}

#[test]
fn move_path_refuses_background_and_locked() {
    let mut doc = doc_with(vec![
        pixel_layer("Background", 4, 4, 0),
        locked_layer("locked"),
        pixel_layer("normal", 4, 4, 3),
    ]);

    assert!(!move_path(&mut doc, "0", 1), "Background refuses");
    assert_eq!(doc.layers[0].name, "Background", "Background unchanged");
    assert_eq!(doc.layers[1].name, "locked");
    assert!(!move_path(&mut doc, "1", -1), "fully locked refuses");
    assert_eq!(doc.layers[0].name, "Background", "locked state unchanged");
    assert_eq!(doc.layers[1].name, "locked");

    assert!(move_path(&mut doc, "2", -1), "ordinary layer moves");
    assert_eq!(doc.layers[1].name, "normal");
    assert_eq!(doc.layers[2].name, "locked");
}
