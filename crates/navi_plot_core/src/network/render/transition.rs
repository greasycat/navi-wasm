use super::*;
use crate::font_family;

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

pub(in crate::network) fn render_transition_with_layout<DB>(
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

    draw_network_title(root, spec)?;

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
                let text_style = TextStyle::from(
                    (font_family(spec.font_family.as_deref()), label_size).into_font(),
                )
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
            spec.font_family.as_deref(),
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

pub(in crate::network) fn render_transition_nodes_with_layout(
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
