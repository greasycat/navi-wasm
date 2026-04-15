use crate::types::HeatmapSpec;
use crate::viewport::{ensure_finite, ScreenTransform};
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError};
use plotters::prelude::*;

const CAPTION_AREA_SIZE: i32 = 40;
const LEGEND_WIDTH: i32 = 50;
const COL_LABEL_FONT_PX: i32 = 16;
const CHAR_W: i32 = 6;

/// Interpolate between two colors at parameter `t` in [0,1].
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
                    let seg_t = (t - STOPS[i - 1].0) / (STOPS[i].0 - STOPS[i - 1].0);
                    return lerp_color(STOPS[i - 1].1, STOPS[i].1, seg_t);
                }
            }
            lerp_color(STOPS[3].1, STOPS[4].1, 1.0)
        }
        "greens" => lerp_color((240, 253, 244), (22, 163, 74), t),
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
        for &value in row {
            ensure_finite("value", value)?;
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
        for &value in row {
            lo = lo.min(value);
            hi = hi.max(value);
        }
    }

    if (hi - lo).abs() < f64::EPSILON {
        (lo - 1.0, lo + 1.0)
    } else {
        (lo, hi)
    }
}

#[derive(Debug, Clone, Copy)]
struct Layout {
    left: i32,
    top: i32,
    cell_w: i32,
    cell_h: i32,
}

impl Layout {
    fn right(self, n_cols: usize) -> i32 {
        self.left + self.cell_w * n_cols as i32
    }

    fn bottom(self, n_rows: usize) -> i32 {
        self.top + self.cell_h * n_rows as i32
    }

    fn bounds(self, n_rows: usize, n_cols: usize) -> (i32, i32, i32, i32) {
        (self.left, self.top, self.right(n_cols), self.bottom(n_rows))
    }
}

fn compute_layout(spec: &HeatmapSpec, n_rows: usize, n_cols: usize) -> Layout {
    let row_label_w = if spec.row_labels.is_empty() { 0 } else { 60 };
    let col_label_h = if spec.col_labels.is_empty() { 0 } else { 28 };

    let left = spec.margin as i32 + row_label_w;
    let top = spec.margin as i32 + CAPTION_AREA_SIZE + col_label_h;
    let right = (spec.width as i32 - spec.margin as i32 - LEGEND_WIDTH).max(left + 1);
    let bottom = (spec.height as i32 - spec.margin as i32).max(top + 1);

    Layout {
        left,
        top,
        cell_w: ((right - left) / n_cols as i32).max(1),
        cell_h: ((bottom - top) / n_rows as i32).max(1),
    }
}

#[derive(Debug, Clone)]
pub struct HeatmapSession {
    spec: HeatmapSpec,
    n_rows: usize,
    n_cols: usize,
    value_range: (f64, f64),
    layout: Layout,
    view: ScreenTransform,
}

impl HeatmapSession {
    pub fn new(spec: HeatmapSpec) -> Result<Self, PlotError> {
        let (n_rows, n_cols) = validate(&spec)?;
        let value_range = value_range(&spec);
        let layout = compute_layout(&spec, n_rows, n_cols);

        Ok(Self {
            spec,
            n_rows,
            n_cols,
            value_range,
            layout,
            view: ScreenTransform::new(0.0, 0.0),
        })
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        render_with_layout(
            &root,
            &self.spec,
            self.n_rows,
            self.n_cols,
            self.value_range,
            self.layout,
            &self.view,
        )
    }

    pub fn pick_cell(&self, canvas_x: f64, canvas_y: f64) -> Option<[usize; 2]> {
        if !canvas_x.is_finite() || !canvas_y.is_finite() {
            return None;
        }

        let (left, top, right, bottom) = self.layout.bounds(self.n_rows, self.n_cols);
        let cx = canvas_x.round() as i32;
        let cy = canvas_y.round() as i32;
        if cx < left || cx >= right || cy < top || cy >= bottom {
            return None;
        }

        let (logical_x, logical_y) = self.view.inverse((canvas_x, canvas_y));
        let logical_x = logical_x.round() as i32 - self.layout.left;
        let logical_y = logical_y.round() as i32 - self.layout.top;
        if logical_x < 0 || logical_y < 0 {
            return None;
        }

        let col = logical_x / self.layout.cell_w;
        let row = logical_y / self.layout.cell_h;
        if row as usize >= self.n_rows || col as usize >= self.n_cols {
            return None;
        }

        Some([row as usize, col as usize])
    }

    pub fn zoom_at(&mut self, canvas_x: f64, canvas_y: f64, factor: f64) -> Result<(), PlotError> {
        let (left, top, right, bottom) = self.layout.bounds(self.n_rows, self.n_cols);
        let anchor_x = canvas_x.clamp(f64::from(left), f64::from(right.saturating_sub(1)));
        let anchor_y = canvas_y.clamp(f64::from(top), f64::from(bottom.saturating_sub(1)));
        self.view.zoom_at(anchor_x, anchor_y, factor)
    }

