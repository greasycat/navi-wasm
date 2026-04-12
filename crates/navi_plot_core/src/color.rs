use crate::PlotError;
use plotters::prelude::RGBColor;

pub(crate) fn parse_color(value: &str) -> Result<RGBColor, PlotError> {
    let normalized = value.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "black" => Ok(RGBColor(0, 0, 0)),
        "blue" => Ok(RGBColor(36, 99, 235)),
        "green" => Ok(RGBColor(22, 163, 74)),
        "red" => Ok(RGBColor(220, 38, 38)),
        "orange" => Ok(RGBColor(249, 115, 22)),
        "purple" => Ok(RGBColor(147, 51, 234)),
        "teal" => Ok(RGBColor(13, 148, 136)),
        "gray" | "grey" => Ok(RGBColor(107, 114, 128)),
        _ if normalized.starts_with('#') && normalized.len() == 7 => {
            let red = u8::from_str_radix(&normalized[1..3], 16);
            let green = u8::from_str_radix(&normalized[3..5], 16);
            let blue = u8::from_str_radix(&normalized[5..7], 16);

            match (red, green, blue) {
                (Ok(red), Ok(green), Ok(blue)) => Ok(RGBColor(red, green, blue)),
                _ => Err(PlotError::InvalidColor {
                    color: value.to_string(),
                }),
            }
        }
        _ => Err(PlotError::InvalidColor {
            color: value.to_string(),
        }),
    }
}
