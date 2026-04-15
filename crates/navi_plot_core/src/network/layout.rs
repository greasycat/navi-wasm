use super::*;

pub(super) fn validate(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
    ensure_dimensions(spec.width, spec.height)?;
    if spec.nodes.is_empty() {
        return Err(PlotError::EmptyNetwork);
    }

    // Check for duplicate node IDs
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for node in &spec.nodes {
        if !seen_ids.insert(node.id.as_str()) {
            return Err(PlotError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }
    }

    // Check for unknown node references and duplicate edges
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

pub(super) fn resolve_nodes(
    spec: &NetworkPlotSpec,
) -> Result<BTreeMap<String, ResolvedNode>, PlotError> {
    spec.nodes
        .iter()
        .map(|n| {
            let style = resolve_node_style(NodeStyleContext {
                default_fill_color: DEFAULT_NODE_COLOR,
                default_radius: spec.node_radius,
                default_label_visible: spec.show_labels,
                graph_style: spec.default_node_style.as_ref(),
                legacy_fill_color: n.color.as_deref(),
                legacy_shape: n.shape.as_ref(),
                legacy_label_inside: n.label_inside,
                item_style: n.style.as_ref(),
            })?;
            Ok((
                n.id.clone(),
                ResolvedNode {
                    label: if n.label.is_empty() {
                        n.id.clone()
                    } else {
                        n.label.clone()
                    },
                    style,
                    media: node::resolve_node_media(n.media.as_ref())?,
                },
            ))
        })
        .collect()
}

pub(super) fn validate_explicit_positions(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
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

pub(super) fn node_is_pinned(node: &crate::NetworkNode) -> bool {
    node.x.is_some() && node.y.is_some()
}

pub(super) fn deterministic_offset(node_id: &str, radius: f64) -> (f64, f64) {
    let hash = node_id.bytes().fold(0u64, |acc, byte| {
        acc.wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(byte) + 1)
    });
    let angle = ((hash % 3600) as f64 / 3600.0) * TAU;
    let scaled_radius = radius * (0.7 + (((hash / 3600) % 5) as f64) * 0.12);
    (scaled_radius * angle.cos(), scaled_radius * angle.sin())
}

pub(super) fn edge_weight(edge: &crate::NetworkEdge) -> f64 {
    edge.weight.unwrap_or(1.0).max(MIN_LAYOUT_EDGE_WEIGHT)
}

pub(super) fn edge_is_structural(edge: &crate::NetworkEdge) -> bool {
    edge.weight.unwrap_or(1.0) >= STRUCTURAL_EDGE_WEIGHT_THRESHOLD
}

pub(super) fn neighbor_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
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

pub(super) fn adjacency_map(spec: &NetworkPlotSpec) -> BTreeMap<String, BTreeSet<String>> {
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

pub(super) fn topology_identity(
    spec: &NetworkPlotSpec,
) -> (BTreeSet<String>, BTreeSet<(String, String)>) {
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

pub(super) fn topology_changed(
    previous_spec: &NetworkPlotSpec,
    next_spec: &NetworkPlotSpec,
) -> bool {
    topology_identity(previous_spec) != topology_identity(next_spec)
}

pub(super) fn path_edge_exists(spec: &NetworkPlotSpec, from_node: &str, to_node: &str) -> bool {
    spec.edges.iter().any(|edge| {
        (edge.source == from_node && edge.target == to_node)
            || (edge.source == to_node && edge.target == from_node)
    })
}

pub(super) fn node_property_is_true(node: &crate::NetworkNode, key: &str) -> bool {
    node.properties.get(key).is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes"
        )
    })
}

pub(super) fn node_has_toggle_badge(node: &crate::NetworkNode) -> bool {
    node_property_is_true(node, TOGGLEABLE_PROPERTY_KEY)
}

pub(super) fn node_badge_expanded(node: &crate::NetworkNode) -> bool {
    node_property_is_true(node, EXPANDED_PROPERTY_KEY)
}

pub(super) fn structural_parent_map(spec: &NetworkPlotSpec) -> HashMap<&str, &str> {
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

pub(super) fn nearest_shared_ancestor(
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

pub(super) fn structural_depths(spec: &NetworkPlotSpec) -> HashMap<String, usize> {
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

pub(super) fn transition_anchor_distance(
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

pub(super) fn choose_transition_anchor(
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

pub(super) fn estimate_world_span(
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

pub(super) fn seed_position(
    spec: &NetworkPlotSpec,
    node_id: &str,
    positions: &BTreeMap<String, (f64, f64)>,
) -> (f64, f64) {
    if let Some(position) = seed_position_from_parent_gaps(spec, node_id, positions) {
        return position;
    }

    let neighbors: Vec<(f64, f64)> = neighbor_ids(spec, node_id)
        .into_iter()
        .filter_map(|neighbor_id| positions.get(&neighbor_id).copied())
        .collect();
    let world_span = estimate_world_span(spec, positions.values().copied());
    let offset = deterministic_offset(node_id, WORLD_NODE_SPACING * 0.55);
    if !neighbors.is_empty() {
        let (sum_x, sum_y) = neighbors
            .iter()
            .fold((0.0, 0.0), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));
        let base = (
            sum_x / neighbors.len() as f64,
            sum_y / neighbors.len() as f64,
        );
        return (base.0 + offset.0, base.1 + offset.1);
    }
    let existing: Vec<(f64, f64)> = positions.values().copied().collect();
    if !existing.is_empty() {
        let (sum_x, sum_y) = existing
            .iter()
            .fold((0.0, 0.0), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));
        let base = (sum_x / existing.len() as f64, sum_y / existing.len() as f64);
        return (base.0 + offset.0, base.1 + offset.1);
    }
    let grid_side = (spec.nodes.len().max(1) as f64).sqrt().ceil() as usize;
    let step = world_span / (grid_side as f64 + 1.0);
    let col = (positions.len() % grid_side) as f64;
    let row = (positions.len() / grid_side) as f64;
    let left = -world_span / 2.0;
    let top = -world_span / 2.0;
    (
        left + step * (col + 1.0) + offset.0 * 0.2,
        top + step * (row + 1.0) + offset.1 * 0.2,
    )
}

pub(super) fn parent_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    spec.edges
        .iter()
        .filter(|edge| edge_is_structural(edge) && edge.target == node_id)
        .map(|edge| edge.source.clone())
        .collect()
}

pub(super) fn child_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    spec.edges
        .iter()
        .filter(|edge| edge_is_structural(edge) && edge.source == node_id)
        .map(|edge| edge.target.clone())
        .collect()
}

pub(super) fn normalized_angle(angle: f64) -> f64 {
    angle.rem_euclid(TAU)
}

pub(super) fn angle_between(origin: (f64, f64), target: (f64, f64)) -> f64 {
    normalized_angle((target.1 - origin.1).atan2(target.0 - origin.0))
}

pub(super) fn candidate_seed_score(
    candidate: (f64, f64),
    positions: &BTreeMap<String, (f64, f64)>,
) -> f64 {
    positions
        .values()
        .map(|&(x, y)| {
            let dx = candidate.0 - x;
            let dy = candidate.1 - y;
            dx * dx + dy * dy
        })
        .fold(f64::INFINITY, f64::min)
}

pub(super) fn best_gap_candidate(
    spec: &NetworkPlotSpec,
    node_id: &str,
    parent_id: &str,
    parent_pos: (f64, f64),
    positions: &BTreeMap<String, (f64, f64)>,
) -> Option<(f64, f64)> {
    let siblings: Vec<(String, (f64, f64))> = child_ids(spec, parent_id)
        .into_iter()
        .filter(|child_id| child_id != node_id)
        .filter_map(|child_id| positions.get(&child_id).copied().map(|pos| (child_id, pos)))
        .collect();
    let base_distance = if siblings.is_empty() {
        WORLD_NODE_SPACING
    } else {
        let avg_distance = siblings
            .iter()
            .map(|(_, pos)| (pos.0 - parent_pos.0).hypot(pos.1 - parent_pos.1))
            .sum::<f64>()
            / siblings.len() as f64;
        avg_distance.max(WORLD_NODE_SPACING * 0.95)
    };

    if siblings.is_empty() {
        let (ux, uy) = deterministic_unit(&format!("{parent_id}:{node_id}:seed"));
        return Some((
            parent_pos.0 + ux * base_distance,
            parent_pos.1 + uy * base_distance,
        ));
    }

    let mut occupied_angles: Vec<f64> = siblings
        .iter()
        .map(|(_, pos)| angle_between(parent_pos, *pos))
        .collect();
    occupied_angles.sort_by(|left, right| left.total_cmp(right));

    let mut best_candidate = None;
    let mut best_score = f64::NEG_INFINITY;
    for idx in 0..occupied_angles.len() {
        let start = occupied_angles[idx];
        let end = if idx + 1 < occupied_angles.len() {
            occupied_angles[idx + 1]
        } else {
            occupied_angles[0] + TAU
        };
        let gap = end - start;
        let mid = start + gap * 0.5;
        for scale in [1.0, 1.2, 1.45] {
            let distance = base_distance * scale;
            let candidate = (
                parent_pos.0 + mid.cos() * distance,
                parent_pos.1 + mid.sin() * distance,
            );
            let score = candidate_seed_score(candidate, positions) + gap * WORLD_NODE_SPACING;
            if score > best_score {
                best_score = score;
                best_candidate = Some(candidate);
            }
        }
    }

    best_candidate
}

pub(super) fn seed_position_from_parent_gaps(
    spec: &NetworkPlotSpec,
    node_id: &str,
    positions: &BTreeMap<String, (f64, f64)>,
) -> Option<(f64, f64)> {
    let mut best_candidate = None;
    let mut best_score = f64::NEG_INFINITY;
    for parent_id in parent_ids(spec, node_id) {
        let Some(&parent_pos) = positions.get(&parent_id) else {
            continue;
        };
        let sibling_count = child_ids(spec, &parent_id)
            .into_iter()
            .filter(|child_id| child_id != node_id && positions.contains_key(child_id))
            .count() as f64;
        let Some(candidate) = best_gap_candidate(spec, node_id, &parent_id, parent_pos, positions)
        else {
            continue;
        };
        let score = candidate_seed_score(candidate, positions)
            + sibling_count * WORLD_NODE_SPACING * WORLD_NODE_SPACING;
        if score > best_score {
            best_score = score;
            best_candidate = Some(candidate);
        }
    }
    best_candidate
}

pub(super) fn layout_zoom(view_zoom: f64) -> f64 {
    view_zoom.max(MIN_LAYOUT_ZOOM)
}

pub(super) fn deterministic_unit(seed: &str) -> (f64, f64) {
    let (x, y) = deterministic_offset(seed, 1.0);
    let len = (x * x + y * y).sqrt().max(0.01);
    (x / len, y / len)
}

pub(super) fn order_children_by_sibling_edges(
    spec: &NetworkPlotSpec,
    children: &[usize],
    id_to_idx: &HashMap<&str, usize>,
) -> Vec<usize> {
    if children.len() <= 1 {
        return children.to_vec();
    }

    let child_set: HashSet<usize> = children.iter().copied().collect();
    let mut indegree = HashMap::<usize, usize>::new();
    let mut next_by_child = HashMap::<usize, usize>::new();
    let mut sibling_edge_count = 0usize;

    for edge in &spec.edges {
        if edge_is_structural(edge) {
            continue;
        }
        let Some(&source_idx) = id_to_idx.get(edge.source.as_str()) else {
            continue;
        };
        let Some(&target_idx) = id_to_idx.get(edge.target.as_str()) else {
            continue;
        };
        if !child_set.contains(&source_idx) || !child_set.contains(&target_idx) {
            continue;
        }
        sibling_edge_count += 1;
        if next_by_child.insert(source_idx, target_idx).is_some() {
            return children.to_vec();
        }
        let entry = indegree.entry(target_idx).or_insert(0);
        *entry += 1;
        if *entry > 1 {
            return children.to_vec();
        }
    }

    if sibling_edge_count != children.len() - 1 {
        return children.to_vec();
    }

    let starts: Vec<usize> = children
        .iter()
        .copied()
        .filter(|child_idx| indegree.get(child_idx).copied().unwrap_or(0) == 0)
        .collect();
    if starts.len() != 1 {
        return children.to_vec();
    }

    let mut ordered = Vec::with_capacity(children.len());
    let mut current = starts[0];
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(current) {
            return children.to_vec();
        }
        ordered.push(current);
        let Some(&next) = next_by_child.get(&current) else {
            break;
        };
        current = next;
    }

    if ordered.len() == children.len() {
        ordered
    } else {
        children.to_vec()
    }
}

pub(super) fn build_hierarchical_layout(spec: &NetworkPlotSpec) -> Option<HierarchicalLayout> {
    let id_to_idx: HashMap<&str, usize> = spec
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.id.as_str(), idx))
        .collect();
    let mut parent_by_idx = vec![None; spec.nodes.len()];
    let mut children_by_idx = vec![Vec::new(); spec.nodes.len()];

    for edge in &spec.edges {
        if !edge_is_structural(edge) {
            continue;
        }
        let Some(&source_idx) = id_to_idx.get(edge.source.as_str()) else {
            return None;
        };
        let Some(&target_idx) = id_to_idx.get(edge.target.as_str()) else {
            return None;
        };
        if parent_by_idx[target_idx].is_some() {
            return None;
        }
        parent_by_idx[target_idx] = Some(source_idx);
        children_by_idx[source_idx].push(target_idx);
    }

    let roots: Vec<usize> = parent_by_idx
        .iter()
        .enumerate()
        .filter_map(|(idx, parent)| parent.is_none().then_some(idx))
        .collect();
    if roots.len() != 1 {
        return None;
    }
    let root_idx = roots[0];
    if spec
        .nodes
        .iter()
        .enumerate()
        .any(|(idx, node)| idx != root_idx && node_is_pinned(node))
    {
        return None;
    }

    for children in &mut children_by_idx {
        *children = order_children_by_sibling_edges(spec, children, &id_to_idx);
    }

    let mut depth_by_idx = vec![usize::MAX; spec.nodes.len()];
    let mut stack = vec![root_idx];
    depth_by_idx[root_idx] = 0;
    let mut visited = 0usize;
    while let Some(node_idx) = stack.pop() {
        visited += 1;
        let next_depth = depth_by_idx[node_idx] + 1;
        for &child_idx in &children_by_idx[node_idx] {
            if depth_by_idx[child_idx] != usize::MAX {
                return None;
            }
            depth_by_idx[child_idx] = next_depth;
            stack.push(child_idx);
        }
    }
    if visited != spec.nodes.len() {
        return None;
    }

    fn assign_subtree_size(
        node_idx: usize,
        children_by_idx: &[Vec<usize>],
        subtree_size_by_idx: &mut [usize],
    ) -> usize {
        let size = 1 + children_by_idx[node_idx]
            .iter()
            .copied()
            .map(|child_idx| assign_subtree_size(child_idx, children_by_idx, subtree_size_by_idx))
            .sum::<usize>();
        subtree_size_by_idx[node_idx] = size;
        size
    }

    let mut subtree_size_by_idx = vec![0usize; spec.nodes.len()];
    assign_subtree_size(root_idx, &children_by_idx, &mut subtree_size_by_idx);

    Some(HierarchicalLayout {
        root_idx,
        parent_by_idx,
        children_by_idx,
        depth_by_idx,
        subtree_size_by_idx,
    })
}

