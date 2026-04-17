use crate::PlotError;

pub(crate) const CHART_MARGIN: u32 = 24;
pub(crate) const X_LABEL_AREA_SIZE: u32 = 42;
pub(crate) const Y_LABEL_AREA_SIZE: u32 = 54;
pub(crate) const CAPTION_AREA_SIZE: u32 = 29;
const MIN_SCREEN_ZOOM: f64 = 0.05;
const MAX_SCREEN_ZOOM: f64 = 8.0;
const MIN_AXIS_SPAN: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PixelBounds {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl PixelBounds {
    pub(crate) fn from_canvas(width: u32, height: u32) -> Self {
        Self {
            left: 0,
            right: width.max(1) as i32,
            top: 0,
            bottom: height.max(1) as i32,
        }
    }

    pub(crate) fn from_dimensions(width: u32, height: u32) -> Self {
        let left = CHART_MARGIN.saturating_add(Y_LABEL_AREA_SIZE) as i32;
        let right = width.saturating_sub(CHART_MARGIN) as i32;
        let top = CHART_MARGIN.saturating_add(CAPTION_AREA_SIZE) as i32;
        let bottom = height.saturating_sub(CHART_MARGIN.saturating_add(X_LABEL_AREA_SIZE)) as i32;

        Self {
            left,
            right: right.max(left + 1),
            top,
            bottom: bottom.max(top + 1),
        }
    }

    pub(crate) fn max_x(self) -> i32 {
        self.right.saturating_sub(1)
    }

    pub(crate) fn max_y(self) -> i32 {
        self.bottom.saturating_sub(1)
    }

    pub(crate) fn span_x(self) -> f64 {
        f64::from((self.max_x() - self.left).max(1))
    }

    pub(crate) fn span_y(self) -> f64 {
        f64::from((self.max_y() - self.top).max(1))
    }

    pub(crate) fn center(self) -> (i32, i32) {
        (
            self.left + (self.max_x() - self.left) / 2,
            self.top + (self.max_y() - self.top) / 2,
        )
    }

    pub(crate) fn clamp_x(self, value: i32) -> i32 {
        value.clamp(self.left, self.max_x())
    }

    pub(crate) fn clamp_y(self, value: i32) -> i32 {
        value.clamp(self.top, self.max_y())
    }

    pub(crate) fn contains(self, point: (i32, i32)) -> bool {
        point.0 >= self.left
            && point.0 <= self.max_x()
            && point.1 >= self.top
            && point.1 <= self.max_y()
    }

    pub(crate) fn intersects_circle(self, center: (i32, i32), radius: i32) -> bool {
        let radius = radius.max(0);
        let nearest_x = center.0.clamp(self.left, self.max_x());
        let nearest_y = center.1.clamp(self.top, self.max_y());
        let dx = i64::from(center.0 - nearest_x);
        let dy = i64::from(center.1 - nearest_y);
        let radius = i64::from(radius);
        dx * dx + dy * dy <= radius * radius
    }

    pub(crate) fn clip_line(
        self,
        start: (i32, i32),
        end: (i32, i32),
    ) -> Option<((i32, i32), (i32, i32))> {
        let mut t0: f64 = 0.0;
        let mut t1: f64 = 1.0;
        let dx = f64::from(end.0 - start.0);
        let dy = f64::from(end.1 - start.1);
        let x_min = f64::from(self.left);
        let x_max = f64::from(self.max_x());
        let y_min = f64::from(self.top);
        let y_max = f64::from(self.max_y());

        for (p, q) in [
            (-dx, f64::from(start.0) - x_min),
            (dx, x_max - f64::from(start.0)),
            (-dy, f64::from(start.1) - y_min),
            (dy, y_max - f64::from(start.1)),
        ] {
            if p.abs() < f64::EPSILON {
                if q < 0.0 {
                    return None;
                }
                continue;
            }

            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return None;
                }
                t0 = t0.max(r);
            } else {
                if r < t0 {
                    return None;
                }
                t1 = t1.min(r);
            }
        }

        if t0 > t1 {
            return None;
        }

        let clipped_start = (
            (f64::from(start.0) + dx * t0).round() as i32,
            (f64::from(start.1) + dy * t0).round() as i32,
        );
        let clipped_end = (
            (f64::from(start.0) + dx * t1).round() as i32,
            (f64::from(start.1) + dy * t1).round() as i32,
        );

        Some((clipped_start, clipped_end))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CartesianViewport {
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub plot_bounds: PixelBounds,
}

impl CartesianViewport {
    pub(crate) fn new(width: u32, height: u32, x_range: (f64, f64), y_range: (f64, f64)) -> Self {
        Self {
            x_range,
            y_range,
            plot_bounds: PixelBounds::from_dimensions(width, height),
        }
    }

    pub(crate) fn with_plot_bounds(mut self, plot_bounds: PixelBounds) -> Self {
        self.plot_bounds = plot_bounds;
        self
    }

    pub(crate) fn translate(self, point: (f64, f64)) -> (i32, i32) {
        let x_ratio = normalize(point.0, self.x_range);
        let y_ratio = normalize(point.1, self.y_range);

        (
            self.plot_bounds.left + (x_ratio * self.plot_bounds.span_x()).round() as i32,
            self.plot_bounds.top + ((1.0 - y_ratio) * self.plot_bounds.span_y()).round() as i32,
        )
    }

    pub(crate) fn reverse_translate(self, pixel: (i32, i32)) -> (f64, f64) {
        let x_ratio = f64::from(pixel.0 - self.plot_bounds.left) / self.plot_bounds.span_x();
        let y_ratio = f64::from(pixel.1 - self.plot_bounds.top) / self.plot_bounds.span_y();
        let x = self.x_range.0 + x_ratio * (self.x_range.1 - self.x_range.0);
        let y = self.y_range.1 - y_ratio * (self.y_range.1 - self.y_range.0);
        (x, y)
    }

    pub(crate) fn zoom_at(
        &mut self,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), PlotError> {
        ensure_zoom_factor(factor)?;

        let anchor_pixel = (
            self.plot_bounds.clamp_x(canvas_x.round() as i32),
            self.plot_bounds.clamp_y(canvas_y.round() as i32),
        );
        let anchor = self.reverse_translate(anchor_pixel);
        let anchor_x_ratio = normalize(anchor.0, self.x_range);
        let anchor_y_ratio = normalize(anchor.1, self.y_range);

        let next_x_span = ((self.x_range.1 - self.x_range.0) / factor).max(MIN_AXIS_SPAN);
        let next_y_span = ((self.y_range.1 - self.y_range.0) / factor).max(MIN_AXIS_SPAN);

        let next_x = (
            anchor.0 - anchor_x_ratio * next_x_span,
            anchor.0 + (1.0 - anchor_x_ratio) * next_x_span,
        );
        let next_y = (
            anchor.1 - anchor_y_ratio * next_y_span,
            anchor.1 + (1.0 - anchor_y_ratio) * next_y_span,
        );

        self.x_range = next_x;
        self.y_range = next_y;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScreenTransform {
    pub zoom: f64,
    pub translate_x: f64,
    pub translate_y: f64,
}

impl ScreenTransform {
    pub(crate) fn new(translate_x: f64, translate_y: f64) -> Self {
        Self {
            zoom: 1.0,
            translate_x,
            translate_y,
        }
    }

    pub(crate) fn with_view(
        zoom: f64,
        translate_x: f64,
        translate_y: f64,
    ) -> Result<Self, PlotError> {
        ensure_zoom_factor(zoom)?;
        ensure_finite("translate_x", translate_x)?;
        ensure_finite("translate_y", translate_y)?;
        Ok(Self {
            zoom: zoom.clamp(MIN_SCREEN_ZOOM, MAX_SCREEN_ZOOM),
            translate_x,
            translate_y,
        })
    }

    pub(crate) fn pan_by(&mut self, delta_x: f64, delta_y: f64) {
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return;
        }
        self.translate_x += delta_x;
        self.translate_y += delta_y;
    }

    pub(crate) fn zoom_at(
        &mut self,
        canvas_x: f64,
        canvas_y: f64,
        factor: f64,
    ) -> Result<(), PlotError> {
        ensure_zoom_factor(factor)?;

        let next_zoom = (self.zoom * factor).clamp(MIN_SCREEN_ZOOM, MAX_SCREEN_ZOOM);
        let ratio = next_zoom / self.zoom;
        self.translate_x = canvas_x - (canvas_x - self.translate_x) * ratio;
        self.translate_y = canvas_y - (canvas_y - self.translate_y) * ratio;
        self.zoom = next_zoom;
        Ok(())
    }

    pub(crate) fn apply(self, point: (f64, f64)) -> (i32, i32) {
        (
            (point.0 * self.zoom + self.translate_x).round() as i32,
            (point.1 * self.zoom + self.translate_y).round() as i32,
        )
    }

    pub(crate) fn inverse(self, point: (f64, f64)) -> (f64, f64) {
        (
            (point.0 - self.translate_x) / self.zoom,
            (point.1 - self.translate_y) / self.zoom,
        )
    }
}

pub(crate) fn normalize(value: f64, range: (f64, f64)) -> f64 {
    let span = range.1 - range.0;
    if span.abs() < f64::EPSILON {
        0.5
    } else {
        (value - range.0) / span
    }
}

pub(crate) fn ensure_finite(axis: &'static str, value: f64) -> Result<(), PlotError> {
    if !value.is_finite() {
        return Err(PlotError::NonFinitePointValue { axis, value });
    }
    Ok(())
}

pub(crate) fn ensure_zoom_factor(factor: f64) -> Result<(), PlotError> {
    if !factor.is_finite() || factor <= 0.0 {
        return Err(PlotError::InvalidZoomFactor { factor });
    }
    Ok(())
}

pub(crate) fn resolve_axis_range(
    axis: &'static str,
    provided: Option<[f64; 2]>,
    observed_min: f64,
    observed_max: f64,
) -> Result<(f64, f64), PlotError> {
    if let Some([min, max]) = provided {
        ensure_finite(axis, min)?;
        ensure_finite(axis, max)?;
        if min >= max {
            return Err(PlotError::InvalidRange { axis, min, max });
        }
        return Ok((min, max));
    }

    if (observed_max - observed_min).abs() < f64::EPSILON {
        return Ok((observed_min - 1.0, observed_max + 1.0));
    }

    let padding = (observed_max - observed_min) * 0.05;
    Ok((observed_min - padding, observed_max + padding))
}
