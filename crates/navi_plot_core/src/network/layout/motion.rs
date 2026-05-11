use super::*;
use crate::types::NetworkMotionMode;

const MOTION_HASH_OFFSET: u64 = 14_695_981_039_346_656_037;
const MOTION_HASH_PRIME: u64 = 1_099_511_628_211;
const MOTION_UNIT_MASK: u64 = (1 << 24) - 1;
const MIN_ORBIT_RADIUS_SCALE: f64 = 0.72;
const ORBIT_RADIUS_SCALE_RANGE: f64 = 0.28;
const MIN_ORBIT_ELLIPSE_SCALE: f64 = 0.62;
const ORBIT_ELLIPSE_SCALE_RANGE: f64 = 0.32;
const MIN_ORBIT_SPEED_SCALE: f64 = 0.72;
const ORBIT_SPEED_SCALE_RANGE: f64 = 0.56;
const MIN_DRIFT_RADIUS_SCALE: f64 = 0.62;
const DRIFT_RADIUS_SCALE_RANGE: f64 = 0.38;
const MIN_DRIFT_SECONDARY_SPEED: f64 = 0.31;
const DRIFT_SECONDARY_SPEED_RANGE: f64 = 0.36;
const MIN_BREATHE_RADIUS_SCALE: f64 = 0.46;
const BREATHE_RADIUS_SCALE_RANGE: f64 = 0.34;
const BREATHE_PULSE_SCALE: f64 = 0.5;
const BREATHE_BRANCH_ROTATION_SPEED_SCALE: f64 = 0.016;

pub(in crate::network) fn validate_motion(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
    let Some(motion) = spec.motion.as_ref() else {
        return Ok(());
    };
    if !motion.enabled {
        return Ok(());
    }
    if !motion.amplitude.is_finite() || motion.amplitude < 0.0 {
        return Err(PlotError::InvalidStyleValue {
            field: "motion.amplitude",
            value: motion.amplitude,
            reason: "must be finite and greater than or equal to zero",
        });
    }
    if !motion.speed.is_finite() || motion.speed < 0.0 {
        return Err(PlotError::InvalidStyleValue {
            field: "motion.speed",
            value: motion.speed,
            reason: "must be finite and greater than or equal to zero",
        });
    }
    Ok(())
}

pub(in crate::network) fn animated_radial_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    hierarchy: Option<&HierarchicalLayout>,
    view: &ScreenTransform,
    time_seconds: f64,
) -> Option<BTreeMap<String, (f64, f64)>> {
    let motion = spec.motion.as_ref()?;
    if !motion.enabled || motion.amplitude <= 0.0 {
        return None;
    }
    let hierarchy = hierarchy?;
    let time_seconds = if time_seconds.is_finite() {
        time_seconds
    } else {
        0.0
    };
    let visual_amplitude = motion.amplitude.max(0.0) / view.zoom.max(0.001);
    if visual_amplitude <= 0.0 {
        return None;
    }

    let root_point = spec
        .nodes
        .get(hierarchy.root_idx)
        .and_then(|node| layout.get(&node.id).copied());

    if matches!(motion.mode, NetworkMotionMode::Breathe) {
        return Some(animated_breathe_layout(
            spec,
            layout,
            hierarchy,
            root_point,
            visual_amplitude,
            motion.seed,
            motion.speed,
            time_seconds,
        ));
    }

    let mut animated = layout.clone();
    for (idx, node) in spec.nodes.iter().enumerate() {
        if idx == hierarchy.root_idx {
            continue;
        }
        let Some(point) = animated.get_mut(&node.id) else {
            continue;
        };
        let (dx, dy) = match motion.mode {
            NetworkMotionMode::Orbital => orbital_motion_offset(
                &node.id,
                motion.seed,
                visual_amplitude,
                motion.speed,
                time_seconds,
            ),
            NetworkMotionMode::Drift => drift_motion_offset(
                &node.id,
                motion.seed,
                visual_amplitude,
                motion.speed,
                time_seconds,
            ),
            NetworkMotionMode::Breathe => unreachable!("breathe motion is handled earlier"),
        };
        point.0 += dx;
        point.1 += dy;
    }
    Some(animated)
}

fn animated_breathe_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    hierarchy: &HierarchicalLayout,
    root_point: Option<(f64, f64)>,
    amplitude: f64,
    seed: u64,
    speed: f64,
    time_seconds: f64,
) -> BTreeMap<String, (f64, f64)> {
    let Some(root_point) = root_point else {
        return layout.clone();
    };
    let mut animated = layout.clone();
    for (idx, node) in spec.nodes.iter().enumerate() {
        if idx == hierarchy.root_idx {
            continue;
        }
        let Some(base_point) = layout.get(&node.id).copied() else {
            continue;
        };
        let Some(point) = animated.get_mut(&node.id) else {
            continue;
        };
        if first_level_branch_idx(hierarchy, idx).is_none() {
            continue;
        }
        let rotation_angle = breathe_branch_rotation_angle(seed, speed, time_seconds);
        let rotated_point = rotate_around(root_point, base_point, rotation_angle);
        let (dx, dy) = if hierarchy.depth_by_idx[idx] <= 1 {
            (0.0, 0.0)
        } else {
            breathe_motion_offset(
                &node.id,
                seed,
                amplitude,
                speed,
                time_seconds,
                rotated_point,
                Some(root_point),
            )
        };
        point.0 = rotated_point.0 + dx;
        point.1 = rotated_point.1 + dy;
    }
    animated
}

fn first_level_branch_idx(hierarchy: &HierarchicalLayout, node_idx: usize) -> Option<usize> {
    let mut current = node_idx;
    loop {
        let parent = hierarchy.parent_by_idx[current]?;
        if parent == hierarchy.root_idx {
            return Some(current);
        }
        current = parent;
    }
}

