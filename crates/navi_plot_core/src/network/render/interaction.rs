use super::*;

pub(in crate::network) fn pick_hit_from_layout(
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

pub(in crate::network) fn render_nodes_with_layout(
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

pub(in crate::network) fn node_intersects_viewport(
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

pub(in crate::network) fn scale_node_style(
    style: &ResolvedNodeStyle,
    zoom: f64,
) -> ResolvedNodeStyle {
    let mut scaled = style.clone();
    scaled.radius = ((scaled.radius.max(1) as f64) * zoom).round() as i32;
    scaled.radius = scaled.radius.max(1);
    if let Some(ref mut shadow) = scaled.shadow {
        shadow.blur = ((shadow.blur as f64) * zoom).round() as i32;
        shadow.offset_x = ((shadow.offset_x as f64) * zoom).round() as i32;
        shadow.offset_y = ((shadow.offset_y as f64) * zoom).round() as i32;
    }
    scaled
}

pub(in crate::network) fn scale_selection_style(
    style: &ResolvedSelectionStyle,
    zoom: f64,
) -> ResolvedSelectionStyle {
    let mut scaled = style.clone();
    scaled.padding = ((scaled.padding.max(0) as f64) * zoom).round() as i32;
    scaled.padding = scaled.padding.max(0);
    scaled
}
