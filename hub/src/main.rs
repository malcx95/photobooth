mod camera;

use gphoto2::Context;
use gphoto2::camera::Camera;
use gphoto2::file::CameraFilePath;
use gphoto2::widget::Widget;
use serialport::{SerialPort, SerialPortInfo};
use core::panic;
use std::f32::consts::PI;
use std::io::Cursor;
use std::path::Path;
use std::thread;
use std::time::{self, Duration, Instant};
use macroquad::prelude::*;
use image::{DynamicImage, ImageReader, RgbaImage};
use std::sync::mpsc;

use crate::camera::{CameraCommand, ImageMessage};

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

const IMAGE_WIDTH: f32 = 1024.0;
const IMAGE_HEIGHT: f32 = 680.0;

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
    let (image_tx, image_rx) = mpsc::channel::<camera::ImageMessage>();
    let (camera_tx, camera_rx) = mpsc::channel::<camera::CameraCommand>();

    thread::spawn(move || {
        camera::camera_loop(image_tx, camera_rx);
    });

    let mut port = find_port();


    println!("Connecting to camera...");
    match image_rx.recv().unwrap() {
        ImageMessage::FailedToStartCamera(e) => {
            panic!("Failed to start camera: {e}");
        }
        ImageMessage::CameraStarted => {
            println!("Successfully started camera");
        }
        _ => {}
    }

    let mut state = ProgramState::Preview;

    let mut countdown_start = Instant::now();

    camera_tx.send(CameraCommand::CapturePreview).unwrap();
    let mut curr_preview_image = RgbaImage::new(1000, 1000);
    let mut last_captured_image = RgbaImage::new(1000, 1000);
    let mut last_captured_path: CameraFilePath;

    loop {
        let button_press = read_buttons(&mut port);

        match state {
            ProgramState::Preview => {
                let preview_response = image_rx.recv_timeout(Duration::from_millis(300));
                match preview_response {
                    Ok(ImageMessage::ImagePreview(image)) => {
                        curr_preview_image = image;
                    },
                    _ => { }
                };

                match button_press {
                    Some(ButtonPress::TakePhoto) => {
                        state = ProgramState::Countdown;
                        countdown_start = Instant::now();
                    }
                    _ => {
                        camera_tx.send(CameraCommand::CapturePreview).unwrap();
                    },
                }

                draw_preview(&curr_preview_image, IMAGE_HEIGHT, IMAGE_WIDTH);
                draw_buttons(IMAGE_HEIGHT, IMAGE_WIDTH, &state);
            },
            ProgramState::Countdown => {
                draw_preview(&curr_preview_image, IMAGE_HEIGHT, IMAGE_WIDTH);
                let secs_left = 3.0 - countdown_start.elapsed().as_secs_f32();
                if secs_left <= 0.0 {
                    state = ProgramState::Capturing;
                    camera_tx.send(CameraCommand::CaptureImage).unwrap();
                }
                else {
                    draw_countdown(secs_left, IMAGE_HEIGHT, IMAGE_WIDTH);
                }
            },
            ProgramState::Capturing => {
                let capture_response = image_rx.try_recv();
                match capture_response {
                    Ok(ImageMessage::Captured(path)) => {
                        camera_tx.send(CameraCommand::FetchImage(path)).unwrap();
                        state = ProgramState::FetchingImage;
                    },
                    Ok(ImageMessage::CaptureFailed) => {
                        println!("Failed to capture image");
                        state = ProgramState::Preview;
                    },
                    _ => { }
                };
                draw_preview(&curr_preview_image, IMAGE_HEIGHT, IMAGE_WIDTH);
                draw_cheese_frame(IMAGE_HEIGHT, IMAGE_WIDTH);
            },
            ProgramState::FetchingImage => {
                println!("Waiting for image...");
                let fetch_response = image_rx.try_recv();
                println!("Got image");
                match fetch_response {
                    Ok(ImageMessage::FetchedImage(image, path)) => {
                        println!("Got image");
                        last_captured_image = image;
                        last_captured_path = path;
                        state = ProgramState::Review;
                    },
                    Ok(ImageMessage::FetchFailed) => {
                        println!("Failed to capture image");
                        state = ProgramState::Preview;
                    },
                    _ => { }
                };
                draw_preview(&curr_preview_image, IMAGE_HEIGHT, IMAGE_WIDTH);
                draw_loading_frame(IMAGE_HEIGHT, IMAGE_WIDTH);
            },
            ProgramState::Review => {
                // state = ProgramState::Preview;
                draw_captured_image(&last_captured_image, IMAGE_HEIGHT, IMAGE_WIDTH);
                draw_buttons(IMAGE_HEIGHT, IMAGE_WIDTH, &state);
                draw_review_frame(IMAGE_HEIGHT, IMAGE_WIDTH);
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


fn draw_buttons(image_height: f32, image_width: f32, program_state: &ProgramState) {
    let button_x = image_width + 100.0;
    let label_x = button_x + 60.0;
    let button_y = 100.0;
    let button_y_separation = 100.0;
    let font_size = 40.0;

    let mut buttons = match program_state {
        ProgramState::Review => vec![
            ("Accept", GREEN),
            ("Reject", RED),
        ],
        _ => vec![
            ("Capture", WHITE),
            ("Accept", GREEN),
            ("Reject", RED),
        ]
    };

    for (i, (text, color)) in buttons.into_iter().enumerate() {
        draw_circle(button_x, button_y + (i as f32) * button_y_separation, 40.0, color);
        draw_text(text, label_x, button_y + (i as f32) * button_y_separation + font_size / 4.0, font_size, WHITE);
    }
}


// fn capture_image(camera: &Camera) {
//     let file = camera.capture_image().wait().unwrap();
//     let _ = camera
//         .fs()
//         .download_to(&file.folder(), &file.name(), Path::new("/tmp/image.jpg"))
//         .wait();
// }


fn draw_captured_image(image: &RgbaImage, target_height: f32, target_width: f32) -> (f32, f32) {
    let width = image.width() as f32;
    let height = image.height() as f32;
    let texture = Texture2D::from_rgba8(width as u16, height as u16, &image);

    draw_texture_ex(&texture, 0., 0., WHITE, DrawTextureParams { dest_size: Some(vec2(target_width, target_height)), ..Default::default() });

    (height as f32, width as f32)
}


fn draw_preview(image: &RgbaImage, target_height: f32, target_width: f32) {
    let width = image.width() as u16;
    let height = image.height() as u16;
    let texture = Texture2D::from_rgba8(width, height, &image);

    draw_texture_ex(&texture, 0., 0., WHITE, DrawTextureParams { dest_size: Some(vec2(target_width, target_height)), ..Default::default() });
}

fn draw_cheese_frame(image_height: f32, image_width: f32) {
    let font_size = 400.0;
    let center = get_text_center("CHEESE!", Option::None, font_size as u16, 1.0, 0.0);
    draw_text("CHEESE!", image_width / 2.0 - center.x / 2.0, image_height / 2.0 - center.y / 2.0, font_size, YELLOW);
}

fn draw_loading_frame(image_height: f32, image_width: f32) {
    let font_size = 400.0;

    let center = get_text_center("Loading...", Option::None, font_size as u16, 1.0, 0.0);
    draw_text("Loading...", image_width / 2.0 - center.x / 2.0, image_height / 2.0 - center.y / 2.0, font_size, YELLOW);
}

fn draw_review_frame(image_height: f32, image_width: f32) {
    // let font_size = 400.0;

    // let center = get_text_center("Loading...", Option::None, font_size as u16, 1.0, 0.0);
    // draw_text("Loading...", image_width / 2.0 - center.x / 2.0, image_height / 2.0 - center.y / 2.0, font_size, YELLOW);
}
