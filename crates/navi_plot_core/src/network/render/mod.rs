use super::*;

mod badges;
mod interaction;
mod tracking;
mod transition;

pub(in crate::network) use self::badges::*;
pub(in crate::network) use self::interaction::*;
pub(in crate::network) use self::tracking::*;
pub(in crate::network) use self::transition::*;

pub(in crate::network) fn draw_network_title<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    if spec.title.is_empty() {
        return Ok(());
    }

    let title_size = (20.0 * spec.pixel_ratio.max(0.25)).round() as u32;
    root.draw(&Text::new(
        spec.title.clone(),
        (
            spec.width as i32 / 2 - spec.title.len() as i32 * 7,
            spec.margin as i32 / 2,
        ),
        ("sans-serif", title_size).into_font(),
    ))
    .map_err(backend_error)?;
    Ok(())
}

pub(in crate::network) fn render_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    tracking: Option<&NetworkTrackedPath>,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    root.fill(&WHITE).map_err(backend_error)?;
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    let parent_by_id = structural_parent_map(spec);

    draw_network_title(root, spec)?;

    for edge in &spec.edges {
        let Some(&src_pos) = layout.get(&edge.source) else {
            continue;
        };
        let Some(&tgt_pos) = layout.get(&edge.target) else {
            continue;
        };
        let edge_style = resolve_network_edge_style(spec, edge)?;
        let target_radius = resolved
            .get(&edge.target)
            .map(|node| scale_node_style(&node.style, view.zoom).radius)
            .unwrap_or(spec.node_radius.max(1) as i32);
        let (sx, sy) = view.apply(src_pos);
        let (tx, ty) = view.apply(tgt_pos);
        let Some(((line_start_x, line_start_y), (line_end_x, line_end_y))) =
            viewport.clip_line((sx, sy), (tx, ty))
        else {
            continue;
        };

        if edge_style.stroke_width > 0 {
            let shape_style = ShapeStyle::from(&edge_style.stroke_color.mix(edge_style.opacity))
                .stroke_width(edge_style.stroke_width);
            for (segment_start, segment_end) in edge_line_segments(
                (line_start_x, line_start_y),
                (line_end_x, line_end_y),
                edge_style.dash_pattern.as_deref(),
            ) {
                root.draw(&PathElement::new(
                    vec![segment_start, segment_end],
                    shape_style,
                ))
                .map_err(backend_error)?;
            }
        }

        if edge_style.arrow_visible
            && edge_style.stroke_width > 0
            && ((line_start_x, line_start_y) != (line_end_x, line_end_y))
        {
            let dx = line_end_x as f64 - line_start_x as f64;
            let dy = line_end_y as f64 - line_start_y as f64;
            let len = (dx * dx + dy * dy).sqrt().max(0.01);
            let ux = dx / len;
            let uy = dy / len;
            let perp_x = -uy;
            let perp_y = ux;
            let target_visible = resolved
                .get(&edge.target)
                .map(|node| {
                    let scaled_style = scale_node_style(&node.style, view.zoom);
                    viewport.intersects_circle((tx, ty), scaled_style.radius.max(1))
                })
                .unwrap_or(false);

            let mut tip_x = line_end_x;
            let mut tip_y = line_end_y;
            if target_visible {
                let candidate = (
                    (tx as f64 - ux * target_radius as f64).round() as i32,
                    (ty as f64 - uy * target_radius as f64).round() as i32,
                );
                if viewport.contains(candidate) {
                    tip_x = candidate.0;
                    tip_y = candidate.1;
                }
            }
            let b1_x =
                (tip_x as f64 - ux * ARROW_LENGTH + perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b1_y =
                (tip_y as f64 - uy * ARROW_LENGTH + perp_y * ARROW_HALF_WIDTH).round() as i32;
            let b2_x =
                (tip_x as f64 - ux * ARROW_LENGTH - perp_x * ARROW_HALF_WIDTH).round() as i32;
            let b2_y =
                (tip_y as f64 - uy * ARROW_LENGTH - perp_y * ARROW_HALF_WIDTH).round() as i32;

            root.draw(&Polygon::new(
                vec![(tip_x, tip_y), (b1_x, b1_y), (b2_x, b2_y)],
                edge_style.stroke_color.mix(edge_style.opacity).filled(),
            ))
            .map_err(backend_error)?;
        }

        if edge_style.label_visible {
            if let Some(label) = edge.label.as_deref().filter(|label| !label.is_empty()) {
                let label_size = (12.0 * spec.pixel_ratio.max(0.25)).round() as u32;
                let label_color = edge_style.label_color.unwrap_or(edge_style.stroke_color);
                let text_color = label_color.mix(edge_style.opacity);
                let text_style = TextStyle::from(("sans-serif", label_size).into_font())
                    .pos(Pos::new(HPos::Center, VPos::Bottom))
                    .color(&text_color);
                let mid_x = ((line_start_x + line_end_x) as f64 / 2.0).round() as i32;
                let mid_y = ((line_start_y + line_end_y) as f64 / 2.0).round() as i32 - 4;
                root.draw(&Text::new(label.to_owned(), (mid_x, mid_y), text_style))
                    .map_err(backend_error)?;
            }
        }
    }

    draw_tracking_edges(root, spec, layout, view, tracking)?;

    for node_spec in &spec.nodes {
        let Some(&pos) = layout.get(&node_spec.id) else {
            continue;
        };
        let resolved_node = resolved.get(&node_spec.id).expect("resolved network node");
        let label = resolved_node.label.as_str();
        let (cx, cy) = view.apply(pos);
        let is_selected = spec
            .selected_node_id
            .as_deref()
            .is_some_and(|id| id == node_spec.id.as_str());
        let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        if !node_intersects_viewport(
            viewport,
            (cx, cy),
            &scaled_style,
            &scaled_selection_style,
            is_selected,
        ) {
            continue;
        }

        node::draw_node(
            root,
            cx,
            cy,
            &scaled_style,
            resolved_node.media.as_ref(),
            label,
            is_selected,
            &scaled_selection_style,
            spec.pixel_ratio,
        )?;
        if tracking.is_some_and(|tracking| tracking.is_traversed_node(node_spec.id.as_str())) {
            let tracking_outline = ShapeStyle::from(&TRACKING_NODE_BORDER_COLOR.mix(0.95))
                .stroke_width(TRACKING_NODE_BORDER_WIDTH.max(scaled_style.stroke_width));
            node::draw_shape(
                root,
                cx,
                cy,
                scaled_style.radius,
                &scaled_style.shape,
                tracking_outline,
            )?;
        }
        if let Some(badge) =
            toggle_badge_for_node(spec, node_spec, layout, resolved, view, &parent_by_id)
        {
            draw_toggle_badge(root, badge, 1.0)?;
        }
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

pub fn render_network_on<DB>(root: PlotArea<DB>, spec: &NetworkPlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    NetworkSession::new(spec.clone())?.render_on(root)
}

pub fn network_render_nodes(spec: &NetworkPlotSpec) -> Result<Vec<GraphNodeRenderInfo>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.render_nodes())
}