pub(super) fn radial_origin(spec: &NetworkPlotSpec, hierarchy: &HierarchicalLayout) -> (f64, f64) {
    let root = &spec.nodes[hierarchy.root_idx];
    (root.x.unwrap_or(0.0), root.y.unwrap_or(0.0))
}

pub(super) fn polar_to_cartesian(origin: (f64, f64), radius: f64, angle: f64) -> (f64, f64) {
    (
        origin.0 + radius * angle.cos(),
        origin.1 + radius * angle.sin(),
    )
}

pub(super) fn unwrap_angle_near(angle: f64, reference: f64) -> f64 {
    let mut unwrapped = angle;
    while unwrapped - reference > PI {
        unwrapped -= TAU;
    }
    while unwrapped - reference < -PI {
        unwrapped += TAU;
    }
    unwrapped
}

pub(super) fn polar_from_position_unwrapped(
    origin: (f64, f64),
    position: (f64, f64),
    reference_angle: f64,
) -> (f64, f64) {
    let dx = position.0 - origin.0;
    let dy = position.1 - origin.1;
    let radius = (dx * dx + dy * dy).sqrt();
    if radius <= 0.01 {
        return (0.0, reference_angle);
    }
    let angle = normalized_angle(dy.atan2(dx));
    (radius, unwrap_angle_near(angle, reference_angle))
}

