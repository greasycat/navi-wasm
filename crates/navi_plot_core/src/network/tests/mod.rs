use super::*;
use crate::node::ResolvedNodeMediaKind;
use crate::types::{
    BuiltinNodeIcon, GraphEdgeStyle, GraphNodeStyle, NetworkEdge, NetworkNode, NetworkPlotSpec,
    NetworkView, NodeMedia, NodeMediaFit, NodeMediaKind, NodeShape, SelectionStyle,
};
use plotters::drawing::IntoDrawingArea;
use plotters_svg::SVGBackend;

mod interaction;
mod layout;
mod render;
mod session;
mod tracking;
mod validation;

fn sample_spec() -> NetworkPlotSpec {
    NetworkPlotSpec {
        width: 480,
        height: 360,
        title: "Test Network".to_string(),
        font_family: None,
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
        spring_length_scale: 1.0,
        temperature_scale: 1.0,
        cooling_rate: 0.92,
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
