use super::*;

#[test]
fn network_pan_updates_offsets() {
    let spec = sample_spec();
    let updated = pan_network_spec(&spec, 30.0, -15.0).unwrap();
    assert_eq!(updated.offset_x, 30);
    assert_eq!(updated.offset_y, -15);
}

#[test]
fn network_hit_test_returns_closest_node() {
    let spec = positioned_spec();
    let session = NetworkSession::new(spec).unwrap();
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
