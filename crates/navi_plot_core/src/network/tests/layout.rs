use super::*;

#[test]
fn network_layout_positions_are_finite() {
    let spec = sample_spec();
    let layout = fr_layout(&spec);
    for (_, &(x, y)) in &layout {
        assert!(x.is_finite(), "x={x} must be finite");
        assert!(y.is_finite(), "y={y} must be finite");
    }
}

#[test]
fn network_seed_position_prefers_open_parent_gap() {
    let spec = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(0.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "left".to_string(),
                label: "Left".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "right".to_string(),
                label: "Right".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "new".to_string(),
                label: "New".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "left".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "right".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "new".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        ..sample_spec()
    };
    let positions = BTreeMap::from([
        ("root".to_string(), (0.0, 0.0)),
        ("left".to_string(), (-WORLD_NODE_SPACING, 0.0)),
        ("right".to_string(), (WORLD_NODE_SPACING, 0.0)),
    ]);

    let seeded = seed_position(&spec, "new", &positions);

    assert!(seeded.1.abs() > WORLD_NODE_SPACING * 0.5);
    assert!(seeded.0.abs() < WORLD_NODE_SPACING * 0.4);
}

#[test]
fn network_structural_helpers_ignore_lightweight_sibling_edges() {
    let spec = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(0.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "alpha".to_string(),
                label: "Alpha".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "beta".to_string(),
                label: "Beta".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "alpha".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "beta".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "alpha".to_string(),
                target: "beta".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
        ],
        ..sample_spec()
    };

    assert_eq!(parent_ids(&spec, "beta"), vec!["root".to_string()]);
    assert_eq!(
        child_ids(&spec, "root"),
        vec!["alpha".to_string(), "beta".to_string()]
    );
    assert!(child_ids(&spec, "alpha").is_empty());
}

#[test]
fn network_user_supplied_positions_are_used_directly() {
    let spec = positioned_spec();
    let layout = compute_layout(&spec).unwrap();
    assert_eq!(layout["a"], (100.0, 100.0));
    assert_eq!(layout["b"], (300.0, 200.0));
}

#[test]
fn network_user_supplied_positions_are_not_clamped_to_canvas() {
    let mut spec = positioned_spec();
    spec.nodes[0].x = Some(-40.0);
    spec.nodes[0].y = Some(420.0);

    let layout = compute_layout(&spec).unwrap();

    assert_eq!(layout["a"], (-40.0, 420.0));
}

#[test]
fn network_fr_layout_is_deterministic() {
    let spec = sample_spec();
    let layout1 = fr_layout(&spec);
    let layout2 = fr_layout(&spec);
    for (id, &pos1) in &layout1 {
        let pos2 = layout2[id];
        assert!((pos1.0 - pos2.0).abs() < 0.001 && (pos1.1 - pos2.1).abs() < 0.001);
    }
}

