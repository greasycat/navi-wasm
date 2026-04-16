use super::*;

pub(super) fn normalized_collapsed_node_ids(
    spec: &TreePlotSpec,
    validated: &ValidatedTree,
) -> BTreeSet<String> {
    spec.collapsed_node_ids
        .iter()
        .filter(|node_id| {
            validated.node_ids.contains(node_id.as_str()) && validated.has_children(node_id)
        })
        .cloned()
        .collect()
}

pub(super) fn sync_collapsed_node_ids(
    spec: &mut TreePlotSpec,
    collapsed_node_ids: &BTreeSet<String>,
) {
    spec.collapsed_node_ids = collapsed_node_ids.iter().cloned().collect();
}

pub(super) fn build_visible_tree(
    root_id: &str,
    validated: &ValidatedTree,
    collapsed_node_ids: &BTreeSet<String>,
) -> VisibleTree {
    fn visit(
        node_id: &str,
        validated: &ValidatedTree,
        collapsed_node_ids: &BTreeSet<String>,
        visible: &mut VisibleTree,
    ) {
        let children = validated
            .children_by_parent
            .get(node_id)
            .cloned()
            .unwrap_or_default();

        if children.is_empty() {
            visible
                .children_by_parent
                .insert(node_id.to_string(), Vec::new());
            return;
        }

        if collapsed_node_ids.contains(node_id) {
            visible
                .children_by_parent
                .insert(node_id.to_string(), Vec::new());
            visible
                .collapsed_marker_node_ids
                .insert(node_id.to_string());
            return;
        }

        visible
            .children_by_parent
            .insert(node_id.to_string(), children.clone());
        for child in &children {
            visit(child, validated, collapsed_node_ids, visible);
        }
    }

    let mut visible = VisibleTree::default();
    visit(root_id, validated, collapsed_node_ids, &mut visible);
    visible
}

pub(super) fn sync_selected_node_id(
    selected_node_id: &mut Option<String>,
    layout: &BTreeMap<String, LayoutPoint>,
    validated: &ValidatedTree,
    preferred_selection: Option<&str>,
) {
    let Some(current) = selected_node_id.as_deref() else {
        return;
    };

    if layout.contains_key(current) {
        return;
    }

    if let Some(preferred) = preferred_selection.filter(|node_id| layout.contains_key(*node_id)) {
        *selected_node_id = Some(preferred.to_string());
        return;
    }

    *selected_node_id = nearest_visible_ancestor(current, layout, validated);
}

fn nearest_visible_ancestor(
    node_id: &str,
    layout: &BTreeMap<String, LayoutPoint>,
    validated: &ValidatedTree,
) -> Option<String> {
    let mut cursor = node_id;
    while let Some(parent) = validated.parent_by_child.get(cursor) {
        if layout.contains_key(parent.as_str()) {
            return Some(parent.clone());
        }
        cursor = parent;
    }
    None
}

