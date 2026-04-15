use crate::graph_style::{
    edge_line_segments, resolve_edge_style, resolve_node_style, resolve_selection_style,
    EdgeStyleContext, NodeStyleContext, ResolvedNodeStyle,
};
use crate::node::{self, GraphNodeRenderInfo, ResolvedNodeMedia};
use crate::viewport::{PixelBounds, ScreenTransform};
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
const COLLAPSE_MARKER_FILL: RGBColor = RGBColor(15, 23, 42);
const COLLAPSE_MARKER_STROKE: RGBColor = RGBColor(255, 255, 255);

#[derive(Debug, Clone, Copy, PartialEq)]
struct LayoutPoint {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct ValidatedTree {
    node_ids: BTreeSet<String>,
    children_by_parent: BTreeMap<String, Vec<String>>,
    parent_by_child: BTreeMap<String, String>,
}

impl ValidatedTree {
    fn has_children(&self, node_id: &str) -> bool {
        self.children_by_parent
            .get(node_id)
            .is_some_and(|children| !children.is_empty())
    }
}

#[derive(Debug, Clone, Default)]
struct VisibleTree {
    children_by_parent: BTreeMap<String, Vec<String>>,
    collapsed_marker_node_ids: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct ResolvedTreeNode {
    style: ResolvedNodeStyle,
    media: Option<ResolvedNodeMedia>,
}

#[derive(Debug, Clone)]
struct TreeTransition {
    from_layout: BTreeMap<String, LayoutPoint>,
    from_visible: VisibleTree,
    from_selected_node_id: Option<String>,
    anchor_node_id: String,
}

#[derive(Debug, Clone)]
pub struct TreeSession {
    spec: TreePlotSpec,
    validated: ValidatedTree,
    visible: VisibleTree,
    layout: BTreeMap<String, LayoutPoint>,
    resolved_nodes: BTreeMap<String, ResolvedTreeNode>,
    selection_style: crate::graph_style::ResolvedSelectionStyle,
    view: ScreenTransform,
    transition: Option<TreeTransition>,
}

impl TreeSession {
    pub fn new(spec: TreePlotSpec) -> Result<Self, PlotError> {
        ensure_dimensions(spec.width, spec.height)?;
        let spec = spec;
        let validated = validate_tree(&spec)?;
        let resolved_nodes = resolve_tree_nodes(&spec)?;
        let selection_style =
            resolve_selection_style(SELECTION_RING_PADDING, spec.selection_style.as_ref())?;
        let view = ScreenTransform::new(spec.offset_x as f64, spec.offset_y as f64);
        let mut session = Self {
            spec,
            validated,
            visible: VisibleTree::default(),
            layout: BTreeMap::new(),
            resolved_nodes,
            selection_style,
            view,
            transition: None,
        };
        session.refresh_visible_state(None);
        Ok(session)
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        render_with_layout(
            &root,
            &self.spec,
            &self.visible,
            &self.layout,
            &self.resolved_nodes,
            &self.selection_style,
            &self.view,
        )
    }

    pub fn render_transition_on<DB>(
        &self,
        root: PlotArea<DB>,
        progress: f64,
    ) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_on(root);
        };

        let progress = progress.clamp(0.0, 1.0);
        root.fill(&WHITE).map_err(backend_error)?;
        draw_tree_title(&root, &self.spec)?;
        let viewport = PixelBounds::from_canvas(self.spec.width, self.spec.height);
        let anchor_frame = transition_anchor_frame(transition, &self.layout, progress);

        for edge in &self.spec.edges {
            let edge_style = resolve_tree_edge_style(&self.spec, edge)?;
            let Some(source_state) = transition_node_frame(
                &edge.source,
                transition,
                &self.layout,
                anchor_frame,
                progress,
            ) else {
                continue;
            };
            let Some(target_state) = transition_node_frame(
                &edge.target,
                transition,
                &self.layout,
                anchor_frame,
                progress,
            ) else {
                continue;
            };
            let edge_alpha = source_state
                .opacity
                .min(target_state.opacity)
                .clamp(0.0, 1.0);
            if edge_alpha <= 0.0 || edge_style.stroke_width == 0 {
                continue;
            }

            let source = project_point_f64(source_state.point, &self.view);
            let target = project_point_f64(target_state.point, &self.view);
            let Some((clipped_source, clipped_target)) = viewport.clip_line(source, target) else {
                continue;
            };

            let shape_style =
                ShapeStyle::from(&edge_style.stroke_color.mix(edge_style.opacity * edge_alpha))
                    .stroke_width(edge_style.stroke_width);
            for (segment_start, segment_end) in edge_line_segments(
                clipped_source,
                clipped_target,
                edge_style.dash_pattern.as_deref(),
            ) {
                root.draw(&PathElement::new(
                    vec![segment_start, segment_end],
                    shape_style,
                ))
                .map_err(backend_error)?;
            }
        }

