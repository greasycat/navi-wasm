use crate::color::parse_color;
use crate::types::{BarChartSpec, BarVariant};
use crate::viewport::{ensure_finite, ScreenTransform};
use crate::{backend_error, ensure_dimensions, PlotArea, PlotError};
use plotters::prelude::*;

/// Default series color palette.
const SERIES_PALETTE: [RGBColor; 6] = [
    RGBColor(37, 99, 235),
    RGBColor(22, 163, 74),
    RGBColor(249, 115, 22),
    RGBColor(220, 38, 38),
    RGBColor(147, 51, 234),
    RGBColor(15, 118, 110),
];

const CAPTION_AREA_SIZE: u32 = 29;
const X_LABEL_AREA_SIZE: u32 = 52;
const Y_LABEL_AREA_SIZE: u32 = 54;
const BAR_GROUP_GAP: i32 = 8;
const BAR_INNER_GAP: i32 = 2;

#[derive(Debug, Clone, Copy)]
struct BarRect {
    series_index: usize,
    category_index: usize,
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

#[derive(Debug, Clone)]
struct ResolvedBar {
    label: String,
    color: RGBColor,
    values: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct BarSession {
    spec: BarChartSpec,
    resolved: Vec<ResolvedBar>,
    y_max: f64,
    view: ScreenTransform,
}

impl BarSession {
    pub fn new(spec: BarChartSpec) -> Result<Self, PlotError> {
        validate(&spec)?;
        let resolved = resolve_series(&spec)?;
        let y_max = compute_y_max(&spec).max(f64::EPSILON);

        Ok(Self {
            spec,
            resolved,
            y_max,
            view: ScreenTransform::new(0.0, 0.0),
        })
    }

    pub fn render_on<DB>(&self, root: PlotArea<DB>) -> Result<(), PlotError>
    where
        DB: DrawingBackend,
        DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
    {
        let (pl, pt, pr, pb) = plot_area(&self.spec);
        let n_cats = self.spec.categories.len();

        root.fill(&WHITE).map_err(backend_error)?;

        let title = if self.spec.title.is_empty() {
            "Bar Chart"
        } else {
            &self.spec.title
        };

        let mut chart = ChartBuilder::on(&root)
            .margin(self.spec.margin)
            .caption(title, ("sans-serif", 24))
            .x_label_area_size(X_LABEL_AREA_SIZE)
            .y_label_area_size(Y_LABEL_AREA_SIZE)
            .build_cartesian_2d(0f64..(n_cats as f64), 0f64..self.y_max)
            .map_err(backend_error)?;

        chart
            .configure_mesh()
            .x_labels(0)
            .y_desc(if self.spec.y_label.is_empty() {
                "Value"
            } else {
                &self.spec.y_label
            })
            .bold_line_style(RGBColor(209, 213, 219))
            .light_line_style(RGBColor(229, 231, 235))
            .axis_style(BLACK.mix(0.85))
            .draw()
            .map_err(backend_error)?;

        let drawing_area = root.clone();
        let plot_clip = (pl, pt, pr, pb);

        for rect in compute_hit_rects(&self.spec) {
            let transformed = transform_bar_rect(rect, &self.view, plot_clip);
            let Some(transformed) = transformed else {
                continue;
            };
            let series = &self.resolved[rect.series_index];
            let is_selected = self.spec.selected_bar.is_some_and(|[sel_si, sel_ci]| {
                sel_si == rect.series_index && sel_ci == rect.category_index
            });

            drawing_area
                .draw(&Rectangle::new(
                    [
                        (transformed.left, transformed.top),
                        (transformed.right, transformed.bottom),
                    ],
                    series.color.filled(),
                ))
                .map_err(backend_error)?;

            if is_selected {
                drawing_area
                    .draw(&Rectangle::new(
                        [
                            (transformed.left, transformed.top),
                            (transformed.right, transformed.bottom),
                        ],
                        ShapeStyle::from(&BLACK).stroke_width(2),
                    ))
                    .map_err(backend_error)?;
            }
        }

        let plot_w = (pr - pl).max(1);
        let group_w = plot_w / n_cats as i32;
        for (ci, cat) in self.spec.categories.iter().enumerate() {
            let group_left = pl + ci as i32 * group_w;
            let label_x = transform_x(group_left + group_w / 2, &self.view);
            let label_y = pb + 6;
            if label_x < pl || label_x > pr {
                continue;
            }
            drawing_area
                .draw(&Text::new(
                    cat.to_owned(),
                    (label_x, label_y),
                    ("sans-serif", 13).into_font(),
                ))
                .map_err(backend_error)?;
        }

        if self.spec.show_legend {
            let legend_x = pr - 120;
            let legend_y = pt + 8;
            for (si, series) in self.resolved.iter().enumerate() {
                let y = legend_y + si as i32 * 20;
                drawing_area
                    .draw(&Rectangle::new(
                        [(legend_x, y), (legend_x + 14, y + 12)],
                        series.color.filled(),
                    ))
                    .map_err(backend_error)?;
                drawing_area
                    .draw(&Text::new(
                        series.label.as_str(),
                        (legend_x + 18, y),
                        ("sans-serif", 12).into_font(),
                    ))
                    .map_err(backend_error)?;
            }
        }

        root.present().map_err(backend_error)?;
        Ok(())
    }

