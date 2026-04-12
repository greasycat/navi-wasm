use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Specification for a scatter plot.
///
/// Pass this as the `spec` argument to `render_scatter` / `create_scatter_session`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScatterPlotSpec {
    /// Canvas width in pixels.
    pub width: u32,
    /// Canvas height in pixels.
    pub height: u32,
    /// Optional chart title drawn at the top.
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub x_label: String,
    #[serde(default)]
    pub y_label: String,
    /// `[min, max]` for the x-axis. `None` auto-scales from data.
    #[serde(default)]
    pub x_range: Option<[f64; 2]>,
    /// `[min, max]` for the y-axis. `None` auto-scales from data.
    #[serde(default)]
    pub y_range: Option<[f64; 2]>,
    /// Index into `points` of the currently selected point. `None` clears selection.
    #[serde(default)]
    pub selected_point_index: Option<usize>,
    pub points: Vec<ScatterPoint>,
}

/// A single data point in a scatter plot. Both `x` and `y` must be finite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScatterPoint {
    pub x: f64,
    pub y: f64,
    /// Human-readable name shown in the details panel.
    #[serde(default)]
    pub name: Option<String>,
    /// Short label rendered next to the point on the canvas.
    #[serde(default)]
    pub label: Option<String>,
    /// CSS hex color string (e.g. `"#ef4444"`). Falls back to a palette color.
    #[serde(default)]
    pub color: Option<String>,
    /// Radius in canvas pixels. Default: 5.
    #[serde(default)]
    pub radius: Option<u32>,
    /// Arbitrary key/value metadata surfaced in the details panel.
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreePlotSpec {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub title: String,
    pub root_id: String,
    pub nodes: Vec<TreeNode>,
    pub edges: Vec<TreeEdge>,
    #[serde(default = "default_tree_node_radius")]
    pub node_radius: u32,
    #[serde(default = "default_tree_level_gap")]
    pub level_gap: u32,
    #[serde(default = "default_tree_sibling_gap")]
    pub sibling_gap: u32,
    #[serde(default = "default_tree_margin")]
    pub margin: u32,
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
    #[serde(default)]
    pub selected_node_id: Option<String>,
    /// Device pixel ratio used to scale fonts for HiDPI canvases. Default: 1.0.
    /// Set to `window.devicePixelRatio` in JS and multiply `width`, `height`,
    /// `margin`, `node_radius`, `level_gap`, and `sibling_gap` by the same factor.
    #[serde(default = "default_pixel_ratio")]
    pub pixel_ratio: f64,
}

/// Shape used to render tree and network nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeShape {
    /// Circular node (default).
    #[default]
    Circle,
    /// Axis-aligned square node.
    Square,
    /// Diamond (square rotated 45°).
    Diamond,
    /// Upward-pointing triangle.
    Triangle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeNode {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub label: String,
    #[serde(default)]
    pub color: Option<String>,
    /// Shape of the node. Default: `"circle"`.
    #[serde(default)]
    pub shape: NodeShape,
    /// Render the label inside the node shape instead of below it. Default: `false`.
    #[serde(default)]
    pub label_inside: bool,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeEdge {
    pub source: String,
    pub target: String,
}

fn default_tree_node_radius() -> u32 {
    18
}

fn default_tree_level_gap() -> u32 {
    90
}

fn default_tree_sibling_gap() -> u32 {
    96
}

fn default_tree_margin() -> u32 {
    32
}

// ─── Line chart ─────────────────────────────────────────────────────────────

/// Specification for a line / time-series chart.
///
/// Supports multiple series with automatic color assignment or explicit colors.
/// Drag to pan; click a point to select it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinePlotSpec {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub x_label: String,
    #[serde(default)]
    pub y_label: String,
    /// `[min, max]` for x. `None` auto-scales across all series.
    #[serde(default)]
    pub x_range: Option<[f64; 2]>,
    /// `[min, max]` for y. `None` auto-scales across all series.
    #[serde(default)]
    pub y_range: Option<[f64; 2]>,
    pub series: Vec<LineSeries>,
    /// `[series_index, point_index]`. `None` clears selection.
    #[serde(default)]
    pub selected_point: Option<[usize; 2]>,
    /// Render a circle marker at each data point. Default: `true`.
    #[serde(default = "default_true")]
    pub show_points: bool,
    /// Render a series legend. Default: `true`.
    #[serde(default = "default_true")]
    pub show_legend: bool,
}

/// One data series within a [`LinePlotSpec`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineSeries {
    pub label: String,
    /// CSS hex color. `None` picks from the automatic palette.
    #[serde(default)]
    pub color: Option<String>,
    pub points: Vec<LinePoint>,
    /// Stroke width in pixels. Default: 2.
    #[serde(default = "default_line_stroke_width")]
    pub stroke_width: u32,
}

/// A single data point in a line series. Both `x` and `y` must be finite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinePoint {
    pub x: f64,
    pub y: f64,
    /// Optional label shown in the details panel when selected.
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

fn default_true() -> bool {
    true
}

fn default_line_stroke_width() -> u32 {
    2
}

// ─── Bar chart ───────────────────────────────────────────────────────────────