#[test]
fn network_mixed_layout_pins_supplied_nodes_and_places_free_ones() {
    let spec = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "a".into(),
                label: "A".into(),
                color: None,
                x: Some(200.0),
                y: Some(150.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "b".into(),
                label: "B".into(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "c".into(),
                label: "C".into(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![NetworkEdge {
            source: "a".into(),
            target: "b".into(),
            label: None,
            color: None,
            weight: None,
            style: None,
        }],
        ..sample_spec()
    };
    let layout = compute_layout(&spec).unwrap();
    assert_eq!(layout["a"], (200.0, 150.0));
    for id in ["b", "c"] {
        let (x, y) = layout[id];
        assert!(x.is_finite(), "{id} x={x} must be finite");
        assert!(y.is_finite(), "{id} y={y} must be finite");
        assert_ne!((x, y), layout["a"]);
    }
}

#[test]
fn network_layout_separates_long_sibling_labels() {
    let spec = NetworkPlotSpec {
        width: 960,
        height: 720,
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(0.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "alpha".to_string(),
                label: "Alpha label needs clearance".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "beta".to_string(),
                label: "Beta label needs clearance".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "gamma".to_string(),
                label: "Gamma label needs clearance".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "alpha".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "beta".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "gamma".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        layout_iterations: 220,
        node_radius: 24,
        ..sample_spec()
    };

    let layout = compute_layout(&spec).unwrap();

    assert_no_layout_collisions(&spec, &layout, 1.0);
}

#[test]
fn network_layout_spreads_root_siblings_radially() {
    let spec = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(0.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "alpha".to_string(),
                label: "Alpha chapter with a long label".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "beta".to_string(),
                label: "Beta chapter with a long label".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "gamma".to_string(),
                label: "Gamma chapter with a long label".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "alpha".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "beta".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "gamma".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "alpha".to_string(),
                target: "beta".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
            NetworkEdge {
                source: "beta".to_string(),
                target: "gamma".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
        ],
        layout_iterations: 220,
        node_radius: 24,
        ..sample_spec()
    };

    let layout = compute_layout(&spec).unwrap();

    let xs = [layout["alpha"].0, layout["beta"].0, layout["gamma"].0];
    let ys = [layout["alpha"].1, layout["beta"].1, layout["gamma"].1];

    assert!(xs.iter().copied().any(|x| x < -WORLD_NODE_SPACING * 0.2));
    assert!(xs.iter().copied().any(|x| x > WORLD_NODE_SPACING * 0.2));
    assert!(ys.iter().copied().any(|y| y < -WORLD_NODE_SPACING * 0.2));
    assert!(ys.iter().copied().any(|y| y > WORLD_NODE_SPACING * 0.2));
    assert_no_layout_collisions(&spec, &layout, 1.0);
}

#[test]
fn network_layout_spreads_nested_siblings_around_parent() {
    let spec = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(0.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "chapter".to_string(),
                label: "Chapter".to_string(),
                color: None,
                x: Some(WORLD_NODE_SPACING * 1.4),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "section-a".to_string(),
                label: "Section A with a long label".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "section-b".to_string(),
                label: "Section B with a long label".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "section-c".to_string(),
                label: "Section C with a long label".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "chapter".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "chapter".to_string(),
                target: "section-a".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "chapter".to_string(),
                target: "section-b".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "chapter".to_string(),
                target: "section-c".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "section-a".to_string(),
                target: "section-b".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
            NetworkEdge {
                source: "section-b".to_string(),
                target: "section-c".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
        ],
        layout_iterations: 220,
        node_radius: 24,
        ..sample_spec()
    };

    let layout = compute_layout(&spec).unwrap();
    let chapter = layout["chapter"];
    let child_positions = [
        layout["section-a"],
        layout["section-b"],
        layout["section-c"],
    ];

    assert!(child_positions.iter().all(|&(x, y)| {
        let dx = x - chapter.0;
        let dy = y - chapter.1;
        (dx * dx + dy * dy).sqrt() > WORLD_NODE_SPACING * 0.35
    }));
    assert!(child_positions
        .iter()
        .copied()
        .any(|(_, y)| y < chapter.1 - WORLD_NODE_SPACING * 0.2));
    assert!(child_positions
        .iter()
        .copied()
        .any(|(_, y)| y > chapter.1 + WORLD_NODE_SPACING * 0.2));
    assert_no_layout_collisions(&spec, &layout, 1.0);
}

#[test]
fn network_layout_pushes_crowded_sibling_groups_farther_out_than_sparse_ones() {
    let spec = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(0.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "sparse".to_string(),
                label: "Sparse chapter".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "dense".to_string(),
                label: "Dense chapter".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "sparse-leaf".to_string(),
                label: "Sparse leaf".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "dense-a".to_string(),
                label: "Dense leaf A".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "dense-b".to_string(),
                label: "Dense leaf B".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "dense-c".to_string(),
                label: "Dense leaf C".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "dense-d".to_string(),
                label: "Dense leaf D".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "sparse".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "dense".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "sparse".to_string(),
                target: "sparse-leaf".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "dense".to_string(),
                target: "dense-a".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "dense".to_string(),
                target: "dense-b".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "dense".to_string(),
                target: "dense-c".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "dense".to_string(),
                target: "dense-d".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "sparse".to_string(),
                target: "dense".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
            NetworkEdge {
                source: "dense-a".to_string(),
                target: "dense-b".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
            NetworkEdge {
                source: "dense-b".to_string(),
                target: "dense-c".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
            NetworkEdge {
                source: "dense-c".to_string(),
                target: "dense-d".to_string(),
                label: None,
                color: None,
                weight: Some(0.15),
                style: None,
            },
        ],
        layout_iterations: 220,
        node_radius: 24,
        ..sample_spec()
    };

    let layout = compute_layout(&spec).unwrap();
    let root = layout["root"];
    let sparse = layout["sparse"];
    let sparse_leaf = layout["sparse-leaf"];
    let dense = layout["dense"];
    let dense_children = [
        layout["dense-a"],
        layout["dense-b"],
        layout["dense-c"],
        layout["dense-d"],
    ];

    let sparse_parent_distance =
        ((sparse_leaf.0 - sparse.0).powi(2) + (sparse_leaf.1 - sparse.1).powi(2)).sqrt();
    let dense_parent_distance = dense_children
        .iter()
        .map(|&(x, y)| ((x - dense.0).powi(2) + (y - dense.1).powi(2)).sqrt())
        .sum::<f64>()
        / dense_children.len() as f64;
    let sparse_root_distance =
        ((sparse_leaf.0 - root.0).powi(2) + (sparse_leaf.1 - root.1).powi(2)).sqrt();
    let dense_root_distance = dense_children
        .iter()
        .map(|&(x, y)| ((x - root.0).powi(2) + (y - root.1).powi(2)).sqrt())
        .sum::<f64>()
        / dense_children.len() as f64;

    assert!(dense_parent_distance > sparse_parent_distance + WORLD_NODE_SPACING * 0.15);
    assert!(dense_root_distance > sparse_root_distance + WORLD_NODE_SPACING * 0.15);
    assert_no_layout_collisions(&spec, &layout, 1.0);
}
