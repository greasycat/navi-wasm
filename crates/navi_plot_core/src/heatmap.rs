use crate::viewport::ensure_finite;
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError};
use crate::types::HeatmapSpec;
use plotters::prelude::*;

const CAPTION_AREA_SIZE: i32 = 40;
const LEGEND_WIDTH: i32 = 50;

/// Interpolate between two colors at parameter `t` ∈ \[0,1\].
fn lerp_color(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    let r = (a.0 as f64 + t * (b.0 as f64 - a.0 as f64)).round() as u8;
    let g = (a.1 as f64 + t * (b.1 as f64 - a.1 as f64)).round() as u8;
    let b_ = (a.2 as f64 + t * (b.2 as f64 - a.2 as f64)).round() as u8;
    RGBColor(r, g, b_)
}

fn interpolate_color(palette: &str, t: f64) -> RGBColor {
    let t = t.clamp(0.0, 1.0);
    match palette {
        "viridis" => {
            const STOPS: [(f64, (u8, u8, u8)); 5] = [
                (0.00, (68, 1, 84)),
                (0.25, (59, 82, 139)),
                (0.50, (33, 145, 140)),
                (0.75, (94, 201, 98)),
                (1.00, (253, 231, 37)),
            ];
            for i in 1..STOPS.len() {
                if t <= STOPS[i].0 {
                    let seg_t =
                        (t - STOPS[i - 1].0) / (STOPS[i].0 - STOPS[i - 1].0);
                    return lerp_color(STOPS[i - 1].1, STOPS[i].1, seg_t);
                }
            }
            lerp_color(STOPS[3].1, STOPS[4].1, 1.0)
        }
        "greens" => lerp_color((240, 253, 244), (22, 163, 74), t),
        // "blue_white_red" and default
        _ => {
            if t < 0.5 {
                lerp_color((37, 99, 235), (255, 255, 255), t * 2.0)
            } else {
                lerp_color((255, 255, 255), (220, 38, 38), (t - 0.5) * 2.0)
            }
        }
    }
}

fn validate(spec: &HeatmapSpec) -> Result<(usize, usize), PlotError> {
    ensure_dimensions(spec.width, spec.height)?;
    if spec.cells.is_empty() || spec.cells[0].is_empty() {
        return Err(PlotError::EmptyHeatmapData);
    }
    let n_cols = spec.cells[0].len();
    for (row_index, row) in spec.cells.iter().enumerate() {
        if row.len() != n_cols {
            return Err(PlotError::HeatmapShapeMismatch {
                expected_cols: n_cols,
                row_index,
                actual_cols: row.len(),
            });
        }
        for &v in row {
            ensure_finite("value", v)?;
        }
    }
    Ok((spec.cells.len(), n_cols))
}

