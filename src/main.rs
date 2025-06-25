// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod serial;

use crate::serial::packets::{PedalPacket, VelocityPacket};
use serial::packets::{LightsPacket, MotorStatusPacket, MotorTempaturePacket};
use serialport::{Error, SerialPort};
use slint::{ComponentHandle, SharedString, ToSharedString, SharedPixelBuffer};
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::thread;
use std::time::Duration;
slint::include_modules!();


use opencv::{core, imgproc, prelude::*, videoio, Result};

const CAN_PACKET_ID: u8 = 0x01;
const LIGHTS_PACKET_ID: u8 = 0x02;
const VELOCITY_PACKET_ID: u8 = 0x03;
const PEDAL_PACKET_ID: u8 = 0x04;
const MOTOR_TEMPATURE_PACKET_ID: u8 = 0x05;
const MOTOR_STATUS_PACKET_ID: u8 = 0x06;

fn make_connection() -> Box<dyn SerialPort> {
    loop {
        let available_ports = serialport::available_ports().unwrap().into_iter();
        
        println!("ports: {:?}",available_ports);

        for serial_port_info in available_ports {
            println!("port_name: {}",serial_port_info.port_name);

            let port = serialport::new(serial_port_info.port_name, 9600)
                .timeout(Duration::from_millis(50))
                .open();

            if port.is_ok() {
                let mut port = port.unwrap();
                let mut buf: Vec<u8> = vec![32; 0];
                
                if port.read(buf.as_mut_slice()).is_ok(){
                    println!("connection: {:?}",buf);
                    return port;
                }
            }

        }

        println!("port not found, trying again");
        thread::sleep(Duration::from_millis(50));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = Dashboard::new()?; // From the Slint DSL 
    let ui_handle = window.as_weak();
    //initialize default values for window
    let _ = ui_handle.upgrade_in_event_loop(move |window| {
        window.set_speed(0.0 as f32);
        window.set_leftBlinkerOn(false);
        window.set_rightBlinkerOn(false);
        window.set_throttle(0 as i32);
        window.set_tempBMS(0 as i32);
        window.set_tempMotor(0.0 as f32);
        window.set_headlightsOn(false);
        window.set_limitId(0 as i32);
        window.set_errorOut("".to_shared_string());
        window.set_serialConnection(false);
    });


    thread::spawn(move || {
        let mut port = make_connection();
        // Circular buff implmentation would be nice at some point
        let mut queue: VecDeque<u8> = VecDeque::new();
        let mut serial_buf: Vec<u8> = vec![0; 32];

        //Camera
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
        // let opened = videoio::VideoCapture::is_opened(&cam).unwrap();

        let mut speed = 0.0;
        let mut throttle = 0;
        let mut reversed: bool = false;
        let mut headlights_on = false;
        let mut left_on = false;
        let mut right_on = false;
        let mut temp_motor = 0.0;
        let mut limit_code: u16 = 0;
        let mut error_out: SharedString = "".to_shared_string();
        let mut has_serial_connection = true;

        loop {
            let temp_bms = 0;
            //let mut error_out: SharedString = "".to_shared_string();
            
            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    println!("Buf:{:?}",serial_buf);
                    has_serial_connection = true;
                    for item in &serial_buf[..t] {
                        queue.push_back(item.to_owned());
                    }
                }
                Err(e) => {
                    if e.kind() == ErrorKind::BrokenPipe {
                        port = make_connection();
                        println!("Recovered port!");
                    }
                    if e.kind() == ErrorKind::TimedOut{
                        has_serial_connection = false;
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

            //backup camera
            if reversed {
                if let Some(buffer) = update_frame(&mut cam) {
                    let _ = ui_handle.upgrade_in_event_loop( move |window| {
                        window.set_backupCamera(slint::Image::from_rgb8(buffer))
                    });

                }

                continue;
            }

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
                window.set_serialConnection(has_serial_connection);
            });
        }
    });

    _ = window.run();
    Ok(())
}


fn update_frame(cam:&mut videoio::VideoCapture) -> Option<SharedPixelBuffer::<slint::Rgb8Pixel>> {
    let mut frame = core::Mat::default();
    cam.read(&mut frame).unwrap();
    // if frame.size()?.width == 0 {
    //     return;
    // }

    // Convert to RGB
    let mut rgb = core::Mat::default();
    imgproc::cvt_color(&frame, &mut rgb, imgproc::COLOR_BGR2RGB, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT).unwrap();

    let size = rgb.size().unwrap();
    let width = size.width as usize;
    let height = size.height as usize;

    // Convert OpenCV Mat to SharedPixelBuffer
    let data = rgb.data_bytes().unwrap();
    // SharedPixelBuffer::<slint::Rgb8Pixel>::from(data);
    let buffer = SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
        data,
        width as u32,
        height as u32,
    );

    return  Some(buffer);
}
