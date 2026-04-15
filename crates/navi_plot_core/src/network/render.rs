use super::*;

pub(super) fn draw_tracking_edges<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    view: &ScreenTransform,
    tracking: Option<&NetworkTrackedPath>,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let Some(tracking) = tracking else {
        return Ok(());
    };

    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    for (edge_index, node_pair) in tracking.node_ids.windows(2).enumerate() {
        let completion = tracking.edge_completion(edge_index);
        if completion <= 0.0 {
            continue;
        }

        let Some(&source) = layout.get(node_pair[0].as_str()) else {
            continue;
        };
        let Some(&target) = layout.get(node_pair[1].as_str()) else {
            continue;
        };

        let partial_target = if completion >= 1.0 {
            target
        } else {
            lerp_point(source, target, completion)
        };
        let source = view.apply(source);
        let partial_target = view.apply(partial_target);
        let Some((clipped_source, clipped_target)) = viewport.clip_line(source, partial_target)
        else {
            continue;
        };

        let stroke_color = tracking_edge_color(tracking);
        let opacity = tracking_edge_opacity(tracking, completion);
        let shape_style = ShapeStyle::from(&stroke_color.mix(opacity.clamp(0.0, 1.0)))
            .stroke_width(TRACKING_EDGE_WIDTH);
        root.draw(&PathElement::new(
            vec![clipped_source, clipped_target],
            shape_style,
        ))
        .map_err(backend_error)?;
    }

    Ok(())
}

pub(super) fn tracking_breath_strength(phase: f64) -> f64 {
    let phase = if phase.is_finite() {
        phase.rem_euclid(1.0)
    } else {
        0.0
    };
    0.5 - 0.5 * (TAU * phase).cos()
}

pub(super) fn interpolate_rgb(from: RGBColor, to: RGBColor, t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    RGBColor(
        (from.0 as f64 + (to.0 as f64 - from.0 as f64) * t).round() as u8,
        (from.1 as f64 + (to.1 as f64 - from.1 as f64) * t).round() as u8,
        (from.2 as f64 + (to.2 as f64 - from.2 as f64) * t).round() as u8,
    )
}

pub(super) fn tracking_edge_color(tracking: &NetworkTrackedPath) -> RGBColor {
    if tracking.progress >= 1.0 {
        interpolate_rgb(
            TRACKING_EDGE_BREATH_COLOR,
            TRACKING_EDGE_COLOR,
            tracking_breath_strength(tracking.breath_phase),
        )
    } else {
        TRACKING_EDGE_COLOR
    }
}

pub(super) fn tracking_edge_opacity(tracking: &NetworkTrackedPath, completion: f64) -> f64 {
    if tracking.progress >= 1.0 {
        TRACKING_EDGE_OPACITY
    } else {
        TRACKING_EDGE_OPACITY * (0.35 + 0.65 * completion)
    }
}

pub(super) fn outward_unit_from_parent(
    spec: &NetworkPlotSpec,
    node_id: &str,
    node_point: (f64, f64),
    parent_point: Option<(f64, f64)>,
) -> (f64, f64) {
    let Some((parent_x, parent_y)) = parent_point else {
        return deterministic_unit(&format!("toggle:{node_id}:{}", spec.title));
    };
    let dx = node_point.0 - parent_x;
    let dy = node_point.1 - parent_y;
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.01 {
        (dx / len, dy / len)
    } else {
        deterministic_unit(&format!("toggle:{node_id}:{}", spec.title))
    }
}

pub(super) fn toggle_badge_for_node_frame(
    spec: &NetworkPlotSpec,
    node_spec: &crate::NetworkNode,
    resolved_node: &ResolvedNode,
    node_point: (f64, f64),
    parent_point: Option<(f64, f64)>,
    view: &ScreenTransform,
) -> Option<ToggleBadge> {
    if !node_has_toggle_badge(node_spec) {
        return None;
    }
    let (cx, cy) = view.apply(node_point);
    let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
    let badge_radius = (((scaled_style.radius.max(1) as f64) * 0.28).round() as i32)
        .clamp(TOGGLE_BADGE_MIN_RADIUS, TOGGLE_BADGE_MAX_RADIUS);
    let center_offset = (scaled_style.radius + badge_radius).max(badge_radius + 1) as f64;
    let (ux, uy) = outward_unit_from_parent(spec, &node_spec.id, node_point, parent_point);

    Some(ToggleBadge {
        center_x: (f64::from(cx) + ux * center_offset).round() as i32,
        center_y: (f64::from(cy) + uy * center_offset).round() as i32,
        radius: badge_radius,
        expanded: node_badge_expanded(node_spec),
    })
}

