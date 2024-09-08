
pub struct Speedometer {
    // colors
    pub long_notch_color: Color,
    pub short_notch_color: Color,
    pub notch_text_color: Color,
    pub speed_bar_color: Color,
    pub speed_arc_color: Color,
    pub speed_display_text_color: Color,
    pub outer_circle_outline_color: Color,
    pub outer_circle_fill_color: Color,
    pub inner_circle_outline_color: Color,
    // idk
    pub width: i32,
    pub height: i32,
    pub long_notch_length: i8,
    pub short_notch_length: i8,
    pub notch_interval: i16,
    pub font_size: f64,
    pub speed_font_size: f64,
    pub bounds: (i16, i16)
}

impl Speedometer {
    pub fn defualt() -> Speedometer{
        return Speedometer{
            long_notch_color: Color{red: 0.27, green: 0.76, blue: 0.80, alpha: 1.0},
            short_notch_color: Color{red: 0.7, green: 0.88, blue: 0.88, alpha: 1.0},
            notch_text_color: Color{red: 0.0, green: 0.49, blue: 0.2, alpha: 1.0},
            speed_bar_color: Color{red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0},
            speed_arc_color: Color{red: 0.75, green: 0.84, blue: 0.19, alpha: 1.0},
            speed_display_text_color: Color{red: 0.44, green: 0.75, blue: 0.27, alpha: 1.0},
            outer_circle_outline_color: Color{red: 0.0, green: 0.48, blue: 0.20, alpha: 1.0},
            outer_circle_fill_color: Color{red: 0.08, green: 0.28, blue: 0.20, alpha: 1.0},
            inner_circle_outline_color: Color{red: 0.0, green: 0.49, blue: 0.20, alpha: 1.0},
            width: 800,
            height: 800,
            long_notch_length: 70,
            short_notch_length: 50,
            notch_interval: 20,
            font_size: 44.0,
            speed_font_size: 50.0,
            bounds: (0,160),
        };
    }
}

pub struct Color {
    pub red: f64,
    pub blue: f64,
    pub green: f64,
    pub alpha: f64,
}