use super::*;

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
                force_layers: None,
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
fn network_session_exposes_layout_snapshot() {
    let session = NetworkSession::new(positioned_spec()).unwrap();
    let snapshot = session.layout_snapshot();

    assert_eq!(snapshot.len(), session.layout.len());
    assert_eq!(snapshot["a"].x, session.layout["a"].0);
    assert_eq!(snapshot["a"].y, session.layout["a"].1);
}

#[test]
fn network_session_rotate_about_canvas_anchor_updates_layout_and_picking() {
    let mut session = NetworkSession::new(positioned_spec()).unwrap();
    session
        .set_view(NetworkView {
            zoom: 1.2,
            translate_x: 50.0,
            translate_y: -20.0,
        })
        .unwrap();

    let anchor = session.layout["a"];
    let anchor_canvas_x = anchor.0 * 1.2 + 50.0;
    let anchor_canvas_y = anchor.1 * 1.2 - 20.0;
    session
        .rotate_about(
            anchor_canvas_x,
            anchor_canvas_y,
            std::f64::consts::FRAC_PI_2,
        )
        .unwrap();

    assert!((session.layout["a"].0 - anchor.0).abs() < 0.001);
    assert!((session.layout["a"].1 - anchor.1).abs() < 0.001);
    assert!((session.layout["b"].0 - 0.0).abs() < 0.001);
    assert!((session.layout["b"].1 - 300.0).abs() < 0.001);

    let b_canvas_x = session.layout["b"].0 * 1.2 + 50.0;
    let b_canvas_y = session.layout["b"].1 * 1.2 - 20.0;
    assert_eq!(
        session.pick_node(b_canvas_x, b_canvas_y),
        Some("b".to_string())
    );
}

