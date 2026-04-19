/**
 * TypeScript interfaces for navi-wasm spec objects and hit results.
 *
 * These types describe the plain JS objects passed to and returned from the
 * WASM API. Import or reference this file for IDE autocomplete:
 *
 *   /// <reference types="../types/navi_plot_specs.d.ts" />
 *
 * After `just build` the reference is automatically appended to
 * pkg/navi_plot_wasm.d.ts, so consumers of the pkg get types for free.
 */

// ─── Scatter ─────────────────────────────────────────────────────────────────

export interface ScatterPoint {
  x: number;
  y: number;
  name?: string | null;
  label?: string | null;
  /** CSS hex color, e.g. "#ef4444". Falls back to a default palette color. */
  color?: string | null;
  /** Radius in canvas pixels. Default: 5. */
  radius?: number | null;
  properties?: Record<string, string>;
}

export interface ScatterPlotSpec {
  width: number;
  height: number;
  title?: string;
  /** Font family used for chart text. Example: "Roboto, sans-serif". Default: "sans-serif" */
  font_family?: string | null;
  x_label?: string;
  y_label?: string;
  /** [min, max]. Null triggers auto-range from data. */
  x_range?: [number, number] | null;
  /** [min, max]. Null triggers auto-range from data. */
  y_range?: [number, number] | null;
  selected_point_index?: number | null;
  points: ScatterPoint[];
}

// ─── Shared node types ────────────────────────────────────────────────────────

/**
 * Shape used to render tree and network graph nodes.
 * - `"circle"` — circular node (default)
 * - `"square"` — axis-aligned square
 * - `"diamond"` — square rotated 45°
 * - `"triangle"` — upward-pointing triangle
 */
export type NodeShape = "circle" | "square" | "diamond" | "triangle";

export type BuiltinNodeIcon =
  | "star"
  | "galaxy"
  | "planet"
  | "moon"
  | "telescope"
  | "camera"
  | "alert"
  | "archive"
  | "database"
  | "broker"
  | "dish"
  | "spectrograph";

export type NodeMediaFit = "contain" | "cover";

export interface NodeMedia {
  kind: "icon" | "image";
  /** Required when `kind` is `"icon"`. */
  icon?: BuiltinNodeIcon | null;
  /** Required when `kind` is `"image"`. Use a key registered with `register_graph_image`. */
  image_key?: string | null;
  /** Image scaling mode inside the node bounds. Default: `"contain"`. */
  fit?: NodeMediaFit | null;
  /** Relative media size within the node. Valid range: `0.2..=1.0`. Default: `0.7`. */
  scale?: number | null;
  /** Tint color for built-in icons. CSS hex. Default: white. */
  tint_color?: string | null;
  /** Built-in icon shown if an image key is missing or the backend cannot draw images. */
  fallback_icon?: BuiltinNodeIcon | null;
}

export interface GraphNodeStyle {
  /** Node fill color. CSS hex, e.g. "#3b82f6". */
  fill_color?: string | null;
  /** Optional outline color. Defaults to the fill color when stroke_width is set. */
  stroke_color?: string | null;
  /** Outline width in pixels. 0 disables the outline. */
  stroke_width?: number | null;
  /** Node radius in canvas pixels. Must be at least 1. */
  radius?: number | null;
  /** Opacity applied to the node fill, outline, and label. Range: 0..1. */
  opacity?: number | null;
  shape?: NodeShape | null;
  /** Controls whether the label is drawn for this node. */
  label_visible?: boolean | null;
  /** Label text color. Defaults to white for inside labels and black for outside labels. */
  label_color?: string | null;
  /** Render the label centered inside the node shape instead of below it. */
  label_inside?: boolean | null;
}

export interface GraphEdgeStyle {
  /** Edge stroke color. CSS hex, e.g. "#64748b". */
  stroke_color?: string | null;
  /** Edge stroke width in pixels. 0 hides the line. */
  stroke_width?: number | null;
  /** Alternating draw / gap lengths in pixels, e.g. [6, 4] for dashed lines. */
  dash_pattern?: number[] | null;
  /** Opacity applied to the edge stroke, arrowhead, and label. Range: 0..1. */
  opacity?: number | null;
  /** Override arrowhead visibility. Ignored for tree graphs. */
  arrow_visible?: boolean | null;
  /** Controls whether the edge label is drawn when one exists. */
  label_visible?: boolean | null;
  /** Edge label text color. Defaults to the resolved stroke color. */
  label_color?: string | null;
}

