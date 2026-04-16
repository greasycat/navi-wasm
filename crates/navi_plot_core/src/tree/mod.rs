use crate::graph_style::{
    edge_line_segments, resolve_edge_style, resolve_node_style, resolve_selection_style,
    EdgeStyleContext, NodeStyleContext, ResolvedNodeStyle,
};
use crate::node::{self, GraphNodeRenderInfo, ResolvedNodeMedia};
use crate::viewport::{PixelBounds, ScreenTransform};
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError, TreePlotSpec};
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::Dfs;
use petgraph::Direction;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::collections::{BTreeMap, BTreeSet};

mod layout;
mod render;
mod session;
#[cfg(test)]
mod tests;

use self::layout::*;
use self::render::*;

pub use self::render::{pan_tree_spec, pick_tree_node, render_tree_on, tree_render_nodes};
pub use self::session::TreeSession;

const DEFAULT_NODE_COLOR: RGBColor = RGBColor(14, 116, 144);
const DEFAULT_EDGE_COLOR: RGBColor = RGBColor(100, 116, 139);
const SELECTION_RING_PADDING: i32 = 8;
const COLLAPSE_MARKER_FILL: RGBColor = RGBColor(15, 23, 42);
const COLLAPSE_MARKER_STROKE: RGBColor = RGBColor(255, 255, 255);

#[derive(Debug, Clone, Copy, PartialEq)]
struct LayoutPoint {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct ValidatedTree {
    node_ids: BTreeSet<String>,
    children_by_parent: BTreeMap<String, Vec<String>>,
    parent_by_child: BTreeMap<String, String>,
}

impl ValidatedTree {
    fn has_children(&self, node_id: &str) -> bool {
        self.children_by_parent
            .get(node_id)
            .is_some_and(|children| !children.is_empty())
    }
}

#[derive(Debug, Clone, Default)]
struct VisibleTree {
    children_by_parent: BTreeMap<String, Vec<String>>,
    collapsed_marker_node_ids: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct ResolvedTreeNode {
    style: ResolvedNodeStyle,
    media: Option<ResolvedNodeMedia>,
}

#[derive(Debug, Clone)]
struct TreeTransition {
    from_layout: BTreeMap<String, LayoutPoint>,
    from_visible: VisibleTree,
    from_selected_node_id: Option<String>,
    anchor_node_id: String,
}
