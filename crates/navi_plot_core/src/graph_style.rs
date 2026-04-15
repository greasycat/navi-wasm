use crate::color::parse_color;
use crate::types::{GraphEdgeStyle, GraphNodeStyle, NodeShape, SelectionStyle};
use crate::PlotError;
use plotters::style::{RGBColor, BLACK};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedNodeStyle {
    pub fill_color: RGBColor,
    pub stroke_color: Option<RGBColor>,
    pub stroke_width: u32,
    pub radius: i32,
    pub opacity: f64,
    pub shape: NodeShape,
    pub label_visible: bool,
    pub label_color: Option<RGBColor>,
    pub label_inside: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedEdgeStyle {
    pub stroke_color: RGBColor,
    pub stroke_width: u32,
    pub dash_pattern: Option<Vec<u32>>,
    pub opacity: f64,
    pub arrow_visible: bool,
    pub label_visible: bool,
    pub label_color: Option<RGBColor>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedSelectionStyle {
    pub stroke_color: RGBColor,
    pub stroke_width: u32,
    pub padding: i32,
    pub opacity: f64,
}

pub(crate) struct NodeStyleContext<'a> {
    pub default_fill_color: RGBColor,
    pub default_radius: u32,
    pub default_label_visible: bool,
    pub graph_style: Option<&'a GraphNodeStyle>,
    pub legacy_fill_color: Option<&'a str>,
    pub legacy_shape: Option<&'a NodeShape>,
    pub legacy_label_inside: Option<bool>,
    pub item_style: Option<&'a GraphNodeStyle>,
}

pub(crate) struct EdgeStyleContext<'a> {
    pub default_stroke_color: RGBColor,
    pub default_stroke_width: u32,
    pub default_arrow_visible: bool,
    pub default_label_visible: bool,
    pub graph_style: Option<&'a GraphEdgeStyle>,
    pub legacy_stroke_color: Option<&'a str>,
    pub item_style: Option<&'a GraphEdgeStyle>,
}

pub(crate) fn resolve_node_style(
    ctx: NodeStyleContext<'_>,
) -> Result<ResolvedNodeStyle, PlotError> {
    let mut fill_color = ctx.default_fill_color;
    let mut stroke_color = None;
    let mut stroke_width = 0;
    let mut radius = ctx.default_radius.max(1) as i32;
    let mut opacity = 1.0;
    let mut shape = NodeShape::Circle;
    let mut label_visible = ctx.default_label_visible;
    let mut label_color = None;
    let mut label_inside = false;

    apply_node_style(
        ctx.graph_style,
        &mut fill_color,
        &mut stroke_color,
        &mut stroke_width,
        &mut radius,
        &mut opacity,
        &mut shape,
        &mut label_visible,
        &mut label_color,
        &mut label_inside,
    )?;

    if let Some(value) = ctx.legacy_fill_color {
        fill_color = parse_color(value)?;
    }
    if let Some(value) = ctx.legacy_shape {
        shape = value.clone();
    }
    if let Some(value) = ctx.legacy_label_inside {
        label_inside = value;
    }

    apply_node_style(
        ctx.item_style,
        &mut fill_color,
        &mut stroke_color,
        &mut stroke_width,
        &mut radius,
        &mut opacity,
        &mut shape,
        &mut label_visible,
        &mut label_color,
        &mut label_inside,
    )?;

    Ok(ResolvedNodeStyle {
        fill_color,
        stroke_color,
        stroke_width,
        radius,
        opacity,
        shape,
        label_visible,
        label_color,
        label_inside,
    })
}

pub(crate) fn resolve_edge_style(
    ctx: EdgeStyleContext<'_>,
) -> Result<ResolvedEdgeStyle, PlotError> {
    let mut stroke_color = ctx.default_stroke_color;
    let mut stroke_width = ctx.default_stroke_width;
    let mut dash_pattern = None;
    let mut opacity = 1.0;
    let mut arrow_visible = ctx.default_arrow_visible;
    let mut label_visible = ctx.default_label_visible;
    let mut label_color = None;

    apply_edge_style(
        ctx.graph_style,
        &mut stroke_color,
        &mut stroke_width,
        &mut dash_pattern,
        &mut opacity,
        &mut arrow_visible,
        &mut label_visible,
        &mut label_color,
    )?;

    if let Some(value) = ctx.legacy_stroke_color {
        stroke_color = parse_color(value)?;
    }

    apply_edge_style(
        ctx.item_style,
        &mut stroke_color,
        &mut stroke_width,
        &mut dash_pattern,
        &mut opacity,
        &mut arrow_visible,
        &mut label_visible,
        &mut label_color,
    )?;

    Ok(ResolvedEdgeStyle {
        stroke_color,
        stroke_width,
        dash_pattern,
        opacity,
        arrow_visible,
        label_visible,
        label_color,
    })
}

pub(crate) fn resolve_selection_style(
    default_padding: i32,
    style: Option<&SelectionStyle>,
) -> Result<ResolvedSelectionStyle, PlotError> {
    let mut stroke_color = BLACK;
    let mut stroke_width = 2;
    let mut padding = default_padding.max(0);
    let mut opacity = 0.9;

    if let Some(style) = style {
        if let Some(value) = style.stroke_color.as_deref() {
            stroke_color = parse_color(value)?;
        }
        if let Some(value) = style.stroke_width {
            stroke_width = validate_size("selection_style.stroke_width", value, 0)?;
        }
        if let Some(value) = style.padding {
            padding = validate_size("selection_style.padding", value, 0)? as i32;
        }
        if let Some(value) = style.opacity {
            opacity = validate_opacity("selection_style.opacity", value)?;
        }
    }

    Ok(ResolvedSelectionStyle {
        stroke_color,
        stroke_width,
        padding,
        opacity,
    })
}

