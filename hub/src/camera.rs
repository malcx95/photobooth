use core::panic;
use std::{io::Cursor, path::Path, sync::mpsc::{Receiver, Sender}};
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

// enum State {
//     Idle,
//     Previewing,
// }

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
    let image_quality = config.get_child_by_id(27).unwrap();
    let setting = match image_quality {
        gphoto2::widget::Widget::Radio(radio_widget) => radio_widget,
        _ => panic!("Missing setting image quality"),
    };
    // setting.set_choice("Fine").unwrap(); // THIS ALMOST WORKS
    camera.set_config(&setting);

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
                let msg = fetch_image(&camera, &context, &path).map_or(
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

fn fetch_image(camera: &Camera, camera_context: &Context, file: &CameraFilePath) -> Option<RgbaImage> {
    println!("Got here");
    let camera_file = camera
        .fs()
        .download(&file.folder(), &file.name())
        .wait()
        .ok()?;

    let data = camera_file.get_data(&camera_context).wait().ok()?;

    let decoded = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg).decode().ok()?;
    let converted = decoded.clone().into_rgba8();

    Some(converted)
}
