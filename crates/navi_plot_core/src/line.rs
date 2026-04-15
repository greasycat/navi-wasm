use crate::color::parse_color;
use crate::types::{LinePlotSpec, LineSeries as LineSeriesSpec};
use crate::viewport::{
    ensure_finite, resolve_axis_range, CartesianViewport, PixelBounds, CHART_MARGIN,
    X_LABEL_AREA_SIZE, Y_LABEL_AREA_SIZE,
};
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError};
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters::series::LineSeries as PlotterLineSeries;

const SELECTION_RING_PADDING: i32 = 6;
const POINT_RADIUS: i32 = 4;

/// Default series color palette (cycles by index).
const SERIES_PALETTE: [RGBColor; 6] = [
    RGBColor(37, 99, 235),  // blue
    RGBColor(22, 163, 74),  // green
    RGBColor(220, 38, 38),  // red
    RGBColor(249, 115, 22), // orange
    RGBColor(147, 51, 234), // purple
    RGBColor(15, 118, 110), // teal
];

type LineCoordSpec = Cartesian2d<RangedCoordf64, RangedCoordf64>;

#[derive(Debug, Clone)]
struct ResolvedSeriesPoint {
    x: f64,
    y: f64,
    series_index: usize,
    point_index: usize,
}

#[derive(Debug, Clone)]
struct ResolvedSeries {
    label: String,
    color: RGBColor,
    stroke_width: u32,
    points: Vec<(f64, f64)>,
}