/// Specification for a bar chart (grouped or stacked).
///
/// Every series must have exactly `categories.len()` values.
/// Stacked mode does not support negative values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BarChartSpec {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub x_label: String,
    #[serde(default)]
    pub y_label: String,
    /// Explicit y-axis upper bound. `None` auto-scales.
    #[serde(default)]
    pub y_max: Option<f64>,
    /// Category labels for the x-axis. Length must match `series[*].values`.
    pub categories: Vec<String>,
    pub series: Vec<BarSeries>,
    /// `"grouped"` (default) or `"stacked"`.
    #[serde(default)]
    pub variant: BarVariant,
    #[serde(default = "default_true")]
    pub show_legend: bool,
    #[serde(default = "default_bar_margin")]
    pub margin: u32,
    /// `[series_index, category_index]`. `None` clears selection.
    #[serde(default)]
    pub selected_bar: Option<[usize; 2]>,
}

/// One data series within a [`BarChartSpec`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BarSeries {
    pub label: String,
    /// CSS hex color. `None` picks from the automatic palette.
    #[serde(default)]
    pub color: Option<String>,
    /// One value per category. Length must equal `BarChartSpec::categories.len()`.
    pub values: Vec<f64>,
}

/// Bar layout variant — serialises as `"grouped"` / `"stacked"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BarVariant {
    #[default]
    Grouped,
    Stacked,
}

fn default_bar_margin() -> u32 {
    32
}

// ─── Heatmap ─────────────────────────────────────────────────────────────────

/// Specification for a heatmap (matrix) chart.
///
/// All rows must have the same number of columns and all values must be finite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatmapSpec {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub title: String,
    /// One label per row. Length must equal `cells.len()` when non-empty.
    #[serde(default)]
    pub row_labels: Vec<String>,
    /// One label per column. Length must equal `cells[0].len()` when non-empty.
    #[serde(default)]
    pub col_labels: Vec<String>,
    /// Row-major 2-D grid: `cells[row][col]`. All rows must have equal length.
    pub cells: Vec<Vec<f64>>,
    /// `[min, max]` for color mapping. `None` auto-scales from cell values.
    #[serde(default)]
    pub value_range: Option<[f64; 2]>,
    /// Color palette: `"blue_white_red"` (default), `"viridis"`, or `"greens"`.
    #[serde(default = "default_heatmap_palette")]
    pub palette: String,
    /// Render the numeric value inside each cell. Default: `true`.
    #[serde(default = "default_true")]
    pub show_values: bool,
    #[serde(default = "default_heatmap_margin")]
    pub margin: u32,
    /// `[row, col]`. `None` clears selection.
    #[serde(default)]
    pub selected_cell: Option<[usize; 2]>,
}

fn default_heatmap_palette() -> String {
    "blue_white_red".to_string()
}

fn default_heatmap_margin() -> u32 {
    32
}

// ─── Network / DAG ───────────────────────────────────────────────────────────

/// Specification for a network or directed-acyclic-graph chart.
///
/// Validation is permissive: cycles, multi-parent nodes, and disconnected
/// sub-graphs are all accepted.
///
/// **Layout:** nodes that provide both `x` and `y` are pinned at those
/// coordinates. Nodes without `x`/`y` are positioned automatically using
/// Fruchterman-Reingold. Mixed pinned+free is supported — you can anchor key
/// nodes while letting others be placed around them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkPlotSpec {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub title: String,
    pub nodes: Vec<NetworkNode>,
    pub edges: Vec<NetworkEdge>,
    /// Node circle radius in pixels. Default: 16.
    #[serde(default = "default_network_node_radius")]
    pub node_radius: u32,
    /// Margin around the drawable area. Default: 40.
    #[serde(default = "default_network_margin")]
    pub margin: u32,
    /// Pan offset in canvas pixels (updated by `pan_network`).
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
    /// ID of the currently selected node. `None` clears selection.
    #[serde(default)]
    pub selected_node_id: Option<String>,
    /// Fruchterman-Reingold iterations for free nodes. Default: 100.
    #[serde(default = "default_fr_iterations")]
    pub layout_iterations: u32,
    /// Draw arrowheads on directed edges. Default: `true`.
    #[serde(default = "default_true")]
    pub show_arrows: bool,
    /// Draw node labels. Default: `true`.
    #[serde(default = "default_true")]
    pub show_labels: bool,
    /// Device pixel ratio used to scale fonts for HiDPI canvases. Default: 1.0.
    #[serde(default = "default_pixel_ratio")]
    pub pixel_ratio: f64,
}

/// A node in a [`NetworkPlotSpec`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkNode {
    /// Unique node identifier used in edge `source`/`target` fields.
    pub id: String,
    /// Display label rendered on the canvas. Defaults to empty string.
    #[serde(default)]
    pub label: String,
    /// CSS hex color. `None` picks from the automatic palette.
    #[serde(default)]
    pub color: Option<String>,
    /// Explicit canvas-relative x coordinate (pixels, 0..width).
    /// When a node provides both `x` and `y`, it is pinned and not moved by
    /// the FR layout algorithm.
    #[serde(default)]
    pub x: Option<f64>,
    /// Explicit canvas-relative y coordinate (pixels, 0..height). See `x`.
    #[serde(default)]
    pub y: Option<f64>,
    /// Shape of the node. Default: `"circle"`.
    #[serde(default)]
    pub shape: NodeShape,
    /// Render the label inside the node shape instead of below it. Default: `false`.
    #[serde(default)]
    pub label_inside: bool,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

/// A directed edge in a [`NetworkPlotSpec`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkEdge {
    /// ID of the source node.
    pub source: String,
    /// ID of the target node.
    pub target: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub weight: Option<f64>,
}

fn default_network_node_radius() -> u32 {
    16
}

fn default_network_margin() -> u32 {
    40
}

fn default_fr_iterations() -> u32 {
    100
}

pub(crate) fn default_pixel_ratio() -> f64 {
    1.0
}
