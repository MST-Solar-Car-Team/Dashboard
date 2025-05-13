// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod serial;

use crate::serial::packets::{PedalPacket, SpeedPacket};
use slint::ComponentHandle;
use std::collections::VecDeque;
use std::thread;
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
        let mut speed = 0.0;
        let mut throttle = 0;
        loop {
            let left_on = false;
            let right_on = true;
            let voltage = 52.3;
            let temp_bms = 32;
            let temp_motor = 45;
            let temp_controller = 38;

            match ports.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    for item in &serial_buf[..t] {
                        queue.push_back(item.to_owned());
                    }
                }
                Err(_) => print!(""),
            }

            while queue.len() > 16 {
                let packet_byte = queue.pop_front();

                if let Some(packet_id) = packet_byte {
                    match packet_id {
                        0x13 => {
                            let pedal_packet = PedalPacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ])
                            .unwrap();

                            throttle = pedal_packet.get_throttle_percentage();

                            queue.drain(0..15);
                        }
                        0x03 => {
                            let pedal_packet = SpeedPacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ])
                            .unwrap();

                            speed = pedal_packet.to_mph().trunc();

                            queue.drain(0..15);
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
        }
    });

    _ = window.run();
    Ok(())
}
