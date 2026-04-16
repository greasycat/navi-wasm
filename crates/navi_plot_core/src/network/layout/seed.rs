use super::*;

pub(in crate::network) fn normalized_angle(angle: f64) -> f64 {
    angle.rem_euclid(TAU)
}

pub(in crate::network) fn angle_between(origin: (f64, f64), target: (f64, f64)) -> f64 {
    normalized_angle((target.1 - origin.1).atan2(target.0 - origin.0))
}

pub(in crate::network) fn layout_zoom(view_zoom: f64) -> f64 {
    view_zoom.max(MIN_LAYOUT_ZOOM)
}

pub(in crate::network) fn deterministic_unit(seed: &str) -> (f64, f64) {
    let (x, y) = deterministic_offset(seed, 1.0);
    let len = (x * x + y * y).sqrt().max(0.01);
    (x / len, y / len)
}

pub(in crate::network) fn candidate_seed_score(
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

pub(in crate::network) fn best_gap_candidate(
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

pub(in crate::network) fn seed_position_from_parent_gaps(
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

pub(in crate::network) fn seed_position(
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
    if !positions.is_empty() {
        let (sum_x, sum_y) = positions
            .values()
            .fold((0.0, 0.0), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));
        let base = (
            sum_x / positions.len() as f64,
            sum_y / positions.len() as f64,
        );
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
