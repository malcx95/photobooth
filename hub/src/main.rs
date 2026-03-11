use gphoto2::Context;
use gphoto2::camera::Camera;
use gphoto2::widget::Widget;
use serialport::{SerialPort, SerialPortInfo};
use core::panic;
use std::f32::consts::PI;
use std::io::Cursor;
use std::path::Path;
use std::time::{self, Duration, Instant};
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

const ACCEPT_BUTTON_PIN: u8 = 6;
const TAKE_PHOTO_BUTTON_PIN: u8 = 5;
const REJECT_BUTTON_PIN: u8 = 4;

enum TXMessage {
    Connect,
}

enum RXMessage {
    Nothing,
    ButtonPress(u8)
}

enum ButtonPress {
    TakePhoto,
    Accept,
    Reject
}

enum ProgramState {
    Preview,
    Countdown,
    Capturing,
    FetchingImage,
    Review
}

#[macroquad::main("photobooth-hub")]
async fn main() -> Result<(), String>  {
    let context = Context::new().expect("Failed to create a context!");
    let camera = context.autodetect_camera().wait()
        .expect("Found no camera!");
    let config = camera.config();
    println!("{:#?}", config.wait());

    let mut port = find_port();

    let mut state = ProgramState::Preview;

    let mut countdown_start = Instant::now();

    loop {
        let button_press = read_buttons(&mut port);

        match state {
            ProgramState::Preview => {
                let (height, width) = preview_camera(&camera, &context);
                draw_buttons(height, width);

                match button_press {
                    Some(ButtonPress::TakePhoto) => {
                        state = ProgramState::Countdown;
                        countdown_start = Instant::now();
                    }
                    _ => {},
                }
            },
            ProgramState::Countdown => {
                let (height, width) = preview_camera(&camera, &context);
                let secs_left = 3.0 - countdown_start.elapsed().as_secs_f32();
                if secs_left <= 0.0 {
                    state = ProgramState::Capturing;
                    capture_image(&camera);
                }
                else {
                    draw_countdown(secs_left, height, width);
                }
            },
            ProgramState::Capturing => {
                state = ProgramState::FetchingImage;
                // match capture_task {
                //     None => {},
                //     Some(task) => {
                //     }
                // }
            },
            ProgramState::FetchingImage => {
                state = ProgramState::Review;
            },
            ProgramState::Review => {
                // TODO review photo
                state = ProgramState::Preview;
            },
        }
        next_frame().await;
    }
}


fn read_buttons(port: &mut Box<dyn SerialPort>) -> Option<ButtonPress> {
    let press_opt = match read_serial(port) {
        RXMessage::Nothing => None,
        RXMessage::ButtonPress(button) => {
            if button == ACCEPT_BUTTON_PIN {
                Some(ButtonPress::Accept)
            } else if button == TAKE_PHOTO_BUTTON_PIN {
                Some(ButtonPress::TakePhoto)
            }
            else {
                Some(ButtonPress::Reject)
            }
        }
    };

    match press_opt {
        Some(button_press) => Some(button_press),
        None => {
            if is_key_down(KeyCode::T) {
                Some(ButtonPress::TakePhoto)
            } else if is_key_down(KeyCode::A) {
                Some(ButtonPress::Accept)
            } else if is_key_down(KeyCode::R) {
                Some(ButtonPress::Reject)
            } else {
                None
            }
        }
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
        match port.read_exact(serial_buf.as_mut_slice()) {
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


fn draw_countdown(count: f32, image_height: f32, image_width: f32) {
    let count_digit = count.ceil() as i32;
    let digit_str = format!("{}", count_digit);
    // let font_size = 160.0 * (-(count * 2.0 * PI).sin() + 2.0) / 2.0;
    let font_size = 260.0 * (count % 1.0 + 0.3);

    let center = get_text_center(&digit_str, Option::None, font_size as u16, 1.0, 0.0);
    draw_text(&digit_str, image_width / 2.0 - center.x / 2.0, image_height / 2.0 - center.y / 2.0, font_size, YELLOW);
}


fn draw_buttons(image_height: f32, image_width: f32) {
    let button_x = image_width + 100.0;
    let label_x = button_x + 60.0;
    let button_y = 100.0;
    let button_y_separation = 100.0;
    let font_size = 40.0;

    let buttons = [
        ("Capture", WHITE),
        ("Accept", GREEN),
        ("Reject", RED),
    ];

    for (i, (text, color)) in buttons.into_iter().enumerate() {
        draw_circle(button_x, button_y + (i as f32) * button_y_separation, 40.0, color);
        draw_text(text, label_x, button_y + (i as f32) * button_y_separation + font_size / 4.0, font_size, WHITE);
    }
}


fn capture_image(camera: &Camera) {
    let file = camera.capture_image().wait().unwrap();
    let _ = camera
        .fs()
        .download_to(&file.folder(), &file.name(), Path::new("/tmp/image.jpg"))
        .wait();
}

fn download_image(camera: &Camera) {

}


fn preview_camera(camera: &Camera, camera_context: &Context) -> (f32, f32) {
    let preview = camera.capture_preview().wait().unwrap();
    let data = preview.get_data(&camera_context).wait().unwrap();

    let decoded = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg).decode().unwrap();
    let width = decoded.width() as u16;
    let height = decoded.height() as u16;
    let converted = decoded.into_rgba8();
    let texture = Texture2D::from_rgba8(width, height, &converted);

    draw_texture(&texture, 0., 0., WHITE);

    (height as f32, width as f32)
}
