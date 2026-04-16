use super::*;

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