#[test]
fn network_session_update_can_restore_cached_target_layout() {
    let spec = positioned_spec();
    let mut session = NetworkSession::new(spec.clone()).unwrap();
    let target = NetworkPlotSpec {
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
                force_layers: None,
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
    let cached = BTreeMap::from([
        ("a".to_string(), NetworkLayoutPoint { x: 10.0, y: 20.0 }),
        ("b".to_string(), NetworkLayoutPoint { x: 30.0, y: 40.0 }),
        ("c".to_string(), NetworkLayoutPoint { x: 50.0, y: 60.0 }),
    ]);

    session
        .update_spec_with_layout(target, Some(cached.clone()))
        .unwrap();

    assert!(session.has_transition());
    assert_eq!(session.layout["a"], (cached["a"].x, cached["a"].y));
    assert_eq!(session.layout["b"], (cached["b"].x, cached["b"].y));
    assert_eq!(session.layout["c"], (cached["c"].x, cached["c"].y));
}

#[test]
fn network_session_ignores_cached_layout_with_changed_nodes() {
    let spec = positioned_spec();
    let mut session = NetworkSession::new(spec.clone()).unwrap();
    let target = NetworkPlotSpec {
        nodes: vec![spec.nodes[0].clone(), spec.nodes[1].clone()],
        edges: vec![spec.edges[0].clone()],
        ..spec
    };
    let stale_cached = BTreeMap::from([
        ("a".to_string(), NetworkLayoutPoint { x: 10.0, y: 20.0 }),
        ("extra".to_string(), NetworkLayoutPoint { x: 30.0, y: 40.0 }),
    ]);

    session
        .update_spec_with_layout(target, Some(stale_cached))
        .unwrap();

    assert!(session.layout.contains_key("a"));
    assert!(session.layout.contains_key("b"));
    assert_eq!(session.layout.len(), 2);
    assert_ne!(session.layout["a"], (10.0, 20.0));
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
fn network_topology_transition_anchors_new_branch_to_parent() {
    let collapsed = NetworkPlotSpec {
        nodes: vec![
            NetworkNode {
                id: "root".to_string(),
                label: "Root".to_string(),
                color: None,
                x: Some(240.0),
                y: Some(180.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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

    let current_parent_by_id = structural_parent_map(session.spec());
    let previous_parent_by_id = structural_parent_map(&transition.from_spec);
    let current_ids = session
        .spec()
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let previous_ids = transition
        .from_spec
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let section_anchor_frame = transition_node_anchor_frame(
        "section",
        transition,
        &session.layout,
        &current_parent_by_id,
        &previous_parent_by_id,
        &current_ids,
        &previous_ids,
        0.5,
    );
    let section = transition_node_frame(
        "section",
        transition,
        &session.layout,
        section_anchor_frame,
        0.5,
    )
    .expect("new node frame");
    let chapter_from = transition.from_layout["chapter"];
    let section_to = session.layout["section"];
    let expected_x = chapter_from.0 + (section_to.0 - chapter_from.0) * 0.5;
    let expected_y = chapter_from.1 + (section_to.1 - chapter_from.1) * 0.5;

    assert!((section.point.0 - expected_x).abs() < 2.0);
    assert!((section.point.1 - expected_y).abs() < 2.0);
    assert!(section.opacity > 0.0 && section.opacity < 1.0);
}

#[test]
fn network_topology_transition_anchors_each_new_branch_to_its_local_parent() {
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
                force_layers: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "left".to_string(),
                label: "Left".to_string(),
                color: None,
                x: Some(120.0),
                y: Some(180.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "right".to_string(),
                label: "Right".to_string(),
                color: None,
                x: Some(360.0),
                y: Some(180.0),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "left".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "root".to_string(),
                target: "right".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
        ],
        ..sample_spec()
    };
    let expanded = NetworkPlotSpec {
        nodes: vec![
            collapsed.nodes[0].clone(),
            collapsed.nodes[1].clone(),
            collapsed.nodes[2].clone(),
            NetworkNode {
                id: "left-leaf".to_string(),
                label: "Left leaf".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "right-leaf".to_string(),
                label: "Right leaf".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            collapsed.edges[0].clone(),
            collapsed.edges[1].clone(),
            NetworkEdge {
                source: "left".to_string(),
                target: "left-leaf".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "right".to_string(),
                target: "right-leaf".to_string(),
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

    let transition = session.transition.as_ref().expect("transition present");
    assert!(transition.anchor_node_id == "left" || transition.anchor_node_id == "right");

    let current_parent_by_id = structural_parent_map(session.spec());
    let previous_parent_by_id = structural_parent_map(&transition.from_spec);
    let current_ids = session
        .spec()
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let previous_ids = transition
        .from_spec
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let left_anchor_frame = transition_node_anchor_frame(
        "left-leaf",
        transition,
        &session.layout,
        &current_parent_by_id,
        &previous_parent_by_id,
        &current_ids,
        &previous_ids,
        0.5,
    );
    let left_leaf = transition_node_frame(
        "left-leaf",
        transition,
        &session.layout,
        left_anchor_frame,
        0.5,
    )
    .expect("left leaf frame");
    let right_anchor_frame = transition_node_anchor_frame(
        "right-leaf",
        transition,
        &session.layout,
        &current_parent_by_id,
        &previous_parent_by_id,
        &current_ids,
        &previous_ids,
        0.5,
    );
    let right_leaf = transition_node_frame(
        "right-leaf",
        transition,
        &session.layout,
        right_anchor_frame,
        0.5,
    )
    .expect("right leaf frame");

    let left_from = transition.from_layout["left"];
    let right_from = transition.from_layout["right"];
    let left_to = session.layout["left-leaf"];
    let right_to = session.layout["right-leaf"];

    let expected_left = (
        left_from.0 + (left_to.0 - left_from.0) * 0.5,
        left_from.1 + (left_to.1 - left_from.1) * 0.5,
    );
    let expected_right = (
        right_from.0 + (right_to.0 - right_from.0) * 0.5,
        right_from.1 + (right_to.1 - right_from.1) * 0.5,
    );

    assert!((left_leaf.point.0 - expected_left.0).abs() < 2.0);
    assert!((left_leaf.point.1 - expected_left.1).abs() < 2.0);
    assert!((right_leaf.point.0 - expected_right.0).abs() < 2.0);
    assert!((right_leaf.point.1 - expected_right.1).abs() < 2.0);
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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
                force_layers: None,
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

    assert!((expanded_distance - collapsed_distance).abs() < WORLD_NODE_SPACING * 0.1);
    assert!((restored_distance - collapsed_distance).abs() < 10.0);
}

fn radial_motion_spec(amplitude: f64, speed: f64) -> NetworkPlotSpec {
    radial_motion_spec_with_mode(NetworkMotionMode::Orbital, amplitude, speed)
}

fn radial_motion_spec_with_mode(
    mode: NetworkMotionMode,
    amplitude: f64,
    speed: f64,
) -> NetworkPlotSpec {
    NetworkPlotSpec {
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
                force_layers: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "child".to_string(),
                label: "Child".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
                properties: Default::default(),
            },
            NetworkNode {
                id: "grandchild".to_string(),
                label: "Grandchild".to_string(),
                color: None,
                x: None,
                y: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                force_layers: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            NetworkEdge {
                source: "root".to_string(),
                target: "child".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "child".to_string(),
                target: "grandchild".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
        ],
        motion: Some(NetworkMotionSpec {
            enabled: true,
            mode,
            amplitude,
            speed,
            seed: 7,
        }),
        ..sample_spec()
    }
}

fn render_info<'a>(nodes: &'a [GraphNodeRenderInfo], id: &str) -> &'a GraphNodeRenderInfo {
    nodes
        .iter()
        .find(|node| node.id == id)
        .expect("render info exists")
}

#[test]
fn network_motion_modes_animate_radial_nodes_without_mutating_layout() {
    for mode in [
        NetworkMotionMode::Orbital,
        NetworkMotionMode::Drift,
        NetworkMotionMode::Breathe,
    ] {
        let session =
            NetworkSession::new(radial_motion_spec_with_mode(mode.clone(), 24.0, 0.5)).unwrap();
        let base_layout = session.layout.clone();
        let first = session.animated_layout(0.0).unwrap();
        let second = session.animated_layout(0.75).unwrap();
        let third = session.animated_layout(1.5).unwrap();

        assert_eq!(session.layout, base_layout);
        assert_eq!(base_layout["root"], first["root"]);
        if matches!(mode, NetworkMotionMode::Breathe) {
            let root = base_layout["root"];
            let base_child_radius =
                (base_layout["child"].0 - root.0).hypot(base_layout["child"].1 - root.1);
            let first_child_radius = (first["child"].0 - root.0).hypot(first["child"].1 - root.1);
            let second_child_radius =
                (second["child"].0 - root.0).hypot(second["child"].1 - root.1);
            let first_angle = (first["child"].1 - root.1).atan2(first["child"].0 - root.0);
            let second_angle = (second["child"].1 - root.1).atan2(second["child"].0 - root.0);
            let third_angle = (third["child"].1 - root.1).atan2(third["child"].0 - root.0);
            let first_delta = second_angle - first_angle;
            let second_delta = third_angle - second_angle;
            assert_ne!(first["child"], second["child"]);
            assert!((first_child_radius - base_child_radius).abs() < 0.001);
            assert!((second_child_radius - base_child_radius).abs() < 0.001);
            assert!(first_delta * second_delta > 0.0);
            assert_ne!(first["grandchild"], second["grandchild"]);
        } else {
            assert_ne!(first["child"], second["child"]);
        }
    }
}

#[test]
fn network_motion_is_ignored_for_non_radial_layouts() {
    let spec = NetworkPlotSpec {
        edges: vec![
            NetworkEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "b".to_string(),
                target: "c".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
            NetworkEdge {
                source: "c".to_string(),
                target: "a".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            },
        ],
        motion: Some(NetworkMotionSpec {
            enabled: true,
            mode: NetworkMotionMode::Orbital,
            amplitude: 24.0,
            speed: 0.5,
            seed: 3,
        }),
        ..sample_spec()
    };
    let session = NetworkSession::new(spec).unwrap();

    assert!(session.radial_hierarchy.is_none());
    assert_eq!(session.render_nodes(), session.render_motion_nodes(1.25));
}

#[test]
fn network_motion_pick_uses_animated_position() {
    let session = NetworkSession::new(radial_motion_spec(80.0, 0.4)).unwrap();
    let time_seconds = 0.25;
    let animated_nodes = session.render_motion_nodes(time_seconds);
    let child = render_info(&animated_nodes, "child");

    let hit = session.pick_node_motion(
        f64::from(child.center_x),
        f64::from(child.center_y),
        time_seconds,
    );

    assert_eq!(hit.as_deref(), Some("child"));
}
