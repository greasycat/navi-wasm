use super::*;

pub(in crate::network) fn validate(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
    ensure_dimensions(spec.width, spec.height)?;
    if spec.nodes.is_empty() {
        return Err(PlotError::EmptyNetwork);
    }

    let mut seen_ids: HashSet<&str> = HashSet::new();
    for node in &spec.nodes {
        if !seen_ids.insert(node.id.as_str()) {
            return Err(PlotError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }
    }

    let mut seen_edges: HashSet<(&str, &str)> = HashSet::new();
    for edge in &spec.edges {
        if !seen_ids.contains(edge.source.as_str()) {
            return Err(PlotError::UnknownNode {
                node_id: edge.source.clone(),
            });
        }
        if !seen_ids.contains(edge.target.as_str()) {
            return Err(PlotError::UnknownNode {
                node_id: edge.target.clone(),
            });
        }
        let key = (edge.source.as_str(), edge.target.as_str());
        if !seen_edges.insert(key) {
            return Err(PlotError::DuplicateEdge {
                from_node: edge.source.clone(),
                to_node: edge.target.clone(),
            });
        }
    }

    resolve_nodes(spec)?;
    let _ = resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
    for edge in &spec.edges {
        let _ = resolve_network_edge_style(spec, edge)?;
    }
    Ok(())
}

pub(in crate::network) fn resolve_nodes(
    spec: &NetworkPlotSpec,
) -> Result<BTreeMap<String, ResolvedNode>, PlotError> {
    spec.nodes
        .iter()
        .map(|node| {
            let style = resolve_node_style(NodeStyleContext {
                default_fill_color: DEFAULT_NODE_COLOR,
                default_radius: spec.node_radius,
                default_label_visible: spec.show_labels,
                graph_style: spec.default_node_style.as_ref(),
                legacy_fill_color: node.color.as_deref(),
                legacy_shape: node.shape.as_ref(),
                legacy_label_inside: node.label_inside,
                item_style: node.style.as_ref(),
            })?;
            Ok((
                node.id.clone(),
                ResolvedNode {
                    label: if node.label.is_empty() {
                        node.id.clone()
                    } else {
                        node.label.clone()
                    },
                    style,
                    media: node::resolve_node_media(node.media.as_ref())?,
                },
            ))
        })
        .collect()
}

pub(in crate::network) fn validate_explicit_positions(
    spec: &NetworkPlotSpec,
) -> Result<(), PlotError> {
    for node in &spec.nodes {
        if let Some(x) = node.x {
            ensure_finite("x", x)?;
        }
        if let Some(y) = node.y {
            ensure_finite("y", y)?;
        }
    }
    Ok(())
}
