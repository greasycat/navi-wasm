use super::*;

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
    assert_eq!(tracking_edge_color(&tracking), TRACKING_EDGE_BREATH_COLOR);
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
        TRACKING_EDGE_BREATH_COLOR
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
