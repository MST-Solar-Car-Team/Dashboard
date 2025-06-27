// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod serial;

use crate::serial::packets::{PedalPacket, VelocityPacket};
use serial::packets::{LightsPacket, MotorStatusPacket, MotorTempaturePacket};
use serialport::{Error, SerialPort};
use slint::{ComponentHandle, SharedString, ToSharedString};
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
slint::include_modules!();

const CAN_PACKET_ID: u8 = 0x01;
const LIGHTS_PACKET_ID: u8 = 0x02;
const VELOCITY_PACKET_ID: u8 = 0x03;
const PEDAL_PACKET_ID: u8 = 0x04;
const MOTOR_TEMPATURE_PACKET_ID: u8 = 0x05;
const MOTOR_STATUS_PACKET_ID: u8 = 0x06;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = Dashboard::new()?; // From the Slint DSL 
    let ui_handle = window.as_weak();

    let data_arc_window: Arc<Mutex<WindowData>> = Arc::new(Mutex::new(WindowData::default()));
    let data_arc_serial: Arc<Mutex<WindowData>> = Arc::clone(&data_arc_window);

    //Window updating thread
    thread::spawn(move || {
        loop {
            let data = data_arc_window.lock();

            if data.is_err() {
                continue;
            }
            let data = data.unwrap().clone();


            // Update UI on the event loop
            let _ = ui_handle.upgrade_in_event_loop(move |window| {
                window.set_speed(data.speed as f32);
                window.set_leftBlinkerOn(data.left_on);
                window.set_rightBlinkerOn(data.right_on);
                window.set_throttle(data.throttle as i32);
                window.set_tempBMS(data.temp_bms as i32);
                window.set_tempMotor(data.temp_motor);
                window.set_headlightsOn(data.headlights_on);
                window.set_limitId(data.limit_code as i32);
                window.set_errorOut(data.error_out);
                window.set_serialError(data.serial_error);
            });

            //don't overload the screen
            thread::sleep(Duration::from_millis(50));

        }
    });

    //serial_updating thread
    thread::spawn(move || {
        update_serial(data_arc_serial);
    });

    _ = window.run();


    Ok(())
}


#[derive(Clone, Debug, Default)]
struct WindowData {
    speed: f32, 
    throttle: i32, 
    headlights_on: bool, 
    left_on: bool, 
    right_on: bool,
    temp_bms: f32,
    temp_motor: f32,
    limit_code: u16 ,
    error_out: SharedString,
    serial_error: SharedString, 
}

fn make_connection() -> Box<dyn SerialPort + 'static> {
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

fn update_serial(data:Arc<Mutex<WindowData>>) { // port:Box<dyn SerialPort + 'static>) -> Result<(),()>{
    let mut port = make_connection();
    // Circular buff implmentation would be nice at some point
    let mut queue: VecDeque<u8> = VecDeque::new();
    let mut serial_buf: Vec<u8> = vec![0; 32];

    loop {
        
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                println!("Buf:{:?}",serial_buf);
                //updates serial working status
                if let Ok(mut mutex) = data.lock(){
                    mutex.serial_error = "None".to_shared_string();
                }
                
                for item in &serial_buf[..t] {
                    queue.push_back(item.to_owned());
                }
            }
            Err(e) => {

                //Prints error to window
                if let Ok(mut mutex) = data.lock(){
                    mutex.serial_error = e.to_shared_string();
                }

                if e.kind() == ErrorKind::BrokenPipe {
                    port = make_connection();
                    println!("Recovered port!");
                }else {
                    println!("Random Error: {}", e);
                }
                                
            }
        }

        while queue.len() > 16 {
            let packet_byte = queue.pop_front();


            if packet_byte.is_none(){
                break; //empty queue
            }
            
            let packet_id = packet_byte.unwrap();
                
            if let Ok(mut data) = data.lock(){
                match packet_id {
                    PEDAL_PACKET_ID => {
                        if let Ok(pedal_packet) = PedalPacket::from_bytes(&[
                            queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                            queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                            queue[12], queue[13], queue[14],
                        ]) {
                            data.throttle = pedal_packet.get_throttle_percentage() as i32;

                            if (pedal_packet.baseline_value
                                < (pedal_packet.pedal_value.clamp(50, 1023) - 50))
                                || (pedal_packet.pedal_value < 475)
                            {
                                data.error_out =
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
                            data.speed = velocity_packet.to_mph().trunc();

                            queue.drain(0..15);
                        }
                    }
                    LIGHTS_PACKET_ID => {
                        if let Ok(lights_packet) = LightsPacket::from_bytes(&[
                            queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                            queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                            queue[12], queue[13], queue[14],
                        ]) {
                            data.left_on = lights_packet.left_blinkers;
                            data.right_on = lights_packet.right_blinkers;
                            data.headlights_on = lights_packet.headlights;

                            queue.drain(0..15);
                        }
                    }
                    MOTOR_TEMPATURE_PACKET_ID => {
                        if let Ok(motor_temp_packet) = MotorTempaturePacket::from_bytes(&[
                            queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                            queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                            queue[12], queue[13], queue[14],
                        ]) {
                            data.temp_motor = motor_temp_packet.motor_temp;
                        }
                    }
                    MOTOR_STATUS_PACKET_ID => {
                        if let Ok(motor_status_packet) = MotorStatusPacket::from_bytes(&[
                            queue[0], queue[1], queue[2], queue[3], queue[4], queue[5],
                            queue[6], queue[7], queue[8], queue[9], queue[10], queue[11],
                            queue[12], queue[13], queue[14],
                        ]) {
                            data.limit_code = motor_status_packet.limit_flags.clone();
                            if motor_status_packet.error_flags != 0 {
                                data.error_out = format!(
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

    }

}