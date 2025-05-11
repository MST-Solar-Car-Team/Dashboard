// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::thread;
use std::time::Duration;
use slint::{ComponentHandle, SharedString};



slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = Dashboard::new()?; // From the Slint DSL

    let ui_handle = window.as_weak();
    thread::spawn(move || {
        let mut speed = 0;
        loop {
            // Increment speed by 1 every second
            speed = (speed + 1) % 121;

            let left_on = false;
            let right_on = true;
            let voltage = 52.3;
            let throttle = 75;
            let temp_bms = 32;
            let temp_motor = 45;
            let temp_controller = 38;

            // Update UI on the event loop
            let _ = ui_handle.upgrade_in_event_loop(move |window| {
                window.set_speed(speed);
                window.set_leftBlinkerOn(left_on);
                window.set_rightBlinkerOn(right_on);
                window.set_voltage(voltage);
                window.set_throttle(throttle);
                window.set_tempBMS(temp_bms);
                window.set_tempMotor(temp_motor);
                window.set_tempController(temp_controller);
            });

            thread::sleep(Duration::from_secs(1));
        }
    });

    window.run();
    Ok(())
}