        for node in &self.spec.nodes {
            let Some(node_state) =
                transition_node_frame(&node.id, transition, &self.layout, anchor_frame, progress)
            else {
                continue;
            };
            if node_state.opacity <= 0.0 {
                continue;
            }

            let position = project_point_f64(node_state.point, &self.view);
            let resolved_node = resolved_tree_node(&self.resolved_nodes, &node.id)?;
            let scaled_style = scale_node_style(&resolved_node.style, self.view.zoom);
            let scaled_selection_style =
                scale_selection_style(&self.selection_style, self.view.zoom);
            let selection_alpha = transition_phase_opacity(
                transition.from_selected_node_id.as_deref() == Some(node.id.as_str()),
                self.spec.selected_node_id.as_deref() == Some(node.id.as_str()),
                progress,
            );
            if !node_intersects_viewport(
                viewport,
                position,
                &scaled_style,
                &scaled_selection_style,
                selection_alpha > 0.0,
            ) {
                continue;
            }

            draw_tree_node(
                &root,
                position,
                &scaled_style,
                resolved_node.media.as_ref(),
                &node.label,
                selection_alpha > 0.0,
                &scaled_selection_style,
                self.spec.pixel_ratio,
                node_state.opacity,
                selection_alpha,
            )?;

            let marker_alpha = transition_phase_opacity(
                transition
                    .from_visible
                    .collapsed_marker_node_ids
                    .contains(&node.id),
                self.visible.collapsed_marker_node_ids.contains(&node.id),
                progress,
            );
            if marker_alpha > 0.0 {
                draw_collapsed_marker(&root, position, &scaled_style, marker_alpha)?;
            }
        }

        root.present().map_err(backend_error)?;
        Ok(())
    }

    pub fn pick_node(&self, canvas_x: f64, canvas_y: f64) -> Option<String> {
        pick_from_layout(
            &self.spec,
            &self.layout,
            &self.resolved_nodes,
            &self.selection_style,
            &self.view,
            canvas_x,
            canvas_y,
        )
    }

    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        self.view.pan_by(delta_x, delta_y);
        self.sync_view_to_spec();
    }

    pub fn zoom_at(&mut self, canvas_x: f64, canvas_y: f64, factor: f64) -> Result<(), PlotError> {
        self.view.zoom_at(canvas_x, canvas_y, factor)?;
        self.sync_view_to_spec();
        Ok(())
    }

    pub fn set_selection(&mut self, node_id: Option<String>) {
        self.spec.selected_node_id = node_id.filter(|id| self.layout.contains_key(id.as_str()));
    }

    pub fn toggle_collapse(&mut self, node_id: &str) -> bool {
        if !self.validated.has_children(node_id) {
            return false;
        }

        let collapsed_node_ids = normalized_collapsed_node_ids(&self.spec, &self.validated);
        let next = !collapsed_node_ids.contains(node_id);
        self.set_collapsed(node_id, next);
        next
    }

    pub fn set_collapsed(&mut self, node_id: &str, collapsed: bool) {
        if !self.validated.has_children(node_id) {
            return;
        }

        let mut collapsed_node_ids = normalized_collapsed_node_ids(&self.spec, &self.validated);
        let was_collapsed = collapsed_node_ids.contains(node_id);
        if was_collapsed == collapsed {
            return;
        }
        let from_layout = self.layout.clone();
        let from_visible = self.visible.clone();
        let from_selected_node_id = self.spec.selected_node_id.clone();
        if collapsed {
            collapsed_node_ids.insert(node_id.to_string());
        } else {
            collapsed_node_ids.remove(node_id);
        }
        sync_collapsed_node_ids(&mut self.spec, &collapsed_node_ids);
        self.refresh_visible_state(collapsed.then_some(node_id));
        self.transition = Some(TreeTransition {
            from_layout,
            from_visible,
            from_selected_node_id,
            anchor_node_id: node_id.to_string(),
        });
    }

    pub fn spec(&self) -> &TreePlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> TreePlotSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }

    pub fn render_nodes(&self) -> Vec<GraphNodeRenderInfo> {
        render_nodes_with_layout(
            &self.spec,
            &self.layout,
            &self.resolved_nodes,
            &self.selection_style,
            &self.view,
        )
    }

    pub fn render_transition_nodes(&self, progress: f64) -> Vec<GraphNodeRenderInfo> {
        let Some(transition) = self.transition.as_ref() else {
            return self.render_nodes();
        };

        let progress = progress.clamp(0.0, 1.0);
        let anchor_frame = transition_anchor_frame(transition, &self.layout, progress);
        let viewport = PixelBounds::from_canvas(self.spec.width, self.spec.height);

        self.spec
            .nodes
            .iter()
            .filter_map(|node| {
                let node_state = transition_node_frame(
                    &node.id,
                    transition,
                    &self.layout,
                    anchor_frame,
                    progress,
                )?;
                if node_state.opacity <= 0.0 {
                    return None;
                }
                let position = project_point_f64(node_state.point, &self.view);
                let resolved_node = self.resolved_nodes.get(&node.id)?;
                let scaled_style = scale_node_style(&resolved_node.style, self.view.zoom);
                let scaled_selection_style =
                    scale_selection_style(&self.selection_style, self.view.zoom);
                let selection_alpha = transition_phase_opacity(
                    transition.from_selected_node_id.as_deref() == Some(node.id.as_str()),
                    self.spec.selected_node_id.as_deref() == Some(node.id.as_str()),
                    progress,
                );
                if !node_intersects_viewport(
                    viewport,
                    position,
                    &scaled_style,
                    &scaled_selection_style,
                    selection_alpha > 0.0,
                ) {
                    return None;
                }
                Some(GraphNodeRenderInfo {
                    id: node.id.clone(),
                    center_x: position.0,
                    center_y: position.1,
                    radius: scaled_style.radius,
                    shape: scaled_style.shape.clone(),
                    opacity: scaled_style.opacity * node_state.opacity,
                    media: resolved_node.media.clone(),
                })
            })
            .collect()
    }

    fn sync_view_to_spec(&mut self) {
        self.spec.offset_x = self.view.translate_x.round() as i32;
        self.spec.offset_y = self.view.translate_y.round() as i32;
    }

    fn refresh_visible_state(&mut self, preferred_selection: Option<&str>) {
        let collapsed_node_ids = normalized_collapsed_node_ids(&self.spec, &self.validated);
        sync_collapsed_node_ids(&mut self.spec, &collapsed_node_ids);
        self.visible = build_visible_tree(&self.spec.root_id, &self.validated, &collapsed_node_ids);
        self.layout = build_layout(&self.spec, &self.visible.children_by_parent);
        sync_selected_node_id(
            &mut self.spec.selected_node_id,
            &self.layout,
            &self.validated,
            preferred_selection,
        );
    }
}

