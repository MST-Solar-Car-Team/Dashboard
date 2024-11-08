use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub speedometer: Speedometer,
}

#[derive(Deserialize, Debug)]
pub struct Speedometer {
    // Colors
    pub long_notch_color: Color,
    pub short_notch_color: Color,
    pub notch_text_color: Color,
    pub speed_bar_color: Color,
    pub speed_arc_color: Color,
    pub speed_display_text_color: Color,
    pub outer_circle_outline_color: Color,
    pub outer_circle_fill_color: Color,
    pub inner_circle_outline_color: Color,
    // Scalables (might rename)
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
    pub fn to_decimal(self) -> Speedometer {
        Speedometer {
            long_notch_color: self.long_notch_color.to_decimal(),
            short_notch_color: self.short_notch_color.to_decimal(),
            notch_text_color: self.notch_text_color.to_decimal(),
            speed_bar_color: self.speed_bar_color.to_decimal(),
            speed_arc_color: self.speed_arc_color.to_decimal(),
            speed_display_text_color: self.speed_display_text_color.to_decimal(),
            outer_circle_outline_color: self.outer_circle_outline_color.to_decimal(),
            outer_circle_fill_color: self.outer_circle_fill_color.to_decimal(),
            inner_circle_outline_color: self.inner_circle_outline_color.to_decimal(),
            ..self
        }
    }
}
 
//what
#[derive(Deserialize, Debug)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,

}

impl Color {
    fn to_decimal (self) -> Color {
        Color {red: self.red / 255.0, green: self.green / 255.0, blue: self.blue / 255.0}
    }
}