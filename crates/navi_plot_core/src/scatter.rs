use crate::color::parse_color;
use crate::viewport::{
    ensure_finite, resolve_axis_range, CartesianViewport, PixelBounds, CHART_MARGIN,
    X_LABEL_AREA_SIZE, Y_LABEL_AREA_SIZE,
};
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError, ScatterPlotSpec};
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;

const DEFAULT_POINT_RADIUS: i32 = 5;
const DEFAULT_POINT_COLOR: RGBColor = RGBColor(37, 99, 235);
const SELECTION_RING_PADDING: i32 = 6;

type ScatterCoordSpec = Cartesian2d<RangedCoordf64, RangedCoordf64>;

#[derive(Debug, Clone)]
struct ResolvedPoint {
    x: f64,
    y: f64,
    name: String,
    label: String,
    color: RGBColor,
    radius: i32,
}

fn pixel_bounds_from_chart<DB>(chart: &ChartContext<'_, DB, ScatterCoordSpec>) -> PixelBounds
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
pub struct ScatterSession {
    spec: ScatterPlotSpec,
    points: Vec<ResolvedPoint>,
    viewport: CartesianViewport,
}

impl ScatterSession {
    pub fn new(spec: ScatterPlotSpec) -> Result<Self, PlotError> {
        ensure_dimensions(spec.width, spec.height)?;
        if spec.points.is_empty() {
            return Err(PlotError::EmptyScatterData);
        }

        let points = spec
            .points
            .iter()
            .map(resolve_point)
            .collect::<Result<Vec<_>, _>>()?;
        let (x_range, y_range) = resolve_ranges_from_spec(&spec)?;

        Ok(Self {
            viewport: CartesianViewport::new(spec.width, spec.height, x_range, y_range),
            spec,
            points,
        })
    }

    pub fn render_on<DB>(&mut self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        root.fill(&WHITE).map_err(backend_error)?;
        let mut chart = build_scatter_chart(
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
            .x_desc(x_axis_label(&self.spec))
            .y_desc(y_axis_label(&self.spec))
            .bold_line_style(RGBColor(209, 213, 219))
            .light_line_style(RGBColor(229, 231, 235))
            .axis_style(BLACK.mix(0.85))
            .draw()
            .map_err(backend_error)?;

        chart
            .draw_series(self.points.iter().map(|point| {
                Circle::new(
                    (point.x, point.y),
                    point.radius,
                    ShapeStyle::from(&point.color).filled(),
                )
            }))
            .map_err(backend_error)?;

        chart
            .draw_series(
                self.points
                    .iter()
                    .filter(|point| !point.label.is_empty())
                    .map(|point| {
                        EmptyElement::at((point.x, point.y))
                            + Text::new(
                                point.label.clone(),
                                (point.radius + 4, -point.radius - 4),
                                ("sans-serif", 12).into_font(),
                            )
                    }),
            )
            .map_err(backend_error)?;

        if let Some(index) = self
            .spec
            .selected_point_index
            .filter(|index| *index < self.points.len())
        {
            let point = &self.points[index];
            chart
                .draw_series(std::iter::once(
                    EmptyElement::at((point.x, point.y))
                        + Circle::new(
                            (0, 0),
                            point.radius + SELECTION_RING_PADDING,
                            ShapeStyle::from(&BLACK.mix(0.9)).stroke_width(2),
                        )
                        + Text::new(
                            point.name.clone(),
                            (point.radius + 10, point.radius + 16),
                            ("sans-serif", 13).into_font().color(&BLACK),
                        ),
                ))
                .map_err(backend_error)?;
        }

        root.present().map_err(backend_error)?;
        Ok(())
    }

    pub fn pick_point(&self, canvas_x: f64, canvas_y: f64) -> Option<usize> {
        if !canvas_x.is_finite() || !canvas_y.is_finite() {
            return None;
        }

        let target = (canvas_x.round() as i32, canvas_y.round() as i32);
        self.points
            .iter()
            .enumerate()
            .filter_map(|(index, point)| {
                let center = self.viewport.translate((point.x, point.y));
                let dx = f64::from(center.0 - target.0);
                let dy = f64::from(center.1 - target.1);
                let threshold = f64::from(point.radius + SELECTION_RING_PADDING);
                let distance_squared = dx * dx + dy * dy;

                (distance_squared <= threshold * threshold).then_some((index, distance_squared))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
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

    pub fn set_selection(&mut self, index: Option<usize>) {
        self.spec.selected_point_index = index.filter(|index| *index < self.points.len());
    }

    pub fn selected_point_index(&self) -> Option<usize> {
        self.spec.selected_point_index
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }

    pub fn spec(&self) -> &ScatterPlotSpec {
        &self.spec
    }

    pub fn into_spec(self) -> ScatterPlotSpec {
        self.spec
    }
}

pub fn render_scatter_on<DB>(root: PlotArea<DB>, spec: &ScatterPlotSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    let mut session = ScatterSession::new(spec.clone())?;
    session.render_on(root)
}

pub fn pick_scatter_point(
    spec: &ScatterPlotSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<usize>, PlotError> {
    Ok(ScatterSession::new(spec.clone())?.pick_point(canvas_x, canvas_y))
}

pub fn pan_scatter_spec(
    spec: &ScatterPlotSpec,
    delta_x: f64,
    delta_y: f64,
) -> Result<ScatterPlotSpec, PlotError> {
    let mut session = ScatterSession::new(spec.clone())?;
    session.pan(delta_x, delta_y);
    Ok(session.into_spec())
}

fn resolve_point(point: &crate::ScatterPoint) -> Result<ResolvedPoint, PlotError> {
    ensure_finite("x", point.x)?;
    ensure_finite("y", point.y)?;

    let color = match point.color.as_deref() {
        Some(value) => parse_color(value)?,
        None => DEFAULT_POINT_COLOR,
    };

    Ok(ResolvedPoint {
        x: point.x,
        y: point.y,
        name: point
            .name
            .clone()
            .or_else(|| point.label.clone())
            .unwrap_or_else(|| format!("Point ({:.2}, {:.2})", point.x, point.y)),
        label: point.label.clone().unwrap_or_default(),
        color,
        radius: point
            .radius
            .unwrap_or(DEFAULT_POINT_RADIUS as u32)
            .min(i32::MAX as u32) as i32,
    })
}

fn resolve_ranges_from_spec(spec: &ScatterPlotSpec) -> Result<((f64, f64), (f64, f64)), PlotError> {
    let data_bounds = scan_data_bounds(&spec.points)?;
    let x_range = resolve_axis_range("x", spec.x_range, data_bounds.0, data_bounds.1)?;
    let y_range = resolve_axis_range("y", spec.y_range, data_bounds.2, data_bounds.3)?;
    Ok((x_range, y_range))
}

fn scan_data_bounds(points: &[crate::ScatterPoint]) -> Result<(f64, f64, f64, f64), PlotError> {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for point in points {
        ensure_finite("x", point.x)?;
        ensure_finite("y", point.y)?;
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    Ok((min_x, max_x, min_y, max_y))
}

fn build_scatter_chart<'a, DB>(
    root: &'a PlotArea<DB>,
    spec: &ScatterPlotSpec,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<ChartContext<'a, DB, ScatterCoordSpec>, PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    ChartBuilder::on(root)
        .margin(CHART_MARGIN)
        .caption(chart_title(spec), ("sans-serif", 24))
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .y_label_area_size(Y_LABEL_AREA_SIZE)
        .build_cartesian_2d(x_range.0..x_range.1, y_range.0..y_range.1)
        .map_err(backend_error)
}

fn chart_title(spec: &ScatterPlotSpec) -> &str {
    if spec.title.is_empty() {
        "Scatter Plot"
    } else {
        spec.title.as_str()
    }
}

fn x_axis_label(spec: &ScatterPlotSpec) -> &str {
    if spec.x_label.is_empty() {
        "x"
    } else {
        spec.x_label.as_str()
    }
}

fn y_axis_label(spec: &ScatterPlotSpec) -> &str {
    if spec.y_label.is_empty() {
        "y"
    } else {
        spec.y_label.as_str()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn sample_spec() -> ScatterPlotSpec {
        ScatterPlotSpec {
            width: 480,
            height: 320,
            title: "Scatter".to_string(),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            x_range: None,
            y_range: None,
            selected_point_index: None,
            points: vec![
                crate::ScatterPoint {
                    x: -2.0,
                    y: 3.5,
                    name: Some("Alpha".to_string()),
                    label: Some("alpha".to_string()),
                    color: Some("#2563eb".to_string()),
                    radius: Some(5),
                    properties: Default::default(),
                },
                crate::ScatterPoint {
                    x: 4.0,
                    y: -1.5,
                    name: Some("Beta".to_string()),
                    label: Some("beta".to_string()),
                    color: Some("#f97316".to_string()),
                    radius: Some(7),
                    properties: Default::default(),
                },
            ],
        }
    }

    #[test]
    fn scatter_auto_range_supports_negative_values() {
        let spec = sample_spec();
        let (x_range, y_range) = resolve_ranges_from_spec(&spec).unwrap();

        assert!(x_range.0 < -2.0);
        assert!(x_range.1 > 4.0);
        assert!(y_range.0 < -1.5);
        assert!(y_range.1 > 3.5);
    }

    #[test]
    fn scatter_auto_range_expands_constant_axis() {
        let range = resolve_axis_range("x", None, 3.0, 3.0).unwrap();
        assert_eq!(range, (2.0, 4.0));
    }

    #[test]
    fn scatter_invalid_explicit_range_is_rejected() {
        let error = resolve_axis_range("x", Some([5.0, 5.0]), 1.0, 1.0).unwrap_err();
        assert_eq!(
            error,
            PlotError::InvalidRange {
                axis: "x",
                min: 5.0,
                max: 5.0,
            }
        );
    }

    #[test]
    fn scatter_svg_output_contains_one_circle_per_point() {
        let mut svg = String::new();
        let spec = sample_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();

        render_scatter_on(area, &spec).unwrap();

        assert_eq!(svg.matches("<circle").count(), spec.points.len());
        assert!(svg.contains("alpha"));
        assert!(svg.contains("beta"));
    }

    #[test]
    fn scatter_viewport_round_trip_is_stable() {
        let spec = sample_spec();
        let session = ScatterSession::new(spec).unwrap();
        let point = (1.25, -0.5);
        let pixel = session.viewport.translate(point);
        let translated = session.viewport.reverse_translate(pixel);

        assert!((translated.0 - point.0).abs() < 0.05);
        assert!((translated.1 - point.1).abs() < 0.05);
    }

    #[test]
    fn scatter_hit_test_returns_expected_index() {
        let spec = sample_spec();
        let session = ScatterSession::new(spec).unwrap();
        let target = session.viewport.translate((4.0, -1.5));

        let selected = session.pick_point(f64::from(target.0), f64::from(target.1));

        assert_eq!(selected, Some(1));
    }

    #[test]
    fn scatter_pan_updates_explicit_ranges() {
        let spec = sample_spec();
        let mut session = ScatterSession::new(spec.clone()).unwrap();
        session.pan(24.0, -12.0);
        let panned = session.into_spec();

        assert!(panned.x_range.is_some());
        assert!(panned.y_range.is_some());
        assert_ne!(panned.x_range, spec.x_range);
        assert_ne!(panned.y_range, spec.y_range);
    }
}