    pub fn pick_bar(&self, canvas_x: f64, canvas_y: f64) -> Option<[usize; 2]> {
        if !canvas_x.is_finite() || !canvas_y.is_finite() {
            return None;
        }
        let (pl, pt, pr, pb) = plot_area(&self.spec);
        let cx = canvas_x.round() as i32;
        let cy = canvas_y.round() as i32;
        if cx < pl || cx > pr || cy < pt || cy > pb {
            return None;
        }

        let (logical_x, logical_y) = self.view.inverse((canvas_x, canvas_y));
        let logical_x = logical_x.round() as i32;
        let logical_y = logical_y.round() as i32;

        compute_hit_rects(&self.spec)
            .into_iter()
            .find(|rect| {
                logical_x >= rect.left
                    && logical_x <= rect.right
                    && logical_y >= rect.top
                    && logical_y <= rect.bottom
            })
            .map(|rect| [rect.series_index, rect.category_index])
    }

    pub fn zoom_at(&mut self, canvas_x: f64, canvas_y: f64, factor: f64) -> Result<(), PlotError> {
        let (pl, pt, pr, pb) = plot_area(&self.spec);
        let anchor_x = canvas_x.clamp(f64::from(pl), f64::from(pr));
        let anchor_y = canvas_y.clamp(f64::from(pt), f64::from(pb));
        self.view.zoom_at(anchor_x, anchor_y, factor)
    }

    pub fn set_selection(&mut self, selection: Option<[usize; 2]>) {
        self.spec.selected_bar = selection.filter(|[si, ci]| {
            self.resolved
                .get(*si)
                .is_some_and(|series| *ci < series.values.len())
        });
    }

    pub fn spec(&self) -> &BarChartSpec {
        &self.spec
    }

    pub fn into_spec(self) -> BarChartSpec {
        self.spec
    }

    pub fn width(&self) -> u32 {
        self.spec.width
    }