pub(super) fn validate_tree(spec: &TreePlotSpec) -> Result<ValidatedTree, PlotError> {
    if spec.nodes.is_empty() {
        return Err(PlotError::EmptyTree);
    }

    let mut graph = StableDiGraph::<String, ()>::new();
    let mut indices_by_id = BTreeMap::<String, NodeIndex>::new();
    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();
    let mut parent_by_child = BTreeMap::<String, String>::new();
    let mut node_ids = BTreeSet::<String>::new();

    for node in &spec.nodes {
        if indices_by_id.contains_key(&node.id) {
            return Err(PlotError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }

        let index = graph.add_node(node.id.clone());
        indices_by_id.insert(node.id.clone(), index);
        node_ids.insert(node.id.clone());
        children_by_parent.entry(node.id.clone()).or_default();
    }

    let root_index = *indices_by_id
        .get(&spec.root_id)
        .ok_or_else(|| PlotError::MissingRoot {
            root_id: spec.root_id.clone(),
        })?;

    let mut seen_edges = BTreeSet::<(String, String)>::new();

    for edge in &spec.edges {
        let source_index =
            indices_by_id
                .get(&edge.source)
                .copied()
                .ok_or_else(|| PlotError::UnknownNode {
                    node_id: edge.source.clone(),
                })?;
        let target_index =
            indices_by_id
                .get(&edge.target)
                .copied()
                .ok_or_else(|| PlotError::UnknownNode {
                    node_id: edge.target.clone(),
                })?;

        if !seen_edges.insert((edge.source.clone(), edge.target.clone())) {
            return Err(PlotError::DuplicateEdge {
                from_node: edge.source.clone(),
                to_node: edge.target.clone(),
            });
        }

        graph.add_edge(source_index, target_index, ());
        children_by_parent
            .entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        parent_by_child.insert(edge.target.clone(), edge.source.clone());
    }

    if is_cyclic_directed(&graph) {
        return Err(PlotError::CycleDetected);
    }

    let roots = graph
        .node_indices()
        .filter(|index| {
            graph
                .neighbors_directed(*index, Direction::Incoming)
                .count()
                == 0
        })
        .collect::<Vec<_>>();

    if roots.len() != 1 {
        return Err(PlotError::InvalidRootCount { count: roots.len() });
    }

    let actual_root = graph[roots[0]].clone();
    if actual_root != spec.root_id {
        return Err(PlotError::RootMismatch {
            declared_root: spec.root_id.clone(),
            actual_root,
        });
    }

    for index in graph.node_indices() {
        let parent_count = graph.neighbors_directed(index, Direction::Incoming).count();
        if index == root_index {
            continue;
        }

        if parent_count != 1 {
            return Err(PlotError::InvalidParentCount {
                node_id: graph[index].clone(),
                parent_count,
            });
        }
    }

    let mut dfs = Dfs::new(&graph, root_index);
    let mut visited = BTreeSet::<String>::new();
    while let Some(index) = dfs.next(&graph) {
        visited.insert(graph[index].clone());
    }

    if visited.len() != graph.node_count() {
        let disconnected = spec
            .nodes
            .iter()
            .find(|node| !visited.contains(&node.id))
            .expect("visited count mismatch implies a disconnected node");

        return Err(PlotError::DisconnectedNode {
            node_id: disconnected.id.clone(),
            root_id: spec.root_id.clone(),
        });
    }

    Ok(ValidatedTree {
        node_ids,
        children_by_parent,
        parent_by_child,
    })
}

pub(super) fn build_layout(
    spec: &TreePlotSpec,
    children_by_parent: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, LayoutPoint> {
    #[derive(Debug, Clone, Copy)]
    struct RawPoint {
        x: f64,
        y: f64,
    }

    fn assign_positions(
        node_id: &str,
        depth: usize,
        children_by_parent: &BTreeMap<String, Vec<String>>,
        sibling_gap: u32,
        level_gap: u32,
        next_leaf_x: &mut f64,
        raw_positions: &mut BTreeMap<String, RawPoint>,
    ) -> f64 {
        let children = children_by_parent
            .get(node_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let x = if children.is_empty() {
            let x = *next_leaf_x;
            *next_leaf_x += sibling_gap.max(1) as f64;
            x
        } else {
            let child_centers = children
                .iter()
                .map(|child| {
                    assign_positions(
                        child,
                        depth + 1,
                        children_by_parent,
                        sibling_gap,
                        level_gap,
                        next_leaf_x,
                        raw_positions,
                    )
                })
                .collect::<Vec<_>>();

            child_centers.iter().sum::<f64>() / child_centers.len() as f64
        };

        raw_positions.insert(
            node_id.to_string(),
            RawPoint {
                x,
                y: depth as f64 * level_gap.max(1) as f64,
            },
        );

        x
    }

    let mut raw_positions = BTreeMap::<String, RawPoint>::new();
    let mut next_leaf_x = 0.0;
    assign_positions(
        &spec.root_id,
        0,
        children_by_parent,
        spec.sibling_gap,
        spec.level_gap,
        &mut next_leaf_x,
        &mut raw_positions,
    );

    let max_raw_x = raw_positions
        .values()
        .map(|point| point.x)
        .fold(0.0, f64::max);

    let left = spec.margin as f64;
    let top = spec.margin as f64 + if spec.title.is_empty() { 0.0 } else { 28.0 };
    let available_width = (spec.width.saturating_sub(2 * spec.margin)) as f64;
    let x_offset = left + (available_width - max_raw_x).max(0.0) / 2.0;

    raw_positions
        .into_iter()
        .map(|(node_id, point)| {
            (
                node_id,
                LayoutPoint {
                    x: (x_offset + point.x).round() as i32,
                    y: (top + point.y).round() as i32,
                },
            )
        })
        .collect()
}

pub(super) fn resolve_tree_nodes(
    spec: &TreePlotSpec,
) -> Result<BTreeMap<String, ResolvedTreeNode>, PlotError> {
    spec.nodes
        .iter()
        .map(|node| {
            let style = resolve_node_style(NodeStyleContext {
                default_fill_color: DEFAULT_NODE_COLOR,
                default_radius: spec.node_radius,
                default_label_visible: true,
                graph_style: spec.default_node_style.as_ref(),
                legacy_fill_color: node.color.as_deref(),
                legacy_shape: node.shape.as_ref(),
                legacy_label_inside: node.label_inside,
                item_style: node.style.as_ref(),
            })?;
            let media = node::resolve_node_media(node.media.as_ref())?;
            Ok((node.id.clone(), ResolvedTreeNode { style, media }))
        })
        .collect()
}

pub(super) fn resolve_tree_edge_style(
    spec: &TreePlotSpec,
    edge: &crate::TreeEdge,
) -> Result<crate::graph_style::ResolvedEdgeStyle, PlotError> {
    resolve_edge_style(EdgeStyleContext {
        default_stroke_color: DEFAULT_EDGE_COLOR,
        default_stroke_width: 2,
        default_arrow_visible: false,
        default_label_visible: false,
        graph_style: spec.default_edge_style.as_ref(),
        legacy_stroke_color: None,
        item_style: edge.style.as_ref(),
    })
}
