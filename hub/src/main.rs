use gphoto2::Context;
use gphoto2::camera::Camera;
use serialport::{SerialPort, SerialPortInfo};
use core::panic;
use std::io::Cursor;
use std::time::{self, Duration};
use macroquad::prelude::*;
use image::ImageReader;

const CONTROLLER_START_WORD: u8 = 0xEE;
const HUB_START_WORD: u8 = 0x41;

const CONTROLLER_BAUD_RATE: u32 = 9600;

const CONNECT_MSG_TYPE: u8 = 0xDE;
const CONNECT_ACK_MSG_TYPE: u8 = 0xAE;
const BUTTON_PRESS_MSG_TYPE: u8 = 0xBB;

const TURN_ON_LIGHT_MSG_TYPE: u8 = 0x4A;

const MSG_TIMEOUT: u64 = 200;

enum TXMessage {
    Connect,
}

enum RXMessage {
    Nothing,
    ButtonPress(u8)
}

#[macroquad::main("photobooth-hub")]
async fn main() -> Result<(), String>  {
    let context = Context::new().expect("Failed to create a context!");
    let camera = context.autodetect_camera().wait().expect("Found no camera!");

    let mut port = find_port();

    loop {
        match read_serial(&mut port) {
            RXMessage::Nothing => {},
            RXMessage::ButtonPress(button) => {
                println!("Pressed button {}", button);
            }
        }
        preview_camera(&camera, &context);
        next_frame().await;
    }
}


fn read_serial(port: &mut Box<dyn SerialPort>) -> RXMessage {
    if port.bytes_to_read().unwrap() < 3 {
        return RXMessage::Nothing;
    }

    let mut serial_buf = [0; 3];
    let read_bytes = match port.read_exact(serial_buf.as_mut_slice()) {
        Ok(b) => b,
        Err(_) => {
            println!("Failed to read message");
            return RXMessage::Nothing;
        }
    };

    if serial_buf[0] == CONTROLLER_START_WORD {
        if serial_buf[1] == BUTTON_PRESS_MSG_TYPE {
            return RXMessage::ButtonPress(serial_buf[2]);
        }
    }
    RXMessage::Nothing
}


fn find_port() -> Box<dyn SerialPort> {
    let ports = serialport::available_ports().expect("No serial ports found!");
    for p in ports {
        let mut port = match serialport::new(&p.port_name, CONTROLLER_BAUD_RATE)
            .timeout(Duration::from_millis(MSG_TIMEOUT))
            .open() {
            Ok(opened) => opened,
            Err(_) => continue,
        };

        println!("Trying port {}", &p.port_name);

        let connect_msg = [1, 1, 1, HUB_START_WORD, CONNECT_MSG_TYPE, 0];
        port.write(&connect_msg).expect("Write failed!");
        println!("This port might respond");

        let mut serial_buf = [0; 3];
        let read_bytes = match port.read_exact(serial_buf.as_mut_slice()) {
            Ok(b) => b,
            Err(_) => {
                println!("This port did not respond properly");
                continue;
            }
        };

        if serial_buf[1] == CONNECT_ACK_MSG_TYPE {
            return port;
        }

    }
    panic!("Found no ports!");
}


fn preview_camera(camera: &Camera, camera_context: &Context)  {
    let preview = camera.capture_preview().wait().unwrap();
    let data = preview.get_data(&camera_context).wait().unwrap();

    let decoded = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg).decode().unwrap();
    let width = decoded.width() as u16;
    let height = decoded.height() as u16;
    let converted = decoded.into_rgba8();
    let texture = Texture2D::from_rgba8(width, height, &converted);

    draw_texture(&texture, 0., 0., WHITE);
}
