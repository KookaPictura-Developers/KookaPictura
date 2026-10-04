use super::*;
use pictura_core::{BitDepth, Channel, CharacterOverrides, ColorMode};

fn doc(w: u32, h: u32, seed: u8) -> Document {
    let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
    doc.composite.data = vec![seed; (w * h * 3) as usize].into();
    doc
}

fn snap(seed: u8) -> Snapshot {
    Snapshot {
        doc: doc(1, 1, seed),
        selection: None,
    }
}

#[test]
fn capture_undo_round_trip_restores_doc_and_selection() {
    let mut history = History::default();
    let open = Snapshot {
        doc: doc(2, 1, 7),
        selection: None,
    };
    let selection = Selection {
        width: 2,
        height: 1,
        data: vec![255, 0],
    };
    let selected = Snapshot {
        doc: doc(2, 1, 7),
        selection: Some(selection.clone()),
    };
    history.capture(open, "Open");
    history.capture(selected, "Select All");

    let restored = history.undo().expect("undo after capture");
    assert_eq!((restored.doc.width, restored.doc.height), (2, 1));
    assert!(restored.selection.is_none());

    let forwarded = history.redo().expect("redo after undo");
    assert_eq!(forwarded.selection, Some(selection));
    assert_eq!(forwarded.doc.composite.data, doc(2, 1, 7).composite.data);
}

#[test]
fn style_sheet_edits_are_one_undoable_state_each() {
    let mut history = History::default();
    let mut doc = doc(1, 1, 7);
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Open",
    );

    doc.text_styles
        .create_character_style(
            "Heading",
            CharacterOverrides {
                size: Some(10.0),
                ..Default::default()
            },
        )
        .unwrap();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "New Character Style",
    );

    doc.text_styles
        .edit_character_style(
            "Heading",
            CharacterOverrides {
                size: Some(24.0),
                ..Default::default()
            },
        )
        .unwrap();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Edit Character Style",
    );

    doc.text_styles.delete_character_style("Heading").unwrap();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Delete Character Style",
    );

    assert_eq!(history.count(), 4);
    assert_eq!(history.depth(), 3);
    assert_eq!(
        history
            .undo()
            .unwrap()
            .doc
            .text_styles
            .character_style("Heading")
            .unwrap()
            .attrs
            .size,
        Some(24.0)
    );
    assert_eq!(
        history
            .undo()
            .unwrap()
            .doc
            .text_styles
            .character_style("Heading")
            .unwrap()
            .attrs
            .size,
        Some(10.0)
    );
    assert!(history
        .undo()
        .unwrap()
        .doc
        .text_styles
        .character_style("Heading")
        .is_none());
    assert_eq!(
        history
            .redo()
            .unwrap()
            .doc
            .text_styles
            .character_style("Heading")
            .unwrap()
            .attrs
            .size,
        Some(10.0)
    );
}