fn pixel_bounds_from_chart<DB>(chart: &ChartContext<'_, DB, LineCoordSpec>) -> PixelBounds
where
    DB: DrawingBackend,
{
    let (x_pixels, y_pixels) = chart.plotting_area().get_pixel_range();
    PixelBounds {
        left: x_pixels.start,
        right: x_pixels.end,
        top: y_pixels.start,
        bottom: y_pixels.end,
    }
}

#[derive(Debug, Clone)]
pub struct LineSession {
    spec: LinePlotSpec,
    series: Vec<ResolvedSeries>,
    flat_points: Vec<ResolvedSeriesPoint>,
    viewport: CartesianViewport,
}

impl LineSession {
    pub fn new(spec: LinePlotSpec) -> Result<Self, PlotError> {
        ensure_dimensions(spec.width, spec.height)?;
        if spec.series.is_empty() {
            return Err(PlotError::EmptyLineSeries);
        }
        for (i, s) in spec.series.iter().enumerate() {
            if s.points.is_empty() {
                return Err(PlotError::EmptySeriesPoints { series_index: i });
            }
        }

        let (x_range, y_range) = resolve_ranges(&spec)?;
        let series = resolve_series(&spec.series)?;
        let flat_points = build_flat_points(&spec.series)?;

        Ok(Self {
            viewport: CartesianViewport::new(spec.width, spec.height, x_range, y_range),
            spec,
            series,
            flat_points,
        })
    }

    pub fn render_on<DB>(&mut self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        root.fill(&WHITE).map_err(backend_error)?;
        let mut chart = build_line_chart(
            &root,
            &self.spec,
            self.viewport.x_range,
            self.viewport.y_range,
        )?;
        self.viewport = self
            .viewport
            .with_plot_bounds(pixel_bounds_from_chart(&chart));

        chart
            .configure_mesh()
            .x_desc(if self.spec.x_label.is_empty() {
                "x"
            } else {
                &self.spec.x_label
            })
            .y_desc(if self.spec.y_label.is_empty() {
                "y"
            } else {
                &self.spec.y_label
            })
            .bold_line_style(RGBColor(209, 213, 219))
            .light_line_style(RGBColor(229, 231, 235))
            .axis_style(BLACK.mix(0.85))
            .draw()
            .map_err(backend_error)?;

        for resolved in &self.series {
            let color = resolved.color;
            let stroke = resolved.stroke_width;
            let label = resolved.label.clone();
            chart
                .draw_series(PlotterLineSeries::new(
                    resolved.points.iter().copied(),
                    ShapeStyle::from(&color).stroke_width(stroke),
                ))
                .map_err(backend_error)?
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], ShapeStyle::from(&color))
                });

            if self.spec.show_points {
                chart
                    .draw_series(
                        resolved
                            .points
                            .iter()
                            .filter(|&&(x, y)| point_overlaps_plot_area(&self.viewport, (x, y), 0))
                            .map(|&pt| Circle::new(pt, POINT_RADIUS, color.filled())),
                    )
                    .map_err(backend_error)?;
            }
        }

        if self.spec.show_legend {
            chart
                .configure_series_labels()
                .position(SeriesLabelPosition::UpperLeft)
                .margin(8)
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK.mix(0.3))
                .draw()
                .map_err(backend_error)?;
        }

        if let Some([si, pi]) = self.spec.selected_point {
            if let Some(resolved) = self.series.get(si) {
                if let Some(&pt) = resolved.points.get(pi) {
                    if point_overlaps_plot_area(&self.viewport, pt, SELECTION_RING_PADDING) {
                        chart
                            .draw_series(std::iter::once(
                                EmptyElement::at(pt)
                                    + Circle::new(
                                        (0, 0),
                                        POINT_RADIUS + SELECTION_RING_PADDING,
                                        ShapeStyle::from(&BLACK.mix(0.9)).stroke_width(2),
                                    ),
                            ))
                            .map_err(backend_error)?;
                    }
                }
            }
        }

        root.present().map_err(backend_error)?;
        Ok(())
    }

    pub fn pick_point(&self, canvas_x: f64, canvas_y: f64) -> Option<[usize; 2]> {
        if !canvas_x.is_finite() || !canvas_y.is_finite() {
            return None;
        }
        let target = (canvas_x.round() as i32, canvas_y.round() as i32);
        let threshold = f64::from(POINT_RADIUS + SELECTION_RING_PADDING);

        self.flat_points
            .iter()
            .filter_map(|p| {
                if !point_overlaps_plot_area(&self.viewport, (p.x, p.y), SELECTION_RING_PADDING) {
                    return None;
                }
                let center = self.viewport.translate((p.x, p.y));
                let dx = f64::from(center.0 - target.0);
                let dy = f64::from(center.1 - target.1);
                let dist_sq = dx * dx + dy * dy;
                (dist_sq <= threshold * threshold)
                    .then_some(([p.series_index, p.point_index], dist_sq))
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(idx, _)| idx)
    }

    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        let center = self.viewport.plot_bounds.center();
        let shifted = (
            self.viewport
                .plot_bounds
                .clamp_x(center.0 - delta_x.round() as i32),
            self.viewport
                .plot_bounds
                .clamp_y(center.1 - delta_y.round() as i32),
        );
        let current_center = self.viewport.reverse_translate(center);
        let shifted_center = self.viewport.reverse_translate(shifted);
        let next_x = (
            self.viewport.x_range.0 + (shifted_center.0 - current_center.0),
            self.viewport.x_range.1 + (shifted_center.0 - current_center.0),
        );
        let next_y = (
            self.viewport.y_range.0 + (shifted_center.1 - current_center.1),
            self.viewport.y_range.1 + (shifted_center.1 - current_center.1),
        );
        self.viewport.x_range = next_x;
        self.viewport.y_range = next_y;
        self.spec.x_range = Some([next_x.0, next_x.1]);
        self.spec.y_range = Some([next_y.0, next_y.1]);
    }

    pub fn zoom_at(&mut self, canvas_x: f64, canvas_y: f64, factor: f64) -> Result<(), PlotError> {
        self.viewport.zoom_at(canvas_x, canvas_y, factor)?;
        self.spec.x_range = Some([self.viewport.x_range.0, self.viewport.x_range.1]);
        self.spec.y_range = Some([self.viewport.y_range.0, self.viewport.y_range.1]);
        Ok(())
    }

    pub fn set_selection(&mut self, index: Option<[usize; 2]>) {
        self.spec.selected_point =
            index.filter(|[si, pi]| self.series.get(*si).is_some_and(|s| *pi < s.points.len()));
    }

    pub fn spec(&self) -> &LinePlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> LinePlotSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }
}