pub(super) fn toggle_badge_for_node(
    spec: &NetworkPlotSpec,
    node_spec: &crate::NetworkNode,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    view: &ScreenTransform,
    parent_by_id: &HashMap<&str, &str>,
) -> Option<ToggleBadge> {
    let resolved_node = resolved.get(&node_spec.id)?;
    let node_point = layout.get(&node_spec.id).copied()?;
    let parent_point = parent_by_id
        .get(node_spec.id.as_str())
        .and_then(|parent_id| layout.get(*parent_id).copied());
    toggle_badge_for_node_frame(
        spec,
        node_spec,
        resolved_node,
        node_point,
        parent_point,
        view,
    )
}

pub(super) fn draw_toggle_badge<DB>(
    root: &PlotArea<DB>,
    badge: ToggleBadge,
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
    node::draw_shape(
        root,
        badge.center_x,
        badge.center_y,
        badge.radius,
        &crate::NodeShape::Circle,
        ShapeStyle::from(&TOGGLE_BADGE_FILL.mix(0.96 * alpha)).filled(),
    )?;

    let symbol_half = (badge.radius as f64 * 0.55).round().clamp(2.0, 5.0) as i32;
    let symbol_width = ((badge.radius as f64) * 0.35).round().clamp(1.0, 2.0) as u32;
    let symbol_style = ShapeStyle::from(&TOGGLE_BADGE_SYMBOL.mix(alpha)).stroke_width(symbol_width);
    root.draw(&PathElement::new(
        vec![
            (badge.center_x - symbol_half, badge.center_y),
            (badge.center_x + symbol_half, badge.center_y),
        ],
        symbol_style,
    ))
    .map_err(backend_error)?;

    if !badge.expanded {
        root.draw(&PathElement::new(
            vec![
                (badge.center_x, badge.center_y - symbol_half),
                (badge.center_x, badge.center_y + symbol_half),
            ],
            symbol_style,
        ))
        .map_err(backend_error)?;
    }

    Ok(())
}