fn breathe_branch_rotation_angle(seed: u64, speed: f64, time_seconds: f64) -> f64 {
    let direction_seed = motion_hash("breathe:rotation-direction", seed);
    let direction = if direction_seed & 1 == 0 { 1.0 } else { -1.0 };
    time_seconds * speed * BREATHE_BRANCH_ROTATION_SPEED_SCALE * TAU * direction
}

fn rotate_around(origin: (f64, f64), point: (f64, f64), angle: f64) -> (f64, f64) {
    if angle.abs() <= f64::EPSILON {
        return point;
    }
    let dx = point.0 - origin.0;
    let dy = point.1 - origin.1;
    let cos = angle.cos();
    let sin = angle.sin();
    (
        origin.0 + dx * cos - dy * sin,
        origin.1 + dx * sin + dy * cos,
    )
}

fn orbital_motion_offset(
    node_id: &str,
    seed: u64,
    amplitude: f64,
    speed: f64,
    time_seconds: f64,
) -> (f64, f64) {
    let hash = motion_hash(node_id, seed);
    let base_phase = unit_from_hash(hash) * TAU;
    let speed_scale =
        MIN_ORBIT_SPEED_SCALE + unit_from_hash(hash.rotate_left(7)) * ORBIT_SPEED_SCALE_RANGE;
    let direction = if hash & (1 << 63) == 0 { 1.0 } else { -1.0 };
    let phase = base_phase + time_seconds * speed * speed_scale * TAU * direction;
    let radius_scale =
        MIN_ORBIT_RADIUS_SCALE + unit_from_hash(hash.rotate_left(17)) * ORBIT_RADIUS_SCALE_RANGE;
    let ellipse_scale =
        MIN_ORBIT_ELLIPSE_SCALE + unit_from_hash(hash.rotate_left(31)) * ORBIT_ELLIPSE_SCALE_RANGE;
    let tilt = unit_from_hash(hash.rotate_left(43)) * TAU;
    let local_x = phase.cos() * amplitude * radius_scale;
    let local_y = phase.sin() * amplitude * radius_scale * ellipse_scale;
    (
        local_x * tilt.cos() - local_y * tilt.sin(),
        local_x * tilt.sin() + local_y * tilt.cos(),
    )
}

fn drift_motion_offset(
    node_id: &str,
    seed: u64,
    amplitude: f64,
    speed: f64,
    time_seconds: f64,
) -> (f64, f64) {
    let hash = motion_hash(node_id, seed);
    let base_phase = unit_from_hash(hash) * TAU;
    let secondary_phase = unit_from_hash(hash.rotate_left(11)) * TAU;
    let speed_scale =
        MIN_ORBIT_SPEED_SCALE + unit_from_hash(hash.rotate_left(7)) * ORBIT_SPEED_SCALE_RANGE;
    let secondary_speed = MIN_DRIFT_SECONDARY_SPEED
        + unit_from_hash(hash.rotate_left(23)) * DRIFT_SECONDARY_SPEED_RANGE;
    let radius_scale =
        MIN_DRIFT_RADIUS_SCALE + unit_from_hash(hash.rotate_left(17)) * DRIFT_RADIUS_SCALE_RANGE;
    let tilt = unit_from_hash(hash.rotate_left(43)) * TAU;
    let phase = base_phase + time_seconds * speed * speed_scale * TAU;
    let secondary = secondary_phase + time_seconds * speed * secondary_speed * TAU;
    let local_x = (phase.sin() * 0.74 + secondary.sin() * 0.26) * amplitude * radius_scale;
    let local_y = (phase.cos() * 0.58 + (secondary * 1.31).sin() * 0.42) * amplitude * radius_scale;
    (
        local_x * tilt.cos() - local_y * tilt.sin(),
        local_x * tilt.sin() + local_y * tilt.cos(),
    )
}

fn breathe_motion_offset(
    node_id: &str,
    seed: u64,
    amplitude: f64,
    speed: f64,
    time_seconds: f64,
    node_point: (f64, f64),
    root_point: Option<(f64, f64)>,
) -> (f64, f64) {
    let hash = motion_hash(node_id, seed);
    let base_phase = unit_from_hash(hash) * TAU;
    let speed_scale = 0.76 + unit_from_hash(hash.rotate_left(7)) * 0.18;
    let radius_scale = MIN_BREATHE_RADIUS_SCALE
        + unit_from_hash(hash.rotate_left(17)) * BREATHE_RADIUS_SCALE_RANGE;
    let (ux, uy) = root_point
        .and_then(|root| {
            let dx = node_point.0 - root.0;
            let dy = node_point.1 - root.1;
            let length = (dx * dx + dy * dy).sqrt();
            (length > 0.01).then_some((dx / length, dy / length))
        })
        .unwrap_or_else(|| deterministic_direction(hash));
    let phase = base_phase + time_seconds * speed * speed_scale * TAU;
    let amount = phase.sin() * amplitude * radius_scale * BREATHE_PULSE_SCALE;
    (ux * amount, uy * amount)
}

fn deterministic_direction(hash: u64) -> (f64, f64) {
    let angle = unit_from_hash(hash.rotate_left(43)) * TAU;
    (angle.cos(), angle.sin())
}

fn motion_hash(node_id: &str, seed: u64) -> u64 {
    let mut hash = MOTION_HASH_OFFSET ^ seed;
    for byte in node_id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(MOTION_HASH_PRIME);
    }
    hash
}

fn unit_from_hash(hash: u64) -> f64 {
    (hash & MOTION_UNIT_MASK) as f64 / MOTION_UNIT_MASK as f64
}