    pub fn height(&self) -> u32 {
        self.spec.height
    }
}

fn validate(spec: &BarChartSpec) -> Result<(), PlotError> {
    ensure_dimensions(spec.width, spec.height)?;
    if spec.categories.is_empty() {
        return Err(PlotError::EmptyBarCategories);
    }
    if spec.series.is_empty() {
        return Err(PlotError::EmptyBarSeries);
    }
    let n = spec.categories.len();
    for (i, series) in spec.series.iter().enumerate() {
        if series.values.len() != n {
            return Err(PlotError::BarValueCountMismatch {
                series_index: i,
                expected: n,
                actual: series.values.len(),
            });
        }
        for (j, &value) in series.values.iter().enumerate() {
            ensure_finite("value", value)?;
            if matches!(spec.variant, BarVariant::Stacked) && value < 0.0 {
                return Err(PlotError::NegativeStackedBarValue {
                    series_index: i,
                    category_index: j,
                });
            }
        }
    }
    Ok(())
}

fn resolve_series(spec: &BarChartSpec) -> Result<Vec<ResolvedBar>, PlotError> {
    spec.series
        .iter()
        .enumerate()
        .map(|(i, series)| {
            let color = match series.color.as_deref() {
                Some(c) => parse_color(c)?,
                None => SERIES_PALETTE[i % SERIES_PALETTE.len()],
            };
            Ok(ResolvedBar {
                label: series.label.clone(),
                color,
                values: series.values.clone(),
            })
        })
        .collect()
}

fn compute_y_max(spec: &BarChartSpec) -> f64 {
    if let Some(max) = spec.y_max {
        return max;
    }
    match spec.variant {
        BarVariant::Grouped => spec
            .series
            .iter()
            .flat_map(|series| series.values.iter().copied())
            .fold(0.0_f64, f64::max),
        BarVariant::Stacked => {
            let n = spec.categories.len();
            (0..n)
                .map(|ci| {
                    spec.series
                        .iter()
                        .map(|series| series.values[ci])
                        .sum::<f64>()
                })
                .fold(0.0_f64, f64::max)
        }
    }
}

fn plot_area(spec: &BarChartSpec) -> (i32, i32, i32, i32) {
    let left = (spec.margin + Y_LABEL_AREA_SIZE) as i32;
    let top = (spec.margin + CAPTION_AREA_SIZE) as i32;
    let right = (spec.width.saturating_sub(spec.margin)) as i32;
    let bottom = (spec.height.saturating_sub(spec.margin + X_LABEL_AREA_SIZE)) as i32;
    (left, top, right.max(left + 1), bottom.max(top + 1))
}

fn compute_hit_rects(spec: &BarChartSpec) -> Vec<BarRect> {
    let n_cats = spec.categories.len();
    let n_series = spec.series.len();
    if n_cats == 0 || n_series == 0 {
        return Vec::new();
    }

    let (pl, pt, pr, pb) = plot_area(spec);
    let y_max = compute_y_max(spec).max(f64::EPSILON);
    let plot_w = (pr - pl).max(1);
    let plot_h = (pb - pt).max(1);
    let group_w = plot_w / n_cats as i32;

    let mut rects = Vec::new();

    match spec.variant {
        BarVariant::Grouped => {
            let bar_w = ((group_w - BAR_GROUP_GAP * 2)
                .saturating_sub(BAR_INNER_GAP * (n_series as i32 - 1)))
                / n_series as i32;
            let bar_w = bar_w.max(1);

            for ci in 0..n_cats {
                let group_left = pl + ci as i32 * group_w + BAR_GROUP_GAP;
                for si in 0..n_series {
                    let value = spec.series[si].values[ci];
                    let bar_left = group_left + si as i32 * (bar_w + BAR_INNER_GAP);
                    let bar_height = ((value / y_max) * plot_h as f64).round() as i32;
                    let bar_top = pb - bar_height;
                    rects.push(BarRect {
                        series_index: si,
                        category_index: ci,
                        left: bar_left,
                        right: bar_left + bar_w,
                        top: bar_top.max(pt),
                        bottom: pb,
                    });
                }
            }
        }
        BarVariant::Stacked => {
            let bar_w = (group_w - BAR_GROUP_GAP * 2).max(1);
            for ci in 0..n_cats {
                let bar_left = pl + ci as i32 * group_w + BAR_GROUP_GAP;
                let mut cumulative = 0.0_f64;
                for si in 0..n_series {
                    let value = spec.series[si].values[ci];
                    let bot = pb - ((cumulative / y_max) * plot_h as f64).round() as i32;
                    cumulative += value;
                    let top = pb - ((cumulative / y_max) * plot_h as f64).round() as i32;
                    rects.push(BarRect {
                        series_index: si,
                        category_index: ci,
                        left: bar_left,
                        right: bar_left + bar_w,
                        top: top.max(pt),
                        bottom: bot,
                    });
                }
            }
        }
    }

    rects
}

fn transform_bar_rect(
    rect: BarRect,
    view: &ScreenTransform,
    clip: (i32, i32, i32, i32),
) -> Option<BarRect> {
    let (left, top) = view.apply((f64::from(rect.left), f64::from(rect.top)));
    let (right, bottom) = view.apply((f64::from(rect.right), f64::from(rect.bottom)));
    let (clip_left, clip_top, clip_right, clip_bottom) = clip;
    let left = left.clamp(clip_left, clip_right);
    let right = right.clamp(clip_left, clip_right);
    let top = top.clamp(clip_top, clip_bottom);
    let bottom = bottom.clamp(clip_top, clip_bottom);

    (left < right && top < bottom).then_some(BarRect {
        series_index: rect.series_index,
        category_index: rect.category_index,
        left,
        right,
        top,
        bottom,
    })
}

fn transform_x(x: i32, view: &ScreenTransform) -> i32 {
    view.apply((f64::from(x), 0.0)).0
}

pub fn render_bar_on<DB>(root: PlotArea<DB>, spec: &BarChartSpec) -> Result<(), PlotError>
where
    DB: DrawingBackend,
    DB::ErrorType: std::fmt::Debug + std::error::Error + Send + Sync,
{
    BarSession::new(spec.clone())?.render_on(root)
}

pub fn pick_bar(
    spec: &BarChartSpec,
    canvas_x: f64,
    canvas_y: f64,
) -> Result<Option<[usize; 2]>, PlotError> {
    Ok(BarSession::new(spec.clone())?.pick_bar(canvas_x, canvas_y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BarSeries;
    use plotters::drawing::IntoDrawingArea;
    use plotters_svg::SVGBackend;

    fn grouped_spec() -> BarChartSpec {
        BarChartSpec {
            width: 480,
            height: 320,
            title: "Test Bar".to_string(),
            x_label: String::new(),
            y_label: String::new(),
            y_max: None,
            categories: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            series: vec![
                BarSeries {
                    label: "S1".to_string(),
                    color: Some("#2563eb".to_string()),
                    values: vec![4.0, 7.0, 3.0],
                },
                BarSeries {
                    label: "S2".to_string(),
                    color: Some("#16a34a".to_string()),
                    values: vec![2.0, 5.0, 8.0],
                },
            ],
            variant: BarVariant::Grouped,
            show_legend: true,
            margin: 32,
            selected_bar: None,
        }
    }

    fn stacked_spec() -> BarChartSpec {
        BarChartSpec {
            variant: BarVariant::Stacked,
            ..grouped_spec()
        }
    }

    #[test]
    fn bar_rejects_empty_categories() {
        let mut spec = grouped_spec();
        spec.categories.clear();
        spec.series
            .iter_mut()
            .for_each(|series| series.values.clear());
        let err = render_bar_on(
            SVGBackend::with_string(&mut String::new(), (480, 320)).into_drawing_area(),
            &spec,
        )
        .unwrap_err();
        assert_eq!(err, PlotError::EmptyBarCategories);
    }

    #[test]
    fn bar_rejects_empty_series() {
        let mut spec = grouped_spec();
        spec.series.clear();
        let err = render_bar_on(
            SVGBackend::with_string(&mut String::new(), (480, 320)).into_drawing_area(),
            &spec,
        )
        .unwrap_err();
        assert_eq!(err, PlotError::EmptyBarSeries);
    }

    #[test]
    fn bar_rejects_value_count_mismatch() {
        let mut spec = grouped_spec();
        spec.series[1].values.push(1.0);
        let err = render_bar_on(
            SVGBackend::with_string(&mut String::new(), (480, 320)).into_drawing_area(),
            &spec,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            PlotError::BarValueCountMismatch {
                series_index: 1,
                ..
            }
        ));
    }

    #[test]
    fn bar_rejects_negative_stacked_values() {
        let mut spec = stacked_spec();
        spec.series[0].values[1] = -1.0;
        let err = render_bar_on(
            SVGBackend::with_string(&mut String::new(), (480, 320)).into_drawing_area(),
            &spec,
        )
        .unwrap_err();
        assert!(matches!(err, PlotError::NegativeStackedBarValue { .. }));
    }

    #[test]
    fn bar_grouped_svg_renders_without_error() {
        let mut svg = String::new();
        let spec = grouped_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_bar_on(area, &spec).unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn bar_stacked_svg_renders_without_error() {
        let mut svg = String::new();
        let spec = stacked_spec();
        let area = SVGBackend::with_string(&mut svg, (spec.width, spec.height)).into_drawing_area();
        render_bar_on(area, &spec).unwrap();
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn bar_auto_y_max_covers_grouped_max() {
        let spec = grouped_spec();
        let y_max = compute_y_max(&spec);
        assert!(y_max >= 8.0);
    }

    #[test]
    fn bar_auto_y_max_covers_stacked_total() {
        let spec = stacked_spec();
        let y_max = compute_y_max(&spec);
        assert!(y_max >= 12.0);
    }

    #[test]
    fn bar_hit_test_grouped_returns_correct_bar() {
        let spec = grouped_spec();
        let rects = compute_hit_rects(&spec);
        let rect = &rects[0];
        let cx = ((rect.left + rect.right) / 2) as f64;
        let cy = ((rect.top + rect.bottom) / 2) as f64;
        let hit = pick_bar(&spec, cx, cy).unwrap();
        assert_eq!(hit, Some([rect.series_index, rect.category_index]));
    }

    #[test]
    fn bar_zoom_changes_pick_geometry() {
        let spec = grouped_spec();
        let rect = compute_hit_rects(&spec)[0];
        let base_center = (
            f64::from((rect.left + rect.right) / 2),
            f64::from((rect.top + rect.bottom) / 2),
        );

        let mut session = BarSession::new(spec).unwrap();
        session.zoom_at(base_center.0, base_center.1, 2.0).unwrap();

        let outside_base_inside_zoom = (base_center.0 + 10.0, base_center.1);

        assert_eq!(
            session.pick_bar(outside_base_inside_zoom.0, outside_base_inside_zoom.1),
            Some([rect.series_index, rect.category_index])
        );
    }
}
