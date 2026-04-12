use crate::PlotError;

pub(crate) const CHART_MARGIN: u32 = 24;
pub(crate) const X_LABEL_AREA_SIZE: u32 = 42;
pub(crate) const Y_LABEL_AREA_SIZE: u32 = 54;
pub(crate) const CAPTION_AREA_SIZE: u32 = 29;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PixelBounds {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

impl PixelBounds {
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
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CartesianViewport {
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub plot_bounds: PixelBounds,
}

impl CartesianViewport {
    pub(crate) fn new(
        width: u32,
        height: u32,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
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
