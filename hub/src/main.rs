use gphoto2::Context;
use std::io::Cursor;
use macroquad::prelude::*;
use image::ImageReader;


#[macroquad::main("photobooth-hub")]
async fn main() -> Result<(), String>  {
    let context = Context::new().unwrap();
    let camera = context.autodetect_camera().wait().unwrap();
    loop {
        let preview = camera.capture_preview().wait().unwrap();
        let data = preview.get_data(&context).wait().unwrap();

        let decoded = ImageReader::with_format(Cursor::new(data), image::ImageFormat::Jpeg).decode().unwrap();
        let width = decoded.width() as u16;
        let height = decoded.height() as u16;
        let converted = decoded.into_rgba8();
        let texture = Texture2D::from_rgba8(width, height, &converted);

        draw_texture(&texture, 0., 0., WHITE);

        next_frame().await;
    }
}

