// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::{ComponentHandle, SharedString};
use std::collections::VecDeque;
use std::error::Error;
use std::thread;
use std::time::Duration;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = Dashboard::new()?; // From the Slint DSL
    // println!("fuck you");
    // Find the first available serial port and use it
    let port_info = serialport::available_ports()?
        .into_iter()
        .next()
        .expect("No serial ports found");
    let mut ports = serialport::new(port_info.port_name, 115200)
        // .timeout(Duration::from_millis(0))
        .open()
        .expect("Failed to open serial port!");

    // Circular buff implmentation would be nice at some point
    let mut queue: VecDeque<u8> = VecDeque::new();
    let mut serial_buf: Vec<u8> = vec![0; 32];

    let ui_handle = window.as_weak();
    thread::spawn(move || {
        let mut speed = 0;
        let mut throttle = 0;
        loop {
            // Increment speed by 1 every second
            //speed = (speed + 1) % 121;

            let left_on = false;
            let right_on = true;
            let voltage = 52.3;
            // let mut throttle: u16;
            let temp_bms = 32;
            let temp_motor = 45;
            let temp_controller = 38;

            // println!("loops");
            match ports.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    for item in &serial_buf[..t] {
                        queue.push_back(item.to_owned());
                    }
                }
                Err(e) => print!(""),
            }

            while queue.len() > 16 {
                let packet_byte = queue.pop_front();

                if let Some(packet_id) = packet_byte {
                    match packet_id {
                        0x13 => {
                            let packet_full = queue.make_contiguous();
                            let packet_full = &packet_full[..15];
                            // read_pedal_packet(&packet_full[..15]);
                            throttle = ((packet_full[2] as u16) << 8) | (packet_full[3] as u16);
                            println!("throttle: {}", throttle);
                            //throttle = (1023 - throttle) * 100 / 1023;
                            println!("new throttle: {}", throttle);
                            for _ in 0..15 {
                                queue.pop_front();
                            }
                        }
                        0x03 => {
                            let packet_full = queue.make_contiguous();
                            let packet_full = &packet_full[..15];

                            speed = ((packet_full[5] as i32) << 24)
                                | ((packet_full[6] as i32) << 16)
                                | ((packet_full[7] as i32) << 8)
                                | (packet_full[8] as i32);

                            for _ in 0..15 {
                                queue.pop_front();
                            }
                        }
                        _ => println!("Packet tossed yo: {:#04X?}", packet_id),
                    }
                }
            }

            // Update UI on the event loop
            let _ = ui_handle.upgrade_in_event_loop(move |window| {
                window.set_speed(speed as f32);
                window.set_leftBlinkerOn(left_on);
                window.set_rightBlinkerOn(right_on);
                window.set_voltage(voltage);
                window.set_throttle(throttle as i32);
                window.set_tempBMS(temp_bms);
                window.set_tempMotor(temp_motor);
                window.set_tempController(temp_controller);
            });
            // thread::sleep(Duration::from_secs(1));
        }
    });

    window.run();
    Ok(())
}

fn read_pedal_packet(data: &[u8]) {
    let baseline: u16 = ((data[0] as u16) << 8) | (data[1] as u16);
    let pedal: u16 = ((data[2] as u16) << 8) | (data[3] as u16);

    println!("baseline: {} pedal: {}", baseline, pedal);
}
