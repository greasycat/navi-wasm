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
