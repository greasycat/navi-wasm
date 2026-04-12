use std::collections::{BTreeMap, HashMap, HashSet};

use crate::color::parse_color;
use crate::node;
use crate::viewport::ensure_finite;
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError};
use crate::types::{NetworkNode, NetworkPlotSpec};
use plotters::prelude::*;

const DEFAULT_NODE_COLOR: RGBColor = RGBColor(59, 130, 246);
const DEFAULT_EDGE_COLOR: RGBColor = RGBColor(107, 114, 128);
const SELECTION_RING_PADDING: i32 = 5;
const ARROW_LENGTH: f64 = 12.0;
const ARROW_HALF_WIDTH: f64 = 5.0;

#[derive(Debug, Clone)]
struct ResolvedNode {
    id: String,
    label: String,
    color: RGBColor,
}

fn validate(spec: &NetworkPlotSpec) -> Result<(), PlotError> {
    ensure_dimensions(spec.width, spec.height)?;
    if spec.nodes.is_empty() {
        return Err(PlotError::EmptyNetwork);
    }

    // Check for duplicate node IDs
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for node in &spec.nodes {
        if !seen_ids.insert(node.id.as_str()) {
            return Err(PlotError::DuplicateNodeId { node_id: node.id.clone() });
        }
    }

    // Check for unknown node references and duplicate edges
    let mut seen_edges: HashSet<(&str, &str)> = HashSet::new();
    for edge in &spec.edges {
        if !seen_ids.contains(edge.source.as_str()) {
            return Err(PlotError::UnknownNode { node_id: edge.source.clone() });
        }
        if !seen_ids.contains(edge.target.as_str()) {
            return Err(PlotError::UnknownNode { node_id: edge.target.clone() });
        }
        let key = (edge.source.as_str(), edge.target.as_str());
        if !seen_edges.insert(key) {
            return Err(PlotError::DuplicateEdge {
                from_node: edge.source.clone(),
                to_node: edge.target.clone(),
            });
        }
    }
    Ok(())
}

fn resolve_nodes(nodes: &[NetworkNode]) -> Result<Vec<ResolvedNode>, PlotError> {
    nodes
        .iter()
        .map(|n| {
            let color = match n.color.as_deref() {
                Some(c) => parse_color(c)?,
                None => DEFAULT_NODE_COLOR,
            };
            Ok(ResolvedNode {
                id: n.id.clone(),
                label: if n.label.is_empty() { n.id.clone() } else { n.label.clone() },
                color,
            })
        })
        .collect()
}

