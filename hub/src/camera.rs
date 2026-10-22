use core::panic;
use gphoto2::Context;
use gphoto2::widget::ToggleWidget;
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

use crate::timer;

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
    Focus,
    CountdownStart,
    Reset,
    SetMode(bool),
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
        "32800",
        "32864",
        "32912",
    ];

    let apertures = vec![
        "f/2.8",
        "f/4",
        "f/6.3",
    ];


    let focuses = vec![
        "Automatic",
        "Manual", // manual
    ];

    let mut effect_index = 0;

    let image_quality = get_camera_setting(&camera, "imagequality");
    let picture_effect = get_camera_setting(&camera, "d21b");
    let aperture = get_camera_setting(&camera, "f-number");
    let focus = get_camera_setting(&camera, "focusmode");

    let focustrigger = get_focus(&camera);

    set_camera_setting(&camera, &image_quality, "Fine");
    set_camera_setting(&camera, &picture_effect, effects[effect_index]);
    set_camera_setting(&camera, &aperture, apertures[0]);
    set_camera_setting(&camera, &focus, "Automatic");

    let config = camera.config().wait().unwrap();
    println!("{:#?}", config);
    image_tx.send(ImageMessage::CameraStarted).unwrap();

    let mut auto_focus = true;

    let mut focus_timer = timer::Timer::new(1000);
    let mut focusing = false;

    let mut pending_manual = false;

    loop {

        if focusing && focus_timer.triggered() {
            focustrigger.set_toggled(false);
            camera.set_config(&focustrigger).wait().unwrap();
            focusing = false;
        }

        if pending_manual && !focusing {
            set_camera_setting(&camera, &focus, "Manual");
            pending_manual = false;
            println!("Setting manual");
        }

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
                println!("{}", effects[effect_index]);
                set_camera_setting(&camera, &picture_effect, effects[effect_index]); // picture effect
            }
            Some(CameraCommand::Focus) => {
                focusing = true;
                focustrigger.set_toggled(true);
                camera.set_config(&focustrigger).wait().unwrap();
            }
            Some(CameraCommand::CountdownStart) => {
                if !auto_focus {
                    pending_manual = true;
                }
            }
            Some(CameraCommand::SetMode(lights_on)) => {
                auto_focus = lights_on;
            }
            Some(CameraCommand::Reset) => {
                effect_index = 0;  // it's only 4 days until the wedding I don't have time to do
                                   // this properly ok
                set_camera_setting(&camera, &picture_effect, effects[effect_index]);
                auto_focus = true;
            }
            Some(CameraCommand::FetchImage(path)) => {
                sleep(Duration::from_secs(1));
                let msg = fetch_image(&camera, &path).map_or(ImageMessage::FetchFailed, |image| {
                    ImageMessage::FetchedImage(image, path)
                });
                image_tx.send(msg).unwrap();
                set_camera_setting(&camera, &focus, "Automatic");
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
    flush_camera_events(camera)?;
    camera.trigger_capture().wait().ok()?;

    let deadline = Instant::now() + Duration::from_secs(20);
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

fn flush_camera_events(camera: &Camera) -> Option<()> {
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
        // try ten times
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

    Some(converted)
}

fn capture_download_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
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

fn get_focus(camera: &Camera) -> ToggleWidget {
    camera
        .config_key::<ToggleWidget>("autofocus")
        .wait()
        .unwrap()
}

fn set_camera_setting(camera: &Camera, setting: &RadioWidget, value: &str) {
    setting.set_choice(value).unwrap();
    camera.set_config(&setting).wait().unwrap()
}
