use core::panic;
use gphoto2::Context;
use gphoto2::{
    camera::{Camera, CameraEvent},
    file::CameraFilePath,
    widget::RadioWidget,
};
use image::{ImageReader, RgbaImage};
use macroquad::prelude::*;
use std::{
    io::Cursor,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub enum ImageMessage {
    CameraStarted,
    FailedToStartCamera(String),
    ImagePreview(RgbaImage),
    ImagePreviewFailed,
    Captured(CameraFilePath),
    CaptureFailed,
    FetchedImage(RgbaImage, CameraFilePath),
    FetchFailed,
}

pub enum CameraCommand {
    CaptureImage,
    CapturePreview,
    CycleEffect,
    FetchImage(CameraFilePath),
    DeleteImage(CameraFilePath),
}

pub fn camera_loop(image_tx: Sender<ImageMessage>, camera_rx: Receiver<CameraCommand>) {
    let context = Context::new().expect("Failed to create a context!");
    let camera: Camera;
    match context.autodetect_camera().wait() {
        Ok(cam) => {
            camera = cam;
        }
        Err(e) => {
            image_tx
                .send(ImageMessage::FailedToStartCamera(e.to_string()))
                .unwrap();
            return;
        }
    }

    let effects = vec![
        "32768",
        "32769",
        "32770",
        "32771",
        "32772",
        "32773",
        "32784",
        "32800",
        "32801",
        "32816",
        "32832",
        "32848",
        "32849",
        "32850",
        "32851",
        "32864",
        "32912",
    ];

    let mut effect_index = 0;

    let image_quality = get_camera_setting(&camera, "imagequality");
    let picture_effect = get_camera_setting(&camera, "d21b");

    set_camera_setting(&camera, &image_quality, "Fine");
    set_camera_setting(&camera, &picture_effect, effects[effect_index]);

    let picture_effect = camera
        .config_key::<RadioWidget>("d21b")
        .wait()
        .unwrap();
    picture_effect.set_choice(effects[effect_index]).unwrap();
    camera.set_config(&picture_effect).wait().unwrap();

    // let shutter_speed = camera.config_key::<RadioWidget>("shutterspeed").wait().unwrap();;
    // shutter_speed.set_choice("1/10").unwrap();
    // camera.set_config(&shutter_speed).wait().unwrap();

    let config = camera.config().wait().unwrap();
    println!("{:#?}", config);
    image_tx.send(ImageMessage::CameraStarted).unwrap();

    loop {
        let msg = camera_rx.recv().ok();

        match msg {
            Some(CameraCommand::CapturePreview) => {
                let msg = preview_camera(&camera, &context)
                    .map_or(ImageMessage::ImagePreviewFailed, |image| {
                        ImageMessage::ImagePreview(image)
                    });
                image_tx.send(msg).unwrap();
            }
            Some(CameraCommand::CaptureImage) => {
                let msg = capture_image(&camera).map_or(ImageMessage::CaptureFailed, |path| {
                    ImageMessage::Captured(path)
                });
                image_tx.send(msg).unwrap();
            }
            Some(CameraCommand::CycleEffect) => {
                effect_index = (effect_index + 1) % effects.len();
                set_camera_setting(&camera, &picture_effect, effects[effect_index]); // picture effect
            }
            Some(CameraCommand::FetchImage(path)) => {
                sleep(Duration::from_secs(1));
                let msg = fetch_image(&camera, &path).map_or(ImageMessage::FetchFailed, |image| {
                    ImageMessage::FetchedImage(image, path)
                });
                println!("Sending image");
                image_tx.send(msg).unwrap();
                println!("Sent image");
            }
            Some(CameraCommand::DeleteImage(path)) => {
                camera
                    .fs()
                    .delete_file(&path.folder(), &path.name())
                    .wait()
                    .ok();
            }
            _ => {}
        }
    }
}

fn preview_camera(camera: &Camera, camera_context: &Context) -> Option<RgbaImage> {
    let preview = camera.capture_preview().wait().ok()?;
    let data = preview.get_data(&camera_context).wait().ok()?;

    let decoded = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg)
        .decode()
        .ok()?;
    let converted = decoded.clone().into_rgba8();

    Some(converted)
}

fn capture_image(camera: &Camera) -> Option<CameraFilePath> {
    // Some camera drivers return the previous image from `gp_camera_capture`.
    // Clear any old notifications, then wait for the NewFile event caused by
    // this trigger so the downloaded path is tied to this shutter press.
    drain_camera_events(camera)?;
    camera.trigger_capture().wait().ok()?;

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let remaining = deadline.checked_duration_since(Instant::now())?;
        match camera
            .wait_event(remaining.min(Duration::from_secs(1)))
            .wait()
            .ok()?
        {
            CameraEvent::NewFile(path) => return Some(path),
            CameraEvent::Timeout => {}
            _ => {}
        }
    }
}

fn drain_camera_events(camera: &Camera) -> Option<()> {
    loop {
        match camera.wait_event(Duration::ZERO).wait().ok()? {
            CameraEvent::Timeout => return Some(()),
            _ => {}
        }
    }
}

fn fetch_image(camera: &Camera, file: &CameraFilePath) -> Option<RgbaImage> {
    let download_path = capture_download_path();
    let mut camera_file = None;

    for _ in 0..10 {
        // try five times
        match camera
            .fs()
            .download_to(&file.folder(), &file.name(), &download_path)
            .wait()
        {
            Ok(f) => {
                camera_file = Some(f);
                break;
            }
            _ => {
                println!("Trying again...");
                sleep(Duration::from_secs(1));
            }
        }
    }

    match camera_file {
        None => {
            return None;
        }
        _ => {}
    }

    let decoded = ImageReader::open(&download_path).ok()?.decode().ok()?;
    let converted = decoded.into_rgba8();
    drop(camera_file);
    if let Err(error) = std::fs::remove_file(&download_path) {
        eprintln!(
            "Failed to remove temporary image {}: {error}",
            download_path.display()
        );
    }

    println!("Yay! we actually got an image");
    Some(converted)
}

fn capture_download_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is before the Unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "photobooth-capture-{}-{timestamp}.jpg",
        std::process::id()
    ))
}

fn get_camera_setting(camera: &Camera, key: &str) -> RadioWidget {
    camera
        .config_key::<RadioWidget>(key)
        .wait()
        .unwrap()
}

fn set_camera_setting(camera: &Camera, setting: &RadioWidget, value: &str) {
    setting.set_choice(value).unwrap();
    camera.set_config(&setting).wait().unwrap()
}