pub(super) fn radial_ring_spacing(
    spec: &NetworkPlotSpec,
    resolved: &BTreeMap<String, ResolvedNode>,
    view_zoom: f64,
) -> f64 {
    let max_radius = resolved
        .values()
        .map(|node| node.style.radius.max(1) as f64)
        .fold(spec.node_radius.max(1) as f64, f64::max);
    let zoom = layout_zoom(view_zoom);
    let font_size = (13.0 * spec.pixel_ratio.max(0.25)).round().max(1.0);
    let max_label_height = resolved
        .values()
        .filter(|node| {
            node.style.label_visible && !node.style.label_inside && !node.label.is_empty()
        })
        .map(|_| (font_size * LABEL_HEIGHT_FACTOR + 8.0) / zoom)
        .fold(0.0, f64::max);
    WORLD_NODE_SPACING.max(max_radius * 2.0 + max_label_height + 24.0 / zoom)
        * RADIAL_RING_SPACING_SCALE
}

pub(super) fn child_intervals(
    start: f64,
    end: f64,
    weights: &[f64],
    desired_centers: Option<&[f64]>,
) -> Vec<(f64, f64)> {
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    if weights.is_empty() {
        return Vec::new();
    }
    if weights.len() == 1 {
        return vec![(start, end)];
    }

    let span = (end - start).max(0.01);
    if span <= 0.02 {
        return vec![(start, end); weights.len()];
    }
    let mut gap = ((span * 0.12) / weights.len() as f64).clamp(0.01, 0.18);
    let max_gap_total = span * 0.22;
    gap = gap.min(max_gap_total / (weights.len() - 1) as f64);
    let available = (span - gap * (weights.len() - 1) as f64).max(0.0);
    if available <= 0.02 {
        let slot = span / weights.len() as f64;
        return (0..weights.len())
            .map(|idx| {
                let slot_start = start + slot * idx as f64;
                (slot_start, slot_start + slot)
            })
            .collect();
    }
    let width_total = available * 0.86;
    let weight_sum = weights.iter().sum::<f64>().max(0.001);
    let widths: Vec<f64> = weights
        .iter()
        .map(|weight| width_total * (*weight / weight_sum))
        .collect();

    let canonical_start = start + (span - (width_total + gap * (weights.len() - 1) as f64)) * 0.5;
    let canonical = widths
        .iter()
        .scan(canonical_start, |cursor, width| {
            let interval = (*cursor, *cursor + *width);
            *cursor += *width + gap;
            Some(interval)
        })
        .collect::<Vec<_>>();

    let Some(desired_centers) = desired_centers else {
        return canonical;
    };

    let mut suffix_required = vec![0.0; widths.len()];
    let mut running = 0.0;
    for idx in (0..widths.len()).rev() {
        suffix_required[idx] = running;
        running += widths[idx];
        if idx + 1 < widths.len() {
            running += gap;
        }
    }

    let mut starts = vec![0.0; widths.len()];
    for idx in 0..widths.len() {
        let canonical_center = (canonical[idx].0 + canonical[idx].1) * 0.5;
        let desired_center = unwrap_angle_near(desired_centers[idx], canonical_center);
        let desired_start = desired_center - widths[idx] * 0.5;
        let min_start = if idx == 0 {
            start
        } else {
            starts[idx - 1] + widths[idx - 1] + gap
        };
        let max_start = end - widths[idx] - suffix_required[idx];
        if max_start < min_start {
            return canonical;
        }
        starts[idx] = desired_start.clamp(min_start, max_start);
    }

    for idx in (0..starts.len().saturating_sub(1)).rev() {
        let max_start = starts[idx + 1] - gap - widths[idx];
        starts[idx] = starts[idx].min(max_start);
    }
    let first_shift = start - starts[0];
    if first_shift > 0.0 {
        for value in &mut starts {
            *value += first_shift;
        }
    }
    let overflow = starts[starts.len() - 1] + widths[widths.len() - 1] - end;
    if overflow > 0.0 {
        for value in &mut starts {
            *value -= overflow;
        }
    }

    starts
        .into_iter()
        .zip(widths)
        .map(|(slot_start, width)| {
            let slot_end = slot_start + width;
            (slot_start.min(slot_end), slot_start.max(slot_end))
        })
        .collect()
}