export interface SelectionStyle {
  /** Selection ring stroke color. CSS hex. */
  stroke_color?: string | null;
  /** Selection ring stroke width in pixels. 0 hides the ring. */
  stroke_width?: number | null;
  /** Extra padding between the node radius and the selection ring radius. */
  padding?: number | null;
  /** Opacity applied to the selection ring. Range: 0..1. */
  opacity?: number | null;
}

// ─── Tree ─────────────────────────────────────────────────────────────────────

export interface TreeNode {
  id: string;
  label: string;
  name?: string | null;
  color?: string | null;
  /** Shape of the node. Default: inherited from `default_node_style`, otherwise `"circle"` */
  shape?: NodeShape;
  /** Render label centered inside the node shape (white text) instead of below it. Default: inherited, otherwise false */
  label_inside?: boolean;
  /** Per-node style overrides. */
  style?: GraphNodeStyle | null;
  /** Optional built-in icon or registered image rendered inside the node. */
  media?: NodeMedia | null;
  properties?: Record<string, string>;
}

export interface TreeEdge {
  source: string;
  target: string;
  /** Per-edge style overrides. */
  style?: GraphEdgeStyle | null;
}

export interface TreePlotSpec {
  width: number;
  height: number;
  /** HiDPI canvas hint. Supports logical CSS sizing and legacy pre-scaled sizing. Default: 1.0 */
  pixel_ratio?: number;
  title?: string;
  /** Font family used for graph text. Example: "Roboto, sans-serif". Default: "sans-serif" */
  font_family?: string | null;
  root_id: string;
  nodes: TreeNode[];
  edges: TreeEdge[];
  /** Default: 18 */
  node_radius?: number;
  /** Graph-level node style defaults. */
  default_node_style?: GraphNodeStyle | null;
  /** Graph-level edge style defaults. */
  default_edge_style?: GraphEdgeStyle | null;
  /** Selection ring styling for the currently selected node. */
  selection_style?: SelectionStyle | null;
  /** Vertical gap between tree levels in pixels. Default: 90 */
  level_gap?: number;
  /** Horizontal gap between siblings in pixels. Default: 96 */
  sibling_gap?: number;
  /** Margin around the plot area. Default: 32 */
  margin?: number;
  /** Pan offset in canvas pixels. */
  offset_x?: number;
  offset_y?: number;
  selected_node_id?: string | null;
  /** Node IDs whose descendants should be hidden and removed from layout. */
  collapsed_node_ids?: string[];
}

// ─── Line / time-series ───────────────────────────────────────────────────────

export interface LinePoint {
  x: number;
  y: number;
  label?: string | null;
  properties?: Record<string, string>;
}

export interface LineSeries {
  label: string;
  /** CSS hex color. Falls back to auto palette. */
  color?: string | null;
  points: LinePoint[];
  /** Stroke width in pixels. Default: 2 */
  stroke_width?: number;
}

export interface LinePlotSpec {
  width: number;
  height: number;
  title?: string;
  /** Font family used for chart text. Example: "Roboto, sans-serif". Default: "sans-serif" */
  font_family?: string | null;
  x_label?: string;
  y_label?: string;
  x_range?: [number, number] | null;
  y_range?: [number, number] | null;
  series: LineSeries[];
  /** [series_index, point_index]. Null clears selection. */
  selected_point?: [number, number] | null;
  /** Render a circle at each data point. Default: true */
  show_points?: boolean;
  /** Render a series legend. Default: true */
  show_legend?: boolean;
}

// ─── Bar chart ────────────────────────────────────────────────────────────────

export interface BarSeries {
  label: string;
  color?: string | null;
  values: number[];
}

export interface BarChartSpec {
  width: number;
  height: number;
  title?: string;
  /** Font family used for chart text. Example: "Roboto, sans-serif". Default: "sans-serif" */
  font_family?: string | null;
  x_label?: string;
  y_label?: string;
  /** Explicit y-axis maximum. Null auto-scales. */
  y_max?: number | null;
  categories: string[];
  series: BarSeries[];
  /** "grouped" | "stacked". Default: "grouped" */
  variant?: "grouped" | "stacked";
  show_legend?: boolean;
  /** Default: 32 */
  margin?: number;
  /** [series_index, category_index]. Null clears selection. */
  selected_bar?: [number, number] | null;
}

// ─── Heatmap ──────────────────────────────────────────────────────────────────

export interface HeatmapSpec {
  width: number;
  height: number;
  title?: string;
  /** Font family used for chart text. Example: "Roboto, sans-serif". Default: "sans-serif" */
  font_family?: string | null;
  /** One label per row. Length must match cells.length. */
  row_labels?: string[];
  /** One label per column. Length must match cells[0].length. */
  col_labels?: string[];
  /** Row-major 2-D array: cells[row][col] */
  cells: number[][];
  /** [min, max]. Null auto-scales. */
  value_range?: [number, number] | null;
  /** "blue_white_red" | "viridis" | "greens". Default: "blue_white_red" */
  palette?: string;
  /** Render numeric value text inside each cell. Default: true */
  show_values?: boolean;
  /** Default: 32 */
  margin?: number;
  /** [row, col]. Null clears selection. */
  selected_cell?: [number, number] | null;
}

