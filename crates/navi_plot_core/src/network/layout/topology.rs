use super::*;

pub(in crate::network) fn node_is_pinned(node: &crate::NetworkNode) -> bool {
    node.x.is_some() && node.y.is_some()
}

pub(in crate::network) fn deterministic_offset(node_id: &str, radius: f64) -> (f64, f64) {
    let hash = node_id.bytes().fold(0u64, |acc, byte| {
        acc.wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(byte) + 1)
    });
    let angle = ((hash % 3600) as f64 / 3600.0) * TAU;
    let scaled_radius = radius * (0.7 + (((hash / 3600) % 5) as f64) * 0.12);
    (scaled_radius * angle.cos(), scaled_radius * angle.sin())
}

pub(in crate::network) fn edge_weight(edge: &crate::NetworkEdge) -> f64 {
    edge.weight.unwrap_or(1.0).max(MIN_LAYOUT_EDGE_WEIGHT)
}

pub(in crate::network) fn edge_is_structural(edge: &crate::NetworkEdge) -> bool {
    edge.weight.unwrap_or(1.0) >= STRUCTURAL_EDGE_WEIGHT_THRESHOLD
}

pub(in crate::network) fn neighbor_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    let mut neighbors = Vec::new();
    for edge in &spec.edges {
        if edge.source == node_id {
            neighbors.push(edge.target.clone());
        } else if edge.target == node_id {
            neighbors.push(edge.source.clone());
        }
    }
    neighbors
}

pub(in crate::network) fn adjacency_map(
    spec: &NetworkPlotSpec,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::new();
    for node in &spec.nodes {
        adjacency
            .entry(node.id.clone())
            .or_insert_with(BTreeSet::new);
    }
    for edge in &spec.edges {
        adjacency
            .entry(edge.source.clone())
            .or_insert_with(BTreeSet::new)
            .insert(edge.target.clone());
        adjacency
            .entry(edge.target.clone())
            .or_insert_with(BTreeSet::new)
            .insert(edge.source.clone());
    }
    adjacency
}

fn topology_identity(spec: &NetworkPlotSpec) -> (BTreeSet<String>, BTreeSet<(String, String)>) {
    let node_ids = spec
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    let edge_ids = spec
        .edges
        .iter()
        .map(|edge| (edge.source.clone(), edge.target.clone()))
        .collect::<BTreeSet<_>>();
    (node_ids, edge_ids)
}

pub(in crate::network) fn topology_changed(
    previous_spec: &NetworkPlotSpec,
    next_spec: &NetworkPlotSpec,
) -> bool {
    topology_identity(previous_spec) != topology_identity(next_spec)
}

pub(in crate::network) fn path_edge_exists(
    spec: &NetworkPlotSpec,
    from_node: &str,
    to_node: &str,
) -> bool {
    spec.edges.iter().any(|edge| {
        (edge.source == from_node && edge.target == to_node)
            || (edge.source == to_node && edge.target == from_node)
    })
}

pub(in crate::network) fn node_property_is_true(node: &crate::NetworkNode, key: &str) -> bool {
    node.properties.get(key).is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes"
        )
    })
}

pub(in crate::network) fn node_has_toggle_badge(node: &crate::NetworkNode) -> bool {
    node_property_is_true(node, TOGGLEABLE_PROPERTY_KEY)
}

pub(in crate::network) fn node_badge_expanded(node: &crate::NetworkNode) -> bool {
    node_property_is_true(node, EXPANDED_PROPERTY_KEY)
}

pub(in crate::network) fn structural_parent_map(spec: &NetworkPlotSpec) -> HashMap<&str, &str> {
    let mut parents = HashMap::new();
    for edge in &spec.edges {
        if edge_is_structural(edge) {
            parents
                .entry(edge.target.as_str())
                .or_insert(edge.source.as_str());
        }
    }
    parents
}

pub(in crate::network) fn nearest_shared_ancestor(
    parent_by_id: &HashMap<&str, &str>,
    shared_ids: &HashSet<&str>,
    node_id: &str,
) -> Option<String> {
    let mut current = parent_by_id.get(node_id).copied();
    while let Some(parent_id) = current {
        if shared_ids.contains(parent_id) {
            return Some(parent_id.to_string());
        }
        current = parent_by_id.get(parent_id).copied();
    }
    None
}

pub(in crate::network) fn structural_depths(spec: &NetworkPlotSpec) -> HashMap<String, usize> {
    let parent_by_id = structural_parent_map(spec);
    spec.nodes
        .iter()
        .map(|node| {
            let mut depth = 0usize;
            let mut current = node.id.as_str();
            let mut seen = HashSet::new();
            while let Some(parent_id) = parent_by_id.get(current).copied() {
                if !seen.insert(current) {
                    break;
                }
                depth += 1;
                current = parent_id;
            }
            (node.id.clone(), depth)
        })
        .collect()
}

pub(in crate::network) fn transition_anchor_distance(
    candidate_id: &str,
    selected_id: &str,
    previous_layout: &BTreeMap<String, (f64, f64)>,
    next_layout: &BTreeMap<String, (f64, f64)>,
) -> f64 {
    let candidate = next_layout
        .get(candidate_id)
        .copied()
        .or_else(|| previous_layout.get(candidate_id).copied());
    let selected = next_layout
        .get(selected_id)
        .copied()
        .or_else(|| previous_layout.get(selected_id).copied());
    match (candidate, selected) {
        (Some(candidate), Some(selected)) => {
            let dx = candidate.0 - selected.0;
            let dy = candidate.1 - selected.1;
            (dx * dx + dy * dy).sqrt()
        }
        _ => f64::INFINITY,
    }
}

