use super::*;

pub(in crate::network) fn relax_positions(
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
        .map(|(index, node)| (node.id.as_str(), index))
        .collect();
    let adjacency: Vec<(usize, usize, f64)> = spec
        .edges
        .iter()
        .filter_map(|edge| {
            let source = id_to_idx.get(edge.source.as_str())?;
            let target = id_to_idx.get(edge.target.as_str())?;
            Some((*source, *target, edge_weight(edge)))
        })
        .collect();
    let world_span = estimate_world_span(spec, positions.iter().copied());
    let spring_length = ((world_span * world_span) / n as f64).sqrt().max(1.0);
    let mut temperature = (world_span * 0.12).max(WORLD_NODE_SPACING * 0.35);

    for _ in 0..iterations {
        let mut displacement = vec![(0.0, 0.0); n];

        for source_idx in 0..n {
            if !movable[source_idx] {
                continue;
            }
            for target_idx in 0..n {
                if source_idx == target_idx {
                    continue;
                }
                let dx = positions[source_idx].0 - positions[target_idx].0;
                let dy = positions[source_idx].1 - positions[target_idx].1;
                let distance = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = spring_length * spring_length / distance;
                displacement[source_idx].0 += dx / distance * force;
                displacement[source_idx].1 += dy / distance * force;
            }
        }

        for &(source_idx, target_idx, weight) in &adjacency {
            let dx = positions[target_idx].0 - positions[source_idx].0;
            let dy = positions[target_idx].1 - positions[source_idx].1;
            let distance = (dx * dx + dy * dy).sqrt().max(0.01);
            let force = (distance * distance / spring_length).max(0.01) * weight;
            let fx = dx / distance * force;
            let fy = dy / distance * force;
            if movable[source_idx] {
                displacement[source_idx].0 += fx;
                displacement[source_idx].1 += fy;
            }
            if movable[target_idx] {
                displacement[target_idx].0 -= fx;
                displacement[target_idx].1 -= fy;
            }
        }

        for idx in 0..n {
            if !movable[idx] {
                continue;
            }
            let displacement_len = (displacement[idx].0 * displacement[idx].0
                + displacement[idx].1 * displacement[idx].1)
                .sqrt()
                .max(0.01);
            let scale = displacement_len.min(temperature) / displacement_len;
            positions[idx].0 += displacement[idx].0 * scale;
            positions[idx].1 += displacement[idx].1 * scale;
        }

        temperature *= 0.92;
    }
}

pub(in crate::network) fn fr_layout(spec: &NetworkPlotSpec) -> BTreeMap<String, (f64, f64)> {
    let mut seeded = BTreeMap::new();
    for node in &spec.nodes {
        let position = if node_is_pinned(node) {
            (node.x.unwrap(), node.y.unwrap())
        } else {
            seed_position(spec, &node.id, &seeded)
        };
        seeded.insert(node.id.clone(), position);
    }

    let mut positions: Vec<(f64, f64)> = spec
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
        &mut positions,
        &movable,
        spec.layout_iterations.max(1) as usize,
    );

    spec.nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.id.clone(), positions[idx]))
        .collect()
}

pub(in crate::network) fn compute_layout_with_zoom(
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
pub(in crate::network) fn compute_layout(
    spec: &NetworkPlotSpec,
) -> Result<BTreeMap<String, (f64, f64)>, PlotError> {
    compute_layout_with_zoom(spec, 1.0)
}

pub(in crate::network) fn compute_layout_from_previous_with_zoom(
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