// ─── Network / DAG ────────────────────────────────────────────────────────────

export interface NetworkNode {
  id: string;
  label?: string;
  color?: string | null;
  /**
   * Explicit canvas-relative x position (0..width).
   * Nodes that supply both `x` and `y` are pinned — the FR layout algorithm
   * will not move them. Nodes without x/y are FR-placed around pinned anchors.
   */
  x?: number | null;
  /** Explicit canvas-relative y position (0..height). See `x`. */
  y?: number | null;
  /** Shape of the node. Default: inherited from `default_node_style`, otherwise `"circle"` */
  shape?: NodeShape;
  /** Render label centered inside the node shape (white text) instead of below it. Default: inherited, otherwise false */
  label_inside?: boolean;
  /** Per-node style overrides. */
  style?: GraphNodeStyle | null;
  /** Optional built-in icon or registered image rendered inside the node. */
  media?: NodeMedia | null;
  properties?: Record<string, string>;
}

export interface NetworkEdge {
  source: string;
  target: string;
  label?: string | null;
  color?: string | null;
  weight?: number | null;
  /** Per-edge style overrides. */
  style?: GraphEdgeStyle | null;
}

export interface NetworkPlotSpec {
  width: number;
  height: number;
  /** HiDPI canvas hint. Supports logical CSS sizing and legacy pre-scaled sizing. Default: 1.0 */
  pixel_ratio?: number;
  title?: string;
  /** Font family used for graph text. Example: "Roboto, sans-serif". Default: "sans-serif" */
  font_family?: string | null;
  nodes: NetworkNode[];
  edges: NetworkEdge[];
  /** Default: 16 */
  node_radius?: number;
  /** Graph-level node style defaults. */
  default_node_style?: GraphNodeStyle | null;
  /** Graph-level edge style defaults. */
  default_edge_style?: GraphEdgeStyle | null;
  /** Selection ring styling for the currently selected node. */
  selection_style?: SelectionStyle | null;
  /** Default: 40 */
  margin?: number;
  /** Pan offset in canvas pixels. */
  offset_x?: number;
  offset_y?: number;
  selected_node_id?: string | null;
  /** FR layout iterations. Ignored for nodes that supply x/y. Default: 100 */
  layout_iterations?: number;
  /** Multiplier on computed FR spring length. Higher = more spacing. Default: 1.0 */
  spring_length_scale?: number;
  /** Multiplier on initial FR temperature. Higher = more initial movement. Default: 1.0 */
  temperature_scale?: number;
  /** Per-iteration temperature cooling factor (0–1). Lower = faster convergence. Default: 0.92 */
  cooling_rate?: number;
  /** Draw arrowheads on directed edges. Default: true */
  show_arrows?: boolean;
  /** Draw node labels. Default: true */
  show_labels?: boolean;
}

export interface NetworkView {
  zoom: number;
  translate_x: number;
  translate_y: number;
}

export type NetworkFocusMode = "node_and_neighbors";

export interface NetworkFocusOptions {
  mode?: NetworkFocusMode | null;
  /** Screen padding in pixels. Default: 48 */
  padding?: number | null;
  /** Minimum world-space span to keep isolated nodes from over-zooming. Default: 160 */
  min_world_span?: number | null;
}

// ─── Hit results ──────────────────────────────────────────────────────────────

export interface ScatterHit {
  index: number;
}

export interface TreeHit {
  node_id: string;
}

export interface LineHit {
  series_index: number;
  point_index: number;
}

export interface BarHit {
  series_index: number;
  category_index: number;
}

export interface HeatmapHit {
  row: number;
  col: number;
}

export interface NetworkHit {
  node_id: string;
}

// ─── Error shape ──────────────────────────────────────────────────────────────

/**
 * All render/pick/pan functions throw a `PlotError` on invalid input.
 * It extends `Error`, so `instanceof Error` is true and `.message` is set.
 * Use `.code` for programmatic error handling:
 *
 * ```ts
 * try { wasm.render_scatter(id, spec) }
 * catch (e: unknown) {
 *   const err = e as PlotError;
 *   if (err.code === "EMPTY_SCATTER_DATA") { ... }
 * }
 * ```
 */
export interface PlotError extends Error {
  /** Machine-readable error code in SCREAMING_SNAKE_CASE. */
  code: string;
}
