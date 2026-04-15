use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::graph_style::{
    edge_line_segments, resolve_edge_style, resolve_node_style, resolve_selection_style,
    EdgeStyleContext, NodeStyleContext, ResolvedNodeStyle, ResolvedSelectionStyle,
};
use crate::node::{self, GraphNodeRenderInfo, ResolvedNodeMedia};
use crate::types::{NetworkFocusMode, NetworkFocusOptions, NetworkPlotSpec, NetworkView};
use crate::viewport::{ensure_finite, PixelBounds, ScreenTransform};
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use serde::{Deserialize, Serialize};

const DEFAULT_NODE_COLOR: RGBColor = RGBColor(59, 130, 246);
const DEFAULT_EDGE_COLOR: RGBColor = RGBColor(107, 114, 128);
const SELECTION_RING_PADDING: i32 = 5;
const ARROW_LENGTH: f64 = 12.0;
const ARROW_HALF_WIDTH: f64 = 5.0;
const WORLD_NODE_SPACING: f64 = 180.0;
const LOCAL_RELAXATION_MAX_ITERATIONS: usize = 60;
const COLLISION_RESOLUTION_MAX_ITERATIONS: usize = 24;
const ENABLE_LAYOUT_COLLISIONS: bool = false;
const COLLISION_GAP: f64 = 6.0;
const NODE_COLLISION_PADDING: f64 = 8.0;
const LABEL_COLLISION_PADDING: f64 = 4.0;
const LABEL_WIDTH_FACTOR: f64 = 0.58;
const LABEL_HEIGHT_FACTOR: f64 = 1.2;
const MIN_LAYOUT_ZOOM: f64 = 0.25;
const STRUCTURAL_EDGE_WEIGHT_THRESHOLD: f64 = 0.5;
const MIN_LAYOUT_EDGE_WEIGHT: f64 = 0.05;
const RADIAL_START_ANGLE: f64 = -FRAC_PI_2;
const RADIAL_RING_SPACING_SCALE: f64 = 1.35;
const RADIAL_RADIUS_SPRING: f64 = 0.24;
const RADIAL_ANGLE_SPRING: f64 = 0.16;
const RADIAL_PARENT_SPRING: f64 = 0.05;
const RADIAL_REPULSION_FORCE: f64 = 0.18;
const TOGGLEABLE_PROPERTY_KEY: &str = "navil_toggleable";
const EXPANDED_PROPERTY_KEY: &str = "navil_expanded";
const TOGGLE_BADGE_MIN_RADIUS: i32 = 5;
const TOGGLE_BADGE_MAX_RADIUS: i32 = 8;
const TOGGLE_BADGE_FILL: RGBColor = RGBColor(148, 163, 184);
const TOGGLE_BADGE_SYMBOL: RGBColor = RGBColor(255, 255, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPickKind {
    Node,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkPickHit {
    pub kind: NetworkPickKind,
    pub node_id: String,
}

#[derive(Debug, Clone)]
struct HierarchicalLayout {
    root_idx: usize,
    parent_by_idx: Vec<Option<usize>>,
    children_by_idx: Vec<Vec<usize>>,
    depth_by_idx: Vec<usize>,
    subtree_size_by_idx: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
struct RadialTarget {
    radius: f64,
    angle: f64,
    min_angle: f64,
    max_angle: f64,
}

#[derive(Debug, Clone, Copy)]
struct ToggleBadge {
    center_x: i32,
    center_y: i32,
    radius: i32,
    expanded: bool,
}

#[derive(Debug, Clone)]
struct ResolvedNode {
    label: String,
    style: ResolvedNodeStyle,
    media: Option<ResolvedNodeMedia>,
}

#[derive(Debug, Clone)]
struct NetworkTransition {
    from_spec: NetworkPlotSpec,
    from_layout: BTreeMap<String, (f64, f64)>,
    from_resolved: BTreeMap<String, ResolvedNode>,
    from_selected_node_id: Option<String>,
    anchor_node_id: String,
}

#[derive(Debug, Clone, Copy)]
struct LabelBox {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl LabelBox {
    fn center(self) -> (f64, f64) {
        (
            (self.left + self.right) * 0.5,
            (self.top + self.bottom) * 0.5,
        )
    }

    fn overlaps(self, other: Self) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
    }

    fn overlap_amount(self, other: Self) -> Option<(f64, f64)> {
        if !self.overlaps(other) {
            return None;
        }
        Some((
            self.right.min(other.right) - self.left.max(other.left),
            self.bottom.min(other.bottom) - self.top.max(other.top),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
struct NodeFootprint {
    center: (f64, f64),
    radius: f64,
    label: Option<LabelBox>,
}

fn validate(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
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

fn resolve_nodes(spec: &NetworkPlotSpec) -> Result<BTreeMap<String, ResolvedNode>, PlotError> {
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

fn validate_explicit_positions(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
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

fn node_is_pinned(node: &crate::NetworkNode) -> bool {
    node.x.is_some() && node.y.is_some()
}

fn deterministic_offset(node_id: &str, radius: f64) -> (f64, f64) {
    let hash = node_id.bytes().fold(0u64, |acc, byte| {
        acc.wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(byte) + 1)
    });
    let angle = ((hash % 3600) as f64 / 3600.0) * TAU;
    let scaled_radius = radius * (0.7 + (((hash / 3600) % 5) as f64) * 0.12);
    (scaled_radius * angle.cos(), scaled_radius * angle.sin())
}

fn edge_weight(edge: &crate::NetworkEdge) -> f64 {
    edge.weight.unwrap_or(1.0).max(MIN_LAYOUT_EDGE_WEIGHT)
}

fn edge_is_structural(edge: &crate::NetworkEdge) -> bool {
    edge.weight.unwrap_or(1.0) >= STRUCTURAL_EDGE_WEIGHT_THRESHOLD
}

fn neighbor_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
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

fn adjacency_map(spec: &NetworkPlotSpec) -> BTreeMap<String, BTreeSet<String>> {
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

fn topology_changed(previous_spec: &NetworkPlotSpec, next_spec: &NetworkPlotSpec) -> bool {
    topology_identity(previous_spec) != topology_identity(next_spec)
}

fn node_property_is_true(node: &crate::NetworkNode, key: &str) -> bool {
    node.properties.get(key).is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes"
        )
    })
}

fn node_has_toggle_badge(node: &crate::NetworkNode) -> bool {
    node_property_is_true(node, TOGGLEABLE_PROPERTY_KEY)
}

fn node_badge_expanded(node: &crate::NetworkNode) -> bool {
    node_property_is_true(node, EXPANDED_PROPERTY_KEY)
}

fn structural_parent_map(spec: &NetworkPlotSpec) -> HashMap<&str, &str> {
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

fn nearest_shared_ancestor(
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

fn structural_depths(spec: &NetworkPlotSpec) -> HashMap<String, usize> {
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

fn transition_anchor_distance(
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

fn choose_transition_anchor(
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
                        transition_anchor_distance(
                            left,
                            selected_id,
                            previous_layout,
                            next_layout,
                        )
                    })
                    .unwrap_or(f64::INFINITY);
                let right_distance = selected_id
                    .map(|selected_id| {
                        transition_anchor_distance(
                            right,
                            selected_id,
                            previous_layout,
                            next_layout,
                        )
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

fn estimate_world_span(spec: &NetworkPlotSpec, positions: impl Iterator<Item = (f64, f64)>) -> f64 {
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

fn seed_position(
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

fn parent_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    spec.edges
        .iter()
        .filter(|edge| edge_is_structural(edge) && edge.target == node_id)
        .map(|edge| edge.source.clone())
        .collect()
}

fn child_ids(spec: &NetworkPlotSpec, node_id: &str) -> Vec<String> {
    spec.edges
        .iter()
        .filter(|edge| edge_is_structural(edge) && edge.source == node_id)
        .map(|edge| edge.target.clone())
        .collect()
}

fn normalized_angle(angle: f64) -> f64 {
    angle.rem_euclid(TAU)
}

fn angle_between(origin: (f64, f64), target: (f64, f64)) -> f64 {
    normalized_angle((target.1 - origin.1).atan2(target.0 - origin.0))
}

fn candidate_seed_score(candidate: (f64, f64), positions: &BTreeMap<String, (f64, f64)>) -> f64 {
    positions
        .values()
        .map(|&(x, y)| {
            let dx = candidate.0 - x;
            let dy = candidate.1 - y;
            dx * dx + dy * dy
        })
        .fold(f64::INFINITY, f64::min)
}

fn best_gap_candidate(
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

fn seed_position_from_parent_gaps(
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

fn layout_zoom(view_zoom: f64) -> f64 {
    view_zoom.max(MIN_LAYOUT_ZOOM)
}

fn deterministic_unit(seed: &str) -> (f64, f64) {
    let (x, y) = deterministic_offset(seed, 1.0);
    let len = (x * x + y * y).sqrt().max(0.01);
    (x / len, y / len)
}

fn order_children_by_sibling_edges(
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

fn build_hierarchical_layout(spec: &NetworkPlotSpec) -> Option<HierarchicalLayout> {
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

fn radial_origin(spec: &NetworkPlotSpec, hierarchy: &HierarchicalLayout) -> (f64, f64) {
    let root = &spec.nodes[hierarchy.root_idx];
    (root.x.unwrap_or(0.0), root.y.unwrap_or(0.0))
}

fn polar_to_cartesian(origin: (f64, f64), radius: f64, angle: f64) -> (f64, f64) {
    (
        origin.0 + radius * angle.cos(),
        origin.1 + radius * angle.sin(),
    )
}

fn unwrap_angle_near(angle: f64, reference: f64) -> f64 {
    let mut unwrapped = angle;
    while unwrapped - reference > PI {
        unwrapped -= TAU;
    }
    while unwrapped - reference < -PI {
        unwrapped += TAU;
    }
    unwrapped
}

fn polar_from_position_unwrapped(
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

fn radial_ring_spacing(
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

fn child_intervals(
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

fn root_child_anchor_angles(
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

fn radial_targets(
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

fn constrain_radial_positions(
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

fn relax_radial_positions(
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

fn compute_radial_layout_with_zoom(
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

fn node_label_box(
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

fn node_footprints(
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

fn footprint_bounds(footprint: NodeFootprint) -> LabelBox {
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

fn collision_candidate_pairs(footprints: &[NodeFootprint]) -> Vec<(usize, usize)> {
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

fn circle_separation(
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

fn label_box_separation(source: LabelBox, target: LabelBox) -> Option<(f64, f64)> {
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

fn circle_label_separation(circle: (f64, f64), radius: f64, label: LabelBox) -> Option<(f64, f64)> {
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

fn apply_pair_separation(
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

fn resolve_layout_collisions(
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

fn relax_positions(
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

fn fr_layout(spec: &NetworkPlotSpec) -> BTreeMap<String, (f64, f64)> {
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

fn compute_layout_with_zoom(
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
fn compute_layout(spec: &NetworkPlotSpec) -> Result<BTreeMap<String, (f64, f64)>, PlotError> {
    compute_layout_with_zoom(spec, 1.0)
}

fn compute_layout_from_previous_with_zoom(
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

#[derive(Debug, Clone)]
pub struct NetworkSession {
    spec: NetworkPlotSpec,
    layout: BTreeMap<String, (f64, f64)>,
    resolved: BTreeMap<String, ResolvedNode>,
    selection_style: ResolvedSelectionStyle,
    view: ScreenTransform,
    transition: Option<NetworkTransition>,
}

impl NetworkSession {
    pub fn new(spec: NetworkPlotSpec) -> Result<Self, PlotError> {
        validate(&spec)?;
        let resolved = resolve_nodes(&spec)?;
        let layout = compute_layout_with_zoom(&spec, 1.0)?;
        let selection_style =
            resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
        let view = ScreenTransform::new(spec.offset_x as f64, spec.offset_y as f64);
        Ok(Self {
            spec,
            layout,
            resolved,
            selection_style,
            view,
            transition: None,
        })
    }

    pub fn update_spec(&mut self, spec: NetworkPlotSpec) -> Result<(), PlotError> {
        validate(&spec)?;
        let topology_changed = topology_changed(&self.spec, &spec);
        let previous_spec = self.spec.clone();
        let previous_layout = self.layout.clone();
        let previous_resolved = self.resolved.clone();
        let previous_selected_node_id = self.spec.selected_node_id.clone();
        let resolved = resolve_nodes(&spec)?;
        let layout = compute_layout_from_previous_with_zoom(
            &self.layout,
            &self.spec,
            &spec,
            self.view.zoom,
        )?;
        let selection_style =
            resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
        let view = self.view;
        let anchor_node_id = topology_changed
            .then(|| choose_transition_anchor(&previous_spec, &spec, &previous_layout, &layout));
        self.spec = spec;
        self.layout = layout;
        self.resolved = resolved;
        self.selection_style = selection_style;
        self.view = view;
        self.transition = topology_changed.then(|| NetworkTransition {
            from_spec: previous_spec,
            from_layout: previous_layout,
            from_resolved: previous_resolved,
            from_selected_node_id: previous_selected_node_id,
            anchor_node_id: anchor_node_id.expect("anchor present for topology transition"),
        });
        self.spec.selected_node_id = self
            .spec
            .selected_node_id
            .clone()
            .filter(|node_id| self.layout.contains_key(node_id.as_str()));
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        render_with_layout(
            &root,
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
        )
    }

    pub fn render_transition_on<DB>(
        &self,
        root: PlotArea<DB>,
        progress: f64,
    ) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_on(root);
        };
        render_transition_with_layout(
            &root,
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            transition,
            progress,
        )
    }

    pub fn pick(&self, canvas_x: f64, canvas_y: f64) -> Option<NetworkPickHit> {
        pick_hit_from_layout(
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            canvas_x,
            canvas_y,
        )
    }

    pub fn pick_node(&self, canvas_x: f64, canvas_y: f64) -> Option<String> {
        self.pick(canvas_x, canvas_y)
            .and_then(|hit| (hit.kind == NetworkPickKind::Node).then_some(hit.node_id))
    }

    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        self.view.pan_by(delta_x, delta_y);
        self.sync_view_to_spec();
    }

    pub fn zoom_at(&mut self, canvas_x: f64, canvas_y: f64, factor: f64) -> Result<(), PlotError> {
        self.view.zoom_at(canvas_x, canvas_y, factor)?;
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn set_selection(&mut self, node_id: Option<String>) {
        self.spec.selected_node_id = node_id.filter(|id| self.layout.contains_key(id.as_str()));
    }

    pub fn view(&self) -> NetworkView {
        NetworkView {
            zoom: self.view.zoom,
            translate_x: self.view.translate_x,
            translate_y: self.view.translate_y,
        }
    }

    pub fn set_view(&mut self, view: NetworkView) -> Result<(), PlotError> {
        self.view = ScreenTransform::with_view(view.zoom, view.translate_x, view.translate_y)?;
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn compute_focus_view(
        &self,
        node_id: &str,
        options: Option<NetworkFocusOptions>,
    ) -> Option<NetworkView> {
        let center = self.layout.get(node_id).copied()?;
        let options = options.unwrap_or_default();
        let padding = options.padding.max(0.0);
        let min_world_span = options.min_world_span.max(1.0);
        let mut focused_ids = HashSet::from([node_id.to_string()]);
        match options.mode {
            NetworkFocusMode::NodeAndNeighbors => {
                for neighbor_id in neighbor_ids(&self.spec, node_id) {
                    focused_ids.insert(neighbor_id);
                }
            }
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for focused_id in &focused_ids {
            let Some(&(x, y)) = self.layout.get(focused_id) else {
                continue;
            };
            let radius = self
                .resolved
                .get(focused_id)
                .map(|node| node.style.radius.max(1) as f64)
                .unwrap_or(self.spec.node_radius.max(1) as f64);
            min_x = min_x.min(x - radius);
            min_y = min_y.min(y - radius);
            max_x = max_x.max(x + radius);
            max_y = max_y.max(y + radius);
        }

        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            min_x = center.0 - min_world_span / 2.0;
            max_x = center.0 + min_world_span / 2.0;
            min_y = center.1 - min_world_span / 2.0;
            max_y = center.1 + min_world_span / 2.0;
        }

        let span_x = (max_x - min_x).max(min_world_span);
        let span_y = (max_y - min_y).max(min_world_span);
        let available_width = (self.spec.width as f64 - padding * 2.0).max(1.0);
        let available_height = (self.spec.height as f64 - padding * 2.0).max(1.0);
        let zoom = (available_width / span_x).min(available_height / span_y);
        let view = NetworkView {
            zoom,
            translate_x: self.spec.width as f64 / 2.0 - ((min_x + max_x) / 2.0) * zoom,
            translate_y: self.spec.height as f64 / 2.0 - ((min_y + max_y) / 2.0) * zoom,
        };
        ScreenTransform::with_view(view.zoom, view.translate_x, view.translate_y)
            .ok()
            .map(|clamped| NetworkView {
                zoom: clamped.zoom,
                translate_x: clamped.translate_x,
                translate_y: clamped.translate_y,
            })
    }

    pub fn spec(&self) -> &NetworkPlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> NetworkPlotSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }

    pub fn render_nodes(&self) -> Vec<GraphNodeRenderInfo> {
        render_nodes_with_layout(
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
        )
    }

    pub fn render_transition_nodes(&self, progress: f64) -> Vec<GraphNodeRenderInfo> {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_nodes();
        };
        render_transition_nodes_with_layout(
            &self.spec,
            &self.layout,
            &self.resolved,
            &self.selection_style,
            &self.view,
            transition,
            progress,
        )
    }

    pub fn has_transition(&self) -> bool {
        self.transition.is_some()
    }

    pub fn clear_transition(&mut self) {
        self.transition = None;
    }

    fn sync_view_to_spec(&mut self) {
        self.spec.offset_x = self.view.translate_x.round() as i32;
        self.spec.offset_y = self.view.translate_y.round() as i32;
    }
}

fn outward_unit_from_parent(
    spec: &NetworkPlotSpec,
    node_id: &str,
    node_point: (f64, f64),
    parent_point: Option<(f64, f64)>,
) -> (f64, f64) {
    let Some((parent_x, parent_y)) = parent_point else {
        return deterministic_unit(&format!("toggle:{node_id}:{}", spec.title));
    };
    let dx = node_point.0 - parent_x;
    let dy = node_point.1 - parent_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.01 {
        (dx / len, dy / len)
    } else {
        deterministic_unit(&format!("toggle:{node_id}:{}", spec.title))
    }
}

fn toggle_badge_for_node_frame(
    spec: &NetworkPlotSpec,
    node_spec: &crate::NetworkNode,
    resolved_node: &ResolvedNode,
    node_point: (f64, f64),
    parent_point: Option<(f64, f64)>,
    view: &ScreenTransform,
) -> Option<ToggleBadge> {
    if !node_has_toggle_badge(node_spec) {
        return None;
    }
    let (cx, cy) = view.apply(node_point);
    let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
    let badge_radius = (((scaled_style.radius.max(1) as f64) * 0.28).round() as i32)
        .clamp(TOGGLE_BADGE_MIN_RADIUS, TOGGLE_BADGE_MAX_RADIUS);
    let center_offset = (scaled_style.radius + badge_radius).max(badge_radius + 1) as f64;
    let (ux, uy) = outward_unit_from_parent(spec, &node_spec.id, node_point, parent_point);

    Some(ToggleBadge {
        center_x: (f64::from(cx) + ux * center_offset).round() as i32,
        center_y: (f64::from(cy) + uy * center_offset).round() as i32,
        radius: badge_radius,
        expanded: node_badge_expanded(node_spec),
    })
}

fn toggle_badge_for_node(
    spec: &NetworkPlotSpec,
    node_spec: &crate::NetworkNode,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    view: &ScreenTransform,
    parent_by_id: &HashMap<&str, &str>,
) -> Option<ToggleBadge> {
    let resolved_node = resolved.get(&node_spec.id)?;
    let node_point = layout.get(&node_spec.id).copied()?;
    let parent_point = parent_by_id
        .get(node_spec.id.as_str())
        .and_then(|parent_id| layout.get(*parent_id).copied());
    toggle_badge_for_node_frame(spec, node_spec, resolved_node, node_point, parent_point, view)
}

fn draw_toggle_badge<DB>(
    root: &PlotArea<DB>,
    badge: ToggleBadge,
    opacity: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let alpha = opacity.clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return Ok(());
    }
    node::draw_shape(
        root,
        badge.center_x,
        badge.center_y,
        badge.radius,
        &crate::NodeShape::Circle,
        ShapeStyle::from(&TOGGLE_BADGE_FILL.mix(0.96 * alpha)).filled(),
    )?;

    let symbol_half = (badge.radius as f64 * 0.55).round().clamp(2.0, 5.0) as i32;
    let symbol_width = ((badge.radius as f64) * 0.35).round().clamp(1.0, 2.0) as u32;
    let symbol_style = ShapeStyle::from(&TOGGLE_BADGE_SYMBOL.mix(alpha)).stroke_width(symbol_width);
    root.draw(&PathElement::new(
        vec![
            (badge.center_x - symbol_half, badge.center_y),
            (badge.center_x + symbol_half, badge.center_y),
        ],
        symbol_style,
    ))
    .map_err(backend_error)?;

    if !badge.expanded {
        root.draw(&PathElement::new(
            vec![
                (badge.center_x, badge.center_y - symbol_half),
                (badge.center_x, badge.center_y + symbol_half),
            ],
            symbol_style,
        ))
        .map_err(backend_error)?;
    }

    Ok(())
}

fn render_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    root.fill(&WHITE).map_err(backend_error)?;
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    let parent_by_id = structural_parent_map(spec);

    // Draw title
    if !spec.title.is_empty() {
        let title_size = (20.0 * spec.pixel_ratio.max(0.25)).round() as u32;
        root.draw(&Text::new(
            spec.title.clone(),
            (
                spec.width as i32 / 2 - spec.title.len() as i32 * 7,
                spec.margin as i32 / 2,
            ),
            ("sans-serif", title_size).into_font(),
        ))
        .map_err(backend_error)?;
    }

    // Draw edges first
    for edge in &spec.edges {
        let Some(&src_pos) = layout.get(&edge.source) else {
            continue;
        };
        let Some(&tgt_pos) = layout.get(&edge.target) else {
            continue;
        };
        let edge_style = resolve_network_edge_style(spec, edge)?;
        let target_radius = resolved
            .get(&edge.target)
            .map(|node| scale_node_style(&node.style, view.zoom).radius)
            .unwrap_or(spec.node_radius.max(1) as i32);
        let (sx, sy) = view.apply(src_pos);
        let (tx, ty) = view.apply(tgt_pos);
        let Some(((line_start_x, line_start_y), (line_end_x, line_end_y))) =
            viewport.clip_line((sx, sy), (tx, ty))
        else {
            continue;
        };

        if edge_style.stroke_width > 0 {
            let shape_style = ShapeStyle::from(&edge_style.stroke_color.mix(edge_style.opacity))
                .stroke_width(edge_style.stroke_width);
            for (segment_start, segment_end) in edge_line_segments(
                (line_start_x, line_start_y),
                (line_end_x, line_end_y),
                edge_style.dash_pattern.as_deref(),
            ) {
                root.draw(&PathElement::new(
                    vec![segment_start, segment_end],
                    shape_style,
                ))
                .map_err(backend_error)?;
            }
        }

        // Arrowhead
        if edge_style.arrow_visible
            && edge_style.stroke_width > 0
            && ((line_start_x, line_start_y) != (line_end_x, line_end_y))
        {
            let dx = line_end_x as f64 - line_start_x as f64;
            let dy = line_end_y as f64 - line_start_y as f64;
            let len = (dx * dx + dy * dy).sqrt().max(0.01);
            let ux = dx / len;
            let uy = dy / len;
            let perp_x = -uy;
            let perp_y = ux;
            let target_visible = resolved
                .get(&edge.target)
                .map(|node| {
                    let scaled_style = scale_node_style(&node.style, view.zoom);
                    viewport.intersects_circle((tx, ty), scaled_style.radius.max(1))
                })
                .unwrap_or(false);

            let mut tip_x = line_end_x;
            let mut tip_y = line_end_y;
            if target_visible {
                let candidate = (
                    (tx as f64 - ux * target_radius as f64).round() as i32,
                    (ty as f64 - uy * target_radius as f64).round() as i32,
                );
                if viewport.contains(candidate) {
                    tip_x = candidate.0;
                    tip_y = candidate.1;
                }
            }
            let b1_x =
                (tip_x as f64 - ux * ARROW_LENGTH + perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b1_y =
                (tip_y as f64 - uy * ARROW_LENGTH + perp_y * ARROW_HALF_WIDTH).round() as i32;
            let b2_x =
                (tip_x as f64 - ux * ARROW_LENGTH - perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b2_y =
                (tip_y as f64 - uy * ARROW_LENGTH - perp_y * ARROW_HALF_WIDTH).round() as i32;

            root.draw(&Polygon::new(
                vec![(tip_x, tip_y), (b1_x, b1_y), (b2_x, b2_y)],
                edge_style.stroke_color.mix(edge_style.opacity).filled(),
            ))
            .map_err(backend_error)?;
        }

        if edge_style.label_visible {
            if let Some(label) = edge.label.as_deref().filter(|label| !label.is_empty()) {
                let label_size = (12.0 * spec.pixel_ratio.max(0.25)).round() as u32;
                let label_color = edge_style.label_color.unwrap_or(edge_style.stroke_color);
                let text_color = label_color.mix(edge_style.opacity);
                let text_style = TextStyle::from(("sans-serif", label_size).into_font())
                    .pos(Pos::new(HPos::Center, VPos::Bottom))
                    .color(&text_color);
                let mid_x = ((line_start_x + line_end_x) as f64 / 2.0).round() as i32;
                let mid_y = ((line_start_y + line_end_y) as f64 / 2.0).round() as i32 - 4;
                root.draw(&Text::new(label.to_owned(), (mid_x, mid_y), text_style))
                    .map_err(backend_error)?;
            }
        }
    }

    // Draw nodes on top
    for node_spec in &spec.nodes {
        let Some(&pos) = layout.get(&node_spec.id) else {
            continue;
        };
        let resolved_node = resolved.get(&node_spec.id).expect("resolved network node");
        let label = resolved_node.label.as_str();
        let (cx, cy) = view.apply(pos);
        let is_selected = spec
            .selected_node_id
            .as_deref()
            .is_some_and(|id| id == node_spec.id.as_str());
        let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        if !node_intersects_viewport(
            viewport,
            (cx, cy),
            &scaled_style,
            &scaled_selection_style,
            is_selected,
        ) {
            continue;
        }

        node::draw_node(
            root,
            cx,
            cy,
            &scaled_style,
            resolved_node.media.as_ref(),
            label,
            is_selected,
            &scaled_selection_style,
            spec.pixel_ratio,
        )?;
        if let Some(badge) =
            toggle_badge_for_node(spec, node_spec, layout, resolved, view, &parent_by_id)
        {
            draw_toggle_badge(root, badge, 1.0)?;
        }
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct TransitionNodeState {
    point: (f64, f64),
    opacity: f64,
}

fn project_point_f64(point: (f64, f64), view: &ScreenTransform) -> (i32, i32) {
    view.apply(point)
}

fn lerp_point(from: (f64, f64), to: (f64, f64), progress: f64) -> (f64, f64) {
    (
        from.0 + (to.0 - from.0) * progress,
        from.1 + (to.1 - from.1) * progress,
    )
}

fn transition_anchor_frame(
    transition: &NetworkTransition,
    to_layout: &BTreeMap<String, (f64, f64)>,
    progress: f64,
) -> (f64, f64) {
    let anchor_from = transition
        .from_layout
        .get(&transition.anchor_node_id)
        .copied()
        .or_else(|| to_layout.get(&transition.anchor_node_id).copied())
        .unwrap_or((0.0, 0.0));
    let anchor_to = to_layout
        .get(&transition.anchor_node_id)
        .copied()
        .or_else(|| transition.from_layout.get(&transition.anchor_node_id).copied())
        .unwrap_or(anchor_from);
    lerp_point(anchor_from, anchor_to, progress)
}

fn transition_node_frame(
    node_id: &str,
    transition: &NetworkTransition,
    to_layout: &BTreeMap<String, (f64, f64)>,
    anchor_frame: (f64, f64),
    progress: f64,
) -> Option<TransitionNodeState> {
    let from = transition.from_layout.get(node_id).copied();
    let to = to_layout.get(node_id).copied();
    match (from, to) {
        (Some(from), Some(to)) => Some(TransitionNodeState {
            point: lerp_point(from, to, progress),
            opacity: 1.0,
        }),
        (Some(from), None) => Some(TransitionNodeState {
            point: lerp_point(from, anchor_frame, progress),
            opacity: 1.0 - progress,
        }),
        (None, Some(to)) => Some(TransitionNodeState {
            point: lerp_point(anchor_frame, to, progress),
            opacity: progress,
        }),
        (None, None) => None,
    }
}

fn transition_phase_opacity(from_present: bool, to_present: bool, progress: f64) -> f64 {
    match (from_present, to_present) {
        (true, true) => 1.0,
        (true, false) => 1.0 - progress,
        (false, true) => progress,
        (false, false) => 0.0,
    }
}

fn ordered_transition_node_ids(
    spec: &NetworkPlotSpec,
    previous_spec: &NetworkPlotSpec,
) -> Vec<String> {
    let mut ordered = spec
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let mut seen = ordered.iter().cloned().collect::<HashSet<_>>();
    for node in &previous_spec.nodes {
        if seen.insert(node.id.clone()) {
            ordered.push(node.id.clone());
        }
    }
    ordered
}

fn render_transition_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    transition: &NetworkTransition,
    progress: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let progress = progress.clamp(0.0, 1.0);
    root.fill(&WHITE).map_err(backend_error)?;
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    let anchor_frame = transition_anchor_frame(transition, layout, progress);
    let current_nodes_by_id = spec
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let previous_nodes_by_id = transition
        .from_spec
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let current_parent_by_id = structural_parent_map(spec);
    let previous_parent_by_id = structural_parent_map(&transition.from_spec);

    if !spec.title.is_empty() {
        let title_size = (20.0 * spec.pixel_ratio.max(0.25)).round() as u32;
        root.draw(&Text::new(
            spec.title.clone(),
            (
                spec.width as i32 / 2 - spec.title.len() as i32 * 7,
                spec.margin as i32 / 2,
            ),
            ("sans-serif", title_size).into_font(),
        ))
        .map_err(backend_error)?;
    }

    let mut edge_entries = BTreeMap::<(String, String), (Option<&crate::NetworkEdge>, Option<&crate::NetworkEdge>)>::new();
    for edge in &transition.from_spec.edges {
        edge_entries.insert((edge.source.clone(), edge.target.clone()), (Some(edge), None));
    }
    for edge in &spec.edges {
        edge_entries
            .entry((edge.source.clone(), edge.target.clone()))
            .and_modify(|entry| entry.1 = Some(edge))
            .or_insert((None, Some(edge)));
    }

    for ((source_id, target_id), (from_edge, to_edge)) in edge_entries {
        let Some(source_state) =
            transition_node_frame(&source_id, transition, layout, anchor_frame, progress)
        else {
            continue;
        };
        let Some(target_state) =
            transition_node_frame(&target_id, transition, layout, anchor_frame, progress)
        else {
            continue;
        };
        let edge_alpha = transition_phase_opacity(from_edge.is_some(), to_edge.is_some(), progress)
            .min(source_state.opacity)
            .min(target_state.opacity)
            .clamp(0.0, 1.0);
        if edge_alpha <= 0.0 {
            continue;
        }

        let render_spec = if to_edge.is_some() {
            spec
        } else {
            &transition.from_spec
        };
        let render_edge = to_edge.or(from_edge).expect("edge present in transition");
        let edge_style = resolve_network_edge_style(render_spec, render_edge)?;
        let target_resolved = resolved
            .get(target_id.as_str())
            .or_else(|| transition.from_resolved.get(target_id.as_str()));
        let target_radius = target_resolved
            .map(|node| scale_node_style(&node.style, view.zoom).radius)
            .unwrap_or(render_spec.node_radius.max(1) as i32);
        let source = project_point_f64(source_state.point, view);
        let target = project_point_f64(target_state.point, view);
        let Some((clipped_source, clipped_target)) = viewport.clip_line(source, target) else {
            continue;
        };

        if edge_style.stroke_width > 0 {
            let shape_style =
                ShapeStyle::from(&edge_style.stroke_color.mix(edge_style.opacity * edge_alpha))
                    .stroke_width(edge_style.stroke_width);
            for (segment_start, segment_end) in edge_line_segments(
                clipped_source,
                clipped_target,
                edge_style.dash_pattern.as_deref(),
            ) {
                root.draw(&PathElement::new(
                    vec![segment_start, segment_end],
                    shape_style,
                ))
                .map_err(backend_error)?;
            }
        }

        if edge_style.arrow_visible
            && edge_style.stroke_width > 0
            && (clipped_source != clipped_target)
        {
            let dx = clipped_target.0 as f64 - clipped_source.0 as f64;
            let dy = clipped_target.1 as f64 - clipped_source.1 as f64;
            let len = (dx * dx + dy * dy).sqrt().max(0.01);
            let ux = dx / len;
            let uy = dy / len;
            let perp_x = -uy;
            let perp_y = ux;
            let target_visible = target_resolved
                .map(|node| {
                    let scaled_style = scale_node_style(&node.style, view.zoom);
                    viewport.intersects_circle(target, scaled_style.radius.max(1))
                })
                .unwrap_or(false);

            let mut tip_x = clipped_target.0;
            let mut tip_y = clipped_target.1;
            if target_visible {
                let candidate = (
                    (target.0 as f64 - ux * target_radius as f64).round() as i32,
                    (target.1 as f64 - uy * target_radius as f64).round() as i32,
                );
                if viewport.contains(candidate) {
                    tip_x = candidate.0;
                    tip_y = candidate.1;
                }
            }
            let b1_x =
                (tip_x as f64 - ux * ARROW_LENGTH + perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b1_y =
                (tip_y as f64 - uy * ARROW_LENGTH + perp_y * ARROW_HALF_WIDTH).round() as i32;
            let b2_x =
                (tip_x as f64 - ux * ARROW_LENGTH - perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b2_y =
                (tip_y as f64 - uy * ARROW_LENGTH - perp_y * ARROW_HALF_WIDTH).round() as i32;

            root.draw(&Polygon::new(
                vec![(tip_x, tip_y), (b1_x, b1_y), (b2_x, b2_y)],
                edge_style
                    .stroke_color
                    .mix(edge_style.opacity * edge_alpha)
                    .filled(),
            ))
            .map_err(backend_error)?;
        }

        if edge_style.label_visible {
            if let Some(label) = render_edge.label.as_deref().filter(|label| !label.is_empty()) {
                let label_size = (12.0 * spec.pixel_ratio.max(0.25)).round() as u32;
                let label_color = edge_style.label_color.unwrap_or(edge_style.stroke_color);
                let text_color = label_color.mix(edge_style.opacity * edge_alpha);
                let text_style = TextStyle::from(("sans-serif", label_size).into_font())
                    .pos(Pos::new(HPos::Center, VPos::Bottom))
                    .color(&text_color);
                let mid_x = ((clipped_source.0 + clipped_target.0) as f64 / 2.0).round() as i32;
                let mid_y = ((clipped_source.1 + clipped_target.1) as f64 / 2.0).round() as i32 - 4;
                root.draw(&Text::new(label.to_owned(), (mid_x, mid_y), text_style))
                    .map_err(backend_error)?;
            }
        }
    }

    for node_id in ordered_transition_node_ids(spec, &transition.from_spec) {
        let Some(node_state) =
            transition_node_frame(&node_id, transition, layout, anchor_frame, progress)
        else {
            continue;
        };
        if node_state.opacity <= 0.0 {
            continue;
        }

        let current_node = current_nodes_by_id.get(node_id.as_str()).copied();
        let previous_node = previous_nodes_by_id.get(node_id.as_str()).copied();
        let Some(node_spec) = current_node.or(previous_node) else {
            continue;
        };
        let Some(resolved_node) = resolved
            .get(node_id.as_str())
            .or_else(|| transition.from_resolved.get(node_id.as_str()))
        else {
            continue;
        };
        let position = project_point_f64(node_state.point, view);
        let mut scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        scaled_style.opacity = (scaled_style.opacity * node_state.opacity).clamp(0.0, 1.0);
        let selection_alpha = transition_phase_opacity(
            transition.from_selected_node_id.as_deref() == Some(node_id.as_str()),
            spec.selected_node_id.as_deref() == Some(node_id.as_str()),
            progress,
        );
        let mut scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        scaled_selection_style.opacity =
            (scaled_selection_style.opacity * selection_alpha).clamp(0.0, 1.0);
        if !node_intersects_viewport(
            viewport,
            position,
            &scaled_style,
            &scaled_selection_style,
            selection_alpha > 0.0,
        ) {
            continue;
        }

        node::draw_node(
            root,
            position.0,
            position.1,
            &scaled_style,
            resolved_node.media.as_ref(),
            resolved_node.label.as_str(),
            selection_alpha > 0.0,
            &scaled_selection_style,
            spec.pixel_ratio,
        )?;

        let render_spec = current_node.map(|_| spec).unwrap_or(&transition.from_spec);
        let parent_point = current_parent_by_id
            .get(node_id.as_str())
            .copied()
            .or_else(|| previous_parent_by_id.get(node_id.as_str()).copied())
            .and_then(|parent_id| {
                transition_node_frame(parent_id, transition, layout, anchor_frame, progress)
                    .map(|state| state.point)
            });
        if let Some(badge) = toggle_badge_for_node_frame(
            render_spec,
            node_spec,
            resolved_node,
            node_state.point,
            parent_point,
            view,
        ) {
            draw_toggle_badge(root, badge, node_state.opacity)?;
        }
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

fn render_transition_nodes_with_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    transition: &NetworkTransition,
    progress: f64,
) -> Vec<GraphNodeRenderInfo> {
    let progress = progress.clamp(0.0, 1.0);
    let anchor_frame = transition_anchor_frame(transition, layout, progress);
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);

    ordered_transition_node_ids(spec, &transition.from_spec)
        .into_iter()
        .filter_map(|node_id| {
            let node_state =
                transition_node_frame(&node_id, transition, layout, anchor_frame, progress)?;
            if node_state.opacity <= 0.0 {
                return None;
            }
            let resolved_node = resolved
                .get(node_id.as_str())
                .or_else(|| transition.from_resolved.get(node_id.as_str()))?;
            let position = project_point_f64(node_state.point, view);
            let mut scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            scaled_style.opacity = (scaled_style.opacity * node_state.opacity).clamp(0.0, 1.0);
            let selection_alpha = transition_phase_opacity(
                transition.from_selected_node_id.as_deref() == Some(node_id.as_str()),
                spec.selected_node_id.as_deref() == Some(node_id.as_str()),
                progress,
            );
            let mut scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            scaled_selection_style.opacity =
                (scaled_selection_style.opacity * selection_alpha).clamp(0.0, 1.0);
            if !node_intersects_viewport(
                viewport,
                position,
                &scaled_style,
                &scaled_selection_style,
                selection_alpha > 0.0,
            ) {
                return None;
            }
            Some(GraphNodeRenderInfo {
                id: node_id,
                center_x: position.0,
                center_y: position.1,
                radius: scaled_style.radius,
                shape: scaled_style.shape.clone(),
                opacity: scaled_style.opacity,
                media: resolved_node.media.clone(),
            })
        })
        .collect()
}

fn pick_hit_from_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    canvas_x: f64,
    canvas_y: f64,
) -> Option<NetworkPickHit> {
    if !canvas_x.is_finite() || !canvas_y.is_finite() {
        return None;
    }
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    let parent_by_id = structural_parent_map(spec);

    let toggle_hit = spec.nodes.iter().find_map(|node_spec| {
        let resolved_node = resolved.get(&node_spec.id)?;
        let &(px, py) = layout.get(&node_spec.id)?;
        let (screen_x, screen_y) = view.apply((px, py));
        let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        let is_selected = spec
            .selected_node_id
            .as_deref()
            .is_some_and(|id| id == node_spec.id.as_str());
        if !node_intersects_viewport(
            viewport,
            (screen_x, screen_y),
            &scaled_style,
            &scaled_selection_style,
            is_selected,
        ) {
            return None;
        }
        let badge = toggle_badge_for_node(spec, node_spec, layout, resolved, view, &parent_by_id)?;
        let dx = f64::from(badge.center_x) - canvas_x;
        let dy = f64::from(badge.center_y) - canvas_y;
        (dx * dx + dy * dy <= f64::from(badge.radius * badge.radius)).then_some(NetworkPickHit {
            kind: NetworkPickKind::Toggle,
            node_id: node_spec.id.clone(),
        })
    });
    if toggle_hit.is_some() {
        return toggle_hit;
    }

    spec.nodes
        .iter()
        .filter_map(|node_spec| {
            let &(px, py) = layout.get(&node_spec.id)?;
            let resolved_node = resolved.get(&node_spec.id)?;
            let (screen_x, screen_y) = view.apply((px, py));
            let dx = f64::from(screen_x) - canvas_x;
            let dy = f64::from(screen_y) - canvas_y;
            let dist_sq = dx * dx + dy * dy;
            let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            let is_selected = spec
                .selected_node_id
                .as_deref()
                .is_some_and(|id| id == node_spec.id.as_str());
            if !node_intersects_viewport(
                viewport,
                (screen_x, screen_y),
                &scaled_style,
                &scaled_selection_style,
                is_selected,
            ) {
                return None;
            }
            let hit_radius =
                f64::from(scaled_style.radius.max(1) + scaled_selection_style.padding.max(0));
            node::node_contains(
                &scaled_style.shape,
                f64::from(screen_x),
                f64::from(screen_y),
                hit_radius,
                canvas_x,
                canvas_y,
            )
            .then_some((
                NetworkPickHit {
                    kind: NetworkPickKind::Node,
                    node_id: node_spec.id.clone(),
                },
                dist_sq,
            ))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(hit, _)| hit)
}

fn render_nodes_with_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
) -> Vec<GraphNodeRenderInfo> {
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    spec.nodes
        .iter()
        .filter_map(|node_spec| {
            let &(px, py) = layout.get(&node_spec.id)?;
            let resolved_node = resolved.get(&node_spec.id)?;
            let (screen_x, screen_y) = view.apply((px, py));
            let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            let is_selected = spec
                .selected_node_id
                .as_deref()
                .is_some_and(|id| id == node_spec.id.as_str());
            if !node_intersects_viewport(
                viewport,
                (screen_x, screen_y),
                &scaled_style,
                &scaled_selection_style,
                is_selected,
            ) {
                return None;
            }
            Some(GraphNodeRenderInfo {
                id: node_spec.id.clone(),
                center_x: screen_x,
                center_y: screen_y,
                radius: scaled_style.radius,
                shape: scaled_style.shape.clone(),
                opacity: scaled_style.opacity,
                media: resolved_node.media.clone(),
            })
        })
        .collect()
}

fn node_intersects_viewport(
    viewport: PixelBounds,
    center: (i32, i32),
    style: &ResolvedNodeStyle,
    selection_style: &ResolvedSelectionStyle,
    is_selected: bool,
) -> bool {
    let footprint_radius = style.radius.max(1)
        + if is_selected {
            selection_style.padding.max(0)
        } else {
            0
        };
    viewport.intersects_circle(center, footprint_radius)
}

fn scale_node_style(style: &ResolvedNodeStyle, zoom: f64) -> ResolvedNodeStyle {
    let mut scaled = style.clone();
    scaled.radius = ((scaled.radius.max(1) as f64) * zoom).round() as i32;
    scaled.radius = scaled.radius.max(1);
    scaled
}

fn scale_selection_style(style: &ResolvedSelectionStyle, zoom: f64) -> ResolvedSelectionStyle {
    let mut scaled = style.clone();
    scaled.padding = ((scaled.padding.max(0) as f64) * zoom).round() as i32;
    scaled.padding = scaled.padding.max(0);
    scaled
}

pub fn render_network_on<DB>(root: PlotArea<DB>, spec: &NetworkPlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    NetworkSession::new(spec.clone())?.render_on(root)
}

pub fn network_render_nodes(spec: &NetworkPlotSpec) -> Result<Vec<GraphNodeRenderInfo>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.render_nodes())
}

pub fn pick_network_hit(
    spec: &NetworkPlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<NetworkPickHit>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.pick(canvas_x, canvas_y))
}

pub fn pick_network_node(
    spec: &NetworkPlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<String>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.pick_node(canvas_x, canvas_y))
}

pub fn pan_network_spec(
    spec: &NetworkPlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<NetworkPlotSpec, PlotError> {
    let mut session = NetworkSession::new(spec.clone())?;
    session.pan(delta_x, delta_y);
    Ok(session.into_spec())
}

pub fn focus_network_view(
    spec: &NetworkPlotSpec,
    node_id: &str,
    options: Option<NetworkFocusOptions>,
) -> Result<Option<NetworkView>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.compute_focus_view(node_id, options))
}

fn resolve_network_edge_style(
    spec: &NetworkPlotSpec,
    edge: &crate::NetworkEdge,
) -> Result<crate::graph_style::ResolvedEdgeStyle, PlotError> {
    resolve_edge_style(EdgeStyleContext {
        default_stroke_color: DEFAULT_EDGE_COLOR,
        default_stroke_width: 1,
        default_arrow_visible: spec.show_arrows,
        default_label_visible: false,
        graph_style: spec.default_edge_style.as_ref(),
        legacy_stroke_color: edge.color.as_deref(),
        item_style: edge.style.as_ref(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::ResolvedNodeMediaKind;
    use crate::types::{
        BuiltinNodeIcon, GraphEdgeStyle, GraphNodeStyle, NetworkEdge, NetworkNode, NetworkPlotSpec,
        NetworkView, NodeMedia, NodeMediaFit, NodeMediaKind, NodeShape, SelectionStyle,
    };
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn sample_spec() -> NetworkPlotSpec {
        NetworkPlotSpec {
            width: 480,
            height: 360,
            title: "Test Network".to_string(),
            nodes: vec![
                NetworkNode {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "c".to_string(),
                    label: "C".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "a".to_string(),
                    target: "b".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "b".to_string(),
                    target: "c".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            node_radius: 16,
            default_node_style: None,
            default_edge_style: None,
            selection_style: None,
            margin: 40,
            offset_x: 0,
            offset_y: 0,
            selected_node_id: None,
            layout_iterations: 50,
            show_arrows: true,
            show_labels: true,
            pixel_ratio: 1.0,
        }
    }

    fn positioned_spec() -> NetworkPlotSpec {
        NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    color: None,
                    x: Some(100.0),
                    y: Some(100.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    color: None,
                    x: Some(300.0),
                    y: Some(200.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![NetworkEdge {
                source: "a".to_string(),
                target: "b".to_string(),
                label: None,
                color: None,
                weight: None,
                style: None,
            }],
            ..sample_spec()
        }
    }

    fn toggleable_positioned_spec(expanded: bool) -> NetworkPlotSpec {
        NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(60.0),
                    y: Some(100.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "chapter".to_string(),
                    label: "Chapter".to_string(),
                    color: None,
                    x: Some(140.0),
                    y: Some(100.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: BTreeMap::from([
                        (TOGGLEABLE_PROPERTY_KEY.to_string(), "true".to_string()),
                        (
                            EXPANDED_PROPERTY_KEY.to_string(),
                            if expanded { "true" } else { "false" }.to_string(),
                        ),
                    ]),
                },
                NetworkNode {
                    id: "leaf".to_string(),
                    label: "Leaf".to_string(),
                    color: None,
                    x: Some(220.0),
                    y: Some(100.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "chapter".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "chapter".to_string(),
                    target: "leaf".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
            ],
            ..sample_spec()
        }
    }

    fn assert_no_layout_collisions(
        spec: &NetworkPlotSpec,
        layout: &BTreeMap<String, (f64, f64)>,
        view_zoom: f64,
    ) {
        if !ENABLE_LAYOUT_COLLISIONS {
            return;
        }
        let resolved = resolve_nodes(spec).unwrap();
        let positions: Vec<(f64, f64)> = spec
            .nodes
            .iter()
            .map(|node| {
                layout
                    .get(&node.id)
                    .copied()
                    .expect("layout contains every node")
            })
            .collect();
        let footprints = node_footprints(spec, &resolved, &positions, view_zoom);

        for source_idx in 0..footprints.len() {
            let source = footprints[source_idx];
            let source_id = spec.nodes[source_idx].id.as_str();
            for target_idx in (source_idx + 1)..footprints.len() {
                let target = footprints[target_idx];
                let target_id = spec.nodes[target_idx].id.as_str();
                assert!(
                    circle_separation(
                        source_id,
                        target_id,
                        source.center,
                        source.radius,
                        target.center,
                        target.radius,
                    )
                    .is_none(),
                    "node collision between {source_id} and {target_id}",
                );
                if let (Some(source_label), Some(target_label)) = (source.label, target.label) {
                    assert!(
                        label_box_separation(source_label, target_label).is_none(),
                        "label collision between {source_id} and {target_id}",
                    );
                }
                if let Some(source_label) = source.label {
                    assert!(
                        circle_label_separation(target.center, target.radius, source_label)
                            .is_none(),
                        "label of {source_id} overlaps node {target_id}",
                    );
                }
                if let Some(target_label) = target.label {
                    assert!(
                        circle_label_separation(source.center, source.radius, target_label)
                            .is_none(),
                        "label of {target_id} overlaps node {source_id}",
                    );
                }
            }
        }
    }

    #[test]
    fn network_rejects_empty_nodes() {
        let mut spec = sample_spec();
        spec.nodes.clear();
        spec.edges.clear();
        let err = NetworkSession::new(spec).unwrap_err();
        assert_eq!(err, PlotError::EmptyNetwork);
    }

    #[test]
    fn network_rejects_duplicate_node_ids() {
        let mut spec = sample_spec();
        spec.nodes[1].id = "a".to_string();
        let err = NetworkSession::new(spec).unwrap_err();
        assert!(matches!(err, PlotError::DuplicateNodeId { .. }));
    }

    #[test]
    fn network_rejects_duplicate_edges() {
        let mut spec = sample_spec();
        spec.edges.push(spec.edges[0].clone());
        let err = NetworkSession::new(spec).unwrap_err();
        assert!(matches!(err, PlotError::DuplicateEdge { .. }));
    }

    #[test]
    fn network_rejects_unknown_edge_nodes() {
        let mut spec = sample_spec();
        spec.edges.push(NetworkEdge {
            source: "a".to_string(),
            target: "z".to_string(),
            label: None,
            color: None,
            weight: None,
            style: None,
        });
        let err = NetworkSession::new(spec).unwrap_err();
        assert!(matches!(err, PlotError::UnknownNode { .. }));
    }

    #[test]
    fn network_allows_cycles() {
        let mut spec = sample_spec();
        spec.edges.push(NetworkEdge {
            source: "c".to_string(),
            target: "a".to_string(),
            label: None,
            color: None,
            weight: None,
            style: None,
        });
        assert!(NetworkSession::new(spec).is_ok());
    }

    #[test]
    fn network_allows_multiple_parents() {
        let mut spec = sample_spec();
        // Both "a" and "b" point to "c" — c has 2 parents
        spec.edges.push(NetworkEdge {
            source: "a".to_string(),
            target: "c".to_string(),
            label: None,
            color: None,
            weight: None,
            style: None,
        });
        assert!(NetworkSession::new(spec).is_ok());
    }

    #[test]
    fn network_layout_positions_are_finite() {
        let spec = sample_spec();
        let layout = fr_layout(&spec);
        for (_, &(x, y)) in &layout {
            assert!(x.is_finite(), "x={x} must be finite");
            assert!(y.is_finite(), "y={y} must be finite");
        }
    }

    #[test]
    fn network_seed_position_prefers_open_parent_gap() {
        let spec = NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "left".to_string(),
                    label: "Left".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "right".to_string(),
                    label: "Right".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "new".to_string(),
                    label: "New".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "left".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "right".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "new".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            ..sample_spec()
        };
        let positions = BTreeMap::from([
            ("root".to_string(), (0.0, 0.0)),
            ("left".to_string(), (-WORLD_NODE_SPACING, 0.0)),
            ("right".to_string(), (WORLD_NODE_SPACING, 0.0)),
        ]);

        let seeded = seed_position(&spec, "new", &positions);

        assert!(seeded.1.abs() > WORLD_NODE_SPACING * 0.5);
        assert!(seeded.0.abs() < WORLD_NODE_SPACING * 0.4);
    }

    #[test]
    fn network_structural_helpers_ignore_lightweight_sibling_edges() {
        let spec = NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "alpha".to_string(),
                    label: "Alpha".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "beta".to_string(),
                    label: "Beta".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "alpha".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "beta".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "alpha".to_string(),
                    target: "beta".to_string(),
                    label: None,
                    color: None,
                    weight: Some(0.15),
                    style: None,
                },
            ],
            ..sample_spec()
        };

        assert_eq!(parent_ids(&spec, "beta"), vec!["root".to_string()]);
        assert_eq!(
            child_ids(&spec, "root"),
            vec!["alpha".to_string(), "beta".to_string()]
        );
        assert!(child_ids(&spec, "alpha").is_empty());
    }

    #[test]
    fn network_user_supplied_positions_are_used_directly() {
        let spec = positioned_spec();
        let layout = compute_layout(&spec).unwrap();
        assert_eq!(layout["a"], (100.0, 100.0));
        assert_eq!(layout["b"], (300.0, 200.0));
    }

    #[test]
    fn network_user_supplied_positions_are_not_clamped_to_canvas() {
        let mut spec = positioned_spec();
        spec.nodes[0].x = Some(-40.0);
        spec.nodes[0].y = Some(420.0);

        let layout = compute_layout(&spec).unwrap();

        assert_eq!(layout["a"], (-40.0, 420.0));
    }

    #[test]
    fn network_svg_has_correct_circle_count() {
        let mut svg = String::new();
        let spec = positioned_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_network_on(area, &spec).unwrap();
        assert_eq!(svg.matches("<circle").count(), spec.nodes.len());
    }

    #[test]
    fn network_pan_updates_offsets() {
        let spec = sample_spec();
        let updated = pan_network_spec(&spec, 30.0, -15.0).unwrap();
        assert_eq!(updated.offset_x, 30);
        assert_eq!(updated.offset_y, -15);
    }

    #[test]
    fn network_fr_layout_is_deterministic() {
        let spec = sample_spec();
        let layout1 = fr_layout(&spec);
        let layout2 = fr_layout(&spec);
        for (id, &pos1) in &layout1 {
            let pos2 = layout2[id];
            assert!((pos1.0 - pos2.0).abs() < 0.001 && (pos1.1 - pos2.1).abs() < 0.001);
        }
    }

    #[test]
    fn network_hit_test_returns_closest_node() {
        let spec = positioned_spec();
        let session = NetworkSession::new(spec).unwrap();
        // Click near node "a" at (100, 100)
        let hit = session.pick_node(102.0, 98.0);
        assert_eq!(hit, Some("a".to_string()));
    }

    #[test]
    fn network_toggle_badge_uses_outward_direction_and_state() {
        let spec = toggleable_positioned_spec(false);
        let session = NetworkSession::new(spec.clone()).unwrap();
        let parent_by_id = structural_parent_map(&spec);
        let badge = toggle_badge_for_node(
            &spec,
            &spec.nodes[1],
            &session.layout,
            &session.resolved,
            &session.view,
            &parent_by_id,
        )
        .expect("toggle badge");

        assert!(badge.center_x > 140);
        assert_eq!(badge.center_y, 100);
        assert!(!badge.expanded);

        let expanded_spec = toggleable_positioned_spec(true);
        let expanded_session = NetworkSession::new(expanded_spec.clone()).unwrap();
        let expanded_parents = structural_parent_map(&expanded_spec);
        let expanded_badge = toggle_badge_for_node(
            &expanded_spec,
            &expanded_spec.nodes[1],
            &expanded_session.layout,
            &expanded_session.resolved,
            &expanded_session.view,
            &expanded_parents,
        )
        .expect("expanded toggle badge");
        assert!(expanded_badge.expanded);
    }

    #[test]
    fn network_toggle_badge_hit_distinguishes_toggle_from_node_body() {
        let spec = toggleable_positioned_spec(false);
        let session = NetworkSession::new(spec.clone()).unwrap();
        let parent_by_id = structural_parent_map(&spec);
        let badge = toggle_badge_for_node(
            &spec,
            &spec.nodes[1],
            &session.layout,
            &session.resolved,
            &session.view,
            &parent_by_id,
        )
        .expect("toggle badge");

        let badge_hit = session.pick(badge.center_x as f64, badge.center_y as f64);
        assert_eq!(
            badge_hit,
            Some(NetworkPickHit {
                kind: NetworkPickKind::Toggle,
                node_id: "chapter".to_string(),
            })
        );

        let node_hit = session.pick(140.0, 100.0);
        assert_eq!(
            node_hit,
            Some(NetworkPickHit {
                kind: NetworkPickKind::Node,
                node_id: "chapter".to_string(),
            })
        );

        assert!(toggle_badge_for_node(
            &spec,
            &spec.nodes[0],
            &session.layout,
            &session.resolved,
            &session.view,
            &parent_by_id,
        )
        .is_none());
        assert!(toggle_badge_for_node(
            &spec,
            &spec.nodes[2],
            &session.layout,
            &session.resolved,
            &session.view,
            &parent_by_id,
        )
        .is_none());
    }

    #[test]
    fn network_mixed_layout_pins_supplied_nodes_and_places_free_ones() {
        // "a" has explicit coordinates; "b" and "c" do not.
        let spec = NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "a".into(),
                    label: "A".into(),
                    color: None,
                    x: Some(200.0),
                    y: Some(150.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "b".into(),
                    label: "B".into(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "c".into(),
                    label: "C".into(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![NetworkEdge {
                source: "a".into(),
                target: "b".into(),
                label: None,
                color: None,
                weight: None,
                style: None,
            }],
            ..sample_spec()
        };
        let layout = compute_layout(&spec).unwrap();
        // Pinned node "a" must be exactly at its supplied coordinates
        assert_eq!(layout["a"], (200.0, 150.0));
        // Free nodes "b" and "c" must resolve to finite positions and stay distinct
        for id in ["b", "c"] {
            let (x, y) = layout[id];
            assert!(x.is_finite(), "{id} x={x} must be finite");
            assert!(y.is_finite(), "{id} y={y} must be finite");
            assert_ne!((x, y), layout["a"]);
        }
    }

    #[test]
    fn network_non_circle_shapes_render_without_error() {
        // (shape, expected SVG element tag)
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
            let area =
                SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
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
    fn network_hit_test_uses_per_node_radius_override() {
        let mut spec = positioned_spec();
        spec.node_radius = 12;
        spec.selection_style = Some(SelectionStyle {
            padding: Some(0.0),
            ..Default::default()
        });
        spec.nodes[0].style = Some(GraphNodeStyle {
            radius: Some(40.0),
            ..Default::default()
        });

        let session = NetworkSession::new(spec).unwrap();
        let hit = session.pick_node(135.0, 100.0);

        assert_eq!(hit.as_deref(), Some("a"));
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

    #[test]
    fn network_session_update_preserves_existing_positions() {
        let spec = positioned_spec();
        let mut session = NetworkSession::new(spec.clone()).unwrap();
        let before = session.layout.clone();
        let updated = NetworkPlotSpec {
            nodes: vec![
                spec.nodes[0].clone(),
                spec.nodes[1].clone(),
                NetworkNode {
                    id: "c".to_string(),
                    label: "C".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                spec.edges[0].clone(),
                NetworkEdge {
                    source: "b".to_string(),
                    target: "c".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            ..spec
        };

        session.update_spec(updated).unwrap();

        assert_eq!(session.layout["a"], before["a"]);
        assert_eq!(session.layout["b"], before["b"]);
        assert!(session.layout.contains_key("c"));
    }

    #[test]
    fn network_selection_only_update_does_not_create_transition() {
        let spec = positioned_spec();
        let mut session = NetworkSession::new(spec.clone()).unwrap();

        session
            .update_spec(NetworkPlotSpec {
                selected_node_id: Some("b".to_string()),
                ..spec
            })
            .unwrap();

        assert!(!session.has_transition());
    }

    #[test]
    fn network_topology_transition_anchors_new_branch_to_parent() {
        let collapsed = NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "chapter".to_string(),
                    label: "Chapter".to_string(),
                    color: None,
                    x: Some(120.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: BTreeMap::from([
                        (TOGGLEABLE_PROPERTY_KEY.to_string(), "true".to_string()),
                        (EXPANDED_PROPERTY_KEY.to_string(), "false".to_string()),
                    ]),
                },
            ],
            edges: vec![NetworkEdge {
                source: "root".to_string(),
                target: "chapter".to_string(),
                label: None,
                color: None,
                weight: Some(1.0),
                style: None,
            }],
            ..sample_spec()
        };
        let expanded = NetworkPlotSpec {
            nodes: vec![
                collapsed.nodes[0].clone(),
                NetworkNode {
                    properties: BTreeMap::from([
                        (TOGGLEABLE_PROPERTY_KEY.to_string(), "true".to_string()),
                        (EXPANDED_PROPERTY_KEY.to_string(), "true".to_string()),
                    ]),
                    ..collapsed.nodes[1].clone()
                },
                NetworkNode {
                    id: "section".to_string(),
                    label: "Section".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                collapsed.edges[0].clone(),
                NetworkEdge {
                    source: "chapter".to_string(),
                    target: "section".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
            ],
            ..collapsed.clone()
        };

        let mut session = NetworkSession::new(collapsed).unwrap();
        session.update_spec(expanded).unwrap();

        assert!(session.has_transition());
        let transition = session.transition.as_ref().expect("transition present");
        assert_eq!(transition.anchor_node_id, "chapter");

        let midway = session.render_transition_nodes(0.5);
        let section = midway
            .iter()
            .find(|node| node.id == "section")
            .expect("new node rendered mid-transition");
        let chapter_from = transition.from_layout["chapter"];
        let section_to = session.layout["section"];
        let expected_x = chapter_from.0 + (section_to.0 - chapter_from.0) * 0.5;
        let expected_y = chapter_from.1 + (section_to.1 - chapter_from.1) * 0.5;

        assert!((f64::from(section.center_x) - expected_x).abs() < 2.0);
        assert!((f64::from(section.center_y) - expected_y).abs() < 2.0);
        assert!(section.opacity > 0.0 && section.opacity < 1.0);
    }

    #[test]
    fn network_layout_separates_long_sibling_labels() {
        let spec = NetworkPlotSpec {
            width: 960,
            height: 720,
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "alpha".to_string(),
                    label: "Alpha label needs clearance".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "beta".to_string(),
                    label: "Beta label needs clearance".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "gamma".to_string(),
                    label: "Gamma label needs clearance".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "alpha".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "beta".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "gamma".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            layout_iterations: 220,
            node_radius: 24,
            ..sample_spec()
        };

        let layout = compute_layout(&spec).unwrap();

        assert_no_layout_collisions(&spec, &layout, 1.0);
    }

    #[test]
    fn network_layout_spreads_root_siblings_radially() {
        let spec = NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "alpha".to_string(),
                    label: "Alpha chapter with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "beta".to_string(),
                    label: "Beta chapter with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "gamma".to_string(),
                    label: "Gamma chapter with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "alpha".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "beta".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "gamma".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "alpha".to_string(),
                    target: "beta".to_string(),
                    label: None,
                    color: None,
                    weight: Some(0.15),
                    style: None,
                },
                NetworkEdge {
                    source: "beta".to_string(),
                    target: "gamma".to_string(),
                    label: None,
                    color: None,
                    weight: Some(0.15),
                    style: None,
                },
            ],
            layout_iterations: 220,
            node_radius: 24,
            ..sample_spec()
        };

        let layout = compute_layout(&spec).unwrap();

        let xs = [layout["alpha"].0, layout["beta"].0, layout["gamma"].0];
        let ys = [layout["alpha"].1, layout["beta"].1, layout["gamma"].1];

        assert!(xs.iter().copied().any(|x| x < -WORLD_NODE_SPACING * 0.2));
        assert!(xs.iter().copied().any(|x| x > WORLD_NODE_SPACING * 0.2));
        assert!(ys.iter().copied().any(|y| y < -WORLD_NODE_SPACING * 0.2));
        assert!(ys.iter().copied().any(|y| y > WORLD_NODE_SPACING * 0.2));
        assert_no_layout_collisions(&spec, &layout, 1.0);
    }

    #[test]
    fn network_layout_spreads_nested_siblings_around_parent() {
        let spec = NetworkPlotSpec {
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "chapter".to_string(),
                    label: "Chapter".to_string(),
                    color: None,
                    x: Some(WORLD_NODE_SPACING * 1.4),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "section-a".to_string(),
                    label: "Section A with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "section-b".to_string(),
                    label: "Section B with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "section-c".to_string(),
                    label: "Section C with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "chapter".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "chapter".to_string(),
                    target: "section-a".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "chapter".to_string(),
                    target: "section-b".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "chapter".to_string(),
                    target: "section-c".to_string(),
                    label: None,
                    color: None,
                    weight: Some(1.0),
                    style: None,
                },
                NetworkEdge {
                    source: "section-a".to_string(),
                    target: "section-b".to_string(),
                    label: None,
                    color: None,
                    weight: Some(0.15),
                    style: None,
                },
                NetworkEdge {
                    source: "section-b".to_string(),
                    target: "section-c".to_string(),
                    label: None,
                    color: None,
                    weight: Some(0.15),
                    style: None,
                },
            ],
            layout_iterations: 220,
            node_radius: 24,
            ..sample_spec()
        };

        let layout = compute_layout(&spec).unwrap();
        let chapter = layout["chapter"];
        let child_positions = [
            layout["section-a"],
            layout["section-b"],
            layout["section-c"],
        ];

        assert!(child_positions.iter().all(|&(x, y)| {
            let dx = x - chapter.0;
            let dy = y - chapter.1;
            (dx * dx + dy * dy).sqrt() > WORLD_NODE_SPACING * 0.35
        }));
        assert!(child_positions
            .iter()
            .copied()
            .any(|(_, y)| y < chapter.1 - WORLD_NODE_SPACING * 0.2));
        assert!(child_positions
            .iter()
            .copied()
            .any(|(_, y)| y > chapter.1 + WORLD_NODE_SPACING * 0.2));
        assert_no_layout_collisions(&spec, &layout, 1.0);
    }

    #[test]
    fn network_session_spawned_node_labels_respect_active_zoom() {
        let initial = NetworkPlotSpec {
            width: 960,
            height: 720,
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "alpha".to_string(),
                    label: "Alpha branch with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "beta".to_string(),
                    label: "Beta branch with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "alpha".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "beta".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            layout_iterations: 220,
            node_radius: 24,
            ..sample_spec()
        };
        let updated = NetworkPlotSpec {
            nodes: vec![
                initial.nodes[0].clone(),
                initial.nodes[1].clone(),
                initial.nodes[2].clone(),
                NetworkNode {
                    id: "gamma".to_string(),
                    label: "Gamma branch with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "delta".to_string(),
                    label: "Delta branch with a long label".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                initial.edges[0].clone(),
                initial.edges[1].clone(),
                NetworkEdge {
                    source: "root".to_string(),
                    target: "gamma".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "delta".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            ..initial.clone()
        };

        let mut session = NetworkSession::new(initial).unwrap();
        session
            .set_view(NetworkView {
                zoom: 1.8,
                translate_x: 0.0,
                translate_y: 0.0,
            })
            .unwrap();

        session.update_spec(updated.clone()).unwrap();

        assert_no_layout_collisions(&updated, &session.layout, 1.8);
    }

    #[test]
    fn network_session_collapse_restores_parent_distance() {
        let collapsed = NetworkPlotSpec {
            width: 800,
            height: 600,
            nodes: vec![
                NetworkNode {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "root".to_string(),
                    target: "a".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "root".to_string(),
                    target: "b".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            layout_iterations: 200,
            ..sample_spec()
        };
        let expanded = NetworkPlotSpec {
            nodes: vec![
                collapsed.nodes[0].clone(),
                collapsed.nodes[1].clone(),
                collapsed.nodes[2].clone(),
                NetworkNode {
                    id: "c".to_string(),
                    label: "C".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "d".to_string(),
                    label: "D".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "e".to_string(),
                    label: "E".to_string(),
                    color: None,
                    x: None,
                    y: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                collapsed.edges[0].clone(),
                collapsed.edges[1].clone(),
                NetworkEdge {
                    source: "b".to_string(),
                    target: "c".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "b".to_string(),
                    target: "d".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "b".to_string(),
                    target: "e".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            ..collapsed.clone()
        };

        let mut session = NetworkSession::new(collapsed.clone()).unwrap();
        let collapsed_distance = {
            let root = session.layout["root"];
            let node = session.layout["b"];
            (node.0 - root.0).hypot(node.1 - root.1)
        };

        session.update_spec(expanded).unwrap();
        let expanded_distance = {
            let root = session.layout["root"];
            let node = session.layout["b"];
            (node.0 - root.0).hypot(node.1 - root.1)
        };

        session.update_spec(collapsed).unwrap();
        let restored_distance = {
            let root = session.layout["root"];
            let node = session.layout["b"];
            (node.0 - root.0).hypot(node.1 - root.1)
        };

        assert!((expanded_distance - collapsed_distance).abs() > 10.0);
        assert!((restored_distance - collapsed_distance).abs() < 10.0);
    }

    #[test]
    fn network_focus_view_fits_node_and_neighbors() {
        let spec = NetworkPlotSpec {
            width: 400,
            height: 240,
            nodes: vec![
                NetworkNode {
                    id: "a".to_string(),
                    label: "A".to_string(),
                    color: None,
                    x: Some(-100.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "b".to_string(),
                    label: "B".to_string(),
                    color: None,
                    x: Some(0.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "c".to_string(),
                    label: "C".to_string(),
                    color: None,
                    x: Some(100.0),
                    y: Some(0.0),
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge {
                    source: "a".to_string(),
                    target: "b".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
                NetworkEdge {
                    source: "b".to_string(),
                    target: "c".to_string(),
                    label: None,
                    color: None,
                    weight: None,
                    style: None,
                },
            ],
            ..sample_spec()
        };
        let width = spec.width as f64;
        let height = spec.height as f64;
        let session = NetworkSession::new(spec).unwrap();

        let view = session
            .compute_focus_view("b", Some(NetworkFocusOptions::default()))
            .unwrap();

        assert!(view.zoom > 0.25);
        let (ax, ay) = session.layout["a"];
        let (bx, by) = session.layout["b"];
        let (cx, cy) = session.layout["c"];
        let a_screen = (
            ax * view.zoom + view.translate_x,
            ay * view.zoom + view.translate_y,
        );
        let b_screen = (
            bx * view.zoom + view.translate_x,
            by * view.zoom + view.translate_y,
        );
        let c_screen = (
            cx * view.zoom + view.translate_x,
            cy * view.zoom + view.translate_y,
        );

        assert!(a_screen.0 >= 0.0 && a_screen.0 <= width);
        assert!(c_screen.0 >= 0.0 && c_screen.0 <= width);
        assert!(a_screen.0 < b_screen.0 && b_screen.0 < c_screen.0);
        assert!((b_screen.0 - width / 2.0).abs() < 1.0);
        assert!((b_screen.1 - height / 2.0).abs() < 1.0);
    }

    #[test]
    fn network_focus_view_uses_minimum_world_span_for_isolated_nodes() {
        let spec = positioned_spec();
        let session = NetworkSession::new(spec.clone()).unwrap();

        let view = session
            .compute_focus_view(
                "a",
                Some(NetworkFocusOptions {
                    min_world_span: 200.0,
                    ..Default::default()
                }),
            )
            .unwrap();

        assert!(view.zoom <= (spec.width as f64 - 96.0) / 200.0 + 0.001);
    }

    #[test]
    fn network_rejects_invalid_style_values() {
        let mut spec = sample_spec();
        spec.default_node_style = Some(GraphNodeStyle {
            opacity: Some(1.5),
            ..Default::default()
        });

        let err = NetworkSession::new(spec).unwrap_err();

        assert_eq!(
            err,
            PlotError::InvalidStyleValue {
                field: "node_style.opacity",
                value: 1.5,
                reason: "must be between 0 and 1 inclusive",
            }
        );
    }

    #[test]
    fn network_rejects_invalid_dash_pattern_values() {
        let mut spec = sample_spec();
        spec.default_edge_style = Some(GraphEdgeStyle {
            dash_pattern: Some(vec![0.0, 4.0]),
            ..Default::default()
        });

        let err = NetworkSession::new(spec).unwrap_err();

        assert_eq!(
            err,
            PlotError::InvalidStyleValue {
                field: "edge_style.dash_pattern",
                value: 0.0,
                reason: "must be greater than or equal to 1",
            }
        );
    }
}