    pub fn set_selection(&mut self, selection: Option<[usize; 2]>) {
        self.spec.selected_cell =
            selection.filter(|[row, col]| *row < self.n_rows && *col < self.n_cols);
    }

    pub fn spec(&self) -> &HeatmapSpec {
        &self.spec
    }

    pub fn into_spec(self) -> HeatmapSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }
}

fn transform_rect(
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    view: &ScreenTransform,
    clip: (i32, i32, i32, i32),
) -> Option<((i32, i32), (i32, i32))> {
    let (screen_left, screen_top) = view.apply((f64::from(left), f64::from(top)));
    let (screen_right, screen_bottom) = view.apply((f64::from(right), f64::from(bottom)));
    let (clip_left, clip_top, clip_right, clip_bottom) = clip;
    let clipped_left = screen_left.clamp(clip_left, clip_right);
    let clipped_top = screen_top.clamp(clip_top, clip_bottom);
    let clipped_right = screen_right.clamp(clip_left, clip_right);
    let clipped_bottom = screen_bottom.clamp(clip_top, clip_bottom);

    (clipped_left < clipped_right && clipped_top < clipped_bottom)
        .then_some(((clipped_left, clipped_top), (clipped_right, clipped_bottom)))
}

fn render_with_layout<DB>(
    root: &PlotArea<DB>,
    spec: &HeatmapSpec,
    n_rows: usize,
    n_cols: usize,
    value_range: (f64, f64),
    layout: Layout,
    view: &ScreenTransform,
) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let (v_min, v_max) = value_range;
    let grid_bounds = layout.bounds(n_rows, n_cols);
    let (grid_left, grid_top, grid_right, grid_bottom) = grid_bounds;

    root.fill(&WHITE).map_err(backend_error)?;

    if !spec.title.is_empty() {
        root.draw(&Text::new(
            spec.title.clone(),
            (
                spec.width as i32 / 2 - spec.title.len() as i32 * 7,
                spec.margin as i32 + 8,
            ),
            ("sans-serif", 22).into_font(),
        ))
        .map_err(backend_error)?;
    }

    for (col_index, label) in spec.col_labels.iter().enumerate() {
        let center_x = layout.left + col_index as i32 * layout.cell_w + layout.cell_w / 2;
        let screen_x = view.apply((f64::from(center_x), f64::from(layout.top))).0;
        if screen_x < grid_left - label.len() as i32 * CHAR_W || screen_x > grid_right {
            continue;
        }
        let x = screen_x - label.len() as i32 * CHAR_W / 2;
        let y = (layout.top - COL_LABEL_FONT_PX - 4).max(spec.margin as i32 + CAPTION_AREA_SIZE);
        root.draw(&Text::new(
            label.clone(),
            (x, y),
            ("sans-serif", 12).into_font(),
        ))
        .map_err(backend_error)?;
    }

    for (row_index, label) in spec.row_labels.iter().enumerate() {
        let center_y = layout.top + row_index as i32 * layout.cell_h + layout.cell_h / 2;
        let screen_y = view.apply((f64::from(layout.left), f64::from(center_y))).1;
        if screen_y < grid_top - COL_LABEL_FONT_PX || screen_y > grid_bottom {
            continue;
        }
        let text_w = label.len() as i32 * CHAR_W;
        let x = (layout.left - 8 - text_w).max(spec.margin as i32);
        let y = screen_y - COL_LABEL_FONT_PX / 2;
        root.draw(&Text::new(
            label.clone(),
            (x, y),
            ("sans-serif", 12).into_font(),
        ))
        .map_err(backend_error)?;
    }

    for (row_index, row) in spec.cells.iter().enumerate() {
        for (col_index, &value) in row.iter().enumerate() {
            let t = if (v_max - v_min).abs() < f64::EPSILON {
                0.5
            } else {
                (value - v_min) / (v_max - v_min)
            };
            let fill = interpolate_color(&spec.palette, t);
            let x0 = layout.left + col_index as i32 * layout.cell_w;
            let y0 = layout.top + row_index as i32 * layout.cell_h;
            let x1 = x0 + layout.cell_w;
            let y1 = y0 + layout.cell_h;

            let Some(((left, top), (right, bottom))) =
                transform_rect(x0, y0, x1, y1, view, grid_bounds)
            else {
                continue;
            };

            root.draw(&Rectangle::new(
                [(left, top), (right, bottom)],
                fill.filled(),
            ))
            .map_err(backend_error)?;

            root.draw(&Rectangle::new(
                [(left, top), (right, bottom)],
                ShapeStyle::from(&RGBColor(200, 200, 200)).stroke_width(1),
            ))
            .map_err(backend_error)?;

            if spec.show_values {
                let lum = 0.299 * fill.0 as f64 + 0.587 * fill.1 as f64 + 0.114 * fill.2 as f64;
                let text_color = if lum > 128.0 { &BLACK } else { &WHITE };
                let text = format!("{value:.2}");
                let center_x = x0 + layout.cell_w / 2;
                let center_y = y0 + layout.cell_h / 2;
                let (screen_x, screen_y) = view.apply((f64::from(center_x), f64::from(center_y)));
                if screen_x >= grid_left
                    && screen_x <= grid_right
                    && screen_y >= grid_top
                    && screen_y <= grid_bottom
                {
                    let text_width = text.len() as i32 * 4;
                    root.draw(&Text::new(
                        text,
                        (screen_x - text_width, screen_y - 6),
                        ("sans-serif", 11).into_font().color(text_color),
                    ))
                    .map_err(backend_error)?;
                }
            }

            if spec.selected_cell == Some([row_index, col_index]) {
                root.draw(&Rectangle::new(
                    [(left, top), (right, bottom)],
                    ShapeStyle::from(&BLACK).stroke_width(2),
                ))
                .map_err(backend_error)?;
            }
        }
    }

    let legend_x = layout.right(n_cols) + 10;
    let legend_y_top = layout.top;
    let legend_h = n_rows as i32 * layout.cell_h;
    let legend_bar_w = 16;
    let n_steps = legend_h.max(1);

    for step in 0..n_steps {
        let t = 1.0 - step as f64 / n_steps as f64;
        let fill = interpolate_color(&spec.palette, t);
        root.draw(&Rectangle::new(
            [
                (legend_x, legend_y_top + step),
                (legend_x + legend_bar_w, legend_y_top + step + 1),
            ],
            fill.filled(),
        ))
        .map_err(backend_error)?;
    }

    root.draw(&Text::new(
        format!("{v_max:.2}"),
        (legend_x + legend_bar_w + 4, legend_y_top),
        ("sans-serif", 10).into_font(),
    ))
    .map_err(backend_error)?;
    root.draw(&Text::new(
        format!("{v_min:.2}"),
        (legend_x + legend_bar_w + 4, legend_y_top + legend_h - 10),
        ("sans-serif", 10).into_font(),
    ))
    .map_err(backend_error)?;

    root.present().map_err(backend_error)?;
    Ok(())
}