pub(super) fn root_child_anchor_angles(
    spec: &NetworkPlotSpec,
    hierarchy: &HierarchicalLayout,
    previous_spec: &NetworkPlotSpec,
    previous_hierarchy: &HierarchicalLayout,
    previous_layout: &BTreeMap<String, (f64, f64)>,
) -> HashMap<String, f64> {
    let previous_root_id = previous_spec.nodes[previous_hierarchy.root_idx].id.as_str();
    let next_root_id = spec.nodes[hierarchy.root_idx].id.as_str();
    if previous_root_id != next_root_id {
        return HashMap::new();
    }

    let prev_origin = radial_origin(previous_spec, previous_hierarchy);
    let previous_id_to_idx: HashMap<&str, usize> = previous_spec
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.id.as_str(), idx))
        .collect();
    hierarchy.children_by_idx[hierarchy.root_idx]
        .iter()
        .filter_map(|&child_idx| {
            let child_id = spec.nodes[child_idx].id.as_str();
            let &previous_child_idx = previous_id_to_idx.get(child_id)?;
            if previous_hierarchy.parent_by_idx[previous_child_idx]
                != Some(previous_hierarchy.root_idx)
            {
                return None;
            }
            let position = previous_layout.get(child_id)?;
            Some((
                child_id.to_string(),
                normalized_angle((position.1 - prev_origin.1).atan2(position.0 - prev_origin.0)),
            ))
        })
        .collect()
}

pub(super) fn radial_targets(
    spec: &NetworkPlotSpec,
    resolved: &BTreeMap<String, ResolvedNode>,
    hierarchy: &HierarchicalLayout,
    view_zoom: f64,
    previous_root_anchors: Option<&HashMap<String, f64>>,
) -> (Vec<RadialTarget>, Vec<f64>) {
    let max_depth = hierarchy.depth_by_idx.iter().copied().max().unwrap_or(0);
    let ring_spacing = radial_ring_spacing(spec, resolved, view_zoom);
    let ring_radii: Vec<f64> = (0..=max_depth)
        .map(|depth| depth as f64 * ring_spacing)
        .collect();
    let mut targets = vec![
        RadialTarget {
            radius: 0.0,
            angle: RADIAL_START_ANGLE,
            min_angle: RADIAL_START_ANGLE,
            max_angle: RADIAL_START_ANGLE + TAU,
        };
        spec.nodes.len()
    ];

    fn assign_targets(
        spec: &NetworkPlotSpec,
        hierarchy: &HierarchicalLayout,
        ring_radii: &[f64],
        previous_root_anchors: Option<&HashMap<String, f64>>,
        node_idx: usize,
        start_angle: f64,
        end_angle: f64,
        targets: &mut [RadialTarget],
    ) {
        let children = &hierarchy.children_by_idx[node_idx];
        if children.is_empty() {
            return;
        }

        let weights: Vec<f64> = children
            .iter()
            .map(|&child_idx| hierarchy.subtree_size_by_idx[child_idx] as f64)
            .collect();
        let canonical = child_intervals(start_angle, end_angle, &weights, None);
        let anchored = if node_idx == hierarchy.root_idx {
            let desired_centers: Vec<f64> = children
                .iter()
                .enumerate()
                .map(|(child_pos, &child_idx)| {
                    previous_root_anchors
                        .and_then(|anchors| anchors.get(spec.nodes[child_idx].id.as_str()).copied())
                        .unwrap_or((canonical[child_pos].0 + canonical[child_pos].1) * 0.5)
                })
                .collect();
            child_intervals(start_angle, end_angle, &weights, Some(&desired_centers))
        } else {
            canonical
        };

        for (child_pos, &child_idx) in children.iter().enumerate() {
            let (slot_start, slot_end) = anchored[child_pos];
            let depth = hierarchy.depth_by_idx[child_idx];
            let min_angle = slot_start.min(slot_end);
            let max_angle = slot_start.max(slot_end);
            targets[child_idx] = RadialTarget {
                radius: ring_radii[depth],
                angle: (min_angle + max_angle) * 0.5,
                min_angle,
                max_angle,
            };
            assign_targets(
                spec,
                hierarchy,
                ring_radii,
                previous_root_anchors,
                child_idx,
                slot_start,
                slot_end,
                targets,
            );
        }
    }

    assign_targets(
        spec,
        hierarchy,
        &ring_radii,
        previous_root_anchors,
        hierarchy.root_idx,
        RADIAL_START_ANGLE,
        RADIAL_START_ANGLE + TAU,
        &mut targets,
    );

    (targets, ring_radii)
}