fn render_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &TreePlotSpec,
    visible: &VisibleTree,
    layout: &BTreeMap<String, LayoutPoint>,
    resolved_nodes: &BTreeMap<String, ResolvedTreeNode>,
    selection_style: &crate::graph_style::ResolvedSelectionStyle,
    view: &ScreenTransform,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    root.fill(&WHITE).map_err(backend_error)?;
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    draw_tree_title(root, spec)?;

    for edge in &spec.edges {
        let edge_style = resolve_tree_edge_style(spec, edge)?;
        let Some(source_point) = layout.get(&edge.source).copied() else {
            continue;
        };
        let Some(target_point) = layout.get(&edge.target).copied() else {
            continue;
        };
        let source = project_point(source_point, view);
        let target = project_point(target_point, view);
        let Some((clipped_source, clipped_target)) = viewport.clip_line(source, target) else {
            continue;
        };

        if edge_style.stroke_width > 0 {
            let shape_style = ShapeStyle::from(&edge_style.stroke_color.mix(edge_style.opacity))
                .stroke_width(edge_style.stroke_width);
            for (segment_start, segment_end) in edge_line_segments(
                clipped_source,
                clipped_target,
                edge_style.dash_pattern.as_deref(),
            ) {
                root.draw(&PathElement::new(
                    vec![segment_start, segment_end],
                    shape_style,
                ))
                .map_err(backend_error)?;
            }
        }
    }

    for node in &spec.nodes {
        let Some(layout_point) = layout.get(&node.id).copied() else {
            continue;
        };
        let position = project_point(layout_point, view);
        let resolved_node = resolved_tree_node(resolved_nodes, &node.id)?;
        let is_selected = spec.selected_node_id.as_deref() == Some(node.id.as_str());
        let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        if !node_intersects_viewport(
            viewport,
            position,
            &scaled_style,
            &scaled_selection_style,
            is_selected,
        ) {
            continue;
        }

        draw_tree_node(
            root,
            position,
            &scaled_style,
            resolved_node.media.as_ref(),
            &node.label,
            is_selected,
            &scaled_selection_style,
            spec.pixel_ratio,
            1.0,
            if is_selected { 1.0 } else { 0.0 },
        )?;

        if visible.collapsed_marker_node_ids.contains(&node.id) {
            draw_collapsed_marker(root, position, &scaled_style, 1.0)?;
        }
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

fn draw_tree_title<DB>(root: &PlotArea<DB>, spec: &TreePlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    if spec.title.is_empty() {
        return Ok(());
    }

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
    Ok(())
}

fn resolved_tree_node<'a>(
    resolved_nodes: &'a BTreeMap<String, ResolvedTreeNode>,
    node_id: &str,
) -> Result<&'a ResolvedTreeNode, PlotError> {
    resolved_nodes
        .get(node_id)
        .ok_or_else(|| PlotError::UnknownNode {
            node_id: node_id.to_string(),
        })
}

