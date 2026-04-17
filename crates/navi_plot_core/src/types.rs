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
    /// Font family used for chart text. Example: `"Roboto, sans-serif"`.
    /// Default: `"sans-serif"`.
    #[serde(default)]
    pub font_family: Option<String>,
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
    /// Font family used for graph text. Example: `"Roboto, sans-serif"`.
    /// Default: `"sans-serif"`.
    #[serde(default)]
    pub font_family: Option<String>,
    pub root_id: String,
    pub nodes: Vec<TreeNode>,
    pub edges: Vec<TreeEdge>,
    #[serde(default = "default_tree_node_radius")]
    pub node_radius: u32,
    /// Graph-level node style defaults. Per-node legacy fields and per-node
    /// style overrides still take precedence when provided.
    #[serde(default)]
    pub default_node_style: Option<GraphNodeStyle>,
    /// Graph-level edge style defaults. Per-edge style overrides still take
    /// precedence when provided.
    #[serde(default)]
    pub default_edge_style: Option<GraphEdgeStyle>,
    /// Selection ring styling for the currently selected node.
    #[serde(default)]
    pub selection_style: Option<SelectionStyle>,
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
    /// Node IDs whose descendants should be hidden and removed from layout.
    #[serde(default)]
    pub collapsed_node_ids: Vec<String>,
    /// HiDPI canvas hint. Default: 1.0.
    /// JS callers may either provide logical CSS `width`/`height` with this ratio,
    /// or pre-scale all canvas-space values manually for legacy callers.
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

/// Built-in icon set for tree and network nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinNodeIcon {
    Star,
    Galaxy,
    Planet,
    Moon,
    Telescope,
    Camera,
    Alert,
    Archive,
    Database,
    Broker,
    Dish,
    Spectrograph,
}

/// Scaling mode for node images inside the node shape bounds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeMediaFit {
    #[default]
    Contain,
    Cover,
}

/// Discriminant for node media content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeMediaKind {
    Icon,
    Image,
}

/// Optional icon or registered image content rendered inside a node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeMedia {
    pub kind: NodeMediaKind,
    /// Built-in icon name. Required when `kind == "icon"`.
    #[serde(default)]
    pub icon: Option<BuiltinNodeIcon>,
    /// Registered image key. Required when `kind == "image"`.
    #[serde(default)]
    pub image_key: Option<String>,
    /// Image sizing mode inside the node bounds. Default: `contain`.
    #[serde(default)]
    pub fit: NodeMediaFit,
    /// Relative media size within the node. Valid range: `0.2..=1.0`.
    #[serde(default)]
    pub scale: Option<f64>,
    /// Media tint color. Used by built-in icons. Default: white.
    #[serde(default)]
    pub tint_color: Option<String>,
    /// Built-in icon shown when an image is unavailable.
    #[serde(default)]
    pub fallback_icon: Option<BuiltinNodeIcon>,
}

/// Shared style overrides for tree and network graph nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GraphNodeStyle {
    /// Node fill color. CSS hex (e.g. `"#3b82f6"`).
    #[serde(default)]
    pub fill_color: Option<String>,
    /// Optional outline color. Defaults to the fill color when `stroke_width`
    /// is set but `stroke_color` is omitted.
    #[serde(default)]
    pub stroke_color: Option<String>,
    /// Outline width in pixels. `0` disables the outline. Default: `0`.
    #[serde(default)]
    pub stroke_width: Option<f64>,
    /// Node radius in canvas pixels. Must be at least `1`.
    #[serde(default)]
    pub radius: Option<f64>,
    /// Opacity applied to the node fill, outline, and label. `0..=1`.
    #[serde(default)]
    pub opacity: Option<f64>,
    /// Shape override for the node.
    #[serde(default)]
    pub shape: Option<NodeShape>,
    /// Controls whether the label is drawn for this node. Default: inherited.
    #[serde(default)]
    pub label_visible: Option<bool>,
    /// Label text color. Defaults to white for inside labels and black for
    /// labels rendered outside the node.
    #[serde(default)]
    pub label_color: Option<String>,
    /// Render the label centered inside the node shape instead of below it.
    #[serde(default)]
    pub label_inside: Option<bool>,
    /// Drop-shadow color. CSS hex (e.g. `"#000000"`). No shadow when omitted.
    #[serde(default)]
    pub shadow_color: Option<String>,
    /// Shadow blur radius in pixels (`0` = hard shadow). Default: `6`.
    #[serde(default)]
    pub shadow_blur: Option<f64>,
    /// Shadow horizontal offset in pixels. Default: `2`.
    #[serde(default)]
    pub shadow_offset_x: Option<f64>,
    /// Shadow vertical offset in pixels. Default: `3`.
    #[serde(default)]
    pub shadow_offset_y: Option<f64>,
    /// Shadow opacity `0..=1`. Default: `0.28`.
    #[serde(default)]
    pub shadow_opacity: Option<f64>,
}

