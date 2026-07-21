use core::panic;
use std::{
    io::Cursor, path::PathBuf, sync::mpsc::{Receiver, Sender}, thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}
};
use gphoto2::{camera::Camera, file::CameraFilePath, widget::RadioWidget};
use gphoto2::Context;
use macroquad::prelude::*;
use image::{ImageReader, RgbaImage};

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
    FetchImage(CameraFilePath),
    DeleteImage(CameraFilePath),
}

pub fn camera_loop(image_tx: Sender<ImageMessage>, camera_rx: Receiver<CameraCommand>) {
    let context = Context::new().expect("Failed to create a context!");
    let camera: Camera;
    match context.autodetect_camera().wait() {
        Ok(cam) => {
            camera = cam;
        },
        Err(e) => {
            image_tx.send(ImageMessage::FailedToStartCamera(e.to_string())).unwrap();
            return;
        }
    }

    let config = camera.config().wait().unwrap();
    let image_quality = camera.config_key::<RadioWidget>("imagequality").wait().unwrap();
    image_quality.set_choice("Fine").unwrap();
    camera.set_config(&image_quality).wait().unwrap();

    // let shutter_speed = camera.config_key::<RadioWidget>("shutterspeed").wait().unwrap();;
    // shutter_speed.set_choice("1/10").unwrap();
    // camera.set_config(&shutter_speed).wait().unwrap();

    println!("{:#?}", config);
    image_tx.send(ImageMessage::CameraStarted).unwrap();

    loop {
        let msg = camera_rx.recv().ok();

        match msg {
            Some(CameraCommand::CapturePreview) => {
                let msg = preview_camera(&camera, &context).map_or(
                    ImageMessage::ImagePreviewFailed,
                    |image| { ImageMessage::ImagePreview(image) });
                image_tx.send(msg).unwrap();
            },
            Some(CameraCommand::CaptureImage) => {
                let msg = capture_image(&camera).map_or(
                    ImageMessage::CaptureFailed,
                    |path| { ImageMessage::Captured(path) });
                image_tx.send(msg).unwrap();
            },
            Some(CameraCommand::FetchImage(path)) => {
                sleep(Duration::from_secs(1));
                let msg = fetch_image(&camera, &path).map_or(
                    ImageMessage::FetchFailed,
                    |image| { ImageMessage::FetchedImage(image, path) });
                println!("Sending image");
                image_tx.send(msg).unwrap();
                println!("Sent image");
            },
            Some(CameraCommand::DeleteImage(path)) => {
                camera.fs().delete_file(&path.folder(), &path.name()).wait().ok();
            },
            _ => {}
        }
    }
}

fn preview_camera(camera: &Camera, camera_context: &Context) -> Option<RgbaImage> {
    let preview = camera.capture_preview().wait().ok()?;
    let data = preview.get_data(&camera_context).wait().ok()?;

    let decoded = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg).decode().ok()?;
    let converted = decoded.clone().into_rgba8();

    Some(converted)
}

fn capture_image(camera: &Camera) -> Option<CameraFilePath> {
    camera.capture_image().wait().ok()
}

fn fetch_image(camera: &Camera, file: &CameraFilePath) -> Option<RgbaImage> {
    let download_path = capture_download_path();
    let camera_file = camera
        .fs()
        .download_to(&file.folder(), &file.name(), &download_path)
        .wait()
        .ok()?;

    let decoded = ImageReader::open(&download_path).ok()?.decode().ok()?;
    let converted = decoded.into_rgba8();
    drop(camera_file);
    if let Err(error) = std::fs::remove_file(&download_path) {
        eprintln!("Failed to remove temporary image {}: {error}", download_path.display());
    }

    Some(converted)
}

fn capture_download_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is before the Unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("photobooth-capture-{}-{timestamp}.jpg", std::process::id()))
}