pub(super) fn constrain_radial_positions(
    hierarchy: &HierarchicalLayout,
    targets: &[RadialTarget],
    ring_radii: &[f64],
    origin: (f64, f64),
    positions: &mut [(f64, f64)],
) {
    let ring_spacing = if ring_radii.len() > 1 {
        ring_radii[1] - ring_radii[0]
    } else {
        WORLD_NODE_SPACING
    };
    positions[hierarchy.root_idx] = origin;
    for idx in 0..positions.len() {
        if idx == hierarchy.root_idx {
            continue;
        }
        let target = targets[idx];
        let depth = hierarchy.depth_by_idx[idx];
        let (current_radius, current_angle) =
            polar_from_position_unwrapped(origin, positions[idx], target.angle);
        let min_radius = if depth == 0 {
            0.0
        } else {
            (ring_radii[depth - 1] + ring_radii[depth]) * 0.5
        };
        let max_radius = if depth + 1 < ring_radii.len() {
            (ring_radii[depth] + ring_radii[depth + 1]) * 0.5
        } else {
            ring_radii[depth] + ring_spacing * 0.5
        };
        let min_angle = target.min_angle.min(target.max_angle);
        let max_angle = target.max_angle.max(target.min_angle);
        let clamped_radius = current_radius.clamp(min_radius, max_radius.max(min_radius));
        let clamped_angle = current_angle.clamp(min_angle, max_angle);
        positions[idx] = polar_to_cartesian(origin, clamped_radius, clamped_angle);
    }
}

pub(super) fn relax_radial_positions(
    hierarchy: &HierarchicalLayout,
    targets: &[RadialTarget],
    ring_radii: &[f64],
    origin: (f64, f64),
    positions: &mut [(f64, f64)],
    iterations: usize,
) {
    if positions.is_empty() {
        return;
    }
    let ring_spacing = if ring_radii.len() > 1 {
        ring_radii[1] - ring_radii[0]
    } else {
        WORLD_NODE_SPACING
    };
    let mut temperature = (ring_spacing * 0.18).max(8.0);
    let max_iterations = iterations.max(1).min(LOCAL_RELAXATION_MAX_ITERATIONS);

    for _ in 0..max_iterations {
        let mut displacement = vec![(0.0, 0.0); positions.len()];

        for source_idx in 0..positions.len() {
            if source_idx == hierarchy.root_idx {
                continue;
            }
            for target_idx in (source_idx + 1)..positions.len() {
                if target_idx == hierarchy.root_idx {
                    continue;
                }
                let depth_gap =
                    hierarchy.depth_by_idx[source_idx].abs_diff(hierarchy.depth_by_idx[target_idx]);
                if depth_gap > 1 {
                    continue;
                }

                let dx = positions[source_idx].0 - positions[target_idx].0;
                let dy = positions[source_idx].1 - positions[target_idx].1;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = (RADIAL_REPULSION_FORCE * ring_spacing * ring_spacing / dist)
                    / (depth_gap as f64 + 1.0);
                let fx = dx / dist * force;
                let fy = dy / dist * force;
                displacement[source_idx].0 += fx;
                displacement[source_idx].1 += fy;
                displacement[target_idx].0 -= fx;
                displacement[target_idx].1 -= fy;
            }
        }

        for idx in 0..positions.len() {
            if idx == hierarchy.root_idx {
                continue;
            }

            let target = targets[idx];
            let (radius, angle) =
                polar_from_position_unwrapped(origin, positions[idx], target.angle);
            let radial_dir = if radius > 0.01 {
                (
                    (positions[idx].0 - origin.0) / radius,
                    (positions[idx].1 - origin.1) / radius,
                )
            } else {
                (target.angle.cos(), target.angle.sin())
            };
            let tangent_dir = (-radial_dir.1, radial_dir.0);
            let radius_error = target.radius - radius;
            let angle_error = target.angle - angle;
            displacement[idx].0 += radial_dir.0 * radius_error * RADIAL_RADIUS_SPRING;
            displacement[idx].1 += radial_dir.1 * radius_error * RADIAL_RADIUS_SPRING;
            displacement[idx].0 += tangent_dir.0
                * angle_error
                * target.radius.max(ring_spacing * 0.6)
                * RADIAL_ANGLE_SPRING;
            displacement[idx].1 += tangent_dir.1
                * angle_error
                * target.radius.max(ring_spacing * 0.6)
                * RADIAL_ANGLE_SPRING;

            if let Some(parent_idx) = hierarchy.parent_by_idx[idx] {
                let dx = positions[parent_idx].0 - positions[idx].0;
                let dy = positions[parent_idx].1 - positions[idx].1;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let desired = (targets[idx].radius - targets[parent_idx].radius)
                    .abs()
                    .max(ring_spacing * 0.72);
                let stretch = dist - desired;
                let fx = dx / dist * stretch * RADIAL_PARENT_SPRING;
                let fy = dy / dist * stretch * RADIAL_PARENT_SPRING;
                displacement[idx].0 += fx;
                displacement[idx].1 += fy;
                if parent_idx != hierarchy.root_idx {
                    displacement[parent_idx].0 -= fx;
                    displacement[parent_idx].1 -= fy;
                }
            }
        }

        for idx in 0..positions.len() {
            if idx == hierarchy.root_idx {
                positions[idx] = origin;
                continue;
            }
            let magnitude = (displacement[idx].0 * displacement[idx].0
                + displacement[idx].1 * displacement[idx].1)
                .sqrt()
                .max(0.01);
            let scale = magnitude.min(temperature) / magnitude;
            positions[idx].0 += displacement[idx].0 * scale;
            positions[idx].1 += displacement[idx].1 * scale;
        }
        constrain_radial_positions(hierarchy, targets, ring_radii, origin, positions);
        temperature *= 0.9;
    }
}