#[test]
fn applying_a_style_is_one_undoable_state() {
    use pictura_core::{
        BitDepth, CharacterOverrides, ColorMode, Justify, ParagraphOverrides, TypeSpec,
    };
    use pictura_render::{add_type_layer, apply_type_style, type_layer_spec};

    let mut doc = Document::new(128, 64, ColorMode::Rgb, BitDepth::Eight);
    let path = add_type_layer(
        &mut doc,
        "",
        &TypeSpec::new("Style", "Liberation Sans", 20.0),
    );
    assert!(!path.is_empty(), "a type layer");
    doc.text_styles
        .create_character_style(
            "Big",
            CharacterOverrides {
                font_family: Some("Liberation Sans".into()),
                size: Some(40.0),
                ..Default::default()
            },
        )
        .unwrap();

    let mut history = History::default();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Open",
    );
    assert_eq!(history.depth(), 0);

    assert!(apply_type_style(&mut doc, &path, "Big", false));
    assert_eq!(
        type_layer_spec(&doc.layers[0]).unwrap().character.size,
        40.0
    );
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Apply Type Style",
    );
    assert_eq!(history.depth(), 1, "one state for the apply");

    let undone = history.undo().expect("undo the apply");
    assert_eq!(
        type_layer_spec(&undone.doc.layers[0])
            .unwrap()
            .character
            .size,
        20.0
    );

    // A paragraph style resolves both its character and paragraph sets, but the
    // already-applied character style outranks its character defaults; an
    // unknown name leaves the layer alone.
    doc.text_styles
        .create_paragraph_style(
            "Heading",
            CharacterOverrides {
                font_family: Some("Liberation Sans".into()),
                size: Some(30.0),
                ..Default::default()
            },
            ParagraphOverrides {
                justify: Some(Justify::Center),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(apply_type_style(&mut doc, &path, "Heading", true));
    let spec = type_layer_spec(&doc.layers[0]).unwrap();
    assert_eq!(
        spec.character.size, 40.0,
        "the applied character style still outranks the paragraph default"
    );
    assert_eq!(spec.paragraph.justify, Justify::Center);

    let before = doc.layers[0].type_tool.clone();
    assert!(!apply_type_style(&mut doc, &path, "Ghost", false));
    assert_eq!(doc.layers[0].type_tool, before);
}

#[test]
fn selection_none_round_trips() {
    let mut history = History::default();
    let doc0 = doc(2, 2, 3);
    history.capture(
        Snapshot {
            doc: doc0.clone(),
            selection: Some(Selection {
                width: 2,
                height: 2,
                data: vec![255; 4],
            }),
        },
        "Select All",
    );
    history.capture(
        Snapshot {
            doc: doc(3, 3, 4),
            selection: None,
        },
        "Deselect",
    );

    let cleared = history.undo().expect("undo to selection");
    assert_eq!(cleared.selection.unwrap().data, vec![255; 4]);

    let restored = history.redo().expect("redo to none");
    assert_eq!(restored.doc, doc(3, 3, 4));
    assert!(restored.selection.is_none());
}

#[test]
fn empty_stacks_refuse_undo_and_redo() {
    let mut history = History::default();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
    assert_eq!(history.depth(), 0);
    assert_eq!(history.count(), 0);

    assert!(history.undo().is_none());
    assert!(history.redo().is_none());
    assert!(history.jump(0).is_none());
    assert_eq!(history.depth(), 0);
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn redo_returns_the_stashed_post_state() {
    let mut history = History::default();
    history.capture(snap(0), "open");
    history.capture(snap(1), "op");

    let restored = history.undo().expect("undo state");
    assert_eq!(restored.doc.composite.data, vec![0, 0, 0]);
    assert_eq!(history.depth(), 0);
    assert!(history.can_redo());

    let forwarded = history.redo().expect("redo state");
    assert_eq!(forwarded.doc.composite.data, vec![1, 1, 1]);
    assert_eq!(history.depth(), 1);
    assert!(history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn depth_bounded_at_20_and_drops_oldest() {
    let mut history = History::default();
    history.capture(snap(99), "initial");
    for i in 0..21u8 {
        history.capture(snap(i), "op");
    }
    // One state per undoable step plus the current state, capped at depth 20.
    assert_eq!(history.depth(), 20);
    assert_eq!(history.count(), 21);

    let mut last = None;
    for _ in 0..20 {
        last = history.undo();
    }
    let oldest_kept = last.expect("state 20");
    assert_eq!(oldest_kept.doc.composite.data, vec![0, 0, 0]);
    assert_eq!(history.depth(), 0);
    assert!(history.undo().is_none());
}

#[test]
fn new_capture_truncates_redo() {
    let mut history = History::default();
    history.capture(snap(0), "open");
    history.capture(snap(1), "op");
    assert!(history.undo().is_some());
    assert!(history.can_redo());

    history.capture(snap(2), "op2");
    assert!(!history.can_redo());
    assert_eq!(history.depth(), 1);
    assert!(history.can_undo());
}

#[test]
fn labels_track_captures_and_undo_redo_position() {
    let mut history = History::default();
    history.capture(snap(0), "Open");
    history.capture(snap(1), "Select All");
    history.capture(snap(2), "Filter");

    assert_eq!(history.count(), 3);
    assert_eq!(history.index(), 2);
    assert_eq!(history.label(0), "Open");
    assert_eq!(history.label(1), "Select All");
    assert_eq!(history.label(2), "Filter");
    assert_eq!(history.label(9), "");

    history.undo();
    assert_eq!(history.index(), 1);
    history.undo();
    assert_eq!(history.index(), 0);
    history.redo();
    assert_eq!(history.index(), 1);
}

#[test]
fn jump_restores_state_and_moves_position() {
    let mut history = History::default();
    history.capture(snap(0), "Open");
    history.capture(snap(1), "A");
    history.capture(snap(2), "B");

    let jumped = history.jump(0).expect("state 0");
    assert_eq!(jumped.doc.composite.data, vec![0, 0, 0]);
    assert_eq!(history.index(), 0);
    assert!(history.can_redo());
    assert!(!history.can_undo());

    let forward = history.jump(2).expect("state 2");
    assert_eq!(forward.doc.composite.data, vec![2, 2, 2]);
    assert_eq!(history.index(), 2);

    assert!(history.jump(9).is_none());
    assert_eq!(history.index(), 2);
}

#[test]
fn snapshots_are_capped_and_restorable() {
    let mut history = History::default();
    for i in 0..12u8 {
        history.add_snapshot(&format!("snap {i}"), snap(i));
    }
    assert_eq!(history.snapshot_count(), 10);
    // The two oldest snapshots were dropped.
    assert_eq!(history.snapshot_label(0), "snap 2");
    assert_eq!(history.snapshot_label(9), "snap 11");
    assert_eq!(history.snapshot_label(10), "");

    let restored = history.snapshot(3).expect("snapshot 3");
    assert_eq!(restored.doc.composite.data, vec![5, 5, 5]);
    assert!(history.snapshot(99).is_none());
}

#[test]
fn the_brush_source_follows_its_state_and_is_pinned_before_it_is_dropped() {
    let mut history = History::default();
    history.capture(snap(0), "Open");
    assert_eq!(history.brush_source(), BrushSource::Oldest);
    assert_eq!(
        history.brush_source_doc().unwrap().doc.composite.data,
        vec![0, 0, 0]
    );
    for i in 1..=3u8 {
        history.capture(snap(i), "Brush");
    }
    assert!(history.set_brush_source(BrushSource::State(2)));
    assert!(!history.set_brush_source(BrushSource::State(9)));
    assert_eq!(history.brush_source(), BrushSource::State(2));
    // Depth pruning shifts the index down with its state.
    for i in 4..=22u8 {
        history.capture(snap(i), "Brush");
    }
    assert_eq!(history.brush_source(), BrushSource::State(0));
    assert_eq!(
        history.brush_source_doc().unwrap().doc.composite.data,
        vec![2, 2, 2]
    );
    // Dropping it pins a copy instead of reading another state.
    history.capture(snap(23), "Brush");
    assert_eq!(history.brush_source(), BrushSource::Pinned);
    assert_eq!(
        history.brush_source_doc().unwrap().doc.composite.data,
        vec![2, 2, 2]
    );

    // A redo state discarded by a new capture is pinned the same way.
    assert!(history.set_brush_source(BrushSource::State(20)));
    history.undo();
    history.undo();
    history.capture(snap(99), "Brush");
    assert_eq!(history.brush_source(), BrushSource::Pinned);
    assert_eq!(
        history.brush_source_doc().unwrap().doc.composite.data,
        vec![23, 23, 23]
    );

    for i in 0..11u8 {
        history.add_snapshot("s", snap(100 + i));
        if i == 0 {
            assert!(history.set_brush_source(BrushSource::Snapshot(0)));
        }
    }
    assert_eq!(history.brush_source(), BrushSource::Pinned);
    assert_eq!(
        history.brush_source_doc().unwrap().doc.composite.data,
        vec![100, 100, 100]
    );
}

#[test]
fn region_deltas_retain_only_changed_tiles() {
    let mut history = History::default();
    let mut doc = doc(256, 256, 0);
    let full_state_bytes = doc.composite.data.len();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Open",
    );

    // Ten edits, each touching only the first 64×64 tile.
    for i in 1..=10u8 {
        doc.composite.data[0] = i;
        history.capture(
            Snapshot {
                doc: doc.clone(),
                selection: None,
            },
            "Paint",
        );
    }

    assert_eq!(
        history.full_state_count(),
        1,
        "only the base is a full state"
    );
    let retained = history.retained_tile_bytes();
    assert!(
        retained <= 10 * TILE * TILE * 2,
        "retained {retained} bytes exceeds the changed-tile bound"
    );
    assert!(
        retained < full_state_bytes,
        "retained {retained} bytes should be far below a full state ({full_state_bytes})"
    );

    // Every state still materializes its exact edited byte.
    for i in 0..=10u8 {
        let restored = history.jump(i as usize).expect("state");
        assert_eq!(restored.doc.composite.data[0], i);
    }
}

#[test]
fn every_state_matches_a_reference_copy() {
    let mut doc = doc(200, 130, 5);
    doc.layers = vec![Layer {
        name: "paint".into(),
        rect: pictura_core::PsdRect {
            top: 0,
            left: 0,
            bottom: 130,
            right: 200,
        },
        channels: vec![Channel {
            id: 0,
            data: vec![9u8; 200 * 130].into(),
        }],
        ..Default::default()
    }];

    let mut history = History::default();
    let mut reference: Vec<Snapshot> = Vec::new();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Open",
    );
    reference.push(Snapshot {
        doc: doc.clone(),
        selection: None,
    });

    // A mix of pixel edits, a metadata-only change, and the region path.
    let mut seed = 7u8;
    for step in 0..8usize {
        seed = seed.wrapping_mul(31).wrapping_add(step as u8);
        for k in 0..500usize {
            let idx = (seed as usize * 137 + k * 61) % doc.composite.data.len();
            doc.composite.data[idx] ^= (seed ^ k as u8) | 1;
        }
        let layer = &mut doc.layers[0];
        for k in 0..300usize {
            let idx = (seed as usize * 53 + k * 97) % layer.channels[0].data.len();
            layer.channels[0].data[idx] = seed.wrapping_add(k as u8);
        }
        if step == 3 {
            doc.layers[0].visible = false;
        }
        let selection = Some(Selection {
            width: 200,
            height: 130,
            data: vec![seed; 200 * 130],
        });
        doc.composite.data[0] = seed;
        let snapshot = Snapshot {
            doc: doc.clone(),
            selection,
        };
        history.capture(snapshot.clone(), "Edit");
        reference.push(snapshot);
    }

    for (i, expected) in reference.iter().enumerate() {
        let jumped = history.jump(i).expect("state");
        assert_eq!(&jumped.doc, &expected.doc, "doc differs at state {i}");
        assert_eq!(jumped.selection, expected.selection, "selection at {i}");
    }

    // Walk down and up through undo/redo, still byte-identical.
    while let Some(snapshot) = history.undo() {
        let i = history.index();
        assert_eq!(&snapshot.doc, &reference[i].doc);
    }
    while let Some(snapshot) = history.redo() {
        let i = history.index();
        assert_eq!(&snapshot.doc, &reference[i].doc);
    }
}

#[test]
fn geometry_change_uses_a_full_anchor_and_still_undoes() {
    let mut history = History::default();
    let open = doc(32, 32, 4);
    history.capture(
        Snapshot {
            doc: open.clone(),
            selection: None,
        },
        "Open",
    );

    let resized = doc(48, 48, 9);
    history.capture(
        Snapshot {
            doc: resized.clone(),
            selection: None,
        },
        "Resize",
    );

    let restored = history.undo().expect("undo across geometry");
    assert_eq!(restored.doc, open);
    let forwarded = history.redo().expect("redo across geometry");
    assert_eq!(forwarded.doc, resized);
}

#[test]
fn paint_commit_undo_restores_the_prior_pixels() {
    let mut doc = Document::new(64, 64, ColorMode::Rgb, BitDepth::Eight);
    doc.layers = vec![Layer {
        name: "base".into(),
        rect: pictura_core::PsdRect {
            top: 0,
            left: 0,
            bottom: 64,
            right: 64,
        },
        channels: vec![
            Channel {
                id: 0,
                data: vec![10u8; 64 * 64].into(),
            },
            Channel {
                id: -1,
                data: vec![255u8; 64 * 64].into(),
            },
        ],
        ..Default::default()
    }];
    let prior_layer = doc.layers[0].channels[0].data.clone();
    let prior_composite = doc.composite.data.clone();

    let mut history = History::default();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Open",
    );

    // A paint commit: dab bytes into the layer plane and recomposite.
    let mut painted = doc.clone();
    for p in painted.layers[0].channels[0].data[..64 * 32].iter_mut() {
        *p = 200;
    }
    for p in painted.composite.data[..64 * 32].iter_mut() {
        *p = 200;
    }
    history.capture(
        Snapshot {
            doc: painted,
            selection: None,
        },
        "Brush",
    );

    let undone = history.undo().expect("undo the paint");
    assert_eq!(undone.doc.layers[0].channels[0].data, prior_layer);
    assert_eq!(undone.doc.composite.data, prior_composite);

    let redone = history.redo().expect("redo the paint");
    assert_eq!(redone.doc.layers[0].channels[0].data[0], 200);
    assert_eq!(redone.doc.layers[0].channels[0].data[32 * 64], 10);
    assert_eq!(redone.doc.composite.data[0], 200);
}

#[test]
fn editing_a_style_through_the_operation_updates_every_applying_layer() {
    use pictura_core::{CharacterOverrides, TypeSpec};
    use pictura_render::{add_type_layer, apply_type_style, type_layer_spec};

    let mut doc = Document::new(256, 128, ColorMode::Rgb, BitDepth::Eight);
    let first = add_type_layer(&mut doc, "", &TypeSpec::new("Aa", "Liberation Sans", 20.0));
    let second = add_type_layer(&mut doc, "", &TypeSpec::new("Bb", "Liberation Sans", 20.0));
    assert!((first.as_str(), second.as_str()) == ("0", "1"));

    create_character_style(
        &mut doc,
        "Big",
        CharacterOverrides {
            size: Some(40.0),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(apply_type_style(&mut doc, &first, "Big", false));
    assert!(apply_type_style(&mut doc, &second, "Big", false));
    assert_eq!(
        type_layer_spec(&doc.layers[0]).unwrap().character.size,
        40.0
    );
    assert_eq!(
        type_layer_spec(&doc.layers[1]).unwrap().character.size,
        40.0
    );

    // One edit propagates to both layers, which still reference the style.
    edit_character_style(
        &mut doc,
        "Big",
        CharacterOverrides {
            size: Some(60.0),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        type_layer_spec(&doc.layers[0]).unwrap().character.size,
        60.0
    );
    assert_eq!(
        type_layer_spec(&doc.layers[1]).unwrap().character.size,
        60.0
    );
    assert_eq!(
        type_layer_spec(&doc.layers[0])
            .unwrap()
            .applied_character_style
            .as_deref(),
        Some("Big")
    );
}

#[test]
fn a_style_operation_records_one_undoable_state_each() {
    let mut history = History::default();
    let mut doc = doc(1, 1, 7);
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Open",
    );

    create_character_style(
        &mut doc,
        "Heading",
        CharacterOverrides {
            size: Some(10.0),
            ..Default::default()
        },
    )
    .unwrap();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "New Character Style",
    );
    assert_eq!(history.depth(), 1, "one state for the create");

    edit_character_style(
        &mut doc,
        "Heading",
        CharacterOverrides {
            size: Some(24.0),
            ..Default::default()
        },
    )
    .unwrap();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Edit Character Style",
    );
    assert_eq!(history.depth(), 2, "one state for the edit");
    assert_eq!(
        history
            .undo()
            .unwrap()
            .doc
            .text_styles
            .character_style("Heading")
            .unwrap()
            .attrs
            .size,
        Some(10.0)
    );

    delete_character_style(&mut doc, "Heading").unwrap();
    history.capture(
        Snapshot {
            doc: doc.clone(),
            selection: None,
        },
        "Delete Character Style",
    );
    assert_eq!(history.depth(), 2, "one state for the delete");
    assert!(history
        .undo()
        .unwrap()
        .doc
        .text_styles
        .character_style("Heading")
        .is_some());
}
