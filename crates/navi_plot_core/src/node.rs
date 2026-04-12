use crate::types::NodeShape;
use crate::{backend_error, PlotArea, PlotError};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};

/// Draw a single node onto `root`.
///
/// * Selection ring (same shape, `ring_padding` larger) is drawn when `is_selected`.
/// * When `label_inside` is `true`, the label is centered inside the shape in white.
/// * When `label_inside` is `false`, the label is drawn below the shape in black.
pub(crate) fn draw_node<DB>(
    root: &PlotArea<DB>,
    cx: i32,
    cy: i32,
    radius: i32,
    color: RGBColor,
    shape: &NodeShape,
    label: &str,
    label_inside: bool,
    is_selected: bool,
    ring_padding: i32,
    font_scale: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    // Selection ring — outline in same shape, slightly enlarged
    if is_selected {
        let ring_style = ShapeStyle::from(&BLACK.mix(0.9)).stroke_width(2);
        draw_shape(root, cx, cy, radius + ring_padding, shape, ring_style)?;
    }

    // Node body — filled
    let fill_style = ShapeStyle::from(&color).filled();
    draw_shape(root, cx, cy, radius, shape, fill_style)?;

    // Label — font sizes are scaled by font_scale so they appear correct on HiDPI
    if !label.is_empty() {
        let scale = font_scale.max(0.25);
        if label_inside {
            let size = (12.0 * scale).round() as u32;
            let style = TextStyle::from(("sans-serif", size).into_font())
                .pos(Pos::new(HPos::Center, VPos::Center))
                .color(&WHITE);
            root.draw(&Text::new(label.to_owned(), (cx, cy), style))
                .map_err(backend_error)?;
        } else {
            let size = (13.0 * scale).round() as u32;
            let style = TextStyle::from(("sans-serif", size).into_font())
                .pos(Pos::new(HPos::Center, VPos::Top))
                .color(&BLACK);
            root.draw(&Text::new(label.to_owned(), (cx, cy + radius + 4), style))
                .map_err(backend_error)?;
        }
    }

    Ok(())
}

fn draw_shape<DB>(
    root: &PlotArea<DB>,
    cx: i32,
    cy: i32,
    r: i32,
    shape: &NodeShape,
    style: ShapeStyle,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    match shape {
        NodeShape::Circle => {
            root.draw(&Circle::new((cx, cy), r, style))
                .map_err(backend_error)?;
        }
        NodeShape::Square => {
            root.draw(&Rectangle::new(
                [(cx - r, cy - r), (cx + r, cy + r)],
                style,
            ))
            .map_err(backend_error)?;
        }
        NodeShape::Diamond => {
            root.draw(&Polygon::new(
                vec![(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)],
                style,
            ))
            .map_err(backend_error)?;
        }
        NodeShape::Triangle => {
            // Upward-pointing triangle
            root.draw(&Polygon::new(
                vec![(cx, cy - r), (cx + r, cy + r), (cx - r, cy + r)],
                style,
            ))
            .map_err(backend_error)?;
        }
    }
    Ok(())
}

/// Return `true` if canvas point `(x, y)` falls within the node.
///
/// `radius` should already include any hit-area expansion (e.g. ring padding).
pub(crate) fn node_contains(shape: &NodeShape, cx: f64, cy: f64, radius: f64, x: f64, y: f64) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    match shape {
        NodeShape::Circle => dx * dx + dy * dy <= radius * radius,
        NodeShape::Square => dx.abs() <= radius && dy.abs() <= radius,
        NodeShape::Diamond => dx.abs() + dy.abs() <= radius,
        // Triangle approximated by bounding circle for hit-testing
        NodeShape::Triangle => dx * dx + dy * dy <= radius * radius,
    }
}