fn point_overlaps_plot_area(
    viewport: &CartesianViewport,
    point: (f64, f64),
    extra_radius: i32,
) -> bool {
    let center = viewport.translate(point);
    viewport
        .plot_bounds
        .intersects_circle(center, POINT_RADIUS + extra_radius.max(0))
}

pub fn render_line_on<DB>(root: PlotArea<DB>, spec: &LinePlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let mut session = LineSession::new(spec.clone())?;
    session.render_on(root)
}

pub fn pick_line_point(
    spec: &LinePlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<[usize; 2]>, PlotError> {
    Ok(LineSession::new(spec.clone())?.pick_point(canvas_x, canvas_y))
}

pub fn pan_line_spec(
    spec: &LinePlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<LinePlotSpec, PlotError> {
    let mut session = LineSession::new(spec.clone())?;
    session.pan(delta_x, delta_y);
    Ok(session.into_spec())
}

fn resolve_series(series: &[LineSeriesSpec]) -> Result<Vec<ResolvedSeries>, PlotError> {
    series
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let color = match s.color.as_deref() {
                Some(c) => parse_color(c)?,
                None => SERIES_PALETTE[i % SERIES_PALETTE.len()],
            };
            let points = s
                .points
                .iter()
                .map(|p| {
                    ensure_finite("x", p.x)?;
                    ensure_finite("y", p.y)?;
                    Ok((p.x, p.y))
                })
                .collect::<Result<Vec<_>, PlotError>>()?;
            Ok(ResolvedSeries {
                label: s.label.clone(),
                color,
                stroke_width: s.stroke_width,
                points,
            })
        })
        .collect()
}

fn build_flat_points(series: &[LineSeriesSpec]) -> Result<Vec<ResolvedSeriesPoint>, PlotError> {
    let mut flat = Vec::new();
    for (si, s) in series.iter().enumerate() {
        for (pi, p) in s.points.iter().enumerate() {
            ensure_finite("x", p.x)?;
            ensure_finite("y", p.y)?;
            flat.push(ResolvedSeriesPoint {
                x: p.x,
                y: p.y,
                series_index: si,
                point_index: pi,
            });
        }
    }
    Ok(flat)
}

fn resolve_ranges(spec: &LinePlotSpec) -> Result<((f64, f64), (f64, f64)), PlotError> {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for s in &spec.series {
        for p in &s.points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
    }

    let x_range = resolve_axis_range("x", spec.x_range, min_x, max_x)?;
    let y_range = resolve_axis_range("y", spec.y_range, min_y, max_y)?;
    Ok((x_range, y_range))
}