pub(super) fn render_with_layout<DB>(
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

    // Draw title
    if !spec.title.is_empty() {
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
    }

    // Draw edges first
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

        // Arrowhead
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

    // Draw nodes on top
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

#[derive(Debug, Clone, Copy)]
struct TransitionNodeState {
    point: (f64, f64),
    opacity: f64,
}

fn project_point_f64(point: (f64, f64), view: &ScreenTransform) -> (i32, i32) {
    view.apply(point)
}

fn lerp_point(from: (f64, f64), to: (f64, f64), progress: f64) -> (f64, f64) {
    (
        from.0 + (to.0 - from.0) * progress,
        from.1 + (to.1 - from.1) * progress,
    )
}

fn transition_anchor_frame(
    transition: &NetworkTransition,
    to_layout: &BTreeMap<String, (f64, f64)>,
    progress: f64,
) -> (f64, f64) {
    let anchor_from = transition
        .from_layout
        .get(&transition.anchor_node_id)
        .copied()
        .or_else(|| to_layout.get(&transition.anchor_node_id).copied())
        .unwrap_or((0.0, 0.0));
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
    lerp_point(anchor_from, anchor_to, progress)
}

fn transition_node_frame(
    node_id: &str,
    transition: &NetworkTransition,
    to_layout: &BTreeMap<String, (f64, f64)>,
    anchor_frame: (f64, f64),
    progress: f64,
) -> Option<TransitionNodeState> {
    let from = transition.from_layout.get(node_id).copied();
    let to = to_layout.get(node_id).copied();
    match (from, to) {
        (Some(from), Some(to)) => Some(TransitionNodeState {
            point: lerp_point(from, to, progress),
            opacity: 1.0,
        }),
        (Some(from), None) => Some(TransitionNodeState {
            point: lerp_point(from, anchor_frame, progress),
            opacity: 1.0 - progress,
        }),
        (None, Some(to)) => Some(TransitionNodeState {
            point: lerp_point(anchor_frame, to, progress),
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

fn ordered_transition_node_ids(
    spec: &NetworkPlotSpec,
    previous_spec: &NetworkPlotSpec,
) -> Vec<String> {
    let mut ordered = spec
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let mut seen = ordered.iter().cloned().collect::<HashSet<_>>();
    for node in &previous_spec.nodes {
        if seen.insert(node.id.clone()) {
            ordered.push(node.id.clone());
        }
    }
    ordered
}

pub(super) fn render_transition_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    transition: &NetworkTransition,
    progress: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let progress = progress.clamp(0.0, 1.0);
    root.fill(&WHITE).map_err(backend_error)?;
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    let anchor_frame = transition_anchor_frame(transition, layout, progress);
    let current_nodes_by_id = spec
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let previous_nodes_by_id = transition
        .from_spec
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let current_parent_by_id = structural_parent_map(spec);
    let previous_parent_by_id = structural_parent_map(&transition.from_spec);

    if !spec.title.is_empty() {
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
    }

    let mut edge_entries = BTreeMap::<
        (String, String),
        (Option<&crate::NetworkEdge>, Option<&crate::NetworkEdge>),
    >::new();
    for edge in &transition.from_spec.edges {
        edge_entries.insert(
            (edge.source.clone(), edge.target.clone()),
            (Some(edge), None),
        );
    }
    for edge in &spec.edges {
        edge_entries
            .entry((edge.source.clone(), edge.target.clone()))
            .and_modify(|entry| entry.1 = Some(edge))
            .or_insert((None, Some(edge)));
    }

    for ((source_id, target_id), (from_edge, to_edge)) in edge_entries {
        let Some(source_state) =
            transition_node_frame(&source_id, transition, layout, anchor_frame, progress)
        else {
            continue;
        };
        let Some(target_state) =
            transition_node_frame(&target_id, transition, layout, anchor_frame, progress)
        else {
            continue;
        };
        let edge_alpha = transition_phase_opacity(from_edge.is_some(), to_edge.is_some(), progress)
            .min(source_state.opacity)
            .min(target_state.opacity)
            .clamp(0.0, 1.0);
        if edge_alpha <= 0.0 {
            continue;
        }

        let render_spec = if to_edge.is_some() {
            spec
        } else {
            &transition.from_spec
        };
        let render_edge = to_edge.or(from_edge).expect("edge present in transition");
        let edge_style = resolve_network_edge_style(render_spec, render_edge)?;
        let target_resolved = resolved
            .get(target_id.as_str())
            .or_else(|| transition.from_resolved.get(target_id.as_str()));
        let target_radius = target_resolved
            .map(|node| scale_node_style(&node.style, view.zoom).radius)
            .unwrap_or(render_spec.node_radius.max(1) as i32);
        let source = project_point_f64(source_state.point, view);
        let target = project_point_f64(target_state.point, view);
        let Some((clipped_source, clipped_target)) = viewport.clip_line(source, target) else {
            continue;
        };

        if edge_style.stroke_width > 0 {
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

        if edge_style.arrow_visible
            && edge_style.stroke_width > 0
            && (clipped_source != clipped_target)
        {
            let dx = clipped_target.0 as f64 - clipped_source.0 as f64;
            let dy = clipped_target.1 as f64 - clipped_source.1 as f64;
            let len = (dx * dx + dy * dy).sqrt().max(0.01);
            let ux = dx / len;
            let uy = dy / len;
            let perp_x = -uy;
            let perp_y = ux;
            let target_visible = target_resolved
                .map(|node| {
                    let scaled_style = scale_node_style(&node.style, view.zoom);
                    viewport.intersects_circle(target, scaled_style.radius.max(1))
                })
                .unwrap_or(false);

            let mut tip_x = clipped_target.0;
            let mut tip_y = clipped_target.1;
            if target_visible {
                let candidate = (
                    (target.0 as f64 - ux * target_radius as f64).round() as i32,
                    (target.1 as f64 - uy * target_radius as f64).round() as i32,
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
                edge_style
                    .stroke_color
                    .mix(edge_style.opacity * edge_alpha)
                    .filled(),
            ))
            .map_err(backend_error)?;
        }

        if edge_style.label_visible {
            if let Some(label) = render_edge
                .label
                .as_deref()
                .filter(|label| !label.is_empty())
            {
                let label_size = (12.0 * spec.pixel_ratio.max(0.25)).round() as u32;
                let label_color = edge_style.label_color.unwrap_or(edge_style.stroke_color);
                let text_color = label_color.mix(edge_style.opacity * edge_alpha);
                let text_style = TextStyle::from(("sans-serif", label_size).into_font())
                    .pos(Pos::new(HPos::Center, VPos::Bottom))
                    .color(&text_color);
                let mid_x = ((clipped_source.0 + clipped_target.0) as f64 / 2.0).round() as i32;
                let mid_y = ((clipped_source.1 + clipped_target.1) as f64 / 2.0).round() as i32 - 4;
                root.draw(&Text::new(label.to_owned(), (mid_x, mid_y), text_style))
                    .map_err(backend_error)?;
            }
        }
    }

    for node_id in ordered_transition_node_ids(spec, &transition.from_spec) {
        let Some(node_state) =
            transition_node_frame(&node_id, transition, layout, anchor_frame, progress)
        else {
            continue;
        };
        if node_state.opacity <= 0.0 {
            continue;
        }

        let current_node = current_nodes_by_id.get(node_id.as_str()).copied();
        let previous_node = previous_nodes_by_id.get(node_id.as_str()).copied();
        let Some(node_spec) = current_node.or(previous_node) else {
            continue;
        };
        let Some(resolved_node) = resolved
            .get(node_id.as_str())
            .or_else(|| transition.from_resolved.get(node_id.as_str()))
        else {
            continue;
        };
        let position = project_point_f64(node_state.point, view);
        let mut scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        scaled_style.opacity = (scaled_style.opacity * node_state.opacity).clamp(0.0, 1.0);
        let selection_alpha = transition_phase_opacity(
            transition.from_selected_node_id.as_deref() == Some(node_id.as_str()),
            spec.selected_node_id.as_deref() == Some(node_id.as_str()),
            progress,
        );
        let mut scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        scaled_selection_style.opacity =
            (scaled_selection_style.opacity * selection_alpha).clamp(0.0, 1.0);
        if !node_intersects_viewport(
            viewport,
            position,
            &scaled_style,
            &scaled_selection_style,
            selection_alpha > 0.0,
        ) {
            continue;
        }

        node::draw_node(
            root,
            position.0,
            position.1,
            &scaled_style,
            resolved_node.media.as_ref(),
            resolved_node.label.as_str(),
            selection_alpha > 0.0,
            &scaled_selection_style,
            spec.pixel_ratio,
        )?;

        let render_spec = current_node.map(|_| spec).unwrap_or(&transition.from_spec);
        let parent_point = current_parent_by_id
            .get(node_id.as_str())
            .copied()
            .or_else(|| previous_parent_by_id.get(node_id.as_str()).copied())
            .and_then(|parent_id| {
                transition_node_frame(parent_id, transition, layout, anchor_frame, progress)
                    .map(|state| state.point)
            });
        if let Some(badge) = toggle_badge_for_node_frame(
            render_spec,
            node_spec,
            resolved_node,
            node_state.point,
            parent_point,
            view,
        ) {
            draw_toggle_badge(root, badge, node_state.opacity)?;
        }
    }

    root.present().map_err(backend_error)?;
    Ok(())
}

pub(super) fn render_transition_nodes_with_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    transition: &NetworkTransition,
    progress: f64,
) -> Vec<GraphNodeRenderInfo> {
    let progress = progress.clamp(0.0, 1.0);
    let anchor_frame = transition_anchor_frame(transition, layout, progress);
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);

    ordered_transition_node_ids(spec, &transition.from_spec)
        .into_iter()
        .filter_map(|node_id| {
            let node_state =
                transition_node_frame(&node_id, transition, layout, anchor_frame, progress)?;
            if node_state.opacity <= 0.0 {
                return None;
            }
            let resolved_node = resolved
                .get(node_id.as_str())
                .or_else(|| transition.from_resolved.get(node_id.as_str()))?;
            let position = project_point_f64(node_state.point, view);
            let mut scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            scaled_style.opacity = (scaled_style.opacity * node_state.opacity).clamp(0.0, 1.0);
            let selection_alpha = transition_phase_opacity(
                transition.from_selected_node_id.as_deref() == Some(node_id.as_str()),
                spec.selected_node_id.as_deref() == Some(node_id.as_str()),
                progress,
            );
            let mut scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            scaled_selection_style.opacity =
                (scaled_selection_style.opacity * selection_alpha).clamp(0.0, 1.0);
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
                id: node_id,
                center_x: position.0,
                center_y: position.1,
                radius: scaled_style.radius,
                shape: scaled_style.shape.clone(),
                opacity: scaled_style.opacity,
                media: resolved_node.media.clone(),
            })
        })
        .collect()
}

pub(super) fn pick_hit_from_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
    canvas_x: f64,
    canvas_y: f64,
) -> Option<NetworkPickHit> {
    if !canvas_x.is_finite() || !canvas_y.is_finite() {
        return None;
    }
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    let parent_by_id = structural_parent_map(spec);

    let toggle_hit = spec.nodes.iter().find_map(|node_spec| {
        let resolved_node = resolved.get(&node_spec.id)?;
        let &(px, py) = layout.get(&node_spec.id)?;
        let (screen_x, screen_y) = view.apply((px, py));
        let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
        let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
        let is_selected = spec
            .selected_node_id
            .as_deref()
            .is_some_and(|id| id == node_spec.id.as_str());
        if !node_intersects_viewport(
            viewport,
            (screen_x, screen_y),
            &scaled_style,
            &scaled_selection_style,
            is_selected,
        ) {
            return None;
        }
        let badge = toggle_badge_for_node(spec, node_spec, layout, resolved, view, &parent_by_id)?;
        let dx = f64::from(badge.center_x) - canvas_x;
        let dy = f64::from(badge.center_y) - canvas_y;
        (dx * dx + dy * dy <= f64::from(badge.radius * badge.radius)).then_some(NetworkPickHit {
            kind: NetworkPickKind::Toggle,
            node_id: node_spec.id.clone(),
        })
    });
    if toggle_hit.is_some() {
        return toggle_hit;
    }

    spec.nodes
        .iter()
        .filter_map(|node_spec| {
            let &(px, py) = layout.get(&node_spec.id)?;
            let resolved_node = resolved.get(&node_spec.id)?;
            let (screen_x, screen_y) = view.apply((px, py));
            let dx = f64::from(screen_x) - canvas_x;
            let dy = f64::from(screen_y) - canvas_y;
            let dist_sq = dx * dx + dy * dy;
            let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            let is_selected = spec
                .selected_node_id
                .as_deref()
                .is_some_and(|id| id == node_spec.id.as_str());
            if !node_intersects_viewport(
                viewport,
                (screen_x, screen_y),
                &scaled_style,
                &scaled_selection_style,
                is_selected,
            ) {
                return None;
            }
            let hit_radius =
                f64::from(scaled_style.radius.max(1) + scaled_selection_style.padding.max(0));
            node::node_contains(
                &scaled_style.shape,
                f64::from(screen_x),
                f64::from(screen_y),
                hit_radius,
                canvas_x,
                canvas_y,
            )
            .then_some((
                NetworkPickHit {
                    kind: NetworkPickKind::Node,
                    node_id: node_spec.id.clone(),
                },
                dist_sq,
            ))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(hit, _)| hit)
}

pub(super) fn render_nodes_with_layout(
    spec: &NetworkPlotSpec,
    layout: &BTreeMap<String, (f64, f64)>,
    resolved: &BTreeMap<String, ResolvedNode>,
    selection_style: &ResolvedSelectionStyle,
    view: &ScreenTransform,
) -> Vec<GraphNodeRenderInfo> {
    let viewport = PixelBounds::from_canvas(spec.width, spec.height);
    spec.nodes
        .iter()
        .filter_map(|node_spec| {
            let &(px, py) = layout.get(&node_spec.id)?;
            let resolved_node = resolved.get(&node_spec.id)?;
            let (screen_x, screen_y) = view.apply((px, py));
            let scaled_style = scale_node_style(&resolved_node.style, view.zoom);
            let scaled_selection_style = scale_selection_style(selection_style, view.zoom);
            let is_selected = spec
                .selected_node_id
                .as_deref()
                .is_some_and(|id| id == node_spec.id.as_str());
            if !node_intersects_viewport(
                viewport,
                (screen_x, screen_y),
                &scaled_style,
                &scaled_selection_style,
                is_selected,
            ) {
                return None;
            }
            Some(GraphNodeRenderInfo {
                id: node_spec.id.clone(),
                center_x: screen_x,
                center_y: screen_y,
                radius: scaled_style.radius,
                shape: scaled_style.shape.clone(),
                opacity: scaled_style.opacity,
                media: resolved_node.media.clone(),
            })
        })
        .collect()
}

pub(super) fn node_intersects_viewport(
    viewport: PixelBounds,
    center: (i32, i32),
    style: &ResolvedNodeStyle,
    selection_style: &ResolvedSelectionStyle,
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
    style: &ResolvedSelectionStyle,
    zoom: f64,
) -> ResolvedSelectionStyle {
    let mut scaled = style.clone();
    scaled.padding = ((scaled.padding.max(0) as f64) * zoom).round() as i32;
    scaled.padding = scaled.padding.max(0);
    scaled
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

pub(super) fn resolve_network_edge_style(
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
