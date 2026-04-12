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

// ─── Tree ─────────────────────────────────────────────────────────────────────

export interface TreeNode {
  id: string;
  label: string;
  name?: string | null;
  color?: string | null;
  /** Shape of the node. Default: `"circle"` */
  shape?: NodeShape;
  /** Render label centered inside the node shape (white text) instead of below it. Default: false */
  label_inside?: boolean;
  properties?: Record<string, string>;
}

export interface TreeEdge {
  source: string;
  target: string;
}

export interface TreePlotSpec {
  width: number;
  height: number;
  title?: string;
  root_id: string;
  nodes: TreeNode[];
  edges: TreeEdge[];
  /** Default: 18 */
  node_radius?: number;
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
  /** Shape of the node. Default: `"circle"` */
  shape?: NodeShape;
  /** Render label centered inside the node shape (white text) instead of below it. Default: false */
  label_inside?: boolean;
  properties?: Record<string, string>;
}

export interface NetworkEdge {
  source: string;
  target: string;
  label?: string | null;
  color?: string | null;
  weight?: number | null;
}

export interface NetworkPlotSpec {
  width: number;
  height: number;
  title?: string;
  nodes: NetworkNode[];
  edges: NetworkEdge[];
  /** Default: 16 */
  node_radius?: number;
  /** Default: 40 */
  margin?: number;
  /** Pan offset in canvas pixels. */
  offset_x?: number;
  offset_y?: number;
  selected_node_id?: string | null;
  /** FR layout iterations. Ignored for nodes that supply x/y. Default: 100 */
  layout_iterations?: number;
  /** Draw arrowheads on directed edges. Default: true */
  show_arrows?: boolean;
  /** Draw node labels. Default: true */
  show_labels?: boolean;
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
