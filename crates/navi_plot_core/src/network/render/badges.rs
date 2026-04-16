use super::*;

pub(in crate::network) fn outward_unit_from_parent(
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

pub(in crate::network) fn toggle_badge_for_node_frame(
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

pub(in crate::network) fn toggle_badge_for_node(
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

pub(in crate::network) fn draw_toggle_badge<DB>(
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
