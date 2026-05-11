use super::*;

fn lerp_point(from: (f64, f64), to: (f64, f64), progress: f64) -> (f64, f64) {
    (
        from.0 + (to.0 - from.0) * progress,
        from.1 + (to.1 - from.1) * progress,
    )
}

pub(in crate::network) fn tracking_uses_completed_overlay(tracking: &NetworkTrackedPath) -> bool {
    tracking.progress >= 1.0
}

pub(in crate::network) fn draw_tracking_edges<DB>(
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

    if tracking_uses_completed_overlay(tracking) {
        return Ok(());
    }

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

pub(in crate::network) fn tracking_edge_color(_tracking: &NetworkTrackedPath) -> RGBColor {
    TRACKING_EDGE_COLOR
}

pub(in crate::network) fn tracking_edge_opacity(
    tracking: &NetworkTrackedPath,
    completion: f64,
) -> f64 {
    if tracking.progress >= 1.0 {
        TRACKING_EDGE_OPACITY
    } else {
        TRACKING_EDGE_OPACITY * (0.35 + 0.65 * completion)
    }
}
