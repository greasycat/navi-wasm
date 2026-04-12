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

mod color;
mod error;
pub(crate) mod node;
mod scatter;
mod tree;
mod types;
pub(crate) mod viewport;
mod line;
mod bar;
mod heatmap;
mod network;

pub use error::PlotError;

// ─── Scatter ─────────────────────────────────────────────────────────────────

/// Render a scatter plot onto `root`. Validates the spec first.
pub use scatter::render_scatter_on;
/// Return the updated spec with `x_range`/`y_range` shifted by the given pixel delta.
pub use scatter::pan_scatter_spec;
/// Return the index of the nearest point within hit radius, or `None`.
pub use scatter::pick_scatter_point;
/// Stateful scatter session that caches the resolved viewport between renders.
pub use scatter::ScatterSession;

// ─── Tree ─────────────────────────────────────────────────────────────────────

/// Render a rooted tree onto `root`.
pub use tree::render_tree_on;
/// Return the updated spec with `offset_x`/`offset_y` shifted by the given pixel delta.
pub use tree::pan_tree_spec;
/// Return the ID of the node at `(canvas_x, canvas_y)`, or `None`.
pub use tree::pick_tree_node;

// ─── Line ─────────────────────────────────────────────────────────────────────

/// Render a line/time-series chart onto `root`.
pub use line::render_line_on;
/// Return the updated spec with axis ranges panned by the pixel delta.
pub use line::pan_line_spec;
/// Return `[series_index, point_index]` of the nearest data point, or `None`.
pub use line::pick_line_point;
/// Stateful line session that caches the resolved viewport between renders.
pub use line::LineSession;

// ─── Bar ──────────────────────────────────────────────────────────────────────

/// Render a bar chart (grouped or stacked) onto `root`.
pub use bar::render_bar_on;
/// Return `[series_index, category_index]` for the bar at `(canvas_x, canvas_y)`, or `None`.
pub use bar::pick_bar;

// ─── Heatmap ──────────────────────────────────────────────────────────────────

/// Render a heatmap matrix onto `root`.
pub use heatmap::render_heatmap_on;
/// Return `[row, col]` for the cell at `(canvas_x, canvas_y)`, or `None`.
pub use heatmap::pick_heatmap_cell;

// ─── Network / DAG ────────────────────────────────────────────────────────────

/// Render a network/DAG onto `root`. Runs Fruchterman-Reingold layout if needed.
pub use network::render_network_on;
/// Return the updated spec with `offset_x`/`offset_y` shifted by the given pixel delta.
pub use network::pan_network_spec;
/// Return the ID of the node nearest to `(canvas_x, canvas_y)`, or `None`.
pub use network::pick_network_node;
/// Stateful network session that caches the FR layout between renders.
pub use network::NetworkSession;

pub use types::{NodeShape, ScatterPlotSpec, ScatterPoint, TreeEdge, TreeNode, TreePlotSpec};
pub use types::{
    BarChartSpec, BarSeries, BarVariant,
    HeatmapSpec,
    LinePlotSpec, LineSeries, LinePoint,
    NetworkEdge, NetworkNode, NetworkPlotSpec,
};

use plotters::coord::Shift;
use plotters::prelude::DrawingArea;

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

pub type PlotArea<DB> = DrawingArea<DB, Shift>;
