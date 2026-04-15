# navi-wasm

Rust/WASM plotting library for six browser canvas chart types:

| Chart | Pan | Click-to-select | Session API |
|-------|-----|-----------------|-------------|
| Scatter | ✓ | point | ✓ |
| Rooted tree | ✓ | node | — |
| Line / time-series | ✓ | point | ✓ |
| Bar (grouped / stacked) | — | bar | — |
| Heatmap | — | cell | — |
| Network / DAG | ✓ | node | ✓ |

The repository is a Cargo workspace:

- `crates/navi_plot_core` — pure-Rust rendering, layout, validation, hit-testing
- `crates/navi_plot_wasm` — `wasm-bindgen` bindings for HTML canvas
- `crates/server` — tiny Axum static-file server for the demo
- `demo/` — static browser demo
- `types/navi_plot_specs.d.ts` — TypeScript interfaces for all spec objects

---

## Requirements

- Rust stable (`rustup target add wasm32-unknown-unknown`)
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) (`cargo install wasm-pack --locked`)
- [`just`](https://just.systems/) for the recipe shortcuts (optional but recommended)

---

## Quick start

```bash
just dev          # build WASM + start server at http://127.0.0.1:8080/demo/
```

Or manually:

```bash
wasm-pack build crates/navi_plot_wasm --target web --out-dir ../../pkg
cargo run -p server
```

### Other recipes

```bash
just test         # cargo test -p navi_plot_core
just build        # wasm-pack build + link TS types into pkg/
just serve        # start server (requires a prior build)
just fmt          # cargo fmt --all
just check        # cargo clippy -p navi_plot_core -D warnings
```

---

## WASM API

After `just build` the `pkg/` directory contains the wasm-bindgen JavaScript
module. Import it as an ES module:

```js
import init, * as wasm from "../pkg/navi_plot_wasm.js";
await init();
```

### Error handling

All render, pick, and pan functions throw on invalid input. Errors are real
`Error` objects with a machine-readable `.code` property:

```js
try {
  wasm.render_scatter("my-canvas", spec);
} catch (e) {
  console.error(e.message);            // human-readable description
  if (e.code === "EMPTY_SCATTER_DATA") { /* handle */ }
}
```

See `types/navi_plot_specs.d.ts` for the `PlotError` interface and all error
codes.

### TypeScript types

After `just build`, `pkg/navi_plot_wasm.d.ts` contains a reference to
`types/navi_plot_specs.d.ts`, giving IDEs autocomplete for every spec field.
In a `.ts` file you can also add the reference directly:

```ts
/// <reference types="../types/navi_plot_specs.d.ts" />
import type { ScatterPlotSpec, ScatterHit } from "../types/navi_plot_specs";
```

---

## Chart APIs

### Scatter

```js
// One-shot
wasm.render_scatter(canvasId, spec);
const spec2 = wasm.pan_scatter(spec, dx, dy);        // returns updated spec
const hit   = wasm.pick_scatter_point(spec, x, y);   // → { index } | null

// Session (caches viewport between renders — recommended for interactive use)
const h = wasm.create_scatter_session(canvasId, spec);
wasm.render_scatter_session(h);
wasm.pan_scatter_session(h, dx, dy);
const hit = wasm.pick_scatter_point_session(h, x, y);
wasm.set_scatter_selection(h, index);   // index: number | undefined
wasm.destroy_scatter_session(h);
```

**Spec shape:** `ScatterPlotSpec` — see `types/navi_plot_specs.d.ts`.

### Tree

```js
wasm.render_tree(canvasId, spec);
const spec2 = wasm.pan_tree(spec, dx, dy);
const hit   = wasm.pick_tree_node(spec, x, y);   // → { node_id } | null
```

**Spec shape:** `TreePlotSpec`.

### Line / time-series

```js
// One-shot
wasm.render_line(canvasId, spec);
const spec2 = wasm.pan_line(spec, dx, dy);
const hit   = wasm.pick_line_point(spec, x, y);   // → { series_index, point_index } | null

// Session
const h = wasm.create_line_session(canvasId, spec);
wasm.render_line_session(h);
wasm.pan_line_session(h, dx, dy);
const hit = wasm.pick_line_point_session(h, x, y);
wasm.set_line_selection(h, seriesIndex, pointIndex);   // numbers | undefined
wasm.destroy_line_session(h);
```

**Spec shape:** `LinePlotSpec`.

### Bar

```js
wasm.render_bar(canvasId, spec);
const hit = wasm.pick_bar(spec, x, y);   // → { series_index, category_index } | null
```

**Spec shape:** `BarChartSpec`. Set `variant: "grouped"` (default) or `"stacked"`.

### Heatmap

```js
wasm.render_heatmap(canvasId, spec);
const hit = wasm.pick_heatmap_cell(spec, x, y);   // → { row, col } | null
```

**Spec shape:** `HeatmapSpec`. Palettes: `"blue_white_red"` (default), `"viridis"`, `"greens"`.

### Network / DAG

```js
// One-shot
wasm.render_network(canvasId, spec);
const spec2 = wasm.pan_network(spec, dx, dy);
const hit   = wasm.pick_network_node(spec, x, y);   // → { node_id } | null

// Session (recommended — caches FR layout)
const h = wasm.create_network_session(canvasId, spec);
wasm.render_network_session(h);
wasm.pan_network_session(h, dx, dy);
const hit = wasm.pick_network_node_session(h, x, y);
wasm.set_network_selection(h, nodeId);   // string | undefined
wasm.destroy_network_session(h);
```

**Spec shape:** `NetworkPlotSpec`.

#### Node positioning

Nodes that supply both `x` and `y` are **pinned** at those canvas coordinates.
Nodes without `x`/`y` are automatically placed using Fruchterman-Reingold.
Mixed pinned + free is supported — you can anchor key nodes while letting
others be placed around them:

```js
nodes: [
  { id: "root", x: 360, y: 40, ... },   // pinned
  { id: "child1", ... },                 // FR-placed
  { id: "child2", ... },                 // FR-placed
]
```

#### Graph styling

Tree and network specs both accept graph-level defaults plus per-item style
overrides:

```js
const spec = {
  ...,
  default_node_style: {
    fill_color: "#dbeafe",
    stroke_color: "#1d4ed8",
    stroke_width: 2,
    radius: 20,
    shape: "square",
  },
  default_edge_style: {
    stroke_color: "#64748b",
    stroke_width: 2,
    label_visible: true,   // network edge labels only
  },
  selection_style: {
    stroke_color: "#0f172a",
    stroke_width: 3,
    padding: 8,
  },
  nodes: [
    { id: "a", label: "A" },   // inherits graph defaults
    {
      id: "b",
      label: "B",
      color: "#f59e0b",        // legacy per-node fields still work
      style: { shape: "diamond", radius: 28 },
    },
  ],
  edges: [
    { source: "a", target: "b", label: "calls", style: { stroke_width: 4 } },
  ],
};
```

Resolution order is:

1. Renderer defaults
2. Legacy graph-level fields (`node_radius`, `show_labels`, `show_arrows`)
3. `default_node_style` / `default_edge_style`
4. Legacy per-item fields (`color`, `shape`, `label_inside`)
5. Per-item `style`

#### Node media

Tree and network nodes can also render either a built-in icon or a preloaded
image inside the node body:

```js
await wasm.register_graph_image(
  "planetary-nebula",
  new URL("./demo/assets/planetary-nebula.svg", import.meta.url).href
);

const spec = {
  ...,
  nodes: [
    {
      id: "nebula",
      label: "Nebula",
      media: {
        kind: "image",
        image_key: "planetary-nebula",
        fit: "cover",
        scale: 0.82,
        fallback_icon: "camera",
      },
    },
    {
      id: "broker",
      label: "Broker",
      media: { kind: "icon", icon: "broker", tint_color: "#ffffff" },
    },
  ],
};
```

Supported built-in icons: `"star"`, `"galaxy"`, `"planet"`, `"moon"`,
`"telescope"`, `"camera"`, `"alert"`, `"archive"`, `"database"`, `"broker"`,
`"dish"`, `"spectrograph"`.

Image helpers:

```js
await wasm.register_graph_image(key, src);
wasm.has_graph_image(key);        // boolean
wasm.unregister_graph_image(key); // boolean
wasm.clear_graph_images();
```

Notes:

- Images are currently drawn on the wasm canvas backend only.
- Non-image backends fall back to `fallback_icon` when provided.
- Missing image keys do not fail the render; they fall back to `fallback_icon`
  or leave the node body unchanged.
- When a node has media, its label is rendered below the node instead of inside it.

---

## Session lifecycle

Sessions cache computed state (viewport, FR layout) and are more efficient for
interactive canvases. Typical lifecycle:

```js
// 1. Create once (validates spec + computes layout)
const h = wasm.create_scatter_session("canvas-id", spec);

// 2. Re-render after each interaction
wasm.pan_scatter_session(h, dx, dy);
wasm.render_scatter_session(h);

// 3. Update selection state
const hit = wasm.pick_scatter_point_session(h, x, y);
wasm.set_scatter_selection(h, hit?.index);
wasm.render_scatter_session(h);

// 4. Destroy when the canvas is removed
wasm.destroy_scatter_session(h);
```

---

## Running tests

```bash
just test
# or
cargo test -p navi_plot_core
```

The `navi_plot_core` crate has no browser dependencies and can be tested
natively. The WASM crate requires the `wasm32-unknown-unknown` target.
