# WASM Internals

## Session Store (`crates/navi_plot_wasm/src/wasm_impl/network.rs`)

Thread-local `BTreeMap<u32, NetworkCanvasSession>` keyed by integer handle. All WASM exports follow the pattern:

```rust
with_network_session_mut(handle, |s| { ... })
```

Returns a JS error if handle unknown. Handles are monotonically incrementing with wrap-around collision avoidance.

---

## FR Layout (`crates/navi_plot_core/src/network/layout/force.rs`)

Two forces applied per iteration across all node pairs:

- **Repulsion** (all pairs): `force = spring_length² / distance`
- **Attraction** (connected pairs): `force = (distance² / spring_length) × edge_weight`

Edge weight directly scales attraction strength — hierarchy edges (`1.0`) pull hard, sibling chain edges (`0.15`) barely attract. Structural edges are those with `weight ≥ 0.5` (`STRUCTURAL_EDGE_WEIGHT_THRESHOLD`).

Temperature starts at `max(world_span × 0.12, WORLD_NODE_SPACING × 0.35)` and cools 8% per iteration (`× 0.92`). Temperature caps per-iteration displacement magnitude.

Nodes marked `navil_layout_inert` skip all force computation. Pinned nodes (explicit `x/y`) never move.

**Incremental update**: on topology change only nodes with changed neighbors re-relax (max 60 iterations, `LOCAL_RELAXATION_MAX_ITERATIONS`).

**Position seeding priority** (`layout/seed.rs`):
1. Parent gap-based seeding
2. Average of connected neighbors + deterministic offset
3. Overall graph center + offset
4. Grid fallback with deterministic perturbation

### Key Constants

| Constant | Value | Purpose |
|---|---|---|
| `WORLD_NODE_SPACING` | 180.0 | Natural rest distance between nodes |
| `LOCAL_RELAXATION_MAX_ITERATIONS` | 60 | FR iterations per incremental update |
| `STRUCTURAL_EDGE_WEIGHT_THRESHOLD` | 0.5 | Edge weight ≥ this = structural |
| `MIN_LAYOUT_EDGE_WEIGHT` | 0.05 | Minimum weight before clamping to zero |

---

## Pan / Zoom (`crates/navi_plot_core/src/viewport.rs`)

**View state:**
```rust
struct ScreenTransform {
    zoom: f64,         // clamped [0.05, 8.0]
    translate_x: f64,
    translate_y: f64,
}
```

**World → screen** (`apply`, line 277):
```
screen_x = world_x × zoom + translate_x
screen_y = world_y × zoom + translate_y
```

**Screen → world** (`inverse`, line 284):
```
world_x = (screen_x - translate_x) / zoom
world_y = (screen_y - translate_y) / zoom
```

**Zoom-at-point** (`zoom_at`, line 261) — keeps cursor fixed:
```
next_zoom = clamp(zoom × factor, 0.05, 8.0)
ratio     = next_zoom / zoom
translate_x = canvas_x - (canvas_x - translate_x) × ratio
translate_y = canvas_y - (canvas_y - translate_y) × ratio
```

Pan simply adds deltas to translate. Edges are clipped against viewport via Cohen-Sutherland before drawing.

---

## Hit-Testing (`crates/navi_plot_core/src/network/render/interaction.rs`)

`pick_hit_from_layout()` tests in priority order:

1. **Toggle badges** — project each toggleable node's badge center to screen via `view.apply()`, check `dist(canvas_point, badge_center) ≤ badge_radius`. Returns `kind: Toggle` on first hit.
2. **Nodes** — project each node world pos to screen, test `node_contains()` (shape-aware). Returns `kind: Node` for the closest-by-distance² hit.

Badge hit radius is 5–8 px, zoom-scaled.

---

## Topology Transition (`crates/navi_plot_core/src/network/render/transition.rs`)

Transition state stores old spec + old layout positions. An **anchor node** is chosen from nodes that bridge old↔new topology (prioritises: topology-boundary nodes → selected node → `__start__` → first node).

Per-node interpolation at `progress ∈ [0, 1]`:

| Node state | Position | Opacity |
|---|---|---|
| Present in both | `lerp(from_pos, to_pos, progress)` | `1.0` |
| Added (None → Some) | `lerp(anchor_pos, to_pos, progress)` | `progress` |
| Removed (Some → None) | `lerp(from_pos, anchor_pos, progress)` | `1 - progress` |

Edge alpha = `phase_opacity × source_opacity × target_opacity`.

---

## Tracking Path (`crates/navi_plot_core/src/network/render/tracking.rs`)

Path stored as `Vec<String>` node IDs + `progress ∈ [0, 1]` + `breath_phase ∈ [0, 1]`.

**Partial edge drawing** — for edge at index `i` of `n` total edges:
```
completion = clamp(progress × n - i, 0.0, 1.0)
endpoint   = lerp(source_pos, target_pos, completion)
```

**Color breathing** (activates after `progress = 1.0`):
```
strength = 0.5 - 0.5 × cos(2π × breath_phase)   // oscillates [0, 1]
color    = lerp(TRACKING_EDGE_COLOR, TRACKING_BREATH_COLOR, strength)
```
`TRACKING_EDGE_COLOR = RGB(239, 68, 68)` (red), `TRACKING_BREATH_COLOR` = white. Phase is driven externally by JS at ~2.4 s per cycle.

Edge opacity ramps: `0.95 × (0.35 + 0.65 × completion)` → from 0.33 at start to 0.95 at completion.

---

## Node Rendering (`crates/navi_plot_core/src/node.rs`, `draw_node`)

Layers drawn in order:

1. **Shadow** — blur radius + opacity from `node_style`
2. **Selection ring** — same shape at `radius + selection_style.padding`, drawn only when selected
3. **Fill** — solid shape at `node_style.radius`
4. **Stroke** — outline at same radius if `stroke_width > 0`
5. **Media** (icon/image) — centered, scaled to `media.scale × radius`
6. **Label** — inside (white, centered) or outside (below node at `cy + radius + 4`)

Shapes: `circle`, `square`, `rectangle`, `diamond`, `triangle`.

Zoom-aware scaling applies to radius, shadow blur, and shadow offsets (`scale_node_style`). Minimum rendered radius = 1 px.

---

## Edge Rendering (`crates/navi_plot_core/src/network/render/mod.rs`)

Style resolved by merging: graph default → per-edge override → legacy color field.

1. **Line** — Cohen-Sutherland viewport clip, then drawn as `PathElement` with optional dash pattern via `edge_line_segments()`
2. **Arrow** — triangle at `target_radius` from target center, pointing along edge direction. `ARROW_LENGTH = 12.0`, `ARROW_HALF_WIDTH` proportional to stroke width.
3. **Label** — at edge midpoint, `18pt × pixel_ratio × zoom`, color from edge style.

Edge weight has no direct visual mapping — it only affects FR layout attraction.

---

## Toggle Badges (`crates/navi_plot_core/src/network/render/badges.rs`)

Positioned at `node_center + outward_unit_from_parent() × (radius + badge_radius)`. Draws `—` always; adds `|` when node is collapsed. Hit radius: 5–8 px (zoom-scaled, `TOGGLE_BADGE_MIN_RADIUS` / `TOGGLE_BADGE_MAX_RADIUS`).
