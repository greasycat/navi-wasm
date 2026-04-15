use super::*;
use crate::node::ResolvedNodeMediaKind;
use crate::types::{
    BuiltinNodeIcon, GraphEdgeStyle, GraphNodeStyle, NetworkEdge, NetworkNode, NetworkPlotSpec,
    NetworkView, NodeMedia, NodeMediaFit, NodeMediaKind, NodeShape, SelectionStyle,
};
use plotters::drawing::IntoDrawingArea;
use plotters_svg::SVGBackend;

fn sample_spec() -> NetworkPlotSpec {
    NetworkPlotSpec {
        width: 480,
        height: 360,
        title: "Test Network".to_string(),
        nodes: vec![
            NetworkNode {
                id: "a".to_string(),
                label: "A".to_string(),
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
                id: "b".to_string(),
                label: "B".to_string(),
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
                id: "c".to_string(),
                label: "C".to_string(),
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
                source: "a".to_string(),
                target: "b".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        node_radius: 16,
        default_node_style: None,
        default_edge_style: None,
        selection_style: None,
        margin: 40,
        offset_x: 0,
        offset_y: 0,
        selected_node_id: None,
        layout_iterations: 50,
        show_arrows: true,
        show_labels: true,
        pixel_ratio: 1.0,
    }
}

fn positioned_spec() -> NetworkPlotSpec {
    NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "a".to_string(),
                label: "A".to_string(),
                color: None,
                x: Some(100.0),
                y: Some(100.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "b".to_string(),
                label: "B".to_string(),
                color: None,
                x: Some(300.0),
                y: Some(200.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![NetworkEdge {
            source: "a".to_string(),
            target: "b".to_string(),
            label: None,
            color: None,
            weight: None,
            style: None,
        }],
        ..sample_spec()
    }
}

fn toggleable_positioned_spec(expanded: bool) -> NetworkPlotSpec {
    NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(60.0),
                y: Some(100.0),
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
                x: Some(140.0),
                y: Some(100.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: BTreeMap::from([
                    (TOGGLEABLE_PROPERTY_KEY.to_string(), "true".to_string()),
                    (
                        EXPANDED_PROPERTY_KEY.to_string(),
                        if expanded { "true" } else { "false" }.to_string(),
                    ),
                ]),
            },
            NetworkNode {
                id: "leaf".to_string(),
                label: "Leaf".to_string(),
                color: None,
                x: Some(220.0),
                y: Some(100.0),
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
                target: "leaf".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
        ],
        ..sample_spec()
    }
}

fn assert_no_layout_collisions(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    view_zoom: f64,
) {
    if !ENABLE_LAYOUT_COLLISIONS {
        return;
    }
    let resolved = resolve_nodes(spec).unwrap();
    let positions: Vec<(f64, f64)> = spec
        .nodes
        .iter()
        .map(|node| {
            layout
                .get(&node.id)
                .copied()
                .expect("layout contains every node")
        })
        .collect();
    let footprints = node_footprints(spec, &resolved, &positions, view_zoom);

    for source_idx in 0..footprints.len() {
        let source = footprints[source_idx];
        let source_id = spec.nodes[source_idx].id.as_str();
        for target_idx in (source_idx + 1)..footprints.len() {
            let target = footprints[target_idx];
            let target_id = spec.nodes[target_idx].id.as_str();
            assert!(
                circle_separation(
                    source_id,
                    target_id,
                    source.center,
                    source.radius,
                    target.center,
                    target.radius,
                )
                .is_none(),
                "node collision between {source_id} and {target_id}",
            );
            if let (Some(source_label), Some(target_label)) = (source.label, target.label) {
                assert!(
                    label_box_separation(source_label, target_label).is_none(),
                    "label collision between {source_id} and {target_id}",
                );
            }
            if let Some(source_label) = source.label {
                assert!(
                    circle_label_separation(target.center, target.radius, source_label).is_none(),
                    "label of {source_id} overlaps node {target_id}",
                );
            }
            if let Some(target_label) = target.label {
                assert!(
                    circle_label_separation(source.center, source.radius, target_label).is_none(),
                    "label of {target_id} overlaps node {source_id}",
                );
            }
        }
    }
}

#[test]
fn network_rejects_empty_nodes() {
    let mut spec = sample_spec();
    spec.nodes.clear();
    spec.edges.clear();
    let err = NetworkSession::new(spec).unwrap_err();
    assert_eq!(err, PlotError::EmptyNetwork);
}

#[test]
fn network_rejects_duplicate_node_ids() {
    let mut spec = sample_spec();
    spec.nodes[1].id = "a".to_string();
    let err = NetworkSession::new(spec).unwrap_err();
    assert!(matches!(err, PlotError::DuplicateNodeId { .. }));
}

#[test]
fn network_rejects_duplicate_edges() {
    let mut spec = sample_spec();
    spec.edges.push(spec.edges[0].clone());
    let err = NetworkSession::new(spec).unwrap_err();
    assert!(matches!(err, PlotError::DuplicateEdge { .. }));
}

#[test]
fn network_rejects_unknown_edge_nodes() {
    let mut spec = sample_spec();
    spec.edges.push(NetworkEdge {
        source: "a".to_string(),
        target: "z".to_string(),
        label: None,
        color: None,
        weight: None,
        style: None,
    });
    let err = NetworkSession::new(spec).unwrap_err();
    assert!(matches!(err, PlotError::UnknownNode { .. }));
}

#[test]
fn network_allows_cycles() {
    let mut spec = sample_spec();
    spec.edges.push(NetworkEdge {
        source: "c".to_string(),
        target: "a".to_string(),
        label: None,
        color: None,
        weight: None,
        style: None,
    });
    assert!(NetworkSession::new(spec).is_ok());
}

#[test]
fn network_allows_multiple_parents() {
    let mut spec = sample_spec();
    // Both "a" and "b" point to "c" — c has 2 parents
    spec.edges.push(NetworkEdge {
        source: "a".to_string(),
        target: "c".to_string(),
        label: None,
        color: None,
        weight: None,
        style: None,
    });
    assert!(NetworkSession::new(spec).is_ok());
}

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
fn network_svg_has_correct_circle_count() {
    let mut svg = String::new();
    let spec = positioned_spec();
    let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
    render_network_on(area, &spec).unwrap();
    assert_eq!(svg.matches("<circle").count(), spec.nodes.len());
}

#[test]
fn network_pan_updates_offsets() {
    let spec = sample_spec();
    let updated = pan_network_spec(&spec, 30.0, -15.0).unwrap();
    assert_eq!(updated.offset_x, 30);
    assert_eq!(updated.offset_y, -15);
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
fn network_hit_test_returns_closest_node() {
    let spec = positioned_spec();
    let session = NetworkSession::new(spec).unwrap();
    // Click near node "a" at (100, 100)
    let hit = session.pick_node(102.0, 98.0);
    assert_eq!(hit, Some("a".to_string()));
}

#[test]
fn network_toggle_badge_uses_outward_direction_and_state() {
    let spec = toggleable_positioned_spec(false);
    let session = NetworkSession::new(spec.clone()).unwrap();
    let parent_by_id = structural_parent_map(&spec);
    let badge = toggle_badge_for_node(
        &spec,
        &spec.nodes[1],
        &session.layout,
        &session.resolved,
        &session.view,
        &parent_by_id,
    )
    .expect("toggle badge");

    assert!(badge.center_x > 140);
    assert_eq!(badge.center_y, 100);
    assert!(!badge.expanded);

    let expanded_spec = toggleable_positioned_spec(true);
    let expanded_session = NetworkSession::new(expanded_spec.clone()).unwrap();
    let expanded_parents = structural_parent_map(&expanded_spec);
    let expanded_badge = toggle_badge_for_node(
        &expanded_spec,
        &expanded_spec.nodes[1],
        &expanded_session.layout,
        &expanded_session.resolved,
        &expanded_session.view,
        &expanded_parents,
    )
    .expect("expanded toggle badge");
    assert!(expanded_badge.expanded);
}

#[test]
fn network_toggle_badge_hit_distinguishes_toggle_from_node_body() {
    let spec = toggleable_positioned_spec(false);
    let session = NetworkSession::new(spec.clone()).unwrap();
    let parent_by_id = structural_parent_map(&spec);
    let badge = toggle_badge_for_node(
        &spec,
        &spec.nodes[1],
        &session.layout,
        &session.resolved,
        &session.view,
        &parent_by_id,
    )
    .expect("toggle badge");

    let badge_hit = session.pick(badge.center_x as f64, badge.center_y as f64);
    assert_eq!(
        badge_hit,
        Some(NetworkPickHit {
            kind: NetworkPickKind::Toggle,
            node_id: "chapter".to_string(),
        })
    );

    let node_hit = session.pick(140.0, 100.0);
    assert_eq!(
        node_hit,
        Some(NetworkPickHit {
            kind: NetworkPickKind::Node,
            node_id: "chapter".to_string(),
        })
    );

    assert!(toggle_badge_for_node(
        &spec,
        &spec.nodes[0],
        &session.layout,
        &session.resolved,
        &session.view,
        &parent_by_id,
    )
    .is_none());
    assert!(toggle_badge_for_node(
        &spec,
        &spec.nodes[2],
        &session.layout,
        &session.resolved,
        &session.view,
        &parent_by_id,
    )
    .is_none());
}

#[test]
fn network_mixed_layout_pins_supplied_nodes_and_places_free_ones() {
    // "a" has explicit coordinates; "b" and "c" do not.
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
    // Pinned node "a" must be exactly at its supplied coordinates
    assert_eq!(layout["a"], (200.0, 150.0));
    // Free nodes "b" and "c" must resolve to finite positions and stay distinct
    for id in ["b", "c"] {
        let (x, y) = layout[id];
        assert!(x.is_finite(), "{id} x={x} must be finite");
        assert!(y.is_finite(), "{id} y={y} must be finite");
        assert_ne!((x, y), layout["a"]);
    }
}

#[test]
fn network_non_circle_shapes_render_without_error() {
    // (shape, expected SVG element tag)
    let cases = [
        (NodeShape::Square, "rect"),
        (NodeShape::Diamond, "polygon"),
        (NodeShape::Triangle, "polygon"),
    ];
    for (shape, tag) in cases {
        let spec = NetworkPlotSpec {
            nodes: vec![NetworkNode {
                id: "x".into(),
                label: "X".into(),
                color: None,
                x: None,
                y: None,
                shape: Some(shape.clone()),
                label_inside: Some(true),
                style: None,
                media: None,
                properties: Default::default(),
            }],
            edges: vec![],
            ..sample_spec()
        };
        let mut svg = String::new();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_network_on(area, &spec).expect("render should succeed");
        assert_eq!(
            svg.matches("<circle").count(),
            0,
            "shape={shape:?} should not render circles"
        );
        assert!(
            svg.contains(&format!("<{tag}")),
            "shape={shape:?} should render <{tag}>"
        );
    }
}

#[test]
fn network_node_style_inheritance_and_overrides_resolve_in_order() {
    let mut spec = sample_spec();
    spec.default_node_style = Some(GraphNodeStyle {
        shape: Some(NodeShape::Square),
        radius: Some(22.0),
        label_visible: Some(false),
        ..Default::default()
    });
    spec.nodes[1].shape = Some(NodeShape::Diamond);
    spec.nodes[2].style = Some(GraphNodeStyle {
        shape: Some(NodeShape::Triangle),
        label_visible: Some(true),
        radius: Some(30.0),
        ..Default::default()
    });

    let resolved = resolve_nodes(&spec).unwrap();

    assert_eq!(resolved["a"].style.shape, NodeShape::Square);
    assert_eq!(resolved["b"].style.shape, NodeShape::Diamond);
    assert_eq!(resolved["c"].style.shape, NodeShape::Triangle);
    assert!(!resolved["a"].style.label_visible);
    assert!(resolved["c"].style.label_visible);
    assert_eq!(resolved["c"].style.radius, 30);
}

#[test]
fn network_hit_test_uses_per_node_radius_override() {
    let mut spec = positioned_spec();
    spec.node_radius = 12;
    spec.selection_style = Some(SelectionStyle {
        padding: Some(0.0),
        ..Default::default()
    });
    spec.nodes[0].style = Some(GraphNodeStyle {
        radius: Some(40.0),
        ..Default::default()
    });

    let session = NetworkSession::new(spec).unwrap();
    let hit = session.pick_node(135.0, 100.0);

    assert_eq!(hit.as_deref(), Some("a"));
}

#[test]
fn network_edge_labels_render_when_enabled() {
    let mut svg = String::new();
    let mut spec = positioned_spec();
    spec.default_edge_style = Some(GraphEdgeStyle {
        label_visible: Some(true),
        stroke_width: Some(3.0),
        ..Default::default()
    });
    spec.edges[0].label = Some("AB".to_string());
    let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();

    render_network_on(area, &spec).unwrap();

    assert!(svg.contains("AB"));
    assert!(svg.contains("stroke-width=\"3\""));
}

#[test]
fn network_render_nodes_exposes_media_metadata() {
    let mut spec = positioned_spec();
    spec.nodes[0].media = Some(NodeMedia {
        kind: NodeMediaKind::Image,
        icon: None,
        image_key: Some("survey-hero".to_string()),
        fit: NodeMediaFit::Cover,
        scale: Some(0.75),
        tint_color: None,
        fallback_icon: Some(BuiltinNodeIcon::Camera),
    });

    let nodes = network_render_nodes(&spec).unwrap();
    let node = nodes.iter().find(|node| node.id == "a").unwrap();

    assert!(matches!(
        node.media.as_ref().map(|media| &media.kind),
        Some(ResolvedNodeMediaKind::Image {
            image_key,
            fit: NodeMediaFit::Cover,
            fallback_icon: Some(BuiltinNodeIcon::Camera),
        }) if image_key == "survey-hero"
    ));
}

#[test]
fn network_render_nodes_cull_offscreen_nodes() {
    let mut spec = positioned_spec();
    spec.nodes[0].x = Some(-120.0);
    spec.nodes[0].y = Some(100.0);

    let session = NetworkSession::new(spec).unwrap();
    let nodes = session.render_nodes();

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, "b");
}

#[test]
fn network_session_update_preserves_existing_positions() {
    let spec = positioned_spec();
    let mut session = NetworkSession::new(spec.clone()).unwrap();
    let before = session.layout.clone();
    let updated = NetworkPlotSpec {
        nodes: vec![
            spec.nodes[0].clone(),
            spec.nodes[1].clone(),
            NetworkNode {
                id: "c".to_string(),
                label: "C".to_string(),
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
            spec.edges[0].clone(),
            NetworkEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        ..spec
    };

    session.update_spec(updated).unwrap();

    assert_eq!(session.layout["a"], before["a"]);
    assert_eq!(session.layout["b"], before["b"]);
    assert!(session.layout.contains_key("c"));
}

#[test]
fn network_selection_only_update_does_not_create_transition() {
    let spec = positioned_spec();
    let mut session = NetworkSession::new(spec.clone()).unwrap();

    session
        .update_spec(NetworkPlotSpec {
            selected_node_id: Some("b".to_string()),
            ..spec
        })
        .unwrap();

    assert!(!session.has_transition());
}

#[test]
fn network_tracking_rejects_missing_path_edges() {
    let mut session = NetworkSession::new(sample_spec()).unwrap();

    let err = session
        .set_tracking_path(vec!["a".to_string(), "c".to_string()])
        .unwrap_err();

    assert_eq!(
        err,
        PlotError::InvalidNetworkPath {
            from_node: "a".to_string(),
            to_node: "c".to_string(),
        }
    );
}

#[test]
fn network_tracking_accepts_reverse_edge_lookup() {
    let spec = NetworkPlotSpec {
        edges: vec![NetworkEdge {
            source: "b".to_string(),
            target: "a".to_string(),
            label: None,
            color: None,
            weight: None,
            style: None,
        }],
        ..positioned_spec()
    };
    let mut session = NetworkSession::new(spec).unwrap();

    session
        .set_tracking_path(vec!["a".to_string(), "b".to_string()])
        .unwrap();

    let tracking = session.tracking.as_ref().expect("tracking path");
    assert_eq!(tracking.node_ids, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(tracking.current_node_index(), 0);
}

#[test]
fn network_tracking_progress_advances_edges_and_current_node() {
    let mut session = NetworkSession::new(sample_spec()).unwrap();
    session
        .set_tracking_path(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        .unwrap();

    let tracking = session.tracking.as_ref().expect("tracking path");
    assert_eq!(tracking.current_node_index(), 0);
    assert_eq!(tracking.edge_completion(0), 0.0);

    session.set_tracking_progress(0.25);
    let tracking = session.tracking.as_ref().expect("tracking path");
    assert_eq!(tracking.current_node_index(), 0);
    assert!((tracking.edge_completion(0) - 0.5).abs() < 1e-6);
    assert_eq!(tracking.edge_completion(1), 0.0);

    session.set_tracking_progress(0.75);
    let tracking = session.tracking.as_ref().expect("tracking path");
    assert_eq!(tracking.current_node_index(), 1);
    assert_eq!(tracking.edge_completion(0), 1.0);
    assert!((tracking.edge_completion(1) - 0.5).abs() < 1e-6);

    session.set_tracking_progress(1.0);
    assert_eq!(
        session
            .tracking
            .as_ref()
            .map(NetworkTrackedPath::current_node_index),
        Some(2)
    );
}

#[test]
fn network_tracking_breath_phase_wraps_inputs() {
    let mut session = NetworkSession::new(sample_spec()).unwrap();
    session
        .set_tracking_path(vec!["a".to_string(), "b".to_string()])
        .unwrap();

    session.set_tracking_breath_phase(1.25);
    assert!(
        (session
            .tracking
            .as_ref()
            .expect("tracking path")
            .breath_phase
            - 0.25)
            .abs()
            < 1e-6
    );

    session.set_tracking_breath_phase(-0.2);
    assert!(
        (session
            .tracking
            .as_ref()
            .expect("tracking path")
            .breath_phase
            - 0.8)
            .abs()
            < 1e-6
    );

    session.set_tracking_breath_phase(f64::NAN);
    assert_eq!(
        session
            .tracking
            .as_ref()
            .expect("tracking path")
            .breath_phase,
        0.0
    );
}

#[test]
fn network_tracking_breath_color_reaches_expected_extrema() {
    let tracking = NetworkTrackedPath {
        node_ids: vec!["a".to_string(), "b".to_string()],
        progress: 1.0,
        breath_phase: 0.0,
    };
    assert_eq!(tracking_edge_color(&tracking), TRACKING_EDGE_BREATH_COLOR,);
    assert_eq!(tracking_edge_opacity(&tracking, 1.0), TRACKING_EDGE_OPACITY);

    let peak_tracking = NetworkTrackedPath {
        breath_phase: 0.5,
        ..tracking.clone()
    };
    assert_eq!(tracking_edge_color(&peak_tracking), TRACKING_EDGE_COLOR);
    assert_eq!(
        tracking_edge_opacity(&peak_tracking, 1.0),
        TRACKING_EDGE_OPACITY
    );

    let looped_tracking = NetworkTrackedPath {
        breath_phase: 1.0,
        ..tracking
    };
    assert_eq!(
        tracking_edge_color(&looped_tracking),
        TRACKING_EDGE_BREATH_COLOR,
    );
    assert_eq!(
        tracking_edge_opacity(&looped_tracking, 1.0),
        TRACKING_EDGE_OPACITY
    );
}

#[test]
fn network_tracking_breathing_only_applies_after_reveal_completes() {
    let tracking = NetworkTrackedPath {
        node_ids: vec!["a".to_string(), "b".to_string()],
        progress: 0.5,
        breath_phase: 0.5,
    };

    assert_eq!(tracking_edge_color(&tracking), TRACKING_EDGE_COLOR);
    assert!((tracking_edge_opacity(&tracking, 0.5) - TRACKING_EDGE_OPACITY * 0.675).abs() < 1e-6);
}

#[test]
fn network_tracking_marks_traversed_nodes_through_current_node() {
    let mut tracking = NetworkTrackedPath {
        node_ids: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        progress: 0.0,
        breath_phase: 0.0,
    };

    assert!(tracking.is_traversed_node("a"));
    assert!(!tracking.is_traversed_node("b"));

    tracking.set_progress(0.75);
    assert!(tracking.is_traversed_node("a"));
    assert!(tracking.is_traversed_node("b"));
    assert!(!tracking.is_traversed_node("c"));

    tracking.set_progress(1.0);
    assert!(tracking.is_traversed_node("c"));
}

#[test]
fn network_tracking_survives_selection_updates_and_clears_on_invalid_topology() {
    let spec = positioned_spec();
    let mut session = NetworkSession::new(spec.clone()).unwrap();
    session
        .set_tracking_path(vec!["a".to_string(), "b".to_string()])
        .unwrap();
    session.set_tracking_progress(0.6);
    session.set_tracking_breath_phase(0.4);

    session
        .update_spec(NetworkPlotSpec {
            selected_node_id: Some("b".to_string()),
            ..spec.clone()
        })
        .unwrap();

    let tracking = session.tracking.as_ref().expect("tracking retained");
    assert_eq!(tracking.node_ids, vec!["a".to_string(), "b".to_string()]);
    assert!((tracking.progress - 0.6).abs() < 1e-6);
    assert!((tracking.breath_phase - 0.4).abs() < 1e-6);

    session
        .update_spec(NetworkPlotSpec {
            edges: vec![],
            ..spec
        })
        .unwrap();

    assert!(session.tracking.is_none());
}

#[test]
fn network_topology_transition_anchors_new_branch_to_parent() {
    let collapsed = NetworkPlotSpec {
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
                x: Some(120.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: BTreeMap::from([
                    (TOGGLEABLE_PROPERTY_KEY.to_string(), "true".to_string()),
                    (EXPANDED_PROPERTY_KEY.to_string(), "false".to_string()),
                ]),
            },
        ],
        edges: vec![NetworkEdge {
            source: "root".to_string(),
            target: "chapter".to_string(),
            label: None,
            color: None,
            weight: Some(1.0),
            style: None,
        }],
        ..sample_spec()
    };
    let expanded = NetworkPlotSpec {
        nodes: vec![
            collapsed.nodes[0].clone(),
            NetworkNode {
                properties: BTreeMap::from([
                    (TOGGLEABLE_PROPERTY_KEY.to_string(), "true".to_string()),
                    (EXPANDED_PROPERTY_KEY.to_string(), "true".to_string()),
                ]),
                ..collapsed.nodes[1].clone()
            },
            NetworkNode {
                id: "section".to_string(),
                label: "Section".to_string(),
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
            collapsed.edges[0].clone(),
            NetworkEdge {
                source: "chapter".to_string(),
                target: "section".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
        ],
        ..collapsed.clone()
    };

    let mut session = NetworkSession::new(collapsed).unwrap();
    session.update_spec(expanded).unwrap();

    assert!(session.has_transition());
    let transition = session.transition.as_ref().expect("transition present");
    assert_eq!(transition.anchor_node_id, "chapter");

    let midway = session.render_transition_nodes(0.5);
    let section = midway
        .iter()
        .find(|node| node.id == "section")
        .expect("new node rendered mid-transition");
    let chapter_from = transition.from_layout["chapter"];
    let section_to = session.layout["section"];
    let expected_x = chapter_from.0 + (section_to.0 - chapter_from.0) * 0.5;
    let expected_y = chapter_from.1 + (section_to.1 - chapter_from.1) * 0.5;

    assert!((f64::from(section.center_x) - expected_x).abs() < 2.0);
    assert!((f64::from(section.center_y) - expected_y).abs() < 2.0);
    assert!(section.opacity > 0.0 && section.opacity < 1.0);
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
fn network_session_spawned_node_labels_respect_active_zoom() {
    let initial = NetworkPlotSpec {
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
                label: "Alpha branch with a long label".to_string(),
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
                label: "Beta branch with a long label".to_string(),
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
        ],
        layout_iterations: 220,
        node_radius: 24,
        ..sample_spec()
    };
    let updated = NetworkPlotSpec {
        nodes: vec![
            initial.nodes[0].clone(),
            initial.nodes[1].clone(),
            initial.nodes[2].clone(),
            NetworkNode {
                id: "gamma".to_string(),
                label: "Gamma branch with a long label".to_string(),
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
                id: "delta".to_string(),
                label: "Delta branch with a long label".to_string(),
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
            initial.edges[0].clone(),
            initial.edges[1].clone(),
            NetworkEdge {
                source: "root".to_string(),
                target: "gamma".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "delta".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        ..initial.clone()
    };

    let mut session = NetworkSession::new(initial).unwrap();
    session
        .set_view(NetworkView {
            zoom: 1.8,
            translate_x: 0.0,
            translate_y: 0.0,
        })
        .unwrap();

    session.update_spec(updated.clone()).unwrap();

    assert_no_layout_collisions(&updated, &session.layout, 1.8);
}

#[test]
fn network_session_collapse_restores_parent_distance() {
    let collapsed = NetworkPlotSpec {
        width: 800,
        height: 600,
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
                id: "a".to_string(),
                label: "A".to_string(),
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
                id: "b".to_string(),
                label: "B".to_string(),
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
                target: "a".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "b".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        layout_iterations: 200,
        ..sample_spec()
    };
    let expanded = NetworkPlotSpec {
        nodes: vec![
            collapsed.nodes[0].clone(),
            collapsed.nodes[1].clone(),
            collapsed.nodes[2].clone(),
            NetworkNode {
                id: "c".to_string(),
                label: "C".to_string(),
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
                id: "d".to_string(),
                label: "D".to_string(),
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
                id: "e".to_string(),
                label: "E".to_string(),
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
            collapsed.edges[0].clone(),
            collapsed.edges[1].clone(),
            NetworkEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "b".to_string(),
                target: "d".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "b".to_string(),
                target: "e".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        ..collapsed.clone()
    };

    let mut session = NetworkSession::new(collapsed.clone()).unwrap();
    let collapsed_distance = {
        let root = session.layout["root"];
        let node = session.layout["b"];
        (node.0 - root.0).hypot(node.1 - root.1)
    };

    session.update_spec(expanded).unwrap();
    let expanded_distance = {
        let root = session.layout["root"];
        let node = session.layout["b"];
        (node.0 - root.0).hypot(node.1 - root.1)
    };

    session.update_spec(collapsed).unwrap();
    let restored_distance = {
        let root = session.layout["root"];
        let node = session.layout["b"];
        (node.0 - root.0).hypot(node.1 - root.1)
    };

    assert!((expanded_distance - collapsed_distance).abs() > 10.0);
    assert!((restored_distance - collapsed_distance).abs() < 10.0);
}

#[test]
fn network_focus_view_fits_node_and_neighbors() {
    let spec = NetworkPlotSpec {
        width: 400,
        height: 240,
        nodes: vec![
            NetworkNode {
                id: "a".to_string(),
                label: "A".to_string(),
                color: None,
                x: Some(-100.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "b".to_string(),
                label: "B".to_string(),
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
                id: "c".to_string(),
                label: "C".to_string(),
                color: None,
                x: Some(100.0),
                y: Some(0.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
            NetworkEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            },
        ],
        ..sample_spec()
    };
    let width = spec.width as f64;
    let height = spec.height as f64;
    let session = NetworkSession::new(spec).unwrap();

    let view = session
        .compute_focus_view("b", Some(NetworkFocusOptions::default()))
        .unwrap();

    assert!(view.zoom > 0.25);
    let (ax, ay) = session.layout["a"];
    let (bx, by) = session.layout["b"];
    let (cx, cy) = session.layout["c"];
    let a_screen = (
        ax * view.zoom + view.translate_x,
        ay * view.zoom + view.translate_y,
    );
    let b_screen = (
        bx * view.zoom + view.translate_x,
        by * view.zoom + view.translate_y,
    );
    let c_screen = (
        cx * view.zoom + view.translate_x,
        cy * view.zoom + view.translate_y,
    );

    assert!(a_screen.0 >= 0.0 && a_screen.0 <= width);
    assert!(c_screen.0 >= 0.0 && c_screen.0 <= width);
    assert!(a_screen.0 < b_screen.0 && b_screen.0 < c_screen.0);
    assert!((b_screen.0 - width / 2.0).abs() < 1.0);
    assert!((b_screen.1 - height / 2.0).abs() < 1.0);
}

#[test]
fn network_focus_view_uses_minimum_world_span_for_isolated_nodes() {
    let spec = positioned_spec();
    let session = NetworkSession::new(spec.clone()).unwrap();

    let view = session
        .compute_focus_view(
            "a",
            Some(NetworkFocusOptions {
                min_world_span: 200.0,
                ..Default::default()
            }),
        )
        .unwrap();

    assert!(view.zoom <= (spec.width as f64 - 96.0) / 200.0 + 0.001);
}

#[test]
fn network_rejects_invalid_style_values() {
    let mut spec = sample_spec();
    spec.default_node_style = Some(GraphNodeStyle {
        opacity: Some(1.5),
        ..Default::default()
    });

    let err = NetworkSession::new(spec).unwrap_err();

    assert_eq!(
        err,
        PlotError::InvalidStyleValue {
            field: "node_style.opacity",
            value: 1.5,
            reason: "must be between 0 and 1 inclusive",
        }
    );
}

#[test]
fn network_rejects_invalid_dash_pattern_values() {
    let mut spec = sample_spec();
    spec.default_edge_style = Some(GraphEdgeStyle {
        dash_pattern: Some(vec![0.0, 4.0]),
        ..Default::default()
    });

    let err = NetworkSession::new(spec).unwrap_err();

    assert_eq!(
        err,
        PlotError::InvalidStyleValue {
            field: "edge_style.dash_pattern",
            value: 0.0,
            reason: "must be greater than or equal to 1",
        }
    );
}
