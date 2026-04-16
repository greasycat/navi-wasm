use super::*;

pub(in crate::network) fn node_label_box(
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

pub(in crate::network) fn node_footprints(
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

pub(in crate::network) fn footprint_bounds(footprint: NodeFootprint) -> LabelBox {
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

pub(in crate::network) fn collision_candidate_pairs(
    footprints: &[NodeFootprint],
) -> Vec<(usize, usize)> {
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

pub(in crate::network) fn circle_separation(
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

pub(in crate::network) fn label_box_separation(
    source: LabelBox,
    target: LabelBox,
) -> Option<(f64, f64)> {
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

pub(in crate::network) fn circle_label_separation(
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

pub(in crate::network) fn apply_pair_separation(
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

pub(in crate::network) fn resolve_layout_collisions(
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