pub(super) fn compute_radial_layout_with_zoom(
    spec: &NetworkPlotSpec,
    resolved: &BTreeMap<String, ResolvedNode>,
    hierarchy: &HierarchicalLayout,
    view_zoom: f64,
    previous_spec: Option<&NetworkPlotSpec>,
    previous_hierarchy: Option<&HierarchicalLayout>,
    previous_layout: Option<&BTreeMap<String, (f64, f64)>>,
) -> BTreeMap<String, (f64, f64)> {
    let previous_root_anchors = previous_spec
        .zip(previous_hierarchy)
        .zip(previous_layout)
        .map(|((previous_spec, previous_hierarchy), previous_layout)| {
            root_child_anchor_angles(
                spec,
                hierarchy,
                previous_spec,
                previous_hierarchy,
                previous_layout,
            )
        });
    let (targets, ring_radii) = radial_targets(
        spec,
        resolved,
        hierarchy,
        view_zoom,
        previous_root_anchors.as_ref(),
    );
    let origin = radial_origin(spec, hierarchy);
    let mut positions: Vec<(f64, f64)> = spec
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            if idx == hierarchy.root_idx {
                origin
            } else if let Some(previous_layout) = previous_layout {
                previous_layout.get(&node.id).copied().unwrap_or_else(|| {
                    polar_to_cartesian(origin, targets[idx].radius, targets[idx].angle)
                })
            } else {
                polar_to_cartesian(origin, targets[idx].radius, targets[idx].angle)
            }
        })
        .collect();
    constrain_radial_positions(hierarchy, &targets, &ring_radii, origin, &mut positions);
    relax_radial_positions(
        hierarchy,
        &targets,
        &ring_radii,
        origin,
        &mut positions,
        spec.layout_iterations as usize,
    );

    spec.nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.id.clone(), positions[idx]))
        .collect()
}

pub(super) fn node_label_box(
    spec: &NetworkPlotSpec,
    resolved: &ResolvedNode,
    center: (f64, f64),
    view_zoom: f64,
) -> Option<LabelBox> {
    if !resolved.style.label_visible || resolved.label.is_empty() {
        return None;
    }
    let label_inside = resolved.style.label_inside && resolved.media.is_none();
    if label_inside {
        return None;
    }

    let zoom = layout_zoom(view_zoom);
    let padding = LABEL_COLLISION_PADDING / zoom;
    let font_size = (13.0 * spec.pixel_ratio.max(0.25)).round().max(1.0);
    let label_width =
        ((resolved.label.chars().count() as f64) * font_size * LABEL_WIDTH_FACTOR) / zoom;
    let label_height = (font_size * LABEL_HEIGHT_FACTOR) / zoom;
    let top = center.1 + resolved.style.radius.max(1) as f64 + 4.0 / zoom - padding;
    let half_width = label_width * 0.5 + padding;

    Some(LabelBox {
        left: center.0 - half_width,
        right: center.0 + half_width,
        top,
        bottom: top + label_height + padding * 2.0,
    })
}

pub(super) fn node_footprints(
    spec: &NetworkPlotSpec,
    resolved: &BTreeMap<String, ResolvedNode>,
    positions: &[(f64, f64)],
    view_zoom: f64,
) -> Vec<NodeFootprint> {
    spec.nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            let resolved_node = resolved.get(&node.id).expect("resolved network node");
            let center = positions[idx];
            NodeFootprint {
                center,
                radius: resolved_node.style.radius.max(1) as f64 + NODE_COLLISION_PADDING,
                label: node_label_box(spec, resolved_node, center, view_zoom),
            }
        })
        .collect()
}

pub(super) fn footprint_bounds(footprint: NodeFootprint) -> LabelBox {
    let mut bounds = LabelBox {
        left: footprint.center.0 - footprint.radius,
        right: footprint.center.0 + footprint.radius,
        top: footprint.center.1 - footprint.radius,
        bottom: footprint.center.1 + footprint.radius,
    };
    if let Some(label) = footprint.label {
        bounds.left = bounds.left.min(label.left);
        bounds.right = bounds.right.max(label.right);
        bounds.top = bounds.top.min(label.top);
        bounds.bottom = bounds.bottom.max(label.bottom);
    }
    bounds
}

pub(super) fn collision_candidate_pairs(footprints: &[NodeFootprint]) -> Vec<(usize, usize)> {
    if footprints.len() < 2 {
        return Vec::new();
    }

    let cell_size = WORLD_NODE_SPACING.max(1.0);
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (idx, footprint) in footprints.iter().copied().enumerate() {
        let bounds = footprint_bounds(footprint);
        let min_x = (bounds.left / cell_size).floor() as i32;
        let max_x = (bounds.right / cell_size).floor() as i32;
        let min_y = (bounds.top / cell_size).floor() as i32;
        let max_y = (bounds.bottom / cell_size).floor() as i32;
        for cell_x in min_x..=max_x {
            for cell_y in min_y..=max_y {
                grid.entry((cell_x, cell_y)).or_default().push(idx);
            }
        }
    }

    let mut pairs = HashSet::new();
    for entries in grid.values() {
        if entries.len() < 2 {
            continue;
        }
        for source_idx in 0..entries.len() {
            for target_idx in (source_idx + 1)..entries.len() {
                let left = entries[source_idx].min(entries[target_idx]);
                let right = entries[source_idx].max(entries[target_idx]);
                pairs.insert((left, right));
            }
        }
    }

    let mut pairs: Vec<(usize, usize)> = pairs.into_iter().collect();
    pairs.sort_unstable();
    pairs
}

pub(super) fn circle_separation(
    source_id: &str,
    target_id: &str,
    source: (f64, f64),
    source_radius: f64,
    target: (f64, f64),
    target_radius: f64,
) -> Option<(f64, f64)> {
    let dx = target.0 - source.0;
    let dy = target.1 - source.1;
    let dist = (dx * dx + dy * dy).sqrt();
    let required = source_radius + target_radius + COLLISION_GAP;
    if dist >= required {
        return None;
    }
    if dist > 0.01 {
        let scale = (required - dist) / dist;
        return Some((dx * scale, dy * scale));
    }

    let seed = format!("{source_id}:{target_id}");
    let (ux, uy) = deterministic_unit(&seed);
    Some((ux * required, uy * required))
}

pub(super) fn label_box_separation(source: LabelBox, target: LabelBox) -> Option<(f64, f64)> {
    let (overlap_x, overlap_y) = source.overlap_amount(target)?;
    let source_center = source.center();
    let target_center = target.center();
    if overlap_x <= overlap_y {
        let direction = if target_center.0 >= source_center.0 {
            1.0
        } else {
            -1.0
        };
        Some((direction * (overlap_x + COLLISION_GAP), 0.0))
    } else {
        let direction = if target_center.1 >= source_center.1 {
            1.0
        } else {
            -1.0
        };
        Some((0.0, direction * (overlap_y + COLLISION_GAP)))
    }
}

