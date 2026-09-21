mod camera;
mod timer;
mod imagescore;
mod ui;
mod button;
mod serial;

use core::panic;
use gphoto2::file::CameraFilePath;
use image::EncodableLayout;
use image::buffer::ConvertBuffer;
use image::{DynamicImage, ImageEncoder, ImageReader, RgbaImage};
use macroquad::prelude::*;
use serialport::{SerialPort, SerialPortInfo};
use std::fmt::write;
use std::fs::File;
use std::io::Cursor;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc;
use std::{env, thread};
use std::time::{self, Duration, Instant};

use crate::camera::{CameraCommand, ImageMessage};
use crate::imagescore::{ScoredImage, load_scores, save_scores, score};

const IMAGE_SAVE_DIRECTORY: &str = "/home/malcolm/photoboothimages";

const IMAGE_WIDTH: f32 = 1024.0;
const IMAGE_HEIGHT: f32 = 680.0;

const LED_STATE_STANDBY: u8 = 1;
const LED_STATE_COUNTDOWN_1: u8 = 2;
const LED_STATE_COUNTDOWN_2: u8 = 3;
const LED_STATE_COUNTDOWN_3: u8 = 4;
const LED_STATE_CAPTURING: u8 = 5;

enum ProgramState {
    Preview,
    Countdown,
    Capturing,
    FetchingImage,
    Review,
}


#[macroquad::main("photobooth-hub")]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let ignore_controller = args.len() > 1 && args[1] == "-f";

    let (image_tx, image_rx) = mpsc::channel::<camera::ImageMessage>();
    let (camera_tx, camera_rx) = mpsc::channel::<camera::CameraCommand>();

    thread::spawn(move || {
        camera::camera_loop(image_tx, camera_rx);
    });

    let mut port = serial::find_port(ignore_controller);

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
    serial::send_serial_message(&mut port, serial::TXMessage::SetLightState(LED_STATE_STANDBY));

    let mut countdown_start = Instant::now();

    camera_tx.send(CameraCommand::CapturePreview).unwrap();
    let mut curr_preview_image = RgbaImage::new(1000, 1000);
    let mut last_captured_image = RgbaImage::new(1000, 1000);
    let mut preview_texture = ui::DisplayTexture::new(&curr_preview_image);
    let mut captured_texture = ui::DisplayTexture::new(&last_captured_image);
    let mut last_captured_path: CameraFilePath;

    let mut buttons_timer = timer::Timer::new(100);
    let mut buttons = button::init_buttons();

    let mut ledstrip_timer = timer::Timer::new(150);

    let mut scores = load_scores();

    loop {
        let button_press = button::read_buttons(&mut port);
        let countdown_secs_left = 3.0 - countdown_start.elapsed().as_secs_f32();
        update_enabled_buttons(&mut buttons, &state);

        if buttons_timer.triggered() {
            send_buttons_enabled_message(&mut port, &mut buttons);
        }

        if ledstrip_timer.triggered() {
            send_ledstrip_message(&mut port, &state, countdown_secs_left);
        }

        match state {
            ProgramState::Preview => {
                let preview_response = image_rx.recv_timeout(Duration::from_millis(100));
                match preview_response {
                    Ok(ImageMessage::ImagePreview(image)) => {
                        curr_preview_image = image;
                        preview_texture.update(&curr_preview_image);
                    }
                    _ => {}
                };

                match button_press {
                    Some(button::ButtonPress::TakePhoto) => {
                        state = ProgramState::Countdown;
                        countdown_start = Instant::now();
                    }
                    Some(button::ButtonPress::CycleEffect) => {
                        camera_tx.send(CameraCommand::CycleEffect).unwrap();
                    }
                    _ => {
                        camera_tx.send(CameraCommand::CapturePreview).unwrap();
                    }
                }

                ui::draw_image(&preview_texture.texture, IMAGE_HEIGHT, IMAGE_WIDTH);
                ui::draw_buttons(IMAGE_HEIGHT, IMAGE_WIDTH, &buttons);
            }
            ProgramState::Countdown => {
                camera_tx.send(CameraCommand::CapturePreview).unwrap();
                let preview_response = image_rx.recv_timeout(Duration::from_millis(100));
                match preview_response {
                    Ok(ImageMessage::ImagePreview(image)) => {
                        curr_preview_image = image;
                        preview_texture.update(&curr_preview_image);
                    }
                    _ => {}
                };
                ui::draw_image(&preview_texture.texture, IMAGE_HEIGHT, IMAGE_WIDTH);
                if countdown_secs_left <= 0.0 {
                    state = ProgramState::Capturing;
                    camera_tx.send(CameraCommand::CaptureImage).unwrap();
                } else {
                    ui::draw_countdown(countdown_secs_left, IMAGE_HEIGHT, IMAGE_WIDTH);
                }
            }
            ProgramState::Capturing => {
                let capture_response = image_rx.try_recv();
                match capture_response {
                    Ok(ImageMessage::Captured(path)) => {
                        camera_tx.send(CameraCommand::FetchImage(path)).unwrap();
                        state = ProgramState::FetchingImage;
                    }
                    Ok(ImageMessage::CaptureFailed) => {
                        println!("Failed to capture image");
                        state = ProgramState::Preview;
                    }
                    _ => {}
                };
                ui::draw_image(&preview_texture.texture, IMAGE_HEIGHT, IMAGE_WIDTH);
                ui::draw_cheese_frame(IMAGE_HEIGHT, IMAGE_WIDTH);
            }
            ProgramState::FetchingImage => {
                let fetch_response = image_rx.try_recv();
                match fetch_response {
                    Ok(ImageMessage::FetchedImage(image, path)) => {
                        println!("Got image");
                        last_captured_image = image;
                        captured_texture.update(&last_captured_image);
                        last_captured_path = path;
                        state = ProgramState::Review;
                    }
                    Ok(ImageMessage::FetchFailed) => {
                        println!("Failed to fetch image");
                        state = ProgramState::Preview;
                    }
                    _ => {}
                };
                ui::draw_image(&preview_texture.texture, IMAGE_HEIGHT, IMAGE_WIDTH);
                ui::draw_loading_frame(IMAGE_HEIGHT, IMAGE_WIDTH);
            }
            ProgramState::Review => {
                ui::draw_image(&captured_texture.texture, IMAGE_HEIGHT, IMAGE_WIDTH);
                ui::draw_buttons(IMAGE_HEIGHT, IMAGE_WIDTH, &buttons);
                ui::draw_review_frame(IMAGE_HEIGHT, IMAGE_WIDTH);

                match button_press {
                    Some(button::ButtonPress::Accept) => {
                        state = ProgramState::Preview;
                        save_image(&last_captured_image, &mut scores);
                    }
                    Some(button::ButtonPress::Reject) => {
                        state = ProgramState::Preview;
                    }
                    _ => {}
                }
            }
        }
        next_frame().await;
    }
}