/// Shared style overrides for tree and network graph edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GraphEdgeStyle {
    /// Edge stroke color. CSS hex (e.g. `"#64748b"`).
    #[serde(default)]
    pub stroke_color: Option<String>,
    /// Edge stroke width in pixels. `0` hides the line.
    #[serde(default)]
    pub stroke_width: Option<f64>,
    /// Alternating draw / gap lengths in pixels, e.g. `[6, 4]` for dashed lines.
    #[serde(default)]
    pub dash_pattern: Option<Vec<f64>>,
    /// Opacity applied to the edge stroke, arrowhead, and label. `0..=1`.
    #[serde(default)]
    pub opacity: Option<f64>,
    /// Override arrowhead visibility. Ignored for trees.
    #[serde(default)]
    pub arrow_visible: Option<bool>,
    /// Controls whether the edge label is drawn when one exists.
    #[serde(default)]
    pub label_visible: Option<bool>,
    /// Edge label text color. Defaults to the resolved stroke color.
    #[serde(default)]
    pub label_color: Option<String>,
}

/// Style overrides for the selected-node ring used by tree and network graphs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SelectionStyle {
    /// Selection ring stroke color. CSS hex.
    #[serde(default)]
    pub stroke_color: Option<String>,
    /// Selection ring stroke width in pixels. `0` hides the ring.
    #[serde(default)]
    pub stroke_width: Option<f64>,
    /// Extra padding between the node radius and the selection ring radius.
    #[serde(default)]
    pub padding: Option<f64>,
    /// Opacity applied to the selection ring. `0..=1`.
    #[serde(default)]
    pub opacity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeNode {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub label: String,
    #[serde(default)]
    pub color: Option<String>,
    /// Shape of the node. When omitted, inherits from `default_node_style` or
    /// falls back to `"circle"`.
    #[serde(default)]
    pub shape: Option<NodeShape>,
    /// Render the label inside the node shape instead of below it.
    #[serde(default)]
    pub label_inside: Option<bool>,
    /// Per-node style overrides.
    #[serde(default)]
    pub style: Option<GraphNodeStyle>,
    /// Optional icon or registered image rendered inside the node.
    #[serde(default)]
    pub media: Option<NodeMedia>,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TreeEdge {
    pub source: String,
    pub target: String,
    /// Per-edge style overrides.
    #[serde(default)]
    pub style: Option<GraphEdgeStyle>,
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
    /// Font family used for chart text. Example: `"Roboto, sans-serif"`.
    /// Default: `"sans-serif"`.
    #[serde(default)]
    pub font_family: Option<String>,
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
    /// Font family used for chart text. Example: `"Roboto, sans-serif"`.
    /// Default: `"sans-serif"`.
    #[serde(default)]
    pub font_family: Option<String>,
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
    /// Font family used for chart text. Example: `"Roboto, sans-serif"`.
    /// Default: `"sans-serif"`.
    #[serde(default)]
    pub font_family: Option<String>,
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
    /// Font family used for graph text. Example: `"Roboto, sans-serif"`.
    /// Default: `"sans-serif"`.
    #[serde(default)]
    pub font_family: Option<String>,
    pub nodes: Vec<NetworkNode>,
    pub edges: Vec<NetworkEdge>,
    /// Node circle radius in pixels. Default: 16.
    #[serde(default = "default_network_node_radius")]
    pub node_radius: u32,
    /// Graph-level node style defaults. Per-node legacy fields and per-node
    /// style overrides still take precedence when provided.
    #[serde(default)]
    pub default_node_style: Option<GraphNodeStyle>,
    /// Graph-level edge style defaults. Per-edge legacy fields and per-edge
    /// style overrides still take precedence when provided.
    #[serde(default)]
    pub default_edge_style: Option<GraphEdgeStyle>,
    /// Selection ring styling for the currently selected node.
    #[serde(default)]
    pub selection_style: Option<SelectionStyle>,
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
    /// HiDPI canvas hint. Default: 1.0.
    /// JS callers may either provide logical CSS `width`/`height` with this ratio,
    /// or pre-scale all canvas-space values manually for legacy callers.
    #[serde(default = "default_pixel_ratio")]
    pub pixel_ratio: f64,
}

/// Camera state for a network graph session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkView {
    /// Screen zoom factor. `1.0` means one world unit equals one screen pixel.
    pub zoom: f64,
    /// Screen-space translation in pixels.
    pub translate_x: f64,
    /// Screen-space translation in pixels.
    pub translate_y: f64,
}

/// Focus behavior for computing a target camera view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NetworkFocusOptions {
    #[serde(default)]
    pub mode: NetworkFocusMode,
    /// Screen padding in pixels around the focus bounds.
    #[serde(default = "default_network_focus_padding")]
    pub padding: f64,
    /// Minimum world-space span to keep isolated nodes from over-zooming.
    #[serde(default = "default_network_focus_min_world_span")]
    pub min_world_span: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NetworkFocusMode {
    #[default]
    NodeAndNeighbors,
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
    /// Shape of the node. When omitted, inherits from `default_node_style` or
    /// falls back to `"circle"`.
    #[serde(default)]
    pub shape: Option<NodeShape>,
    /// Render the label inside the node shape instead of below it.
    #[serde(default)]
    pub label_inside: Option<bool>,
    /// Per-node style overrides.
    #[serde(default)]
    pub style: Option<GraphNodeStyle>,
    /// Optional icon or registered image rendered inside the node.
    #[serde(default)]
    pub media: Option<NodeMedia>,
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
    /// Per-edge style overrides.
    #[serde(default)]
    pub style: Option<GraphEdgeStyle>,
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

fn default_network_focus_padding() -> f64 {
    48.0
}

fn default_network_focus_min_world_span() -> f64 {
    160.0
}
