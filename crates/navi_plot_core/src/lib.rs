//! Pure-Rust plotting core for navi-wasm.
//!
//! This crate contains rendering, layout, hit-testing, and validation logic for
//! six chart types: scatter, tree, line, bar, heatmap, and network/DAG.
//! It has no browser or WASM dependencies and can be tested natively with
//! `cargo test -p navi_plot_core`.
//!
//! The companion crate `navi_plot_wasm` wraps these functions with
//! `wasm_bindgen` so they can be called from JavaScript.
#![forbid(unsafe_code)]

mod bar;
mod color;
mod error;
mod graph_style;
mod heatmap;
mod line;
mod network;
pub(crate) mod node;
mod scatter;
mod tree;
mod types;
pub(crate) mod viewport;

pub use error::PlotError;

// ─── Scatter ─────────────────────────────────────────────────────────────────

/// Return the updated spec with `x_range`/`y_range` shifted by the given pixel delta.
pub use scatter::pan_scatter_spec;
/// Return the index of the nearest point within hit radius, or `None`.
pub use scatter::pick_scatter_point;
/// Render a scatter plot onto `root`. Validates the spec first.
pub use scatter::render_scatter_on;
/// Stateful scatter session that caches the resolved viewport between renders.
pub use scatter::ScatterSession;

// ─── Tree ─────────────────────────────────────────────────────────────────────

/// Return the updated spec with `offset_x`/`offset_y` shifted by the given pixel delta.
pub use tree::pan_tree_spec;
/// Return the ID of the node at `(canvas_x, canvas_y)`, or `None`.
pub use tree::pick_tree_node;
/// Render a rooted tree onto `root`.
pub use tree::render_tree_on;
/// Return resolved tree node positions and media metadata for frontend overlays.
pub use tree::tree_render_nodes;
/// Stateful tree session that caches layout and view transforms between renders.
pub use tree::TreeSession;

// ─── Line ─────────────────────────────────────────────────────────────────────

/// Return the updated spec with axis ranges panned by the pixel delta.
pub use line::pan_line_spec;
/// Return `[series_index, point_index]` of the nearest data point, or `None`.
pub use line::pick_line_point;
/// Render a line/time-series chart onto `root`.
pub use line::render_line_on;
/// Stateful line session that caches the resolved viewport between renders.
pub use line::LineSession;

// ─── Bar ──────────────────────────────────────────────────────────────────────

/// Return `[series_index, category_index]` for the bar at `(canvas_x, canvas_y)`, or `None`.
pub use bar::pick_bar;
/// Render a bar chart (grouped or stacked) onto `root`.
pub use bar::render_bar_on;
/// Stateful bar session that caches resolved series and view transforms.
pub use bar::BarSession;

// ─── Heatmap ──────────────────────────────────────────────────────────────────

/// Return `[row, col]` for the cell at `(canvas_x, canvas_y)`, or `None`.
pub use heatmap::pick_heatmap_cell;
/// Render a heatmap matrix onto `root`.
pub use heatmap::render_heatmap_on;
/// Stateful heatmap session that caches layout and view transforms.
pub use heatmap::HeatmapSession;

// ─── Network / DAG ────────────────────────────────────────────────────────────

/// Return a view that fits the target node and its local neighborhood.
pub use network::focus_network_view;
/// Return resolved network node positions and media metadata for frontend overlays.
pub use network::network_render_nodes;
/// Return the updated spec with `offset_x`/`offset_y` shifted by the given pixel delta.
pub use network::pan_network_spec;
/// Return the node body or toggle badge hit at `(canvas_x, canvas_y)`, or `None`.
pub use network::pick_network_hit;
/// Return the ID of the node nearest to `(canvas_x, canvas_y)`, or `None`.
pub use network::pick_network_node;
/// Render a network/DAG onto `root`. Runs Fruchterman-Reingold layout if needed.
pub use network::render_network_on;
/// Stateful network session that caches the FR layout between renders.
pub use network::NetworkSession;

pub use network::{NetworkPickHit, NetworkPickKind};
pub use node::{GraphNodeRenderInfo, ResolvedNodeMedia, ResolvedNodeMediaKind};
pub use types::{
    BarChartSpec, BarSeries, BarVariant, BuiltinNodeIcon, HeatmapSpec, LinePlotSpec, LinePoint,
    LineSeries, NetworkEdge, NetworkFocusMode, NetworkFocusOptions, NetworkNode, NetworkPlotSpec,
    NetworkView, NodeMedia, NodeMediaFit, NodeMediaKind,
};
pub use types::{
    GraphEdgeStyle, GraphNodeStyle, NodeShape, ScatterPlotSpec, ScatterPoint, SelectionStyle,
    TreeEdge, TreeNode, TreePlotSpec,
};

use plotters::coord::Shift;
use plotters::prelude::DrawingArea;
use plotters::style::FontFamily;

pub(crate) fn ensure_dimensions(width: u32, height: u32) -> Result<(), PlotError> {
    if width == 0 || height == 0 {
        return Err(PlotError::InvalidDimensions { width, height });
    }

    Ok(())
}

pub(crate) fn backend_error<DB>(error: plotters::drawing::DrawingAreaErrorKind<DB>) -> PlotError
where
    DB: std::fmt::Debug + std::error::Error + Send + Sync,
{
    PlotError::Backend {
        message: format!("{error:?}"),
    }
}

pub(crate) fn font_family<'a>(value: Option<&'a str>) -> FontFamily<'a> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(FontFamily::from)
        .unwrap_or(FontFamily::Monospace)
}

pub type PlotArea<DB> = DrawingArea<DB, Shift>;
