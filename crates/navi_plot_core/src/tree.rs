use crate::color::parse_color;
use crate::node;
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError, TreePlotSpec};
use petgraph::algo::is_cyclic_directed;
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::Dfs;
use petgraph::Direction;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::collections::{BTreeMap, BTreeSet};

const DEFAULT_NODE_COLOR: RGBColor = RGBColor(14, 116, 144);
const DEFAULT_EDGE_COLOR: RGBColor = RGBColor(100, 116, 139);
const SELECTION_RING_PADDING: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
struct LayoutPoint {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct ValidatedTree {
    children_by_parent: BTreeMap<String, Vec<String>>,
}

pub fn render_tree_on<DB>(root: PlotArea<DB>, spec: &TreePlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    ensure_dimensions(spec.width, spec.height)?;
    let validated = validate_tree(spec)?;
    let layout = build_layout(spec, &validated);

    root.fill(&WHITE).map_err(backend_error)?;

    if !spec.title.is_empty() {
        let title_size = (22.0 * spec.pixel_ratio.max(0.25)).round() as u32;
        let title_style = TextStyle::from(("sans-serif", title_size).into_font())
            .pos(Pos::new(HPos::Center, VPos::Center))
            .color(&BLACK);
        root.draw(&Text::new(
            spec.title.clone(),
            ((spec.width / 2) as i32, (spec.margin.max(28) / 2) as i32),
            title_style,
        ))
        .map_err(backend_error)?;
    }

    for edge in &spec.edges {
        let source = offset_point(
            *layout.get(&edge.source).expect("validated tree source"),
            spec,
        );
        let target = offset_point(
            *layout.get(&edge.target).expect("validated tree target"),
            spec,
        );

        root.draw(&PathElement::new(
            vec![(source.x, source.y), (target.x, target.y)],
            ShapeStyle::from(&DEFAULT_EDGE_COLOR).stroke_width(2),
        ))
        .map_err(backend_error)?;
    }

    for node in &spec.nodes {
        let position = offset_point(*layout.get(&node.id).expect("validated tree node"), spec);
        let color = match node.color.as_deref() {
            Some(value) => parse_color(value)?,
            None => DEFAULT_NODE_COLOR,
        };
        let radius = spec.node_radius.max(1) as i32;
        let is_selected = spec.selected_node_id.as_deref() == Some(node.id.as_str());

        node::draw_node(
            &root,
            position.x,
            position.y,
            radius,
            color,
            &node.shape,
            &node.label,
            node.label_inside,
            is_selected,
            SELECTION_RING_PADDING,
            spec.pixel_ratio,
        )?;
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

pub fn pick_tree_node(
    spec: &TreePlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<String>, PlotError> {
    if !canvas_x.is_finite() || !canvas_y.is_finite() {
        return Ok(None);
    }

    ensure_dimensions(spec.width, spec.height)?;
    let validated = validate_tree(spec)?;
    let layout = build_layout(spec, &validated);
    let hit_radius = f64::from(spec.node_radius.max(1)) + f64::from(SELECTION_RING_PADDING);

    Ok(spec
        .nodes
        .iter()
        .filter_map(|node| {
            let center = offset_point(*layout.get(&node.id)?, spec);
            let cx = f64::from(center.x);
            let cy = f64::from(center.y);
            let dx = cx - canvas_x;
            let dy = cy - canvas_y;
            let dist_sq = dx * dx + dy * dy;

            node::node_contains(&node.shape, cx, cy, hit_radius, canvas_x, canvas_y)
                .then_some((node.id.clone(), dist_sq))
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(node_id, _)| node_id))
}

pub fn pan_tree_spec(
    spec: &TreePlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<TreePlotSpec, PlotError> {
    ensure_dimensions(spec.width, spec.height)?;

    let mut next = spec.clone();
    next.offset_x = next.offset_x.saturating_add(delta_x.round() as i32);
    next.offset_y = next.offset_y.saturating_add(delta_y.round() as i32);
    Ok(next)
}

fn validate_tree(spec: &TreePlotSpec) -> Result<ValidatedTree, PlotError> {
    if spec.nodes.is_empty() {
        return Err(PlotError::EmptyTree);
    }

    let mut graph = StableDiGraph::<String, ()>::new();
    let mut indices_by_id = BTreeMap::<String, NodeIndex>::new();
    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();

    for node in &spec.nodes {
        if indices_by_id.contains_key(&node.id) {
            return Err(PlotError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }

        let index = graph.add_node(node.id.clone());
        indices_by_id.insert(node.id.clone(), index);
        children_by_parent.entry(node.id.clone()).or_default();
    }

    let root_index = *indices_by_id
        .get(&spec.root_id)
        .ok_or_else(|| PlotError::MissingRoot {
            root_id: spec.root_id.clone(),
        })?;

    let mut seen_edges = BTreeSet::<(String, String)>::new();

    for edge in &spec.edges {
        let source_index =
            indices_by_id
                .get(&edge.source)
                .copied()
                .ok_or_else(|| PlotError::UnknownNode {
                    node_id: edge.source.clone(),
                })?;
        let target_index =
            indices_by_id
                .get(&edge.target)
                .copied()
                .ok_or_else(|| PlotError::UnknownNode {
                    node_id: edge.target.clone(),
                })?;

        if !seen_edges.insert((edge.source.clone(), edge.target.clone())) {
            return Err(PlotError::DuplicateEdge {
                from_node: edge.source.clone(),
                to_node: edge.target.clone(),
            });
        }

        graph.add_edge(source_index, target_index, ());
        children_by_parent
            .entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
    }

    if is_cyclic_directed(&graph) {
        return Err(PlotError::CycleDetected);
    }

    let roots = graph
        .node_indices()
        .filter(|index| {
            graph
                .neighbors_directed(*index, Direction::Incoming)
                .count()
                == 0
        })
        .collect::<Vec<_>>();

    if roots.len() != 1 {
        return Err(PlotError::InvalidRootCount { count: roots.len() });
    }

    let actual_root = graph[roots[0]].clone();
    if actual_root != spec.root_id {
        return Err(PlotError::RootMismatch {
            declared_root: spec.root_id.clone(),
            actual_root,
        });
    }

    for index in graph.node_indices() {
        let parent_count = graph.neighbors_directed(index, Direction::Incoming).count();
        if index == root_index {
            continue;
        }

        if parent_count != 1 {
            return Err(PlotError::InvalidParentCount {
                node_id: graph[index].clone(),
                parent_count,
            });
        }
    }

    let mut dfs = Dfs::new(&graph, root_index);
    let mut visited = BTreeSet::<String>::new();
    while let Some(index) = dfs.next(&graph) {
        visited.insert(graph[index].clone());
    }

    if visited.len() != graph.node_count() {
        let disconnected = spec
            .nodes
            .iter()
            .find(|node| !visited.contains(&node.id))
            .expect("visited count mismatch implies a disconnected node");

        return Err(PlotError::DisconnectedNode {
            node_id: disconnected.id.clone(),
            root_id: spec.root_id.clone(),
        });
    }

    Ok(ValidatedTree { children_by_parent })
}

fn build_layout(spec: &TreePlotSpec, validated: &ValidatedTree) -> BTreeMap<String, LayoutPoint> {
    #[derive(Debug, Clone, Copy)]
    struct RawPoint {
        x: f64,
        y: f64,
    }

    fn assign_positions(
        node_id: &str,
        depth: usize,
        children_by_parent: &BTreeMap<String, Vec<String>>,
        sibling_gap: u32,
        level_gap: u32,
        next_leaf_x: &mut f64,
        raw_positions: &mut BTreeMap<String, RawPoint>,
    ) -> f64 {
        let children = children_by_parent
            .get(node_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let x = if children.is_empty() {
            let x = *next_leaf_x;
            *next_leaf_x += sibling_gap.max(1) as f64;
            x
        } else {
            let child_centers = children
                .iter()
                .map(|child| {
                    assign_positions(
                        child,
                        depth + 1,
                        children_by_parent,
                        sibling_gap,
                        level_gap,
                        next_leaf_x,
                        raw_positions,
                    )
                })
                .collect::<Vec<_>>();

            child_centers.iter().sum::<f64>() / child_centers.len() as f64
        };

        raw_positions.insert(
            node_id.to_string(),
            RawPoint {
                x,
                y: depth as f64 * level_gap.max(1) as f64,
            },
        );

        x
    }

    let mut raw_positions = BTreeMap::<String, RawPoint>::new();
    let mut next_leaf_x = 0.0;
    assign_positions(
        &spec.root_id,
        0,
        &validated.children_by_parent,
        spec.sibling_gap,
        spec.level_gap,
        &mut next_leaf_x,
        &mut raw_positions,
    );

    // Raw positions use gap values as direct pixel distances — no normalization.
    // The tree is centered horizontally within the available canvas area.
    let max_raw_x = raw_positions
        .values()
        .map(|point| point.x)
        .fold(0.0, f64::max);

    let left = spec.margin as f64;
    let top = spec.margin as f64 + if spec.title.is_empty() { 0.0 } else { 28.0 };
    let available_width = (spec.width.saturating_sub(2 * spec.margin)) as f64;

    // Center the tree: if the tree is narrower than available_width, add padding.
    // If wider, it overflows to the right (user can pan to see clipped nodes).
    let x_offset = left + (available_width - max_raw_x).max(0.0) / 2.0;

    raw_positions
        .into_iter()
        .map(|(node_id, point)| {
            (
                node_id,
                LayoutPoint {
                    x: (x_offset + point.x).round() as i32,
                    y: (top + point.y).round() as i32,
                },
            )
        })
        .collect()
}

fn offset_point(point: LayoutPoint, spec: &TreePlotSpec) -> LayoutPoint {
    LayoutPoint {
        x: point.x.saturating_add(spec.offset_x),
        y: point.y.saturating_add(spec.offset_y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn sample_tree_spec() -> TreePlotSpec {
        TreePlotSpec {
            width: 640,
            height: 420,
            title: "Tree".to_string(),
            root_id: "root".to_string(),
            nodes: vec![
                crate::TreeNode {
                    id: "root".to_string(),
                    name: Some("Root Hub".to_string()),
                    label: "Root".to_string(),
                    color: Some("#0f766e".to_string()),
                    shape: Default::default(),
                    label_inside: false,
                    properties: Default::default(),
                },
                crate::TreeNode {
                    id: "left".to_string(),
                    name: Some("Left Branch".to_string()),
                    label: "Left".to_string(),
                    color: None,
                    shape: Default::default(),
                    label_inside: false,
                    properties: Default::default(),
                },
                crate::TreeNode {
                    id: "right".to_string(),
                    name: Some("Right Branch".to_string()),
                    label: "Right".to_string(),
                    color: None,
                    shape: Default::default(),
                    label_inside: false,
                    properties: Default::default(),
                },
                crate::TreeNode {
                    id: "leaf".to_string(),
                    name: Some("Leaf Node".to_string()),
                    label: "Leaf".to_string(),
                    color: None,
                    shape: Default::default(),
                    label_inside: false,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                crate::TreeEdge {
                    source: "root".to_string(),
                    target: "left".to_string(),
                },
                crate::TreeEdge {
                    source: "root".to_string(),
                    target: "right".to_string(),
                },
                crate::TreeEdge {
                    source: "right".to_string(),
                    target: "leaf".to_string(),
                },
            ],
            node_radius: 18,
            level_gap: 90,
            sibling_gap: 96,
            margin: 32,
            offset_x: 0,
            offset_y: 0,
            selected_node_id: None,
            pixel_ratio: 1.0,
        }
    }

    #[test]
    fn tree_validation_rejects_duplicate_ids() {
        let mut spec = sample_tree_spec();
        spec.nodes.push(crate::TreeNode {
            id: "left".to_string(),
            name: Some("Duplicate".to_string()),
            label: "Dupe".to_string(),
            color: None,
            shape: Default::default(),
            label_inside: false,
            properties: Default::default(),
        });

        let error = validate_tree(&spec).unwrap_err();
        assert_eq!(
            error,
            PlotError::DuplicateNodeId {
                node_id: "left".to_string(),
            }
        );
    }

    #[test]
    fn tree_validation_rejects_cycles() {
        let mut spec = sample_tree_spec();
        spec.edges.push(crate::TreeEdge {
            source: "leaf".to_string(),
            target: "root".to_string(),
        });

        let error = validate_tree(&spec).unwrap_err();
        assert_eq!(error, PlotError::CycleDetected);
    }

    #[test]
    fn tree_validation_rejects_multiple_parents() {
        let mut spec = sample_tree_spec();
        spec.edges.push(crate::TreeEdge {
            source: "left".to_string(),
            target: "leaf".to_string(),
        });

        let error = validate_tree(&spec).unwrap_err();
        assert_eq!(
            error,
            PlotError::InvalidParentCount {
                node_id: "leaf".to_string(),
                parent_count: 2,
            }
        );
    }

    #[test]
    fn tree_validation_rejects_disconnected_nodes() {
        let mut spec = sample_tree_spec();
        spec.nodes.push(crate::TreeNode {
            id: "orphan".to_string(),
            name: Some("Orphan".to_string()),
            label: "Orphan".to_string(),
            color: None,
            shape: Default::default(),
            label_inside: false,
            properties: Default::default(),
        });

        let error = validate_tree(&spec).unwrap_err();
        assert_eq!(error, PlotError::InvalidRootCount { count: 2 });
    }

    #[test]
    fn tree_layout_keeps_depth_monotonic_and_centers_parents() {
        let spec = sample_tree_spec();
        let validated = validate_tree(&spec).unwrap();
        let layout = build_layout(&spec, &validated);

        let root = layout["root"];
        let left = layout["left"];
        let right = layout["right"];
        let leaf = layout["leaf"];

        assert!(root.y < left.y);
        assert!(root.y < right.y);
        assert!(right.y < leaf.y);
        assert!(left.x < right.x);
        assert!((root.x - ((left.x + right.x) / 2)).abs() <= 1);
    }

    #[test]
    fn tree_svg_output_contains_nodes_and_labels() {
        let mut svg = String::new();
        let spec = sample_tree_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();

        render_tree_on(area, &spec).unwrap();

        assert_eq!(svg.matches("<circle").count(), spec.nodes.len());
        assert!(svg.contains("Root"));
        assert!(svg.contains("Leaf"));
    }

    #[test]
    fn tree_hit_test_respects_offsets() {
        let mut spec = sample_tree_spec();
        spec.offset_x = 32;
        spec.offset_y = -14;
        let validated = validate_tree(&spec).unwrap();
        let layout = build_layout(&spec, &validated);
        let target = offset_point(layout["right"], &spec);

        let selected = pick_tree_node(&spec, f64::from(target.x), f64::from(target.y)).unwrap();

        assert_eq!(selected.as_deref(), Some("right"));
    }

    #[test]
    fn tree_pan_updates_offsets() {
        let spec = sample_tree_spec();
        let panned = pan_tree_spec(&spec, 18.0, -9.0).unwrap();

        assert_eq!(panned.offset_x, 18);
        assert_eq!(panned.offset_y, -9);
    }
}