pub fn pick_network_hit(
    spec: &NetworkPlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<NetworkPickHit>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.pick(canvas_x, canvas_y))
}

pub fn pick_network_node(
    spec: &NetworkPlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<String>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.pick_node(canvas_x, canvas_y))
}

pub fn pan_network_spec(
    spec: &NetworkPlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<NetworkPlotSpec, PlotError> {
    let mut session = NetworkSession::new(spec.clone())?;
    session.pan(delta_x, delta_y);
    Ok(session.into_spec())
}

pub fn focus_network_view(
    spec: &NetworkPlotSpec,
    node_id: &str,
    options: Option<NetworkFocusOptions>,
) -> Result<Option<NetworkView>, PlotError> {
    Ok(NetworkSession::new(spec.clone())?.compute_focus_view(node_id, options))
}

pub(in crate::network) fn resolve_network_edge_style(
    spec: &NetworkPlotSpec,
    edge: &crate::NetworkEdge,
) -> Result<crate::graph_style::ResolvedEdgeStyle, PlotError> {
    resolve_edge_style(EdgeStyleContext {
        default_stroke_color: DEFAULT_EDGE_COLOR,
        default_stroke_width: 1,
        default_arrow_visible: spec.show_arrows,
        default_label_visible: false,
        graph_style: spec.default_edge_style.as_ref(),
        legacy_stroke_color: edge.color.as_deref(),
        item_style: edge.style.as_ref(),
    })
}