pub(super) fn circle_label_separation(
    circle: (f64, f64),
    radius: f64,
    label: LabelBox,
) -> Option<(f64, f64)> {
    let nearest_x = circle.0.clamp(label.left, label.right);
    let nearest_y = circle.1.clamp(label.top, label.bottom);
    let dx = circle.0 - nearest_x;
    let dy = circle.1 - nearest_y;
    let dist_sq = dx * dx + dy * dy;
    let required = radius + COLLISION_GAP;

    if dist_sq > 1e-6 {
        let dist = dist_sq.sqrt();
        if dist >= required {
            return None;
        }
        let overlap = required - dist;
        return Some((-(dx / dist) * overlap, -(dy / dist) * overlap));
    }

    let left = circle.0 - label.left;
    let right = label.right - circle.0;
    let top = circle.1 - label.top;
    let bottom = label.bottom - circle.1;
    let min_side = left.min(right).min(top).min(bottom);
    if (min_side - left).abs() < f64::EPSILON {
        Some((left + required, 0.0))
    } else if (min_side - right).abs() < f64::EPSILON {
        Some((-(right + required), 0.0))
    } else if (min_side - top).abs() < f64::EPSILON {
        Some((0.0, top + required))
    } else {
        Some((0.0, -(bottom + required)))
    }
}

pub(super) fn apply_pair_separation(
    shifts: &mut [(f64, f64)],
    movable: &[bool],
    source_idx: usize,
    target_idx: usize,
    separation: (f64, f64),
) -> bool {
    match (movable[source_idx], movable[target_idx]) {
        (false, false) => false,
        (true, true) => {
            shifts[source_idx].0 -= separation.0 * 0.5;
            shifts[source_idx].1 -= separation.1 * 0.5;
            shifts[target_idx].0 += separation.0 * 0.5;
            shifts[target_idx].1 += separation.1 * 0.5;
            true
        }
        (true, false) => {
            shifts[source_idx].0 -= separation.0;
            shifts[source_idx].1 -= separation.1;
            true
        }
        (false, true) => {
            shifts[target_idx].0 += separation.0;
            shifts[target_idx].1 += separation.1;
            true
        }
    }
}

pub(super) fn resolve_layout_collisions(
    spec: &NetworkPlotSpec,
    resolved: &BTreeMap<String, ResolvedNode>,
    positions: &mut [(f64, f64)],
    movable: &[bool],
    view_zoom: f64,
) {
    if !ENABLE_LAYOUT_COLLISIONS {
        return;
    }
    if positions.len() <= 1 || !movable.iter().any(|&can_move| can_move) {
        return;
    }

    let max_step = (WORLD_NODE_SPACING * 0.45).max(
        estimate_world_span(spec, positions.iter().copied()) / (positions.len().max(1) as f64),
    );
    for _ in 0..COLLISION_RESOLUTION_MAX_ITERATIONS {
        let footprints = node_footprints(spec, resolved, positions, view_zoom);
        let candidate_pairs = collision_candidate_pairs(&footprints);
        let mut shifts = vec![(0.0, 0.0); positions.len()];
        let mut any_overlap = false;

        for (source_idx, target_idx) in candidate_pairs {
            let source = footprints[source_idx];
            let source_id = spec.nodes[source_idx].id.as_str();
            let target = footprints[target_idx];
            let target_id = spec.nodes[target_idx].id.as_str();

            if let Some(separation) = circle_separation(
                source_id,
                target_id,
                source.center,
                source.radius,
                target.center,
                target.radius,
            ) {
                any_overlap |=
                    apply_pair_separation(&mut shifts, movable, source_idx, target_idx, separation);
            }

            if let (Some(source_label), Some(target_label)) = (source.label, target.label) {
                if let Some(separation) = label_box_separation(source_label, target_label) {
                    any_overlap |= apply_pair_separation(
                        &mut shifts,
                        movable,
                        source_idx,
                        target_idx,
                        separation,
                    );
                }
            }

            if let Some(source_label) = source.label {
                if let Some(separation) =
                    circle_label_separation(target.center, target.radius, source_label)
                {
                    any_overlap |= apply_pair_separation(
                        &mut shifts,
                        movable,
                        target_idx,
                        source_idx,
                        separation,
                    );
                }
            }

            if let Some(target_label) = target.label {
                if let Some(separation) =
                    circle_label_separation(source.center, source.radius, target_label)
                {
                    any_overlap |= apply_pair_separation(
                        &mut shifts,
                        movable,
                        source_idx,
                        target_idx,
                        separation,
                    );
                }
            }
        }

        if !any_overlap {
            break;
        }

        let mut max_applied = 0.0_f64;
        for (idx, shift) in shifts.into_iter().enumerate() {
            if !movable[idx] {
                continue;
            }
            let magnitude = (shift.0 * shift.0 + shift.1 * shift.1).sqrt();
            if magnitude <= 0.01 {
                continue;
            }
            let scale = (max_step / magnitude).min(1.0);
            let applied = (shift.0 * scale, shift.1 * scale);
            positions[idx].0 += applied.0;
            positions[idx].1 += applied.1;
            max_applied = max_applied.max((applied.0 * applied.0 + applied.1 * applied.1).sqrt());
        }

        if max_applied <= 0.05 {
            break;
        }
    }
}

pub(super) fn relax_positions(
    spec: &NetworkPlotSpec,
    positions: &mut [(f64, f64)],
    movable: &[bool],
    iterations: usize,
) {
    if iterations == 0 || positions.is_empty() {
        return;
    }
    let n = positions.len();
    let id_to_idx: HashMap<&str, usize> = spec
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();
    let adj: Vec<(usize, usize, f64)> = spec
        .edges
        .iter()
        .filter_map(|edge| {
            let src = id_to_idx.get(edge.source.as_str())?;
            let tgt = id_to_idx.get(edge.target.as_str())?;
            Some((*src, *tgt, edge_weight(edge)))
        })
        .collect();
    let world_span = estimate_world_span(spec, positions.iter().copied());
    let k = ((world_span * world_span) / n as f64).sqrt().max(1.0);
    let mut temperature = (world_span * 0.12).max(WORLD_NODE_SPACING * 0.35);

    for _ in 0..iterations {
        let mut displacement = vec![(0.0, 0.0); n];

        for u in 0..n {
            if !movable[u] {
                continue;
            }
            for v in 0..n {
                if u == v {
                    continue;
                }
                let dx = positions[u].0 - positions[v].0;
                let dy = positions[u].1 - positions[v].1;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = k * k / dist;
                displacement[u].0 += dx / dist * force;
                displacement[u].1 += dy / dist * force;
            }
        }

        for &(src, tgt, weight) in &adj {
            let dx = positions[tgt].0 - positions[src].0;
            let dy = positions[tgt].1 - positions[src].1;
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let force = (dist * dist / k).max(0.01) * weight;
            let fx = dx / dist * force;
            let fy = dy / dist * force;
            if movable[src] {
                displacement[src].0 += fx;
                displacement[src].1 += fy;
            }
            if movable[tgt] {
                displacement[tgt].0 -= fx;
                displacement[tgt].1 -= fy;
            }
        }

        for i in 0..n {
            if !movable[i] {
                continue;
            }
            let disp_len = (displacement[i].0 * displacement[i].0
                + displacement[i].1 * displacement[i].1)
                .sqrt()
                .max(0.01);
            let scale = disp_len.min(temperature) / disp_len;
            positions[i].0 += displacement[i].0 * scale;
            positions[i].1 += displacement[i].1 * scale;
        }

        temperature *= 0.92;
    }
}