pub fn render_heatmap_on<DB>(root: PlotArea<DB>, spec: &HeatmapSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    HeatmapSession::new(spec.clone())?.render_on(root)
}

pub fn pick_heatmap_cell(
    spec: &HeatmapSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<[usize; 2]>, PlotError> {
    Ok(HeatmapSession::new(spec.clone())?.pick_cell(canvas_x, canvas_y))
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
            cells: vec![vec![0.0, 0.5, 1.0], vec![0.3, 0.7, 0.2]],
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
        assert!(matches!(
            err,
            PlotError::HeatmapShapeMismatch { row_index: 1, .. }
        ));
    }

    #[test]
    fn heatmap_svg_has_correct_rect_count() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_heatmap_on(area, &spec).unwrap();
        let rect_count = svg.matches("<rect").count();
        let n_cells = spec.cells.len() * spec.cells[0].len();
        assert!(
            rect_count >= n_cells * 2,
            "expected >= {} rects, got {}",
            n_cells * 2,
            rect_count
        );
    }

    #[test]
    fn heatmap_svg_contains_value_text_when_show_values_true() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_heatmap_on(area, &spec).unwrap();
        assert!(svg.contains("0.50"), "SVG should contain cell value text");
    }

    #[test]
    fn heatmap_color_midpoint_is_white_for_bwr() {
        let color = interpolate_color("blue_white_red", 0.5);
        assert_eq!(color, RGBColor(255, 255, 255));
    }

    #[test]
    fn heatmap_hit_test_returns_correct_row_and_col() {
        let session = HeatmapSession::new(sample_spec()).unwrap();
        let layout = session.layout;
        let center_x = layout.left + 2 * layout.cell_w + layout.cell_w / 2;
        let center_y = layout.top + layout.cell_h + layout.cell_h / 2;
        let hit = session.pick_cell(center_x as f64, center_y as f64);
        assert_eq!(hit, Some([1, 2]));
    }

    #[test]
    fn heatmap_zoom_preserves_anchor_pick() {
        let mut session = HeatmapSession::new(sample_spec()).unwrap();
        let layout = session.layout;
        let center_x = layout.left + layout.cell_w / 2;
        let center_y = layout.top + layout.cell_h / 2;
        session
            .zoom_at(center_x as f64, center_y as f64, 1.5)
            .unwrap();
        assert_eq!(
            session.pick_cell(center_x as f64, center_y as f64),
            Some([0, 0])
        );
    }

    #[test]
    fn heatmap_auto_value_range_covers_all_cells() {
        let spec = sample_spec();
        let (lo, hi) = value_range(&spec);
        assert!(lo <= 0.0);
        assert!(hi >= 1.0);
    }
}