pub(in crate::network) fn choose_transition_anchor(
    previous_spec: &NetworkPlotSpec,
    next_spec: &NetworkPlotSpec,
    previous_layout: &BTreeMap<String, (f64, f64)>,
    next_layout: &BTreeMap<String, (f64, f64)>,
) -> String {
    let previous_ids = previous_spec
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let next_ids = next_spec
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let previous_parent_by_id = structural_parent_map(previous_spec);
    let next_parent_by_id = structural_parent_map(next_spec);
    let mut candidates = BTreeSet::new();

    for node in &next_spec.nodes {
        if !previous_ids.contains(node.id.as_str()) {
            if let Some(anchor_id) =
                nearest_shared_ancestor(&next_parent_by_id, &previous_ids, node.id.as_str())
            {
                candidates.insert(anchor_id);
            }
        }
    }

    for node in &previous_spec.nodes {
        if !next_ids.contains(node.id.as_str()) {
            if let Some(anchor_id) =
                nearest_shared_ancestor(&previous_parent_by_id, &next_ids, node.id.as_str())
            {
                candidates.insert(anchor_id);
            }
        }
    }

    for node in &next_spec.nodes {
        if !previous_ids.contains(node.id.as_str()) {
            continue;
        }
        let previous_parent = previous_parent_by_id.get(node.id.as_str()).copied();
        let next_parent = next_parent_by_id.get(node.id.as_str()).copied();
        if previous_parent != next_parent {
            candidates.insert(node.id.clone());
        }
    }

    if candidates.is_empty() {
        if let Some(selected_id) = next_spec
            .selected_node_id
            .as_ref()
            .filter(|selected_id| previous_ids.contains(selected_id.as_str()))
        {
            return selected_id.clone();
        }
        if previous_ids.contains("__start__") && next_ids.contains("__start__") {
            return "__start__".to_string();
        }
        return next_spec
            .nodes
            .first()
            .or_else(|| previous_spec.nodes.first())
            .map(|node| node.id.clone())
            .unwrap_or_else(|| "__start__".to_string());
    }

    let next_depths = structural_depths(next_spec);
    let previous_depths = structural_depths(previous_spec);
    let mut stable_order = HashMap::new();
    for node in &next_spec.nodes {
        let next_index = stable_order.len();
        stable_order.entry(node.id.clone()).or_insert(next_index);
    }
    for node in &previous_spec.nodes {
        let next_index = stable_order.len();
        stable_order.entry(node.id.clone()).or_insert(next_index);
    }
    let selected_id = next_spec
        .selected_node_id
        .as_deref()
        .filter(|node_id| next_ids.contains(*node_id))
        .or_else(|| {
            previous_spec
                .selected_node_id
                .as_deref()
                .filter(|node_id| previous_ids.contains(*node_id))
        });

    let mut ordered = candidates.into_iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        let left_depth = next_depths
            .get(left)
            .copied()
            .or_else(|| previous_depths.get(left).copied())
            .unwrap_or(0);
        let right_depth = next_depths
            .get(right)
            .copied()
            .or_else(|| previous_depths.get(right).copied())
            .unwrap_or(0);
        right_depth
            .cmp(&left_depth)
            .then_with(|| {
                let left_distance = selected_id
                    .map(|selected_id| {
                        transition_anchor_distance(left, selected_id, previous_layout, next_layout)
                    })
                    .unwrap_or(f64::INFINITY);
                let right_distance = selected_id
                    .map(|selected_id| {
                        transition_anchor_distance(right, selected_id, previous_layout, next_layout)
                    })
                    .unwrap_or(f64::INFINITY);
                left_distance.total_cmp(&right_distance)
            })
            .then_with(|| {
                let left_order = stable_order.get(left).copied().unwrap_or(usize::MAX);
                let right_order = stable_order.get(right).copied().unwrap_or(usize::MAX);
                left_order.cmp(&right_order)
            })
    });

    ordered
        .into_iter()
        .next()
        .unwrap_or_else(|| "__start__".to_string())
}

pub(in crate::network) fn estimate_world_span(
    spec: &NetworkPlotSpec,
    positions: impl Iterator<Item = (f64, f64)>,
) -> f64 {
    let auto_span = (spec.nodes.len().max(1) as f64).sqrt() * WORLD_NODE_SPACING;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut any = false;
    for (x, y) in positions {
        any = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    if !any {
        return auto_span.max(WORLD_NODE_SPACING * 2.0);
    }
    auto_span
        .max((max_x - min_x).abs() + WORLD_NODE_SPACING * 2.0)
        .max((max_y - min_y).abs() + WORLD_NODE_SPACING * 2.0)
}

pub(in crate::network) fn parent_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    spec.edges
        .iter()
        .filter(|edge| edge_is_structural(edge) && edge.target == node_id)
        .map(|edge| edge.source.clone())
        .collect()
}

pub(in crate::network) fn child_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    spec.edges
        .iter()
        .filter(|edge| edge_is_structural(edge) && edge.source == node_id)
        .map(|edge| edge.target.clone())
        .collect()
}
