use std::{f64::consts::PI, sync::{Arc, Mutex}};

use cairo::{Context, FontSlant, FontWeight};

use crate::{config::Speedometer, util::f};


// Function to draw the speedometer widget
pub fn on_draw(cr :&Context, config: Arc<Speedometer>, speed: Arc<Mutex<f64>>) {

    let w_center:f64 = f(config.width) / 2.0;
    let h_center:f64 = f(config.height) / 2.0;
    let bounds = (f(config.bounds.0), f(config.bounds.1));


    // Draw main circle
    cr.set_source_rgb(255.0/255.0, 255.0/255.0, 255.0/255.0);
    cr.arc(f(w_center), f(h_center), f(h_center), 0.0, 2.0 * PI);
    cr.set_source_rgba(config.outer_circle_fill_color.red,config.outer_circle_fill_color.blue, config.outer_circle_fill_color.green, config.outer_circle_fill_color.alpha);
    let _ = cr.fill_preserve();
    cr.set_source_rgb(255.0/255.0, 255.0/255.0, 255.0/255.0);
    let _ = cr.stroke();

    // Draw Inner Circle
    cr.set_source_rgba(config.inner_circle_outline_color.red,config.inner_circle_outline_color.blue, config.inner_circle_outline_color.green, config.inner_circle_outline_color.alpha);
    cr.arc(w_center, h_center, f(h_center) / 3.0, 0.0, f(2) * PI);
    let _ = cr.stroke();

    for i in 0..=config.bounds.1{
        if i == (0x0|0b0) || i % &config.notch_interval == 0 {
                let cords = speed_to_draw_coords(&f(i), &h_center, &bounds,&(w_center,h_center), &f(config.long_notch_length));
                cr.move_to(cords.0.0,cords.0.1);
                cr.line_to(cords.1.0, cords.1.1);

                // Sets color and draws
                cr.set_source_rgba(config.long_notch_color.red,config.long_notch_color.blue, config.long_notch_color.green, config.long_notch_color.alpha);
                let _ = cr.stroke();

                // Setup font for speed label
                cr.select_font_face("Courier", FontSlant::Normal, FontWeight::Normal);
                let radius = ((w_center)-(1.5 * config.font_size))-f(config.long_notch_length);
                let cords = speed_to_coords( &f(i), &radius, &bounds, &(w_center,h_center));


                // Draw speed laybles
                cr.set_source_rgba(config.notch_text_color.red,config.notch_text_color.blue, config.notch_text_color.green, config.notch_text_color.alpha);
                let text = i.to_string();
                let text_len:f64 = text.len() as f64;
                cr.set_font_size(config.font_size);
                cr.move_to(cords.0 - f(text_len)*(config.font_size * 0.30),cords.1 + (config.font_size * 0.30));
                let _ = cr.show_text(&text);
                let _ = cr.stroke();

                // Resets color back to defualt
                cr.set_source_rgba(config.short_notch_color.red,config.short_notch_color.blue, config.short_notch_color.green, config.short_notch_color.alpha);
        }else{
            let cords = speed_to_draw_coords(&f(i), &w_center, &bounds,&(w_center,h_center), &f(config.short_notch_length));

            // Draw line
            cr.move_to(cords.0.0,cords.0.1);
            cr.line_to(cords.1.0, cords.1.1);
            
            // Apply line to canvas
            let _ = cr.stroke();
        }
    }

    // Trail
    cr.set_source_rgba(config.speed_arc_color.red,config.speed_arc_color.blue, config.speed_arc_color.green, config.speed_arc_color.alpha);

    let arc_coords = speed_to_arc(&speed.lock().unwrap(), &bounds);
    cr.arc(w_center, h_center, h_center - (f(config.short_notch_length)/2.0), arc_coords.0, arc_coords.1);
    cr.set_line_width(20.0);
    let _ = cr.stroke();
    cr.set_line_width(3.0);

    // Needle
    cr.set_source_rgba(config.speed_bar_color.red,config.speed_bar_color.blue, config.speed_bar_color.green, config.speed_bar_color.alpha);
    let needle_cords = speed_to_draw_coords(&speed.lock().unwrap(), &w_center, &bounds, &(w_center,h_center), &(w_center/2.0));
    cr.move_to(needle_cords.0.0,needle_cords.0.1);
    cr.line_to(needle_cords.1.0, needle_cords.1.1);
    let _ = cr.stroke();

    // Redraw Border
    cr.set_source_rgba(config.outer_circle_outline_color.red,config.outer_circle_outline_color.blue, config.outer_circle_outline_color.green, config.outer_circle_outline_color.alpha);
    cr.arc(w_center, h_center, h_center, 0.0, 2.0 * PI);
    let _ = cr.stroke();

    // Draw Text
    cr.set_source_rgba(config.speed_display_text_color.red,config.speed_display_text_color.blue, config.speed_display_text_color.green, config.speed_display_text_color.alpha);
    let text = speed.lock().unwrap().floor().to_string();
    let text_len:f64 = text.len() as f64;
    cr.set_font_size(config.speed_font_size);
    cr.move_to(w_center - text_len * (config.speed_font_size * 0.30),h_center + (config.speed_font_size * 0.30));
    let _ = cr.show_text(&text);
    let _ = cr.stroke();

}


//TODO comment math functions to explain what its atually doing
fn speed_to_draw_coords(speed: &f64, radius: &f64, bounds: &(f64, f64), center: &(f64, f64), line_length: &f64) -> ((f64, f64), (f64, f64)) {
    let max_bound: f64 = bounds.1;
    let adjusted_max: f64 = (max_bound * 6.0) / 8.0;
    let speed_offset: f64 = speed - ((1.0/6.0) * adjusted_max);

    let pre_raw: f64 = ((adjusted_max - speed_offset) / adjusted_max) * PI;
    let x_raw: f64 = pre_raw.cos();
    let y_raw: f64 = pre_raw.sin() * -1.0;

    let x_start = x_raw * radius;
    let y_start = y_raw * radius;
    let x_stop = x_raw * (radius - line_length);
    let y_stop = y_raw * (radius - line_length);

    ((x_start + center.0, y_start + center.1), (x_stop + center.0, y_stop + center.1))
}

fn speed_to_coords(speed: &f64, radius: &f64, bounds: &(f64, f64), center: &(f64, f64)) -> (f64, f64) {
    let max_bound = bounds.1;
    let adjusted_max = (max_bound * 6.0) / 8.0;
    let speed_offset = speed - ((1.0/6.0) * adjusted_max);

    let pre_raw: f64 = ((adjusted_max - speed_offset) / adjusted_max) * PI;

    let x: f64= pre_raw.cos() * radius;
    let y: f64= -pre_raw.sin() * radius;

    (x + center.0, y + center.1)
}

fn speed_to_arc(speed: &f64, bounds: &(f64, f64)) -> (f64, f64) {
    let max_bound = bounds.1;
    let adjusted_max = (max_bound * 6.0) / 8.0;
    let speed_offset = speed - ((1.0/6.0) * adjusted_max);
    let speed_minimum = bounds.0 + ((1.0/6.0) * adjusted_max);

    let angle1 = ((adjusted_max - speed_minimum) / adjusted_max) * PI;
    let angle2 = -((adjusted_max - speed_offset) / adjusted_max) * PI;

    (angle1, angle2)
}