fn draw_tree_node<DB>(
    root: &PlotArea<DB>,
    position: (i32, i32),
    style: &ResolvedNodeStyle,
    media: Option<&ResolvedNodeMedia>,
    label: &str,
    is_selected: bool,
    selection_style: &crate::graph_style::ResolvedSelectionStyle,
    font_scale: f64,
    node_opacity_scale: f64,
    selection_opacity_scale: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    if node_opacity_scale <= 0.0 {
        return Ok(());
    }

    let mut faded_style = style.clone();
    faded_style.opacity = (faded_style.opacity * node_opacity_scale).clamp(0.0, 1.0);
    let mut faded_selection_style = selection_style.clone();
    faded_selection_style.opacity =
        (faded_selection_style.opacity * selection_opacity_scale).clamp(0.0, 1.0);

    node::draw_node(
        root,
        position.0,
        position.1,
        &faded_style,
        media,
        label,
        is_selected && faded_selection_style.opacity > 0.0,
        &faded_selection_style,
        font_scale,
    )
}

fn pick_from_layout(
    spec: &TreePlotSpec,
    layout: &BTreeMap<String, LayoutPoint>,
    resolved_nodes: &BTreeMap<String, ResolvedTreeNode>,
    selection_style: &crate::graph_style::ResolvedSelectionStyle,
    view: &ScreenTransform,
    canvas_x: f64,
    canvas_y: f64,
) -> Option<String> {
    if !canvas_x.is_finite() || !canvas_y.is_finite() {
        return None;
    }
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);

    spec.nodes
        .iter()
        .filter_map(|node| {
            let center = project_point(*layout.get(&node.id)?, view);
            let resolved_node = resolved_nodes.get(&node.id)?;
            let cx = f64::from(center.0);
            let cy = f64::from(center.1);
            let dx = cx - canvas_x;
            let dy = cy - canvas_y;
            let dist_sq = dx * dx + dy * dy;
            let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            let is_selected = spec.selected_node_id.as_deref() == Some(node.id.as_str());
            if !node_intersects_viewport(
                viewport,
                center,
                &scaled_style,
                &scaled_selection_style,
                is_selected,
            ) {
                return None;
            }
            let hit_radius =
                f64::from(scaled_style.radius.max(1) + scaled_selection_style.padding.max(0));

            node::node_contains(&scaled_style.shape, cx, cy, hit_radius, canvas_x, canvas_y)
                .then_some((node.id.clone(), dist_sq))
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(node_id, _)| node_id)
}

fn render_nodes_with_layout(
    spec: &TreePlotSpec,
    layout: &BTreeMap<String, LayoutPoint>,
    resolved_nodes: &BTreeMap<String, ResolvedTreeNode>,
    selection_style: &crate::graph_style::ResolvedSelectionStyle,
    view: &ScreenTransform,
) -> Vec<GraphNodeRenderInfo> {
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    spec.nodes
        .iter()
        .filter_map(|node| {
            let center = project_point(*layout.get(&node.id)?, view);
            let resolved_node = resolved_nodes.get(&node.id)?;
            let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            let is_selected = spec.selected_node_id.as_deref() == Some(node.id.as_str());
            if !node_intersects_viewport(
                viewport,
                center,
                &scaled_style,
                &scaled_selection_style,
                is_selected,
            ) {
                return None;
            }
            Some(GraphNodeRenderInfo {
                id: node.id.clone(),
                center_x: center.0,
                center_y: center.1,
                radius: scaled_style.radius,
                shape: scaled_style.shape.clone(),
                opacity: scaled_style.opacity,
                media: resolved_node.media.clone(),
            })
        })
        .collect()
}

pub fn render_tree_on<DB>(root: PlotArea<DB>, spec: &TreePlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    TreeSession::new(spec.clone())?.render_on(root)
}

