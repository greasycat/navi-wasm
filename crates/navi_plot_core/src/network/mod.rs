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

mod layout;
mod render;
mod session;
#[cfg(test)]
mod tests;

use self::layout::*;
use self::render::*;

pub use self::render::{
    focus_network_view, network_render_nodes, pan_network_spec, pick_network_hit,
    pick_network_node, render_network_on,
};
pub use self::session::NetworkSession;

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
const RADIAL_FANOUT_RADIUS_SCALE: f64 = 0.25;
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
const TRACKING_EDGE_COLOR: RGBColor = RGBColor(239, 68, 68);
const TRACKING_EDGE_BREATH_COLOR: RGBColor = RGBColor(255, 255, 255);
const TRACKING_EDGE_WIDTH: u32 = 4;
const TRACKING_EDGE_OPACITY: f64 = 0.95;
const TRACKING_NODE_BORDER_COLOR: RGBColor = RGBColor(239, 68, 68);
const TRACKING_NODE_BORDER_WIDTH: u32 = 3;

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
    min_radius: f64,
    max_radius: f64,
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

#[derive(Debug, Clone, PartialEq)]
struct NetworkTrackedPath {
    node_ids: Vec<String>,
    progress: f64,
    breath_phase: f64,
}

impl NetworkTrackedPath {
    fn resolve(spec: &NetworkPlotSpec, node_ids: Vec<String>) -> Result<Option<Self>, PlotError> {
        if node_ids.len() < 2 {
            return Ok(None);
        }

        let known_ids = spec
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<HashSet<_>>();
        for node_id in &node_ids {
            if !known_ids.contains(node_id.as_str()) {
                return Err(PlotError::UnknownNode {
                    node_id: node_id.clone(),
                });
            }
        }

        for window in node_ids.windows(2) {
            let from_node = window[0].as_str();
            let to_node = window[1].as_str();
            if !path_edge_exists(spec, from_node, to_node) {
                return Err(PlotError::InvalidNetworkPath {
                    from_node: window[0].clone(),
                    to_node: window[1].clone(),
                });
            }
        }

        Ok(Some(Self {
            node_ids,
            progress: 0.0,
            breath_phase: 0.0,
        }))
    }

    fn set_progress(&mut self, progress: f64) {
        self.progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
    }

    fn set_breath_phase(&mut self, breath_phase: f64) {
        self.breath_phase = if breath_phase.is_finite() {
            breath_phase.rem_euclid(1.0)
        } else {
            0.0
        };
    }

    fn edge_count(&self) -> usize {
        self.node_ids.len().saturating_sub(1)
    }

    fn current_node_index(&self) -> usize {
        let edge_count = self.edge_count();
        if edge_count == 0 {
            return 0;
        }
        ((self.progress * edge_count as f64).floor() as usize).min(edge_count)
    }

    fn is_traversed_node(&self, node_id: &str) -> bool {
        self.node_ids
            .iter()
            .take(self.current_node_index().saturating_add(1))
            .any(|tracked_id| tracked_id == node_id)
    }

    fn edge_completion(&self, edge_index: usize) -> f64 {
        let edge_count = self.edge_count();
        if edge_count == 0 || edge_index >= edge_count {
            return 0.0;
        }

        let scaled_progress = self.progress * edge_count as f64;
        (scaled_progress - edge_index as f64).clamp(0.0, 1.0)
    }
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