fn build_line_chart<'a, DB>(
    root: &'a PlotArea<DB>,
    spec: &LinePlotSpec,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<ChartContext<'a, DB, LineCoordSpec>, PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let title = if spec.title.is_empty() {
        "Line Chart"
    } else {
        &spec.title
    };
    ChartBuilder::on(root)
        .margin(CHART_MARGIN)
        .caption(title, ("sans-serif", 24))
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .y_label_area_size(Y_LABEL_AREA_SIZE)
        .build_cartesian_2d(x_range.0..x_range.1, y_range.0..y_range.1)
        .map_err(backend_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LinePoint, LineSeries as LineSeriesSpec};
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn sample_spec() -> LinePlotSpec {
        LinePlotSpec {
            width: 480,
            height: 320,
            title: "Test Line".to_string(),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            x_range: None,
            y_range: None,
            series: vec![
                LineSeriesSpec {
                    label: "A".to_string(),
                    color: Some("#2563eb".to_string()),
                    stroke_width: 2,
                    points: vec![
                        LinePoint {
                            x: 1.0,
                            y: 2.0,
                            label: None,
                            properties: Default::default(),
                        },
                        LinePoint {
                            x: 2.0,
                            y: 4.0,
                            label: None,
                            properties: Default::default(),
                        },
                        LinePoint {
                            x: 3.0,
                            y: 3.0,
                            label: None,
                            properties: Default::default(),
                        },
                    ],
                },
                LineSeriesSpec {
                    label: "B".to_string(),
                    color: Some("#16a34a".to_string()),
                    stroke_width: 2,
                    points: vec![
                        LinePoint {
                            x: 1.0,
                            y: 5.0,
                            label: None,
                            properties: Default::default(),
                        },
                        LinePoint {
                            x: 2.0,
                            y: 3.0,
                            label: None,
                            properties: Default::default(),
                        },
                        LinePoint {
                            x: 3.0,
                            y: 6.0,
                            label: None,
                            properties: Default::default(),
                        },
                    ],
                },
            ],
            selected_point: None,
            show_points: true,
            show_legend: true,
        }
    }

    #[test]
    fn line_auto_range_covers_all_series() {
        let spec = sample_spec();
        let (x_range, y_range) = resolve_ranges(&spec).unwrap();
        assert!(x_range.0 < 1.0);
        assert!(x_range.1 > 3.0);
        assert!(y_range.0 < 2.0);
        assert!(y_range.1 > 6.0);
    }

    #[test]
    fn line_auto_range_expands_constant_axis() {
        let range = resolve_axis_range("x", None, 5.0, 5.0).unwrap();
        assert_eq!(range, (4.0, 6.0));
    }

    #[test]
    fn line_invalid_explicit_range_is_rejected() {
        let err = resolve_axis_range("x", Some([3.0, 3.0]), 0.0, 1.0).unwrap_err();
        assert_eq!(
            err,
            PlotError::InvalidRange {
                axis: "x",
                min: 3.0,
                max: 3.0
            }
        );
    }

    #[test]
    fn line_empty_series_is_rejected() {
        let mut spec = sample_spec();
        spec.series.clear();
        let err = LineSession::new(spec).unwrap_err();
        assert_eq!(err, PlotError::EmptyLineSeries);
    }

    #[test]
    fn line_empty_series_points_is_rejected() {
        let mut spec = sample_spec();
        spec.series[1].points.clear();
        let err = LineSession::new(spec).unwrap_err();
        assert_eq!(err, PlotError::EmptySeriesPoints { series_index: 1 });
    }

    #[test]
    fn line_svg_output_contains_path_per_series() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_line_on(area, &spec).unwrap();
        // Plotters SVG renders line series as <polyline elements
        let polyline_count = svg.matches("<polyline").count();
        assert!(
            polyline_count >= spec.series.len(),
            "expected >= {} polylines, got {}: SVG={}",
            spec.series.len(),
            polyline_count,
            &svg[..svg.len().min(500)]
        );
    }

    #[test]
    fn line_svg_output_contains_circles_when_show_points_true() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_line_on(area, &spec).unwrap();
        let total_points: usize = spec.series.iter().map(|s| s.points.len()).sum();
        assert_eq!(svg.matches("<circle").count(), total_points);
    }

    #[test]
    fn line_hit_test_returns_correct_series_and_point() {
        let spec = sample_spec();
        let session = LineSession::new(spec).unwrap();
        // series 1, point 0 is at (1.0, 5.0)
        let px = session.viewport.translate((1.0, 5.0));
        let hit = session.pick_point(f64::from(px.0), f64::from(px.1));
        assert_eq!(hit, Some([1, 0]));
    }

    #[test]
    fn line_hit_test_ignores_offscreen_points() {
        let mut spec = sample_spec();
        spec.x_range = Some([0.0, 0.5]);
        spec.y_range = Some([0.0, 2.0]);

        let session = LineSession::new(spec).unwrap();
        let px = session.viewport.translate((1.0, 5.0));
        let hit = session.pick_point(f64::from(px.0), f64::from(px.1));

        assert_eq!(hit, None);
    }

    #[test]
    fn line_pan_updates_explicit_ranges() {
        let spec = sample_spec();
        let mut session = LineSession::new(spec.clone()).unwrap();
        session.pan(20.0, -10.0);
        let panned = session.into_spec();
        assert!(panned.x_range.is_some());
        assert!(panned.y_range.is_some());
    }

    #[test]
    fn line_viewport_round_trip_is_stable() {
        let spec = sample_spec();
        let session = LineSession::new(spec).unwrap();
        let point = (2.0, 4.0);
        let px = session.viewport.translate(point);
        let back = session.viewport.reverse_translate(px);
        assert!((back.0 - point.0).abs() < 0.1);
        assert!((back.1 - point.1).abs() < 0.1);
    }
}
