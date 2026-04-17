use super::*;
use crate::font_family;

#[derive(Debug, Clone, Copy)]
pub(super) struct TransitionNodeState {
    pub(super) point: (f64, f64),
    pub(super) opacity: f64,
}

pub(super) fn render_with_layout<DB>(
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
            spec.font_family.as_deref(),
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

pub(super) fn draw_tree_title<DB>(root: &PlotArea<DB>, spec: &TreePlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    if spec.title.is_empty() {
        return Ok(());
    }

    let title_size = (22.0 * spec.pixel_ratio.max(0.25)).round() as u32;
    let title_style =
        TextStyle::from((font_family(spec.font_family.as_deref()), title_size).into_font())
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

pub(super) fn resolved_tree_node<'a>(
    resolved_nodes: &'a BTreeMap<String, ResolvedTreeNode>,
    node_id: &str,
) -> Result<&'a ResolvedTreeNode, PlotError> {
    resolved_nodes
        .get(node_id)
        .ok_or_else(|| PlotError::UnknownNode {
            node_id: node_id.to_string(),
        })
}

pub(super) fn draw_tree_node<DB>(
    root: &PlotArea<DB>,
    position: (i32, i32),
    style: &ResolvedNodeStyle,
    media: Option<&ResolvedNodeMedia>,
    label: &str,
    is_selected: bool,
    selection_style: &crate::graph_style::ResolvedSelectionStyle,
    font_scale: f64,
    font_family_name: Option<&str>,
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
        font_family_name,
    )
}

pub(super) fn pick_from_layout(
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

pub(super) fn render_nodes_with_layout(
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

pub(super) fn project_point(point: LayoutPoint, view: &ScreenTransform) -> (i32, i32) {
    view.apply((point.x as f64, point.y as f64))
}

pub(super) fn project_point_f64(point: (f64, f64), view: &ScreenTransform) -> (i32, i32) {
    view.apply(point)
}

pub(super) fn transition_anchor_frame(
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

pub(super) fn transition_node_frame(
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

pub(super) fn transition_phase_opacity(from_present: bool, to_present: bool, progress: f64) -> f64 {
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

pub(super) fn node_intersects_viewport(
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

pub(super) fn scale_node_style(style: &ResolvedNodeStyle, zoom: f64) -> ResolvedNodeStyle {
    let mut scaled = style.clone();
    scaled.radius = ((scaled.radius.max(1) as f64) * zoom).round() as i32;
    scaled.radius = scaled.radius.max(1);
    scaled
}

pub(super) fn scale_selection_style(
    style: &crate::graph_style::ResolvedSelectionStyle,
    zoom: f64,
) -> crate::graph_style::ResolvedSelectionStyle {
    let mut scaled = style.clone();
    scaled.padding = ((scaled.padding.max(0) as f64) * zoom).round() as i32;
    scaled.padding = scaled.padding.max(0);
    scaled
}

pub(super) fn draw_collapsed_marker<DB>(
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