fn apply_node_style(
    style: Option<&GraphNodeStyle>,
    fill_color: &mut RGBColor,
    stroke_color: &mut Option<RGBColor>,
    stroke_width: &mut u32,
    radius: &mut i32,
    opacity: &mut f64,
    shape: &mut NodeShape,
    label_visible: &mut bool,
    label_color: &mut Option<RGBColor>,
    label_inside: &mut bool,
) -> Result<(), PlotError> {
    let Some(style) = style else {
        return Ok(());
    };

    if let Some(value) = style.fill_color.as_deref() {
        *fill_color = parse_color(value)?;
    }
    if let Some(value) = style.stroke_color.as_deref() {
        *stroke_color = Some(parse_color(value)?);
    }
    if let Some(value) = style.stroke_width {
        *stroke_width = validate_size("node_style.stroke_width", value, 0)?;
    }
    if let Some(value) = style.radius {
        *radius = validate_size("node_style.radius", value, 1)? as i32;
    }
    if let Some(value) = style.opacity {
        *opacity = validate_opacity("node_style.opacity", value)?;
    }
    if let Some(value) = style.shape.as_ref() {
        *shape = value.clone();
    }
    if let Some(value) = style.label_visible {
        *label_visible = value;
    }
    if let Some(value) = style.label_color.as_deref() {
        *label_color = Some(parse_color(value)?);
    }
    if let Some(value) = style.label_inside {
        *label_inside = value;
    }

    Ok(())
}

fn apply_edge_style(
    style: Option<&GraphEdgeStyle>,
    stroke_color: &mut RGBColor,
    stroke_width: &mut u32,
    dash_pattern: &mut Option<Vec<u32>>,
    opacity: &mut f64,
    arrow_visible: &mut bool,
    label_visible: &mut bool,
    label_color: &mut Option<RGBColor>,
) -> Result<(), PlotError> {
    let Some(style) = style else {
        return Ok(());
    };

    if let Some(value) = style.stroke_color.as_deref() {
        *stroke_color = parse_color(value)?;
    }
    if let Some(value) = style.stroke_width {
        *stroke_width = validate_size("edge_style.stroke_width", value, 0)?;
    }
    if let Some(value) = style.dash_pattern.as_deref() {
        *dash_pattern = Some(validate_dash_pattern("edge_style.dash_pattern", value)?);
    }
    if let Some(value) = style.opacity {
        *opacity = validate_opacity("edge_style.opacity", value)?;
    }
    if let Some(value) = style.arrow_visible {
        *arrow_visible = value;
    }
    if let Some(value) = style.label_visible {
        *label_visible = value;
    }
    if let Some(value) = style.label_color.as_deref() {
        *label_color = Some(parse_color(value)?);
    }

    Ok(())
}

pub(crate) fn edge_line_segments(
    start: (i32, i32),
    end: (i32, i32),
    dash_pattern: Option<&[u32]>,
) -> Vec<((i32, i32), (i32, i32))> {
    let Some(pattern) = dash_pattern.filter(|pattern| !pattern.is_empty()) else {
        return vec![(start, end)];
    };

    let dx = end.0 as f64 - start.0 as f64;
    let dy = end.1 as f64 - start.1 as f64;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 0.01 {
        return vec![(start, end)];
    }

    let ux = dx / length;
    let uy = dy / length;
    let mut segments = Vec::new();
    let mut distance = 0.0;
    let mut draw = true;
    let mut pattern_idx = 0usize;

    while distance < length {
        let step = pattern[pattern_idx % pattern.len()].max(1) as f64;
        let next = (distance + step).min(length);
        if draw && next > distance {
            let segment_start = (
                (start.0 as f64 + ux * distance).round() as i32,
                (start.1 as f64 + uy * distance).round() as i32,
            );
            let segment_end = (
                (start.0 as f64 + ux * next).round() as i32,
                (start.1 as f64 + uy * next).round() as i32,
            );
            if segment_start != segment_end {
                segments.push((segment_start, segment_end));
            }
        }
        distance = next;
        draw = !draw;
        pattern_idx += 1;
    }

    if segments.is_empty() {
        vec![(start, end)]
    } else {
        segments
    }
}

fn validate_size(field: &'static str, value: f64, min: u32) -> Result<u32, PlotError> {
    if !value.is_finite() {
        return Err(PlotError::InvalidStyleValue {
            field,
            value,
            reason: "must be finite",
        });
    }
    if value < min as f64 {
        return Err(PlotError::InvalidStyleValue {
            field,
            value,
            reason: if min == 0 {
                "must be greater than or equal to 0"
            } else {
                "must be greater than or equal to 1"
            },
        });
    }
    Ok(value.round() as u32)
}

fn validate_dash_pattern(field: &'static str, values: &[f64]) -> Result<Vec<u32>, PlotError> {
    if values.is_empty() {
        return Err(PlotError::InvalidStyleValue {
            field,
            value: 0.0,
            reason: "must contain at least one segment length",
        });
    }

    let mut resolved = Vec::with_capacity(values.len());
    for &value in values {
        resolved.push(validate_size(field, value, 1)?);
    }
    Ok(resolved)
}

fn validate_opacity(field: &'static str, value: f64) -> Result<f64, PlotError> {
    if !value.is_finite() {
        return Err(PlotError::InvalidStyleValue {
            field,
            value,
            reason: "must be finite",
        });
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(PlotError::InvalidStyleValue {
            field,
            value,
            reason: "must be between 0 and 1 inclusive",
        });
    }
    Ok(value)
}