fn value_range(spec: &HeatmapSpec) -> (f64, f64) {
    if let Some([lo, hi]) = spec.value_range {
        return (lo, hi);
    }
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for row in &spec.cells {
        for &v in row {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    if (hi - lo).abs() < f64::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        (lo, hi)
    }
}

/// Layout constants derived from spec.
struct Layout {
    left: i32,
    top: i32,
    cell_w: i32,
    cell_h: i32,
}

fn compute_layout(spec: &HeatmapSpec, n_rows: usize, n_cols: usize) -> Layout {
    let row_label_w: i32 = if spec.row_labels.is_empty() { 0 } else { 60 };
    let col_label_h: i32 = if spec.col_labels.is_empty() { 0 } else { 28 };

    let left = spec.margin as i32 + row_label_w;
    let top = spec.margin as i32 + CAPTION_AREA_SIZE + col_label_h;
    let right = (spec.width as i32 - spec.margin as i32 - LEGEND_WIDTH).max(left + 1);
    let bottom = (spec.height as i32 - spec.margin as i32).max(top + 1);

    let cell_w = ((right - left) / n_cols as i32).max(1);
    let cell_h = ((bottom - top) / n_rows as i32).max(1);

    Layout { left, top, cell_w, cell_h }
}

pub fn render_heatmap_on<DB>(root: PlotArea<DB>, spec: &HeatmapSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let (n_rows, n_cols) = validate(spec)?;
    let (v_min, v_max) = value_range(spec);
    let layout = compute_layout(spec, n_rows, n_cols);

    root.fill(&WHITE).map_err(backend_error)?;

    // Draw title
    if !spec.title.is_empty() {
        root.draw(&Text::new(
            spec.title.clone(),
            (spec.width as i32 / 2 - spec.title.len() as i32 * 7, spec.margin as i32 + 8),
            ("sans-serif", 22).into_font(),
        ))
        .map_err(backend_error)?;
    }

    // Draw column labels — centered over each cell, placed inside the
    // allocated col_label_h band just above the grid.
    const COL_LABEL_FONT_PX: i32 = 16; // approx pixel height of 12pt
    const CHAR_W: i32 = 6;             // approx pixel width per char at 12pt
    for (ci, label) in spec.col_labels.iter().enumerate() {
        let cell_center_x = layout.left + ci as i32 * layout.cell_w + layout.cell_w / 2;
        let x = cell_center_x - label.len() as i32 * CHAR_W / 2;
        // Vertically: bottom of text sits 4px above the grid top line,
        // clamped so it never overlaps the title area.
        let y = (layout.top - COL_LABEL_FONT_PX - 4)
            .max(spec.margin as i32 + CAPTION_AREA_SIZE);
        root.draw(&Text::new(
            label.clone(),
            (x, y),
            ("sans-serif", 12).into_font(),
        ))
        .map_err(backend_error)?;
    }

    // Draw row labels — right-aligned to the 4px gap before the grid left edge.
    for (ri, label) in spec.row_labels.iter().enumerate() {
        let text_w = label.len() as i32 * CHAR_W;
        let x = (layout.left - 8 - text_w).max(spec.margin as i32);
        let y = layout.top + ri as i32 * layout.cell_h
            + (layout.cell_h - COL_LABEL_FONT_PX) / 2;
        root.draw(&Text::new(
            label.clone(),
            (x, y),
            ("sans-serif", 12).into_font(),
        ))
        .map_err(backend_error)?;
    }

    // Draw cells
    for (ri, row) in spec.cells.iter().enumerate() {
        for (ci, &value) in row.iter().enumerate() {
            let t = if (v_max - v_min).abs() < f64::EPSILON {
                0.5
            } else {
                (value - v_min) / (v_max - v_min)
            };
            let fill = interpolate_color(&spec.palette, t);
            let x0 = layout.left + ci as i32 * layout.cell_w;
            let y0 = layout.top + ri as i32 * layout.cell_h;
            let x1 = x0 + layout.cell_w;
            let y1 = y0 + layout.cell_h;

            root.draw(&Rectangle::new([(x0, y0), (x1, y1)], fill.filled()))
                .map_err(backend_error)?;

            // Cell border
            root.draw(&Rectangle::new(
                [(x0, y0), (x1, y1)],
                ShapeStyle::from(&RGBColor(200, 200, 200)).stroke_width(1),
            ))
            .map_err(backend_error)?;

            // Value text
            if spec.show_values {
                // Choose black or white text based on luminance
                let lum = 0.299 * fill.0 as f64 + 0.587 * fill.1 as f64 + 0.114 * fill.2 as f64;
                let text_color = if lum > 128.0 { &BLACK } else { &WHITE };
                let text = format!("{:.2}", value);
                let text_x = x0 + layout.cell_w / 2 - (text.len() as i32 * 4);
                let text_y = y0 + layout.cell_h / 2 - 6;
                root.draw(&Text::new(
                    text,
                    (text_x, text_y),
                    ("sans-serif", 11).into_font().color(text_color),
                ))
                .map_err(backend_error)?;
            }

            // Selection outline
            if spec.selected_cell == Some([ri, ci]) {
                root.draw(&Rectangle::new(
                    [(x0 + 1, y0 + 1), (x1 - 1, y1 - 1)],
                    ShapeStyle::from(&BLACK).stroke_width(2),
                ))
                .map_err(backend_error)?;
            }
        }
    }

    // Draw color scale legend on the right
    let legend_x = layout.left + n_cols as i32 * layout.cell_w + 10;
    let legend_y_top = layout.top;
    let legend_h = n_rows as i32 * layout.cell_h;
    let legend_bar_w = 16;
    let n_steps = legend_h.max(1);

    for step in 0..n_steps {
        let t = 1.0 - step as f64 / n_steps as f64;
        let fill = interpolate_color(&spec.palette, t);
        root.draw(&Rectangle::new(
            [(legend_x, legend_y_top + step), (legend_x + legend_bar_w, legend_y_top + step + 1)],
            fill.filled(),
        ))
        .map_err(backend_error)?;
    }

    // Min/max labels
    root.draw(&Text::new(
        format!("{:.2}", v_max),
        (legend_x + legend_bar_w + 4, legend_y_top),
        ("sans-serif", 10).into_font(),
    ))
    .map_err(backend_error)?;
    root.draw(&Text::new(
        format!("{:.2}", v_min),
        (legend_x + legend_bar_w + 4, legend_y_top + legend_h - 10),
        ("sans-serif", 10).into_font(),
    ))
    .map_err(backend_error)?;

    root.present().map_err(backend_error)?;
    Ok(())
}

pub fn pick_heatmap_cell(
    spec: &HeatmapSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<[usize; 2]>, PlotError> {
    let (n_rows, n_cols) = validate(spec)?;
    if !canvas_x.is_finite() || !canvas_y.is_finite() {
        return Ok(None);
    }
    let layout = compute_layout(spec, n_rows, n_cols);
    let cx = canvas_x.round() as i32 - layout.left;
    let cy = canvas_y.round() as i32 - layout.top;
    if cx < 0 || cy < 0 {
        return Ok(None);
    }
    let col = cx / layout.cell_w;
    let row = cy / layout.cell_h;
    if row as usize >= n_rows || col as usize >= n_cols {
        return Ok(None);
    }
    Ok(Some([row as usize, col as usize]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HeatmapSpec;
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn sample_spec() -> HeatmapSpec {
        HeatmapSpec {
            width: 480,
            height: 360,
            title: "Test Heatmap".to_string(),
            row_labels: vec!["R0".to_string(), "R1".to_string()],
            col_labels: vec!["C0".to_string(), "C1".to_string(), "C2".to_string()],
            cells: vec![
                vec![0.0, 0.5, 1.0],
                vec![0.3, 0.7, 0.2],
            ],
            value_range: None,
            palette: "blue_white_red".to_string(),
            show_values: true,
            margin: 32,
            selected_cell: None,
        }
    }

    #[test]
    fn heatmap_rejects_empty_data() {
        let mut spec = sample_spec();
        spec.cells.clear();
        let err = render_heatmap_on(
            SVGBackend::with_string(&mut String::new(), (480, 360)).into_drawing_area(),
            &spec,
        )
        .unwrap_err();
        assert_eq!(err, PlotError::EmptyHeatmapData);
    }

    #[test]
    fn heatmap_rejects_jagged_rows() {
        let mut spec = sample_spec();
        spec.cells[1].pop();
        let err = render_heatmap_on(
            SVGBackend::with_string(&mut String::new(), (480, 360)).into_drawing_area(),
            &spec,
        )
        .unwrap_err();
        assert!(matches!(err, PlotError::HeatmapShapeMismatch { row_index: 1, .. }));
    }

    #[test]
    fn heatmap_svg_has_correct_rect_count() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area =
            SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_heatmap_on(area, &spec).unwrap();
        // Each cell produces 2 rects (fill + border), plus legend steps
        let rect_count = svg.matches("<rect").count();
        let n_cells = spec.cells.len() * spec.cells[0].len();
        assert!(rect_count >= n_cells * 2, "expected >= {} rects, got {}", n_cells * 2, rect_count);
    }

    #[test]
    fn heatmap_svg_contains_value_text_when_show_values_true() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area =
            SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_heatmap_on(area, &spec).unwrap();
        // Should contain "0.50" for the middle cell
        assert!(svg.contains("0.50"), "SVG should contain cell value text");
    }

    #[test]
    fn heatmap_color_midpoint_is_white_for_bwr() {
        let c = interpolate_color("blue_white_red", 0.5);
        assert_eq!(c, RGBColor(255, 255, 255));
    }

    #[test]
    fn heatmap_hit_test_returns_correct_row_and_col() {
        let spec = sample_spec();
        let (n_rows, n_cols) = validate(&spec).unwrap();
        let layout = compute_layout(&spec, n_rows, n_cols);
        // Center of cell (1, 2)
        let cx = layout.left + 2 * layout.cell_w + layout.cell_w / 2;
        let cy = layout.top + 1 * layout.cell_h + layout.cell_h / 2;
        let hit = pick_heatmap_cell(&spec, cx as f64, cy as f64).unwrap();
        assert_eq!(hit, Some([1, 2]));
    }

    #[test]
    fn heatmap_auto_value_range_covers_all_cells() {
        let spec = sample_spec();
        let (lo, hi) = value_range(&spec);
        assert!(lo <= 0.0);
        assert!(hi >= 1.0);
    }
}
