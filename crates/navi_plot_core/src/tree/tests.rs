use super::*;
use crate::node::ResolvedNodeMediaKind;
use crate::types::{
    BuiltinNodeIcon, GraphEdgeStyle, GraphNodeStyle, NodeMedia, NodeMediaFit, NodeMediaKind,
    NodeShape, SelectionStyle,
};
use plotters::drawing::IntoDrawingArea;
use plotters_svg::SVGBackend;

fn sample_tree_spec() -> TreePlotSpec {
    TreePlotSpec {
        width: 640,
        height: 420,
        title: "Tree".to_string(),
        font_family: None,
        root_id: "root".to_string(),
        nodes: vec![
            crate::TreeNode {
                id: "root".to_string(),
                name: Some("Root Hub".to_string()),
                label: "Root".to_string(),
                color: Some("#0f766e".to_string()),
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            crate::TreeNode {
                id: "left".to_string(),
                name: Some("Left Branch".to_string()),
                label: "Left".to_string(),
                color: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            crate::TreeNode {
                id: "right".to_string(),
                name: Some("Right Branch".to_string()),
                label: "Right".to_string(),
                color: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
            crate::TreeNode {
                id: "leaf".to_string(),
                name: Some("Leaf Node".to_string()),
                label: "Leaf".to_string(),
                color: None,
                shape: None,
                label_inside: None,
                style: None,
                media: None,
                properties: Default::default(),
            },
        ],
        edges: vec![
            crate::TreeEdge {
                source: "root".to_string(),
                target: "left".to_string(),
                style: None,
            },
            crate::TreeEdge {
                source: "root".to_string(),
                target: "right".to_string(),
                style: None,
            },
            crate::TreeEdge {
                source: "right".to_string(),
                target: "leaf".to_string(),
                style: None,
            },
        ],
        node_radius: 18,
        default_node_style: None,
        default_edge_style: None,
        selection_style: None,
        level_gap: 90,
        sibling_gap: 96,
        margin: 32,
        offset_x: 0,
        offset_y: 0,
        selected_node_id: None,
        collapsed_node_ids: Vec::new(),
        pixel_ratio: 1.0,
    }
}

fn offset_point(point: LayoutPoint, spec: &TreePlotSpec) -> LayoutPoint {
    LayoutPoint {
        x: point.x.saturating_add(spec.offset_x),
        y: point.y.saturating_add(spec.offset_y),
    }
}

#[test]
fn tree_validation_rejects_duplicate_ids() {
    let mut spec = sample_tree_spec();
    spec.nodes.push(crate::TreeNode {
        id: "left".to_string(),
        name: Some("Duplicate".to_string()),
        label: "Dupe".to_string(),
        color: None,
        shape: None,
        label_inside: None,
        style: None,
        media: None,
        properties: Default::default(),
    });

    let error = validate_tree(&spec).unwrap_err();
    assert_eq!(
        error,
        PlotError::DuplicateNodeId {
            node_id: "left".to_string(),
        }
    );
}

#[test]
fn tree_validation_rejects_cycles() {
    let mut spec = sample_tree_spec();
    spec.edges.push(crate::TreeEdge {
        source: "leaf".to_string(),
        target: "root".to_string(),
        style: None,
    });

    let error = validate_tree(&spec).unwrap_err();
    assert_eq!(error, PlotError::CycleDetected);
}

#[test]
fn tree_validation_rejects_multiple_parents() {
    let mut spec = sample_tree_spec();
    spec.edges.push(crate::TreeEdge {
        source: "left".to_string(),
        target: "leaf".to_string(),
        style: None,
    });

    let error = validate_tree(&spec).unwrap_err();
    assert_eq!(
        error,
        PlotError::InvalidParentCount {
            node_id: "leaf".to_string(),
            parent_count: 2,
        }
    );
}

#[test]
fn tree_validation_rejects_disconnected_nodes() {
    let mut spec = sample_tree_spec();
    spec.nodes.push(crate::TreeNode {
        id: "orphan".to_string(),
        name: Some("Orphan".to_string()),
        label: "Orphan".to_string(),
        color: None,
        shape: None,
        label_inside: None,
        style: None,
        media: None,
        properties: Default::default(),
    });

    let error = validate_tree(&spec).unwrap_err();
    assert_eq!(error, PlotError::InvalidRootCount { count: 2 });
}

#[test]
fn tree_layout_keeps_depth_monotonic_and_centers_parents() {
    let spec = sample_tree_spec();
    let validated = validate_tree(&spec).unwrap();
    let layout = build_layout(&spec, &validated.children_by_parent);

    let root = layout["root"];
    let left = layout["left"];
    let right = layout["right"];
    let leaf = layout["leaf"];

    assert!(root.y < left.y);
    assert!(root.y < right.y);
    assert!(right.y < leaf.y);
    assert!(left.x < right.x);
    assert!((root.x - ((left.x + right.x) / 2)).abs() <= 1);
}

#[test]
fn tree_svg_output_contains_nodes_and_labels() {
    let mut svg = String::new();
    let spec = sample_tree_spec();
    let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();

    render_tree_on(area, &spec).unwrap();

    assert_eq!(svg.matches("<circle").count(), spec.nodes.len());
    assert!(svg.contains("Root"));
    assert!(svg.contains("Leaf"));
}

#[test]
fn tree_hit_test_respects_offsets() {
    let mut spec = sample_tree_spec();
    spec.offset_x = 32;
    spec.offset_y = -14;
    let validated = validate_tree(&spec).unwrap();
    let layout = build_layout(&spec, &validated.children_by_parent);
    let target = offset_point(layout["right"], &spec);

    let selected = pick_tree_node(&spec, f64::from(target.x), f64::from(target.y)).unwrap();

    assert_eq!(selected.as_deref(), Some("right"));
}

#[test]
fn tree_render_nodes_exposes_media_metadata() {
    let mut spec = sample_tree_spec();
    spec.nodes[0].media = Some(NodeMedia {
        kind: NodeMediaKind::Icon,
        icon: Some(BuiltinNodeIcon::Galaxy),
        image_key: None,
        fit: NodeMediaFit::Contain,
        scale: Some(0.8),
        tint_color: Some("#ffffff".to_string()),
        fallback_icon: None,
    });

    let nodes = tree_render_nodes(&spec).unwrap();
    let root = nodes.iter().find(|node| node.id == "root").unwrap();

    assert!(matches!(
        root.media.as_ref().map(|media| &media.kind),
        Some(ResolvedNodeMediaKind::Icon(BuiltinNodeIcon::Galaxy))
    ));
    assert_eq!(root.shape, NodeShape::Circle);
}

#[test]
fn tree_render_nodes_cull_offscreen_nodes() {
    let mut spec = sample_tree_spec();
    spec.offset_x = -400;

    let nodes = tree_render_nodes(&spec).unwrap();

    assert!(nodes.len() < spec.nodes.len());
    assert!(!nodes.iter().any(|node| node.id == "root"));
}

#[test]
fn tree_pan_updates_offsets() {
    let spec = sample_tree_spec();
    let panned = pan_tree_spec(&spec, 18.0, -9.0).unwrap();

    assert_eq!(panned.offset_x, 18);
    assert_eq!(panned.offset_y, -9);
}

#[test]
fn tree_node_style_inheritance_and_overrides_resolve_in_order() {
    let mut spec = sample_tree_spec();
    spec.default_node_style = Some(GraphNodeStyle {
        shape: Some(NodeShape::Square),
        radius: Some(24.0),
        label_visible: Some(false),
        ..Default::default()
    });
    spec.nodes[0].shape = Some(NodeShape::Diamond);
    spec.nodes[1].style = Some(GraphNodeStyle {
        shape: Some(NodeShape::Triangle),
        label_visible: Some(true),
        ..Default::default()
    });

    let resolved = resolve_tree_nodes(&spec).unwrap();

    assert_eq!(resolved["root"].style.shape, NodeShape::Diamond);
    assert_eq!(resolved["left"].style.shape, NodeShape::Triangle);
    assert_eq!(resolved["right"].style.shape, NodeShape::Square);
    assert!(!resolved["root"].style.label_visible);
    assert!(resolved["left"].style.label_visible);
    assert_eq!(resolved["right"].style.radius, 24);
}

#[test]
fn tree_hit_test_uses_per_node_radius_override() {
    let mut spec = sample_tree_spec();
    spec.node_radius = 8;
    spec.selection_style = Some(SelectionStyle {
        padding: Some(0.0),
        ..Default::default()
    });
    spec.nodes[1].style = Some(GraphNodeStyle {
        radius: Some(28.0),
        ..Default::default()
    });

    let validated = validate_tree(&spec).unwrap();
    let layout = build_layout(&spec, &validated.children_by_parent);
    let target = layout["left"];
    let selected = pick_tree_node(&spec, f64::from(target.x + 20), f64::from(target.y)).unwrap();

    assert_eq!(selected.as_deref(), Some("left"));
}

#[test]
fn tree_edge_styles_render_graph_defaults_and_per_edge_overrides() {
    let mut svg = String::new();
    let mut spec = sample_tree_spec();
    spec.default_edge_style = Some(GraphEdgeStyle {
        stroke_width: Some(4.0),
        ..Default::default()
    });
    spec.edges[0].style = Some(GraphEdgeStyle {
        stroke_width: Some(6.0),
        ..Default::default()
    });
    let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();

    render_tree_on(area, &spec).unwrap();

    assert!(svg.contains("stroke-width=\"6\""));
    assert!(svg.contains("stroke-width=\"4\""));
}

#[test]
fn tree_collapsing_node_hides_descendants_and_reselects_parent() {
    let mut session = TreeSession::new(sample_tree_spec()).unwrap();
    session.set_selection(Some("leaf".to_string()));

    let collapsed = session.toggle_collapse("right");

    assert!(collapsed);
    assert_eq!(session.spec().selected_node_id.as_deref(), Some("right"));

    let rendered = session.render_nodes();
    assert!(rendered.iter().any(|node| node.id == "right"));
    assert!(!rendered.iter().any(|node| node.id == "leaf"));
}

#[test]
fn tree_collapsed_layout_compacts_hidden_branch() {
    let mut spec = sample_tree_spec();
    spec.nodes.push(crate::TreeNode {
        id: "leaf-two".to_string(),
        name: Some("Second Leaf".to_string()),
        label: "Leaf 2".to_string(),
        color: None,
        shape: None,
        label_inside: None,
        style: None,
        media: None,
        properties: Default::default(),
    });
    spec.edges.push(crate::TreeEdge {
        source: "right".to_string(),
        target: "leaf-two".to_string(),
        style: None,
    });

    let expanded = TreeSession::new(spec.clone()).unwrap();

    let mut collapsed_spec = spec;
    collapsed_spec.collapsed_node_ids = vec!["right".to_string()];
    let collapsed = TreeSession::new(collapsed_spec).unwrap();

    assert!(collapsed.layout["right"].y > collapsed.layout["root"].y);
    assert!(collapsed.layout["left"].x > expanded.layout["left"].x);
    assert!(!collapsed.layout.contains_key("leaf"));
    assert!(!collapsed.layout.contains_key("leaf-two"));
}

#[test]
fn tree_ignores_unknown_or_leaf_collapsed_ids() {
    let mut spec = sample_tree_spec();
    spec.collapsed_node_ids = vec![
        "missing".to_string(),
        "leaf".to_string(),
        "right".to_string(),
        "right".to_string(),
    ];

    let session = TreeSession::new(spec).unwrap();

    assert_eq!(session.spec().collapsed_node_ids, vec!["right".to_string()]);
}

#[test]
fn tree_toggle_collapse_is_noop_for_leaf() {
    let mut session = TreeSession::new(sample_tree_spec()).unwrap();

    let collapsed = session.toggle_collapse("leaf");

    assert!(!collapsed);
    assert!(session.layout.contains_key("leaf"));
    assert!(session.spec().collapsed_node_ids.is_empty());
}

#[test]
fn tree_transition_render_supports_intermediate_frames() {
    let mut session = TreeSession::new(sample_tree_spec()).unwrap();
    session.toggle_collapse("right");

    let mut svg = String::new();
    let area =
        SVGBackend::with_string(&mut svg, (session.width(), session.height())).into_drawing_area();

    session.render_transition_on(area, 0.5).unwrap();
    let nodes = session.render_transition_nodes(0.5);

    assert!(svg.contains("Root"));
    assert!(nodes.iter().any(|node| node.id == "right"));
    assert!(nodes.iter().any(|node| node.id == "leaf"));
}