/// Fruchterman-Reingold force-directed layout.
/// Returns positions keyed by node ID in canvas coordinates (within margin bounds).
/// Fruchterman-Reingold layout with optional pinned nodes.
///
/// Nodes that supply both `x` and `y` are treated as fixed anchors: they are
/// placed at their supplied coordinates and never moved by the algorithm.
/// Nodes without `x`/`y` are positioned freely around the anchors.
/// If no node has coordinates, the result is a fully automatic FR layout.
fn fr_layout(spec: &NetworkPlotSpec) -> BTreeMap<String, (f64, f64)> {
    let n = spec.nodes.len();
    let w = (spec.width as f64 - 2.0 * spec.margin as f64).max(1.0);
    let h = (spec.height as f64 - 2.0 * spec.margin as f64).max(1.0);
    let k = (w * h / n as f64).sqrt();

    // Track which nodes are pinned (have explicit x/y)
    let fixed: Vec<bool> = spec
        .nodes
        .iter()
        .map(|node| node.x.is_some() && node.y.is_some())
        .collect();

    // Initialize positions: pinned nodes use their coordinates (minus margin),
    // free nodes get a deterministic grid position.
    let grid_side = (n as f64).sqrt().ceil() as usize;
    let step_x = w / (grid_side as f64 + 1.0);
    let step_y = h / (grid_side as f64 + 1.0);
    let mut free_grid_idx = 0usize;

    let mut pos: Vec<(f64, f64)> = spec
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            if fixed[i] {
                // Subtract margin: FR works in [0,w] x [0,h], margin is added back later
                let x = (node.x.unwrap() - spec.margin as f64).clamp(0.0, w);
                let y = (node.y.unwrap() - spec.margin as f64).clamp(0.0, h);
                (x, y)
            } else {
                let col = (free_grid_idx % grid_side) as f64;
                let row = (free_grid_idx / grid_side) as f64;
                free_grid_idx += 1;
                (step_x * (col + 1.0), step_y * (row + 1.0))
            }
        })
        .collect();

    // Build adjacency list (index-based)
    let id_to_idx: HashMap<&str, usize> = spec
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();
    let adj: Vec<(usize, usize)> = spec
        .edges
        .iter()
        .filter_map(|e| {
            let src = id_to_idx.get(e.source.as_str())?;
            let tgt = id_to_idx.get(e.target.as_str())?;
            Some((*src, *tgt))
        })
        .collect();

    let iterations = spec.layout_iterations.max(1) as usize;
    let mut temperature = w / 10.0;

    for _iter in 0..iterations {
        let mut displacement: Vec<(f64, f64)> = vec![(0.0, 0.0); n];

        // Repulsive forces between all node pairs
        for u in 0..n {
            if fixed[u] {
                continue;
            }
            for v in 0..n {
                if u == v {
                    continue;
                }
                let dx = pos[u].0 - pos[v].0;
                let dy = pos[u].1 - pos[v].1;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = k * k / dist;
                displacement[u].0 += dx / dist * force;
                displacement[u].1 += dy / dist * force;
            }
        }

        // Attractive forces along edges
        for &(src, tgt) in &adj {
            let dx = pos[tgt].0 - pos[src].0;
            let dy = pos[tgt].1 - pos[src].1;
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let force = dist * dist / k;
            let fx = dx / dist * force;
            let fy = dy / dist * force;
            if !fixed[src] {
                displacement[src].0 += fx;
                displacement[src].1 += fy;
            }
            if !fixed[tgt] {
                displacement[tgt].0 -= fx;
                displacement[tgt].1 -= fy;
            }
        }

        // Apply displacement clamped to temperature (skip pinned nodes)
        for i in 0..n {
            if fixed[i] {
                continue;
            }
            let disp_len = (displacement[i].0 * displacement[i].0
                + displacement[i].1 * displacement[i].1)
                .sqrt()
                .max(0.01);
            let scale = disp_len.min(temperature) / disp_len;
            pos[i].0 = (pos[i].0 + displacement[i].0 * scale).clamp(0.0, w);
            pos[i].1 = (pos[i].1 + displacement[i].1 * scale).clamp(0.0, h);
        }

        // Cool down
        temperature *= 0.95;
    }

    // Convert to canvas coords (add margin offset)
    spec.nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            (node.id.clone(), (pos[i].0 + spec.margin as f64, pos[i].1 + spec.margin as f64))
        })
        .collect()
}

fn compute_layout(spec: &NetworkPlotSpec) -> Result<BTreeMap<String, (f64, f64)>, PlotError> {
    // Validate explicit positions where provided
    for node in &spec.nodes {
        if let Some(x) = node.x { ensure_finite("x", x)?; }
        if let Some(y) = node.y { ensure_finite("y", y)?; }
    }
    // fr_layout handles all cases: all-pinned, all-free, and mixed
    Ok(fr_layout(spec))
}

#[derive(Debug, Clone)]
pub struct NetworkSession {
    spec: NetworkPlotSpec,
    layout: BTreeMap<String, (f64, f64)>,
    resolved: Vec<ResolvedNode>,
}

impl NetworkSession {
    pub fn new(spec: NetworkPlotSpec) -> Result<Self, PlotError> {
        validate(&spec)?;
        let layout = compute_layout(&spec)?;
        let resolved = resolve_nodes(&spec.nodes)?;
        Ok(Self { spec, layout, resolved })
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        render_with_layout(&root, &self.spec, &self.layout, &self.resolved)
    }

    pub fn pick_node(&self, canvas_x: f64, canvas_y: f64) -> Option<String> {
        pick_from_layout(&self.spec, &self.layout, canvas_x, canvas_y)
    }

    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        self.spec.offset_x += delta_x.round() as i32;
        self.spec.offset_y += delta_y.round() as i32;
    }

    pub fn set_selection(&mut self, node_id: Option<String>) {
        self.spec.selected_node_id = node_id
            .filter(|id| self.layout.contains_key(id.as_str()));
    }

    pub fn spec(&self) -> &NetworkPlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> NetworkPlotSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }
}

