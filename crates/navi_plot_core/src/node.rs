use crate::color::parse_color;
use crate::graph_style::{ResolvedNodeStyle, ResolvedSelectionStyle};
use crate::types::{BuiltinNodeIcon, NodeMedia, NodeMediaFit, NodeMediaKind, NodeShape};
use crate::{backend_error, PlotArea, PlotError};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use std::f64::consts::PI;

const DEFAULT_MEDIA_SCALE: f64 = 0.7;

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedNodeMediaKind {
    Icon(BuiltinNodeIcon),
    Image {
        image_key: String,
        fit: NodeMediaFit,
        fallback_icon: Option<BuiltinNodeIcon>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedNodeMedia {
    pub kind: ResolvedNodeMediaKind,
    pub scale: f64,
    pub tint_color: RGBColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphNodeRenderInfo {
    pub id: String,
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub shape: NodeShape,
    pub opacity: f64,
    pub media: Option<ResolvedNodeMedia>,
}

pub(crate) fn resolve_node_media(
    media: Option<&NodeMedia>,
) -> Result<Option<ResolvedNodeMedia>, PlotError> {
    let Some(media) = media else {
        return Ok(None);
    };

    let scale = media.scale.unwrap_or(DEFAULT_MEDIA_SCALE);
    if !scale.is_finite() || !(0.2..=1.0).contains(&scale) {
        return Err(PlotError::InvalidStyleValue {
            field: "node_media.scale",
            value: scale,
            reason: "must be between 0.2 and 1 inclusive",
        });
    }

    let tint_color = match media.tint_color.as_deref() {
        Some(value) => parse_color(value)?,
        None => WHITE,
    };

    let kind = match media.kind {
        NodeMediaKind::Icon => {
            ResolvedNodeMediaKind::Icon(media.icon.clone().ok_or(PlotError::InvalidNodeMedia {
                field: "media.icon",
                reason: "is required when media.kind is \"icon\"",
            })?)
        }
        NodeMediaKind::Image => {
            let image_key = media
                .image_key
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .ok_or(PlotError::InvalidNodeMedia {
                    field: "media.image_key",
                    reason: "is required when media.kind is \"image\"",
                })?;

            ResolvedNodeMediaKind::Image {
                image_key,
                fit: media.fit.clone(),
                fallback_icon: media.fallback_icon.clone(),
            }
        }
    };

    Ok(Some(ResolvedNodeMedia {
        kind,
        scale,
        tint_color,
    }))
}

/// Draw a single node onto `root`.
///
/// * Selection ring (same shape, `ring_padding` larger) is drawn when `is_selected`.
/// * Media is drawn after the node body.
/// * When media is present, labels are rendered below the node even if
///   `label_inside` is `true`.
pub(crate) fn draw_node<DB>(
    root: &PlotArea<DB>,
    cx: i32,
    cy: i32,
    node_style: &ResolvedNodeStyle,
    media: Option<&ResolvedNodeMedia>,
    label: &str,
    is_selected: bool,
    selection_style: &ResolvedSelectionStyle,
    font_scale: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    if is_selected && selection_style.stroke_width > 0 {
        let ring_style =
            ShapeStyle::from(&selection_style.stroke_color.mix(selection_style.opacity))
                .stroke_width(selection_style.stroke_width);
        draw_shape(
            root,
            cx,
            cy,
            node_style.radius + selection_style.padding,
            &node_style.shape,
            ring_style,
        )?;
    }

    let fill_style = ShapeStyle::from(&node_style.fill_color.mix(node_style.opacity)).filled();
    draw_shape(
        root,
        cx,
        cy,
        node_style.radius,
        &node_style.shape,
        fill_style,
    )?;

    if node_style.stroke_width > 0 {
        let outline_color = node_style.stroke_color.unwrap_or(node_style.fill_color);
        let outline_style = ShapeStyle::from(&outline_color.mix(node_style.opacity))
            .stroke_width(node_style.stroke_width);
        draw_shape(
            root,
            cx,
            cy,
            node_style.radius,
            &node_style.shape,
            outline_style,
        )?;
    }

    if let Some(media) = media {
        draw_media(root, cx, cy, node_style, media, node_style.opacity)?;
    }

    if node_style.label_visible && !label.is_empty() {
        let scale = font_scale.max(0.25);
        let label_inside = node_style.label_inside && media.is_none();
        let resolved_label_color =
            node_style
                .label_color
                .unwrap_or(if label_inside { WHITE } else { BLACK });
        let text_color = resolved_label_color.mix(node_style.opacity);

        if label_inside {
            let size = (12.0 * scale).round() as u32;
            let text_style = TextStyle::from(("sans-serif", size).into_font())
                .pos(Pos::new(HPos::Center, VPos::Center))
                .color(&text_color);
            root.draw(&Text::new(label.to_owned(), (cx, cy), text_style))
                .map_err(backend_error)?;
        } else {
            let size = (13.0 * scale).round() as u32;
            let text_style = TextStyle::from(("sans-serif", size).into_font())
                .pos(Pos::new(HPos::Center, VPos::Top))
                .color(&text_color);
            root.draw(&Text::new(
                label.to_owned(),
                (cx, cy + node_style.radius + 4),
                text_style,
            ))
            .map_err(backend_error)?;
        }
    }

    Ok(())
}

fn draw_media<DB>(
    root: &PlotArea<DB>,
    cx: i32,
    cy: i32,
    node_style: &ResolvedNodeStyle,
    media: &ResolvedNodeMedia,
    opacity: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let fallback_icon = match &media.kind {
        ResolvedNodeMediaKind::Icon(icon) => Some(icon),
        ResolvedNodeMediaKind::Image { fallback_icon, .. } => fallback_icon.as_ref(),
    };

    let Some(icon) = fallback_icon else {
        return Ok(());
    };

    let icon_radius = ((node_style.radius.max(2) as f64) * media.scale).round() as i32;
    draw_builtin_icon(
        root,
        cx,
        cy,
        icon_radius.max(2),
        icon,
        media.tint_color,
        node_style.fill_color,
        opacity,
    )
}

pub(crate) fn draw_shape<DB>(
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
            root.draw(&Rectangle::new([(cx - r, cy - r), (cx + r, cy + r)], style))
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
            root.draw(&Polygon::new(
                vec![(cx, cy - r), (cx + r, cy + r), (cx - r, cy + r)],
                style,
            ))
            .map_err(backend_error)?;
        }
    }
    Ok(())
}

fn draw_builtin_icon<DB>(
    root: &PlotArea<DB>,
    cx: i32,
    cy: i32,
    radius: i32,
    icon: &BuiltinNodeIcon,
    tint: RGBColor,
    background: RGBColor,
    opacity: f64,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let alpha = opacity.clamp(0.0, 1.0);
    let fill = ShapeStyle::from(&tint.mix(alpha)).filled();
    let stroke_width = ((radius as f64) * 0.18).round().clamp(1.0, 3.0) as u32;
    let stroke = ShapeStyle::from(&tint.mix(alpha)).stroke_width(stroke_width);
    let cutout = ShapeStyle::from(&background.mix(alpha)).filled();

    match icon {
        BuiltinNodeIcon::Star => {
            let outer = radius as f64;
            let inner = outer * 0.45;
            let points = (0..10)
                .map(|index| {
                    let angle = -PI / 2.0 + index as f64 * PI / 5.0;
                    let r = if index % 2 == 0 { outer } else { inner };
                    (
                        (cx as f64 + angle.cos() * r).round() as i32,
                        (cy as f64 + angle.sin() * r).round() as i32,
                    )
                })
                .collect::<Vec<_>>();
            root.draw(&Polygon::new(points, fill))
                .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Galaxy => {
            let rx = (radius as f64 * 0.95).round() as i32;
            let ry = (radius as f64 * 0.5).round() as i32;
            root.draw(&Circle::new(
                (cx, cy),
                (radius as f64 * 0.16).round() as i32,
                fill,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                ellipse_points(cx, cy, rx, ry, 0.0),
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                ellipse_points(cx, cy, rx, ry, PI / 3.0),
                ShapeStyle::from(&tint.mix(alpha)).stroke_width(stroke_width),
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Planet => {
            root.draw(&Circle::new(
                (cx, cy),
                (radius as f64 * 0.48).round() as i32,
                fill,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                ellipse_points(
                    cx,
                    cy,
                    (radius as f64 * 0.95) as i32,
                    (radius as f64 * 0.35) as i32,
                    -PI / 7.0,
                ),
                stroke,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Moon => {
            let moon_r = (radius as f64 * 0.62).round() as i32;
            root.draw(&Circle::new((cx, cy), moon_r, fill))
                .map_err(backend_error)?;
            root.draw(&Circle::new(
                (
                    cx + (radius as f64 * 0.28).round() as i32,
                    cy - (radius as f64 * 0.06).round() as i32,
                ),
                moon_r,
                cutout,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Telescope => {
            root.draw(&PathElement::new(
                vec![
                    (cx - radius / 2, cy - radius / 6),
                    (cx + radius / 3, cy - radius / 2),
                ],
                ShapeStyle::from(&tint).stroke_width(stroke_width + 1),
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx, cy), (cx - radius / 3, cy + radius / 2)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx, cy), (cx + radius / 6, cy + radius / 2)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx, cy), (cx + radius / 2, cy + radius / 3)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&Circle::new(
                (cx - radius / 5, cy - radius / 10),
                (radius as f64 * 0.11).round() as i32,
                fill,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Camera => {
            let left = cx - (radius as f64 * 0.7).round() as i32;
            let right = cx + (radius as f64 * 0.7).round() as i32;
            let top = cy - (radius as f64 * 0.42).round() as i32;
            let bottom = cy + (radius as f64 * 0.42).round() as i32;
            root.draw(&Rectangle::new([(left, top), (right, bottom)], stroke))
                .map_err(backend_error)?;
            root.draw(&Rectangle::new(
                [(cx - radius / 3, top - radius / 4), (cx + radius / 8, top)],
                fill,
            ))
            .map_err(backend_error)?;
            root.draw(&Circle::new(
                (cx, cy),
                (radius as f64 * 0.28).round() as i32,
                stroke,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Alert => {
            root.draw(&Polygon::new(
                vec![
                    (cx, cy - radius),
                    (cx + radius, cy + radius),
                    (cx - radius, cy + radius),
                ],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx, cy - radius / 3), (cx, cy + radius / 3)],
                ShapeStyle::from(&tint).stroke_width(stroke_width + 1),
            ))
            .map_err(backend_error)?;
            root.draw(&Circle::new(
                (cx, cy + radius / 2),
                (radius as f64 * 0.1).round() as i32,
                fill,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Archive => {
            let left = cx - (radius as f64 * 0.72).round() as i32;
            let right = cx + (radius as f64 * 0.72).round() as i32;
            let top = cy - (radius as f64 * 0.52).round() as i32;
            let bottom = cy + (radius as f64 * 0.52).round() as i32;
            root.draw(&Rectangle::new([(left, top), (right, bottom)], stroke))
                .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(left, cy - radius / 6), (right, cy - radius / 6)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![
                    (cx - radius / 3, cy + radius / 5),
                    (cx + radius / 3, cy + radius / 5),
                ],
                stroke,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Database => {
            let rx = (radius as f64 * 0.72).round() as i32;
            let ry = (radius as f64 * 0.26).round() as i32;
            root.draw(&PathElement::new(
                ellipse_points(cx, cy - radius / 2, rx, ry, 0.0),
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                ellipse_points(cx, cy, rx, ry, 0.0),
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                ellipse_points(cx, cy + radius / 2, rx, ry, 0.0),
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx - rx, cy - radius / 2), (cx - rx, cy + radius / 2)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx + rx, cy - radius / 2), (cx + rx, cy + radius / 2)],
                stroke,
            ))
            .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Broker => {
            let hub_r = (radius as f64 * 0.18).round() as i32;
            let orbit_r = (radius as f64 * 0.14).round() as i32;
            let top = (cx, cy - radius + orbit_r + 2);
            let left = (cx - radius + orbit_r + 2, cy + radius / 3);
            let right = (cx + radius - orbit_r - 2, cy + radius / 3);
            root.draw(&PathElement::new(vec![(cx, cy), top], stroke))
                .map_err(backend_error)?;
            root.draw(&PathElement::new(vec![(cx, cy), left], stroke))
                .map_err(backend_error)?;
            root.draw(&PathElement::new(vec![(cx, cy), right], stroke))
                .map_err(backend_error)?;
            root.draw(&Circle::new((cx, cy), hub_r, fill))
                .map_err(backend_error)?;
            root.draw(&Circle::new(top, orbit_r, fill))
                .map_err(backend_error)?;
            root.draw(&Circle::new(left, orbit_r, fill))
                .map_err(backend_error)?;
            root.draw(&Circle::new(right, orbit_r, fill))
                .map_err(backend_error)?;
        }
        BuiltinNodeIcon::Dish => {
            let arc = arc_points(cx, cy + radius / 6, radius, radius / 2, PI * 0.1, PI * 0.9);
            root.draw(&PathElement::new(arc, stroke))
                .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx, cy + radius / 6), (cx, cy + radius / 2)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![
                    (cx - radius / 3, cy + radius / 2),
                    (cx + radius / 3, cy + radius / 2),
                ],
                stroke,
            ))
            .map_err(backend_error)?;
            for offset in [0.0, 0.18, 0.36] {
                let wave = arc_points(
                    cx + radius / 3,
                    cy - radius / 4,
                    ((radius as f64) * (0.35 + offset)).round() as i32,
                    ((radius as f64) * (0.24 + offset * 0.6)).round() as i32,
                    -PI / 4.0,
                    PI / 4.0,
                );
                root.draw(&PathElement::new(wave, stroke))
                    .map_err(backend_error)?;
            }
        }
        BuiltinNodeIcon::Spectrograph => {
            root.draw(&PathElement::new(
                vec![(cx - radius, cy), (cx - radius / 5, cy)],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&Polygon::new(
                vec![
                    (cx - radius / 6, cy - radius / 2),
                    (cx + radius / 3, cy),
                    (cx - radius / 6, cy + radius / 2),
                ],
                stroke,
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx + radius / 3, cy), (cx + radius, cy - radius / 3)],
                ShapeStyle::from(&tint).stroke_width(1),
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx + radius / 3, cy), (cx + radius, cy - radius / 8)],
                ShapeStyle::from(&tint.mix(0.82)).stroke_width(1),
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx + radius / 3, cy), (cx + radius, cy + radius / 8)],
                ShapeStyle::from(&tint.mix(0.64)).stroke_width(1),
            ))
            .map_err(backend_error)?;
            root.draw(&PathElement::new(
                vec![(cx + radius / 3, cy), (cx + radius, cy + radius / 3)],
                ShapeStyle::from(&tint.mix(0.46)).stroke_width(1),
            ))
            .map_err(backend_error)?;
        }
    }

    Ok(())
}

fn ellipse_points(cx: i32, cy: i32, rx: i32, ry: i32, rotation: f64) -> Vec<(i32, i32)> {
    (0..=36)
        .map(|step| {
            let angle = (step as f64 / 36.0) * 2.0 * PI;
            let x = angle.cos() * rx as f64;
            let y = angle.sin() * ry as f64;
            let xr = x * rotation.cos() - y * rotation.sin();
            let yr = x * rotation.sin() + y * rotation.cos();
            (
                (cx as f64 + xr).round() as i32,
                (cy as f64 + yr).round() as i32,
            )
        })
        .collect()
}

fn arc_points(
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    start_angle: f64,
    end_angle: f64,
) -> Vec<(i32, i32)> {
    (0..=18)
        .map(|step| {
            let t = step as f64 / 18.0;
            let angle = start_angle + (end_angle - start_angle) * t;
            (
                (cx as f64 + angle.cos() * rx as f64).round() as i32,
                (cy as f64 + angle.sin() * ry as f64).round() as i32,
            )
        })
        .collect()
}

/// Return `true` if canvas point `(x, y)` falls within the node.
///
/// `radius` should already include any hit-area expansion (e.g. ring padding).
pub(crate) fn node_contains(
    shape: &NodeShape,
    cx: f64,
    cy: f64,
    radius: f64,
    x: f64,
    y: f64,
) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    match shape {
        NodeShape::Circle => dx * dx + dy * dy <= radius * radius,
        NodeShape::Square => dx.abs() <= radius && dy.abs() <= radius,
        NodeShape::Diamond => dx.abs() + dy.abs() <= radius,
        NodeShape::Triangle => dx * dx + dy * dy <= radius * radius,
    }
}
