use super::*;

#[test]
fn network_svg_has_correct_circle_count() {
    let mut svg = String::new();
    let spec = positioned_spec();
    let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
    render_network_on(area, &spec).unwrap();
    assert_eq!(svg.matches("<circle").count(), spec.nodes.len());
}

#[test]
fn network_non_circle_shapes_render_without_error() {
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