pub(super) fn fr_layout(spec: &NetworkPlotSpec) -> BTreeMap<String, (f64, f64)> {
    let mut seeded = BTreeMap::new();
    for node in &spec.nodes {
        let position = if node_is_pinned(node) {
            (node.x.unwrap(), node.y.unwrap())
        } else {
            seed_position(spec, &node.id, &seeded)
        };
        seeded.insert(node.id.clone(), position);
    }

    let mut pos: Vec<(f64, f64)> = spec
        .nodes
        .iter()
        .map(|node| {
            seeded
                .get(&node.id)
                .copied()
                .expect("layout seeded for every node")
        })
        .collect();
    let movable: Vec<bool> = spec
        .nodes
        .iter()
        .map(|node| !node_is_pinned(node))
        .collect();
    relax_positions(
        spec,
        &mut pos,
        &movable,
        spec.layout_iterations.max(1) as usize,
    );

    spec.nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.clone(), pos[i]))
        .collect()
}

pub(super) fn compute_layout_with_zoom(
    spec: &NetworkPlotSpec,
    view_zoom: f64,
) -> Result<BTreeMap<String, (f64, f64)>, PlotError> {
    validate_explicit_positions(spec)?;
    let resolved = resolve_nodes(spec)?;
    if let Some(hierarchy) = build_hierarchical_layout(spec) {
        return Ok(compute_radial_layout_with_zoom(
            spec, &resolved, &hierarchy, view_zoom, None, None, None,
        ));
    }
    let layout = fr_layout(spec);
    let mut positions: Vec<(f64, f64)> = spec
        .nodes
        .iter()
        .map(|node| {
            layout
                .get(&node.id)
                .copied()
                .expect("layout seeded for every node")
        })
        .collect();
    let movable: Vec<bool> = spec
        .nodes
        .iter()
        .map(|node| !node_is_pinned(node))
        .collect();
    resolve_layout_collisions(spec, &resolved, &mut positions, &movable, view_zoom);
    Ok(spec
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.id.clone(), positions[idx]))
        .collect())
}

#[cfg(test)]
pub(super) fn compute_layout(
    spec: &NetworkPlotSpec,
) -> Result<BTreeMap<String, (f64, f64)>, PlotError> {
    compute_layout_with_zoom(spec, 1.0)
}

pub(super) fn compute_layout_from_previous_with_zoom(
    previous_layout: &BTreeMap<String, (f64, f64)>,
    previous_spec: &NetworkPlotSpec,
    spec: &NetworkPlotSpec,
    view_zoom: f64,
) -> Result<BTreeMap<String, (f64, f64)>, PlotError> {
    validate_explicit_positions(spec)?;
    let resolved = resolve_nodes(spec)?;
    let previous_hierarchy = build_hierarchical_layout(previous_spec);
    if let Some(hierarchy) = build_hierarchical_layout(spec) {
        return Ok(compute_radial_layout_with_zoom(
            spec,
            &resolved,
            &hierarchy,
            view_zoom,
            Some(previous_spec),
            previous_hierarchy.as_ref(),
            Some(previous_layout),
        ));
    }

    let previous_adjacency = adjacency_map(previous_spec);
    let next_adjacency = adjacency_map(spec);
    let removed_neighbor_ids: HashSet<String> = spec
        .nodes
        .iter()
        .filter_map(|node| {
            let previous_neighbors = previous_adjacency.get(&node.id)?;
            let next_neighbors = next_adjacency.get(&node.id).cloned().unwrap_or_default();
            previous_neighbors
                .difference(&next_neighbors)
                .next()
                .map(|_| node.id.clone())
        })
        .collect();
    if !removed_neighbor_ids.is_empty() {
        return compute_layout_with_zoom(spec, view_zoom);
    }

    let mut next_layout = BTreeMap::new();
    let mut changed_ids = HashSet::new();
    for node in &spec.nodes {
        let previous_neighbors = previous_adjacency.get(&node.id);
        let next_neighbors = next_adjacency.get(&node.id);
        let neighbors_changed = previous_neighbors != next_neighbors;
        let position = if node_is_pinned(node) {
            let position = (node.x.unwrap(), node.y.unwrap());
            if previous_layout.get(&node.id).copied() != Some(position) {
                changed_ids.insert(node.id.clone());
            }
            position
        } else if let Some(previous) = previous_layout.get(&node.id).copied() {
            if neighbors_changed {
                changed_ids.insert(node.id.clone());
            }
            previous
        } else {
            changed_ids.insert(node.id.clone());
            seed_position(spec, &node.id, &next_layout)
        };
        next_layout.insert(node.id.clone(), position);
    }

    if changed_ids.is_empty() {
        return Ok(next_layout);
    }

    let mut relaxed_ids = changed_ids.clone();
    for node_id in &changed_ids {
        for neighbor_id in neighbor_ids(spec, node_id) {
            relaxed_ids.insert(neighbor_id);
        }
    }

    let mut positions: Vec<(f64, f64)> = spec
        .nodes
        .iter()
        .map(|node| {
            next_layout
                .get(&node.id)
                .copied()
                .expect("layout seeded for every node")
        })
        .collect();
    let movable: Vec<bool> = spec
        .nodes
        .iter()
        .map(|node| !node_is_pinned(node) && relaxed_ids.contains(node.id.as_str()))
        .collect();
    let iterations = spec
        .layout_iterations
        .max(1)
        .min(LOCAL_RELAXATION_MAX_ITERATIONS as u32) as usize;
    relax_positions(spec, &mut positions, &movable, iterations);
    resolve_layout_collisions(spec, &resolved, &mut positions, &movable, view_zoom);

    Ok(spec
        .nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.id.clone(), positions[idx]))
        .collect())
}