fn render_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &[ResolvedNode],
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    root.fill(&WHITE).map_err(backend_error)?;

    let nr = spec.node_radius as i32;
    let ox = spec.offset_x;
    let oy = spec.offset_y;

    // Draw title
    if !spec.title.is_empty() {
        let title_size = (20.0 * spec.pixel_ratio.max(0.25)).round() as u32;
        root.draw(&Text::new(
            spec.title.clone(),
            (spec.width as i32 / 2 - spec.title.len() as i32 * 7, spec.margin as i32 / 2),
            ("sans-serif", title_size).into_font(),
        ))
        .map_err(backend_error)?;
    }

    // Draw edges first
    for edge in &spec.edges {
        let Some(&src_pos) = layout.get(&edge.source) else { continue };
        let Some(&tgt_pos) = layout.get(&edge.target) else { continue };

        let sx = src_pos.0 as i32 + ox;
        let sy = src_pos.1 as i32 + oy;
        let tx = tgt_pos.0 as i32 + ox;
        let ty = tgt_pos.1 as i32 + oy;

        let edge_color = match edge.color.as_deref() {
            Some(c) => parse_color(c).unwrap_or(DEFAULT_EDGE_COLOR),
            None => DEFAULT_EDGE_COLOR,
        };

        // Edge line (slightly shortened at target to leave room for arrowhead)
        root.draw(&PathElement::new(
            vec![(sx, sy), (tx, ty)],
            ShapeStyle::from(&edge_color).stroke_width(1),
        ))
        .map_err(backend_error)?;

        // Arrowhead
        if spec.show_arrows && (sx != tx || sy != ty) {
            let dx = tx as f64 - sx as f64;
            let dy = ty as f64 - sy as f64;
            let len = (dx * dx + dy * dy).sqrt().max(0.01);
            let ux = dx / len;
            let uy = dy / len;
            let perp_x = -uy;
            let perp_y = ux;

            // Tip at target boundary
            let tip_x = (tx as f64 - ux * nr as f64).round() as i32;
            let tip_y = (ty as f64 - uy * nr as f64).round() as i32;
            let b1_x = (tip_x as f64 - ux * ARROW_LENGTH + perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b1_y = (tip_y as f64 - uy * ARROW_LENGTH + perp_y * ARROW_HALF_WIDTH).round() as i32;
            let b2_x = (tip_x as f64 - ux * ARROW_LENGTH - perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b2_y = (tip_y as f64 - uy * ARROW_LENGTH - perp_y * ARROW_HALF_WIDTH).round() as i32;

            root.draw(&Polygon::new(
                vec![(tip_x, tip_y), (b1_x, b1_y), (b2_x, b2_y)],
                edge_color.filled(),
            ))
            .map_err(backend_error)?;
        }
    }

    // Draw nodes on top
    for node_spec in &spec.nodes {
        let Some(&pos) = layout.get(&node_spec.id) else { continue };
        let resolved_node = resolved.iter().find(|r| r.id == node_spec.id);
        let color = resolved_node.map(|r| r.color).unwrap_or(DEFAULT_NODE_COLOR);
        let label = if spec.show_labels {
            resolved_node.map(|r| r.label.as_str()).unwrap_or(node_spec.id.as_str())
        } else {
            ""
        };

        let cx = pos.0 as i32 + ox;
        let cy = pos.1 as i32 + oy;
        let is_selected = spec
            .selected_node_id
            .as_deref()
            .is_some_and(|id| id == node_spec.id.as_str());

        node::draw_node(
            root,
            cx,
            cy,
            nr,
            color,
            &node_spec.shape,
            label,
            node_spec.label_inside,
            is_selected,
            SELECTION_RING_PADDING,
            spec.pixel_ratio,
        )?;
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

fn pick_from_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    canvas_x: f64,
    canvas_y: f64,
) -> Option<String> {
    if !canvas_x.is_finite() || !canvas_y.is_finite() {
        return None;
    }
    let target_x = canvas_x - spec.offset_x as f64;
    let target_y = canvas_y - spec.offset_y as f64;
    let hit_radius = spec.node_radius as f64 + SELECTION_RING_PADDING as f64;

    spec.nodes
        .iter()
        .filter_map(|node_spec| {
            let &(px, py) = layout.get(&node_spec.id)?;
            let dx = px - target_x;
            let dy = py - target_y;
            let dist_sq = dx * dx + dy * dy;
            node::node_contains(&node_spec.shape, px, py, hit_radius, target_x, target_y)
                .then_some((node_spec.id.clone(), dist_sq))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(id, _)| id)
}

pub fn render_network_on<DB>(root: PlotArea<DB>, spec: &NetworkPlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    validate(spec)?;
    let layout = compute_layout(spec)?;
    let resolved = resolve_nodes(&spec.nodes)?;
    render_with_layout(&root, spec, &layout, &resolved)
}

pub fn pick_network_node(
    spec: &NetworkPlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<String>, PlotError> {
    validate(spec)?;
    let layout = compute_layout(spec)?;
    Ok(pick_from_layout(spec, &layout, canvas_x, canvas_y))
}

pub fn pan_network_spec(
    spec: &NetworkPlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<NetworkPlotSpec, PlotError> {
    validate(spec)?;
    let mut updated = spec.clone();
    if delta_x.is_finite() && delta_y.is_finite() {
        updated.offset_x += delta_x.round() as i32;
        updated.offset_y += delta_y.round() as i32;
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NetworkEdge, NetworkNode, NetworkPlotSpec};
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn sample_spec() -> NetworkPlotSpec {
        NetworkPlotSpec {
            width: 480,
            height: 360,
            title: "Test Network".to_string(),
            nodes: vec![
                NetworkNode {
                    id: "a".to_string(), label: "A".to_string(), color: None,
                    x: None, y: None, shape: Default::default(), label_inside: false,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "b".to_string(), label: "B".to_string(), color: None,
                    x: None, y: None, shape: Default::default(), label_inside: false,
                    properties: Default::default(),
                },
                NetworkNode {
                    id: "c".to_string(), label: "C".to_string(), color: None,
                    x: None, y: None, shape: Default::default(), label_inside: false,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                NetworkEdge { source: "a".to_string(), target: "b".to_string(), label: None, color: None, weight: None },
                NetworkEdge { source: "b".to_string(), target: "c".to_string(), label: None, color: None, weight: None },
            ],
            node_radius: 16,
            margin: 40,
            offset_x: 0,
            offset_y: 0,
            selected_node_id: None,
            layout_iterations: 50,
            show_arrows: true,
            show_labels: true,
            pixel_ratio: 1.0,
        }
    }

    fn positioned_spec() -> NetworkPlotSpec {
        NetworkPlotSpec {
            nodes: vec![
                NetworkNode { id: "a".to_string(), label: "A".to_string(), color: None,
                    x: Some(100.0), y: Some(100.0), shape: Default::default(),
                    label_inside: false, properties: Default::default() },
                NetworkNode { id: "b".to_string(), label: "B".to_string(), color: None,
                    x: Some(300.0), y: Some(200.0), shape: Default::default(),
                    label_inside: false, properties: Default::default() },
            ],
            edges: vec![
                NetworkEdge { source: "a".to_string(), target: "b".to_string(), label: None, color: None, weight: None },
            ],
            ..sample_spec()
        }
    }

    #[test]
    fn network_rejects_empty_nodes() {
        let mut spec = sample_spec();
        spec.nodes.clear();
        spec.edges.clear();
        let err = NetworkSession::new(spec).unwrap_err();
        assert_eq!(err, PlotError::EmptyNetwork);
    }

    #[test]
    fn network_rejects_duplicate_node_ids() {
        let mut spec = sample_spec();
        spec.nodes[1].id = "a".to_string();
        let err = NetworkSession::new(spec).unwrap_err();
        assert!(matches!(err, PlotError::DuplicateNodeId { .. }));
    }

    #[test]
    fn network_rejects_duplicate_edges() {
        let mut spec = sample_spec();
        spec.edges.push(spec.edges[0].clone());
        let err = NetworkSession::new(spec).unwrap_err();
        assert!(matches!(err, PlotError::DuplicateEdge { .. }));
    }

    #[test]
    fn network_rejects_unknown_edge_nodes() {
        let mut spec = sample_spec();
        spec.edges.push(NetworkEdge {
            source: "a".to_string(), target: "z".to_string(),
            label: None, color: None, weight: None,
        });
        let err = NetworkSession::new(spec).unwrap_err();
        assert!(matches!(err, PlotError::UnknownNode { .. }));
    }

    #[test]
    fn network_allows_cycles() {
        let mut spec = sample_spec();
        spec.edges.push(NetworkEdge {
            source: "c".to_string(), target: "a".to_string(),
            label: None, color: None, weight: None,
        });
        assert!(NetworkSession::new(spec).is_ok());
    }

    #[test]
    fn network_allows_multiple_parents() {
        let mut spec = sample_spec();
        // Both "a" and "b" point to "c" — c has 2 parents
        spec.edges.push(NetworkEdge {
            source: "a".to_string(), target: "c".to_string(),
            label: None, color: None, weight: None,
        });
        assert!(NetworkSession::new(spec).is_ok());
    }

    #[test]
    fn network_layout_positions_are_within_canvas_bounds() {
        let spec = sample_spec();
        let layout = fr_layout(&spec);
        for (_, &(x, y)) in &layout {
            assert!(x >= 0.0 && x <= spec.width as f64, "x={} out of bounds", x);
            assert!(y >= 0.0 && y <= spec.height as f64, "y={} out of bounds", y);
        }
    }

    #[test]
    fn network_user_supplied_positions_are_used_directly() {
        let spec = positioned_spec();
        let layout = compute_layout(&spec).unwrap();
        assert_eq!(layout["a"], (100.0, 100.0));
        assert_eq!(layout["b"], (300.0, 200.0));
    }

    #[test]
    fn network_svg_has_correct_circle_count() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area =
            SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_network_on(area, &spec).unwrap();
        assert_eq!(svg.matches("<circle").count(), spec.nodes.len());
    }

    #[test]
    fn network_pan_updates_offsets() {
        let spec = sample_spec();
        let updated = pan_network_spec(&spec, 30.0, -15.0).unwrap();
        assert_eq!(updated.offset_x, 30);
        assert_eq!(updated.offset_y, -15);
    }

    #[test]
    fn network_fr_layout_is_deterministic() {
        let spec = sample_spec();
        let layout1 = fr_layout(&spec);
        let layout2 = fr_layout(&spec);
        for (id, &pos1) in &layout1 {
            let pos2 = layout2[id];
            assert!((pos1.0 - pos2.0).abs() < 0.001 && (pos1.1 - pos2.1).abs() < 0.001);
        }
    }

    #[test]
    fn network_hit_test_returns_closest_node() {
        let spec = positioned_spec();
        let session = NetworkSession::new(spec).unwrap();
        // Click near node "a" at (100, 100)
        let hit = session.pick_node(102.0, 98.0);
        assert_eq!(hit, Some("a".to_string()));
    }

    #[test]
    fn network_mixed_layout_pins_supplied_nodes_and_places_free_ones() {
        // "a" has explicit coordinates; "b" and "c" do not.
        let spec = NetworkPlotSpec {
            nodes: vec![
                NetworkNode { id: "a".into(), label: "A".into(), color: None,
                    x: Some(200.0), y: Some(150.0), shape: Default::default(),
                    label_inside: false, properties: Default::default() },
                NetworkNode { id: "b".into(), label: "B".into(), color: None,
                    x: None, y: None, shape: Default::default(),
                    label_inside: false, properties: Default::default() },
                NetworkNode { id: "c".into(), label: "C".into(), color: None,
                    x: None, y: None, shape: Default::default(),
                    label_inside: false, properties: Default::default() },
            ],
            edges: vec![
                NetworkEdge { source: "a".into(), target: "b".into(),
                    label: None, color: None, weight: None },
            ],
            ..sample_spec()
        };
        let layout = compute_layout(&spec).unwrap();
        // Pinned node "a" must be exactly at its supplied coordinates
        assert_eq!(layout["a"], (200.0, 150.0));
        // Free nodes "b" and "c" must be within canvas bounds
        for id in ["b", "c"] {
            let (x, y) = layout[id];
            assert!(x >= 0.0 && x <= spec.width as f64, "{id} x={x} out of bounds");
            assert!(y >= 0.0 && y <= spec.height as f64, "{id} y={y} out of bounds");
        }
    }

    #[test]
    fn network_non_circle_shapes_render_without_error() {
        use crate::types::NodeShape;
        // (shape, expected SVG element tag)
        let cases = [
            (NodeShape::Square, "rect"),
            (NodeShape::Diamond, "polygon"),
            (NodeShape::Triangle, "polygon"),
        ];
        for (shape, tag) in cases {
            let spec = NetworkPlotSpec {
                nodes: vec![
                    NetworkNode { id: "x".into(), label: "X".into(), color: None,
                        x: None, y: None, shape: shape.clone(), label_inside: true,
                        properties: Default::default() },
                ],
                edges: vec![],
                ..sample_spec()
            };
            let mut svg = String::new();
            let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height))
                .into_drawing_area();
            render_network_on(area, &spec).expect("render should succeed");
            assert_eq!(svg.matches("<circle").count(), 0,
                "shape={shape:?} should not render circles");
            assert!(svg.contains(&format!("<{tag}")),
                "shape={shape:?} should render <{tag}>");
        }
    }
}
