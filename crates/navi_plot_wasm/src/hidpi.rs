#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CanvasHiDpiMode {
    Standard,
    Logical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CanvasHiDpiPlan {
    pub(crate) mode: CanvasHiDpiMode,
    pub(crate) logical_width: u32,
    pub(crate) logical_height: u32,
    pub(crate) backing_width: u32,
    pub(crate) backing_height: u32,
    pub(crate) transform_scale: f64,
    pub(crate) set_logical_style_size: bool,
}

fn sanitized_device_pixel_ratio(value: f64) -> f64 {
    if value.is_finite() && value > 1.0 {
        value.min(8.0)
    } else {
        1.0
    }
}

fn within_tolerance(actual: u32, expected: f64, tolerance: i64) -> bool {
    (i64::from(actual) - expected.round() as i64).abs() <= tolerance
}

pub(crate) fn resolve_canvas_hidpi_plan(
    width: u32,
    height: u32,
    client_width: u32,
    client_height: u32,
    device_pixel_ratio: f64,
) -> CanvasHiDpiPlan {
    let device_pixel_ratio = sanitized_device_pixel_ratio(device_pixel_ratio);
    let scaled_tolerance = device_pixel_ratio.ceil() as i64 + 1;
    let logical_tolerance = 2;
    let looks_like_legacy_scaled = device_pixel_ratio > 1.0
        && client_width > 0
        && client_height > 0
        && within_tolerance(
            width,
            f64::from(client_width) * device_pixel_ratio,
            scaled_tolerance,
        )
        && within_tolerance(
            height,
            f64::from(client_height) * device_pixel_ratio,
            scaled_tolerance,
        );

    if looks_like_legacy_scaled {
        return CanvasHiDpiPlan {
            mode: CanvasHiDpiMode::Standard,
            logical_width: width,
            logical_height: height,
            backing_width: width,
            backing_height: height,
            transform_scale: 1.0,
            set_logical_style_size: false,
        };
    }

    if device_pixel_ratio <= 1.0 {
        return CanvasHiDpiPlan {
            mode: CanvasHiDpiMode::Standard,
            logical_width: width,
            logical_height: height,
            backing_width: width,
            backing_height: height,
            transform_scale: 1.0,
            set_logical_style_size: false,
        };
    }

    let set_logical_style_size = client_width == 0
        || client_height == 0
        || (within_tolerance(width, f64::from(client_width), logical_tolerance)
            && within_tolerance(height, f64::from(client_height), logical_tolerance));

    CanvasHiDpiPlan {
        mode: CanvasHiDpiMode::Logical,
        logical_width: width,
        logical_height: height,
        backing_width: (f64::from(width) * device_pixel_ratio).round().max(1.0) as u32,
        backing_height: (f64::from(height) * device_pixel_ratio).round().max(1.0) as u32,
        transform_scale: device_pixel_ratio,
        set_logical_style_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_standard_mode_when_device_pixel_ratio_is_one() {
        let plan = resolve_canvas_hidpi_plan(640, 480, 640, 480, 1.0);
        assert_eq!(plan.mode, CanvasHiDpiMode::Standard);
        assert_eq!(plan.backing_width, 640);
        assert_eq!(plan.backing_height, 480);
        assert_eq!(plan.transform_scale, 1.0);
    }

    #[test]
    fn detects_legacy_scaled_canvas_dimensions() {
        let plan = resolve_canvas_hidpi_plan(1280, 960, 640, 480, 2.0);
        assert_eq!(plan.mode, CanvasHiDpiMode::Standard);
        assert_eq!(plan.backing_width, 1280);
        assert_eq!(plan.backing_height, 960);
    }

    #[test]
    fn upgrades_logical_canvas_dimensions_to_hidpi_backing_store() {
        let plan = resolve_canvas_hidpi_plan(640, 480, 640, 480, 2.0);
        assert_eq!(plan.mode, CanvasHiDpiMode::Logical);
        assert_eq!(plan.logical_width, 640);
        assert_eq!(plan.logical_height, 480);
        assert_eq!(plan.backing_width, 1280);
        assert_eq!(plan.backing_height, 960);
        assert!(plan.set_logical_style_size);
    }

    #[test]
    fn preserves_css_scaled_layout_without_forcing_inline_size() {
        let plan = resolve_canvas_hidpi_plan(720, 420, 360, 210, 3.0);
        assert_eq!(plan.mode, CanvasHiDpiMode::Logical);
        assert_eq!(plan.backing_width, 2160);
        assert_eq!(plan.backing_height, 1260);
        assert!(!plan.set_logical_style_size);
    }
}