fn send_buttons_enabled_message(port: &mut Box<dyn SerialPort>, buttons: &mut Vec<button::Button>) {
    let mut bitmask: u8 = 0;
    for (i, button) in buttons.iter().enumerate() {
        bitmask |= (button.enabled as u8) << i;
    }
    let _ = serial::send_serial_message(port, serial::TXMessage::SetEnabledButtons(bitmask));
}

fn send_ledstrip_message(port: &mut Box<dyn SerialPort>, state: &ProgramState, countdown: f32) {
    let light_state = match state {
        ProgramState::Preview => LED_STATE_STANDBY,
        ProgramState::Countdown => {
            if countdown > 2.0 {
                LED_STATE_COUNTDOWN_3
            }
            else if countdown > 1.0 {
                LED_STATE_COUNTDOWN_2
            }
            else {
                LED_STATE_COUNTDOWN_1
            }
        }
        ProgramState::Capturing => LED_STATE_CAPTURING,
        ProgramState::FetchingImage => LED_STATE_STANDBY,
        ProgramState::Review => LED_STATE_STANDBY,
    };
    let _ = serial::send_serial_message(port, serial::TXMessage::SetLightState(light_state));
}

fn update_enabled_buttons(buttons: &mut Vec<button::Button>, state: &ProgramState) {
    let enabled_pins = match state {
        ProgramState::Preview => vec![button::BIG_WHITE_BUTTON, button::BLUE_BUTTON],
        ProgramState::Countdown => vec![button::RED_BUTTON],
        ProgramState::Capturing => vec![],
        ProgramState::FetchingImage => vec![],
        ProgramState::Review => vec![button::RED_BUTTON, button::GREEN_BUTTON],
    };

    for button in buttons.iter_mut() {
        let should_be_enabled = enabled_pins.contains(&button.pin);
        button.enabled = should_be_enabled;
    }
}

fn save_image(image: &RgbaImage, scores: &mut Vec<ScoredImage>) {
    if let Err(error) = std::fs::create_dir_all(IMAGE_SAVE_DIRECTORY) {
        eprintln!("Failed to create image directory {IMAGE_SAVE_DIRECTORY}: {error}");
        return;
    }

    let mut image_number = 1;
    let image_path = loop {
        let path = Path::new(IMAGE_SAVE_DIRECTORY).join(format!("{image_number:04}.jpg"));
        if !path.exists() {
            break path;
        }
        image_number += 1;
    };

    let jpeg_image = DynamicImage::ImageRgba8(image.clone()).to_rgb8();
    if let Err(error) = jpeg_image.save_with_format(&image_path, image::ImageFormat::Jpeg) {
        eprintln!("Failed to save image to {}: {error}", image_path.display());
    }

    let score = score(image);
    scores.push(ScoredImage { path: image_path.to_str().unwrap().to_string(), score: score });

    save_scores(scores);
}



// fn capture_image(camera: &Camera) {
//     let file = camera.capture_image().wait().unwrap();
//     let _ = camera
//         .fs()
//         .download_to(&file.folder(), &file.name(), Path::new("/tmp/image.jpg"))
//         .wait();
// }
