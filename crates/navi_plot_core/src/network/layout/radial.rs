use super::*;

pub(in crate::network) fn order_children_by_sibling_edges(
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

pub(in crate::network) fn build_hierarchical_layout(
    spec: &NetworkPlotSpec,
) -> Option<HierarchicalLayout> {
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

pub(in crate::network) fn radial_origin(
    spec: &NetworkPlotSpec,
    hierarchy: &HierarchicalLayout,
) -> (f64, f64) {
    let root = &spec.nodes[hierarchy.root_idx];
    (root.x.unwrap_or(0.0), root.y.unwrap_or(0.0))
}

pub(in crate::network) fn polar_to_cartesian(
    origin: (f64, f64),
    radius: f64,
    angle: f64,
) -> (f64, f64) {
    (
        origin.0 + radius * angle.cos(),
        origin.1 + radius * angle.sin(),
    )
}

pub(in crate::network) fn unwrap_angle_near(angle: f64, reference: f64) -> f64 {
    let mut unwrapped = angle;
    while unwrapped - reference > PI {
        unwrapped -= TAU;
    }
    while unwrapped - reference < -PI {
        unwrapped += TAU;
    }
    unwrapped
}

pub(in crate::network) fn polar_from_position_unwrapped(
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

pub(in crate::network) fn radial_ring_spacing(
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

pub(in crate::network) fn child_intervals(
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

pub(in crate::network) fn root_child_anchor_angles(
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

pub(in crate::network) fn radial_targets(
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

pub(in crate::network) fn constrain_radial_positions(
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

pub(in crate::network) fn relax_radial_positions(
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

pub(in crate::network) fn compute_radial_layout_with_zoom(
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
