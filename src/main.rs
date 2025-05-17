// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod serial;

use crate::serial::packets::{PedalPacket, VelocityPacket};
use serial::packets::{LightsPacket, MotorStatusPacket, MotorTempaturePacket};
use serialport::SerialPort;
use slint::{ComponentHandle, SharedString, ToSharedString};
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::thread;
use std::time::Duration;
slint::include_modules!();

const CAN_PACKET_ID: u8 = 0x01;
const LIGHTS_PACKET_ID: u8 = 0x02;
const VELOCITY_PACKET_ID: u8 = 0x03;
const PEDAL_PACKET_ID: u8 = 0x04;
const MOTOR_TEMPATURE_PACKET_ID: u8 = 0x05;
const MOTOR_STATUS_PACKET_ID: u8 = 0x06;

fn make_connection() -> Box<dyn SerialPort> {
    loop {
        let serial_port_info = serialport::available_ports().unwrap().into_iter().next();

        if serial_port_info.is_some() {
            let serial_port_info = serial_port_info.unwrap();

            let port = serialport::new(serial_port_info.port_name, 9600)
                .timeout(Duration::from_millis(50))
                .open();

            if port.is_ok() {
                return port.unwrap();
            }
        }

        println!("port not found, trying again");
        thread::sleep(Duration::from_millis(50));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = Dashboard::new()?; // From the Slint DSL 

    let mut port = make_connection();
    // Circular buff implmentation would be nice at some point
    let mut queue: VecDeque<u8> = VecDeque::new();
    let mut serial_buf: Vec<u8> = vec![0; 32];
    let ui_handle = window.as_weak();
    thread::spawn(move || {
        let mut speed = 0.0;
        let mut throttle = 0;
        let mut headlights_on = false;
        let mut left_on = false;
        let mut right_on = false;
        let mut temp_motor = 0.0;
        let mut limit_code: u16 = 0;
        let mut error_out: SharedString = "".to_shared_string();

        loop {
            let temp_bms = 0;
            //let mut error_out: SharedString = "".to_shared_string();

            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    for item in &serial_buf[..t] {
                        queue.push_back(item.to_owned());
                    }
                }
                Err(e) => {
                    if e.kind() == ErrorKind::BrokenPipe {
                        port = make_connection();
                        println!("Recovered port!");
                    }
                    println!("Random Error: {}", e);
                }
            }

            while queue.len() > 16 {
                let packet_byte = queue.pop_front();

                if let Some(packet_id) = packet_byte {
                    match packet_id {
                        PEDAL_PACKET_ID => {
                            if let Ok(pedal_packet) = PedalPacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ]) {
                                throttle = pedal_packet.get_throttle_percentage();

                                if (pedal_packet.baseline_value
                                    < (pedal_packet.pedal_value.clamp(50, 1023) - 50))
                                    || (pedal_packet.pedal_value < 475)
                                {
                                    error_out =
                                        "Warning: Recent pedal fault detected!".to_shared_string();
                                }

                                queue.drain(0..15);
                            }
                        }
                        VELOCITY_PACKET_ID => {
                            if let Ok(velocity_packet) = VelocityPacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ]) {
                                speed = velocity_packet.to_mph().trunc();

                                queue.drain(0..15);
                            }
                        }
                        LIGHTS_PACKET_ID => {
                            if let Ok(lights_packet) = LightsPacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ]) {
                                left_on = lights_packet.left_blinkers;
                                right_on = lights_packet.right_blinkers;
                                headlights_on = lights_packet.headlights;

                                queue.drain(0..15);
                            }
                        }
                        MOTOR_TEMPATURE_PACKET_ID => {
                            if let Ok(motor_temp_packet) = MotorTempaturePacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ]) {
                                temp_motor = motor_temp_packet.motor_temp;
                            }
                        }
                        MOTOR_STATUS_PACKET_ID => {
                            if let Ok(motor_status_packet) = MotorStatusPacket::from_bytes(&[
                                queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                                queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                                queue[12], queue[13], queue[14],
                            ]) {
                                limit_code = motor_status_packet.limit_flags;
                                if motor_status_packet.error_flags != 0 {
                                    error_out = format!(
                                        "Warning: Error code {}",
                                        motor_status_packet.error_flags
                                    )
                                    .to_shared_string();
                                }
                            }
                        }
                        _ => print!(""), //println!("Packet tossed yo: {:#04X?}", packet_id),
                    }
                }
            }

            let warning = error_out.clone();

            // Update UI on the event loop
            let _ = ui_handle.upgrade_in_event_loop(move |window| {
                window.set_speed(speed as f32);
                window.set_leftBlinkerOn(left_on);
                window.set_rightBlinkerOn(right_on);
                window.set_throttle(throttle as i32);
                window.set_tempBMS(temp_bms);
                window.set_tempMotor(temp_motor);
                window.set_headlightsOn(headlights_on);
                window.set_limitId(limit_code as i32);
                window.set_errorOut(warning);
            });
        }
    });

    _ = window.run();
    Ok(())
}