pub fn pick_tree_node(
    spec: &TreePlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<String>, PlotError> {
    Ok(TreeSession::new(spec.clone())?.pick_node(canvas_x, canvas_y))
}

pub fn tree_render_nodes(spec: &TreePlotSpec) -> Result<Vec<GraphNodeRenderInfo>, PlotError> {
    let session = TreeSession::new(spec.clone())?;
    Ok(session.render_nodes())
}

pub fn pan_tree_spec(
    spec: &TreePlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<TreePlotSpec, PlotError> {
    let mut session = TreeSession::new(spec.clone())?;
    session.pan(delta_x, delta_y);
    Ok(session.into_spec())
}

fn project_point(point: LayoutPoint, view: &ScreenTransform) -> (i32, i32) {
    view.apply((point.x as f64, point.y as f64))
}

fn project_point_f64(point: (f64, f64), view: &ScreenTransform) -> (i32, i32) {
    view.apply(point)
}

#[derive(Debug, Clone, Copy)]
struct TransitionNodeState {
    point: (f64, f64),
    opacity: f64,
}

fn transition_anchor_frame(
    transition: &TreeTransition,
    to_layout: &BTreeMap<String, LayoutPoint>,
    progress: f64,
) -> (f64, f64) {
    let anchor_from = transition
        .from_layout
        .get(&transition.anchor_node_id)
        .copied()
        .or_else(|| to_layout.get(&transition.anchor_node_id).copied())
        .unwrap_or(LayoutPoint { x: 0, y: 0 });
    let anchor_to = to_layout
        .get(&transition.anchor_node_id)
        .copied()
        .or_else(|| {
            transition
                .from_layout
                .get(&transition.anchor_node_id)
                .copied()
        })
        .unwrap_or(anchor_from);
    lerp_point(
        layout_point_to_f64(anchor_from),
        layout_point_to_f64(anchor_to),
        progress,
    )
}

fn transition_node_frame(
    node_id: &str,
    transition: &TreeTransition,
    to_layout: &BTreeMap<String, LayoutPoint>,
    anchor_frame: (f64, f64),
    progress: f64,
) -> Option<TransitionNodeState> {
    let from = transition.from_layout.get(node_id).copied();
    let to = to_layout.get(node_id).copied();
    match (from, to) {
        (Some(from), Some(to)) => Some(TransitionNodeState {
            point: lerp_point(layout_point_to_f64(from), layout_point_to_f64(to), progress),
            opacity: 1.0,
        }),
        (Some(from), None) => Some(TransitionNodeState {
            point: lerp_point(layout_point_to_f64(from), anchor_frame, progress),
            opacity: 1.0 - progress,
        }),
        (None, Some(to)) => Some(TransitionNodeState {
            point: lerp_point(anchor_frame, layout_point_to_f64(to), progress),
            opacity: progress,
        }),
        (None, None) => None,
    }
}

fn transition_phase_opacity(from_present: bool, to_present: bool, progress: f64) -> f64 {
    match (from_present, to_present) {
        (true, true) => 1.0,
        (true, false) => 1.0 - progress,
        (false, true) => progress,
        (false, false) => 0.0,
    }
}

fn layout_point_to_f64(point: LayoutPoint) -> (f64, f64) {
    (point.x as f64, point.y as f64)
}

fn lerp_point(from: (f64, f64), to: (f64, f64), progress: f64) -> (f64, f64) {
    (
        from.0 + (to.0 - from.0) * progress,
        from.1 + (to.1 - from.1) * progress,
    )
}

fn node_intersects_viewport(
    viewport: PixelBounds,
    center: (i32, i32),
    style: &ResolvedNodeStyle,
    selection_style: &crate::graph_style::ResolvedSelectionStyle,
    is_selected: bool,
) -> bool {
    let footprint_radius = style.radius.max(1)
        + if is_selected {
            selection_style.padding.max(0)
        } else {
            0
        };
    viewport.intersects_circle(center, footprint_radius)
}

fn scale_node_style(style: &ResolvedNodeStyle, zoom: f64) -> ResolvedNodeStyle {
    let mut scaled = style.clone();
    scaled.radius = ((scaled.radius.max(1) as f64) * zoom).round() as i32;
    scaled.radius = scaled.radius.max(1);
    scaled
}

fn scale_selection_style(
    style: &crate::graph_style::ResolvedSelectionStyle,
    zoom: f64,
) -> crate::graph_style::ResolvedSelectionStyle {
    let mut scaled = style.clone();
    scaled.padding = ((scaled.padding.max(0) as f64) * zoom).round() as i32;
    scaled.padding = scaled.padding.max(0);
    scaled
}

fn draw_collapsed_marker<DB>(
    root: &PlotArea<DB>,
    center: (i32, i32),
    style: &ResolvedNodeStyle,
    opacity: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let alpha = opacity.clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return Ok(());
    }
    let marker_radius = ((style.radius.max(1) as f64) * 0.38).round() as i32;
    let marker_radius = marker_radius.clamp(5, 10);
    let marker_center = (
        center.0 + style.radius - marker_radius / 2,
        center.1 - style.radius + marker_radius / 2,
    );

    root.draw(&Circle::new(
        marker_center,
        marker_radius,
        ShapeStyle::from(&COLLAPSE_MARKER_FILL.mix(alpha)).filled(),
    ))
    .map_err(backend_error)?;
    root.draw(&Circle::new(
        marker_center,
        marker_radius,
        ShapeStyle::from(&COLLAPSE_MARKER_STROKE.mix(alpha)).stroke_width(1),
    ))
    .map_err(backend_error)?;

    let plus_half = (marker_radius / 2).max(2);
    root.draw(&PathElement::new(
        vec![
            (marker_center.0 - plus_half, marker_center.1),
            (marker_center.0 + plus_half, marker_center.1),
        ],
        ShapeStyle::from(&COLLAPSE_MARKER_STROKE.mix(alpha)).stroke_width(2),
    ))
    .map_err(backend_error)?;
    root.draw(&PathElement::new(
        vec![
            (marker_center.0, marker_center.1 - plus_half),
            (marker_center.0, marker_center.1 + plus_half),
        ],
        ShapeStyle::from(&COLLAPSE_MARKER_STROKE.mix(alpha)).stroke_width(2),
    ))
    .map_err(backend_error)?;

    Ok(())
}

fn normalized_collapsed_node_ids(
    spec: &TreePlotSpec,
    validated: &ValidatedTree,
) -> BTreeSet<String> {
    spec.collapsed_node_ids
        .iter()
        .filter(|node_id| {
            validated.node_ids.contains(node_id.as_str()) && validated.has_children(node_id)
        })
        .cloned()
        .collect()
}

fn sync_collapsed_node_ids(spec: &mut TreePlotSpec, collapsed_node_ids: &BTreeSet<String>) {
    spec.collapsed_node_ids = collapsed_node_ids.iter().cloned().collect();
}

fn build_visible_tree(
    root_id: &str,
    validated: &ValidatedTree,
    collapsed_node_ids: &BTreeSet<String>,
) -> VisibleTree {
    fn visit(
        node_id: &str,
        validated: &ValidatedTree,
        collapsed_node_ids: &BTreeSet<String>,
        visible: &mut VisibleTree,
    ) {
        let children = validated
            .children_by_parent
            .get(node_id)
            .cloned()
            .unwrap_or_default();

        if children.is_empty() {
            visible
                .children_by_parent
                .insert(node_id.to_string(), Vec::new());
            return;
        }

        if collapsed_node_ids.contains(node_id) {
            visible
                .children_by_parent
                .insert(node_id.to_string(), Vec::new());
            visible
                .collapsed_marker_node_ids
                .insert(node_id.to_string());
            return;
        }

        visible
            .children_by_parent
            .insert(node_id.to_string(), children.clone());
        for child in &children {
            visit(child, validated, collapsed_node_ids, visible);
        }
    }

    let mut visible = VisibleTree::default();
    visit(root_id, validated, collapsed_node_ids, &mut visible);
    visible
}

fn sync_selected_node_id(
    selected_node_id: &mut Option<String>,
    layout: &BTreeMap<String, LayoutPoint>,
    validated: &ValidatedTree,
    preferred_selection: Option<&str>,
) {
    let Some(current) = selected_node_id.as_deref() else {
        return;
    };

    if layout.contains_key(current) {
        return;
    }

    if let Some(preferred) = preferred_selection.filter(|node_id| layout.contains_key(*node_id)) {
        *selected_node_id = Some(preferred.to_string());
        return;
    }

    *selected_node_id = nearest_visible_ancestor(current, layout, validated);
}

fn nearest_visible_ancestor(
    node_id: &str,
    layout: &BTreeMap<String, LayoutPoint>,
    validated: &ValidatedTree,
) -> Option<String> {
    let mut cursor = node_id;
    while let Some(parent) = validated.parent_by_child.get(cursor) {
        if layout.contains_key(parent.as_str()) {
            return Some(parent.clone());
        }
        cursor = parent;
    }
    None
}

fn validate_tree(spec: &TreePlotSpec) -> Result<ValidatedTree, PlotError> {
    if spec.nodes.is_empty() {
        return Err(PlotError::EmptyTree);
    }

    let mut graph = StableDiGraph::<String, ()>::new();
    let mut indices_by_id = BTreeMap::<String, NodeIndex>::new();
    let mut children_by_parent = BTreeMap::<String, Vec<String>>::new();
    let mut parent_by_child = BTreeMap::<String, String>::new();
    let mut node_ids = BTreeSet::<String>::new();

    for node in &spec.nodes {
        if indices_by_id.contains_key(&node.id) {
            return Err(PlotError::DuplicateNodeId {
                node_id: node.id.clone(),
            });
        }

        let index = graph.add_node(node.id.clone());
        indices_by_id.insert(node.id.clone(), index);
        node_ids.insert(node.id.clone());
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
        parent_by_child.insert(edge.target.clone(), edge.source.clone());
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

    Ok(ValidatedTree {
        node_ids,
        children_by_parent,
        parent_by_child,
    })
}

fn build_layout(
    spec: &TreePlotSpec,
    children_by_parent: &BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, LayoutPoint> {
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
        children_by_parent,
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

#[cfg(test)]
fn offset_point(point: LayoutPoint, spec: &TreePlotSpec) -> LayoutPoint {
    LayoutPoint {
        x: point.x.saturating_add(spec.offset_x),
        y: point.y.saturating_add(spec.offset_y),
    }
}

fn resolve_tree_nodes(
    spec: &TreePlotSpec,
) -> Result<BTreeMap<String, ResolvedTreeNode>, PlotError> {
    spec.nodes
        .iter()
        .map(|node| {
            let style = resolve_node_style(NodeStyleContext {
                default_fill_color: DEFAULT_NODE_COLOR,
                default_radius: spec.node_radius,
                default_label_visible: true,
                graph_style: spec.default_node_style.as_ref(),
                legacy_fill_color: node.color.as_deref(),
                legacy_shape: node.shape.as_ref(),
                legacy_label_inside: node.label_inside,
                item_style: node.style.as_ref(),
            })?;
            let media = node::resolve_node_media(node.media.as_ref())?;
            Ok((node.id.clone(), ResolvedTreeNode { style, media }))
        })
        .collect()
}

fn resolve_tree_edge_style(
    spec: &TreePlotSpec,
    edge: &crate::TreeEdge,
) -> Result<crate::graph_style::ResolvedEdgeStyle, PlotError> {
    resolve_edge_style(EdgeStyleContext {
        default_stroke_color: DEFAULT_EDGE_COLOR,
        default_stroke_width: 2,
        default_arrow_visible: false,
        default_label_visible: false,
        graph_style: spec.default_edge_style.as_ref(),
        legacy_stroke_color: None,
        item_style: edge.style.as_ref(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::ResolvedNodeMediaKind;
    use crate::types::{
        BuiltinNodeIcon, GraphEdgeStyle, GraphNodeStyle, NodeMedia, NodeMediaFit, NodeMediaKind,
        NodeShape, SelectionStyle,
    };
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
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                crate::TreeNode {
                    id: "left".to_string(),
                    name: Some("Left Branch".to_string()),
                    label: "Left".to_string(),
                    color: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                crate::TreeNode {
                    id: "right".to_string(),
                    name: Some("Right Branch".to_string()),
                    label: "Right".to_string(),
                    color: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
                crate::TreeNode {
                    id: "leaf".to_string(),
                    name: Some("Leaf Node".to_string()),
                    label: "Leaf".to_string(),
                    color: None,
                    shape: None,
                    label_inside: None,
                    style: None,
                    media: None,
                    properties: Default::default(),
                },
            ],
            edges: vec![
                crate::TreeEdge {
                    source: "root".to_string(),
                    target: "left".to_string(),
                    style: None,
                },
                crate::TreeEdge {
                    source: "root".to_string(),
                    target: "right".to_string(),
                    style: None,
                },
                crate::TreeEdge {
                    source: "right".to_string(),
                    target: "leaf".to_string(),
                    style: None,
                },
            ],
            node_radius: 18,
            default_node_style: None,
            default_edge_style: None,
            selection_style: None,
            level_gap: 90,
            sibling_gap: 96,
            margin: 32,
            offset_x: 0,
            offset_y: 0,
            selected_node_id: None,
            collapsed_node_ids: Vec::new(),
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
            shape: None,
            label_inside: None,
            style: None,
            media: None,
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
            style: None,
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
            style: None,
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
            shape: None,
            label_inside: None,
            style: None,
            media: None,
            properties: Default::default(),
        });

        let error = validate_tree(&spec).unwrap_err();
        assert_eq!(error, PlotError::InvalidRootCount { count: 2 });
    }

    #[test]
    fn tree_layout_keeps_depth_monotonic_and_centers_parents() {
        let spec = sample_tree_spec();
        let validated = validate_tree(&spec).unwrap();
        let layout = build_layout(&spec, &validated.children_by_parent);

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
        let layout = build_layout(&spec, &validated.children_by_parent);
        let target = offset_point(layout["right"], &spec);

        let selected = pick_tree_node(&spec, f64::from(target.x), f64::from(target.y)).unwrap();

        assert_eq!(selected.as_deref(), Some("right"));
    }

    #[test]
    fn tree_render_nodes_exposes_media_metadata() {
        let mut spec = sample_tree_spec();
        spec.nodes[0].media = Some(NodeMedia {
            kind: NodeMediaKind::Icon,
            icon: Some(BuiltinNodeIcon::Galaxy),
            image_key: None,
            fit: NodeMediaFit::Contain,
            scale: Some(0.8),
            tint_color: Some("#ffffff".to_string()),
            fallback_icon: None,
        });

        let nodes = tree_render_nodes(&spec).unwrap();
        let root = nodes.iter().find(|node| node.id == "root").unwrap();

        assert!(matches!(
            root.media.as_ref().map(|media| &media.kind),
            Some(ResolvedNodeMediaKind::Icon(BuiltinNodeIcon::Galaxy))
        ));
        assert_eq!(root.shape, NodeShape::Circle);
    }

    #[test]
    fn tree_render_nodes_cull_offscreen_nodes() {
        let mut spec = sample_tree_spec();
        spec.offset_x = -400;

        let nodes = tree_render_nodes(&spec).unwrap();

        assert!(nodes.len() < spec.nodes.len());
        assert!(!nodes.iter().any(|node| node.id == "root"));
    }

    #[test]
    fn tree_pan_updates_offsets() {
        let spec = sample_tree_spec();
        let panned = pan_tree_spec(&spec, 18.0, -9.0).unwrap();

        assert_eq!(panned.offset_x, 18);
        assert_eq!(panned.offset_y, -9);
    }

    #[test]
    fn tree_node_style_inheritance_and_overrides_resolve_in_order() {
        let mut spec = sample_tree_spec();
        spec.default_node_style = Some(GraphNodeStyle {
            shape: Some(NodeShape::Square),
            radius: Some(24.0),
            label_visible: Some(false),
            ..Default::default()
        });
        spec.nodes[0].shape = Some(NodeShape::Diamond);
        spec.nodes[1].style = Some(GraphNodeStyle {
            shape: Some(NodeShape::Triangle),
            label_visible: Some(true),
            ..Default::default()
        });

        let resolved = resolve_tree_nodes(&spec).unwrap();

        assert_eq!(resolved["root"].style.shape, NodeShape::Diamond);
        assert_eq!(resolved["left"].style.shape, NodeShape::Triangle);
        assert_eq!(resolved["right"].style.shape, NodeShape::Square);
        assert!(!resolved["root"].style.label_visible);
        assert!(resolved["left"].style.label_visible);
        assert_eq!(resolved["right"].style.radius, 24);
    }

    #[test]
    fn tree_hit_test_uses_per_node_radius_override() {
        let mut spec = sample_tree_spec();
        spec.node_radius = 8;
        spec.selection_style = Some(SelectionStyle {
            padding: Some(0.0),
            ..Default::default()
        });
        spec.nodes[1].style = Some(GraphNodeStyle {
            radius: Some(28.0),
            ..Default::default()
        });

        let validated = validate_tree(&spec).unwrap();
        let layout = build_layout(&spec, &validated.children_by_parent);
        let target = layout["left"];
        let selected =
            pick_tree_node(&spec, f64::from(target.x + 20), f64::from(target.y)).unwrap();

        assert_eq!(selected.as_deref(), Some("left"));
    }

    #[test]
    fn tree_edge_styles_render_graph_defaults_and_per_edge_overrides() {
        let mut svg = String::new();
        let mut spec = sample_tree_spec();
        spec.default_edge_style = Some(GraphEdgeStyle {
            stroke_width: Some(4.0),
            ..Default::default()
        });
        spec.edges[0].style = Some(GraphEdgeStyle {
            stroke_width: Some(6.0),
            ..Default::default()
        });
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();

        render_tree_on(area, &spec).unwrap();

        assert!(svg.contains("stroke-width=\"6\""));
        assert!(svg.contains("stroke-width=\"4\""));
    }

    #[test]
    fn tree_collapsing_node_hides_descendants_and_reselects_parent() {
        let mut session = TreeSession::new(sample_tree_spec()).unwrap();
        session.set_selection(Some("leaf".to_string()));

        let collapsed = session.toggle_collapse("right");

        assert!(collapsed);
        assert_eq!(session.spec().selected_node_id.as_deref(), Some("right"));

        let rendered = session.render_nodes();
        assert!(rendered.iter().any(|node| node.id == "right"));
        assert!(!rendered.iter().any(|node| node.id == "leaf"));
    }

    #[test]
    fn tree_collapsed_layout_compacts_hidden_branch() {
        let mut spec = sample_tree_spec();
        spec.nodes.push(crate::TreeNode {
            id: "leaf-two".to_string(),
            name: Some("Second Leaf".to_string()),
            label: "Leaf 2".to_string(),
            color: None,
            shape: None,
            label_inside: None,
            style: None,
            media: None,
            properties: Default::default(),
        });
        spec.edges.push(crate::TreeEdge {
            source: "right".to_string(),
            target: "leaf-two".to_string(),
            style: None,
        });

        let expanded = TreeSession::new(spec.clone()).unwrap();

        let mut collapsed_spec = spec;
        collapsed_spec.collapsed_node_ids = vec!["right".to_string()];
        let collapsed = TreeSession::new(collapsed_spec).unwrap();

        assert!(collapsed.layout["right"].y > collapsed.layout["root"].y);
        assert!(collapsed.layout["left"].x > expanded.layout["left"].x);
        assert!(!collapsed.layout.contains_key("leaf"));
        assert!(!collapsed.layout.contains_key("leaf-two"));
    }

    #[test]
    fn tree_ignores_unknown_or_leaf_collapsed_ids() {
        let mut spec = sample_tree_spec();
        spec.collapsed_node_ids = vec![
            "missing".to_string(),
            "leaf".to_string(),
            "right".to_string(),
            "right".to_string(),
        ];

        let session = TreeSession::new(spec).unwrap();

        assert_eq!(session.spec().collapsed_node_ids, vec!["right".to_string()]);
    }

    #[test]
    fn tree_toggle_collapse_is_noop_for_leaf() {
        let mut session = TreeSession::new(sample_tree_spec()).unwrap();

        let collapsed = session.toggle_collapse("leaf");

        assert!(!collapsed);
        assert!(session.layout.contains_key("leaf"));
        assert!(session.spec().collapsed_node_ids.is_empty());
    }

    #[test]
    fn tree_transition_render_supports_intermediate_frames() {
        let mut session = TreeSession::new(sample_tree_spec()).unwrap();
        session.toggle_collapse("right");

        let mut svg = String::new();
        let area = SVGBackend::with_string(&mut svg, (session.width(), session.height()))
            .into_drawing_area();

        session.render_transition_on(area, 0.5).unwrap();
        let nodes = session.render_transition_nodes(0.5);

        assert!(svg.contains("Root"));
        assert!(nodes.iter().any(|node| node.id == "right"));
        assert!(nodes.iter().any(|node| node.id == "leaf"));
    }
}
