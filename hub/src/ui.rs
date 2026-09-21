use macroquad::prelude::*;
use image::{DynamicImage, ImageEncoder, ImageReader, RgbaImage};
use std::f32::consts::PI;

use crate::button;


pub struct DisplayTexture {
    pub texture: Texture2D,
    pub width: u32,
    pub height: u32,
}

impl DisplayTexture {
    pub fn new(image: &RgbaImage) -> Self {
        let width =
            u16::try_from(image.width()).unwrap();
        let height =
            u16::try_from(image.height()).unwrap();

        Self {
            texture: Texture2D::from_rgba8(width, height, image.as_raw()),
            width: image.width(),
            height: image.height(),
        }
    }

    pub fn update(&mut self, image: &RgbaImage) {
        if self.width != image.width() || self.height != image.height() {
            *self = Self::new(image);
        } else {
            self.texture
                .update_from_bytes(self.width, self.height, image.as_raw());
        }
    }
}

pub fn draw_image(texture: &Texture2D, target_height: f32, target_width: f32) {
    draw_texture_ex(
        texture,
        0.,
        0.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(target_width, target_height)),
            ..Default::default()
        },
    );
}

pub fn draw_cheese_frame(image_height: f32, image_width: f32) {
    let font_size = 400.0;
    let center = get_text_center("CHEESE!", Option::None, font_size as u16, 1.0, 0.0);
    draw_text(
        "CHEESE!",
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        font_size,
        YELLOW,
    );
}

pub fn draw_loading_frame(image_height: f32, image_width: f32) {
    let font_size = 400.0;

    let center = get_text_center("Loading...", Option::None, font_size as u16, 1.0, 0.0);
    draw_text(
        "Loading...",
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        font_size,
        YELLOW,
    );
}

pub fn draw_review_frame(image_height: f32, image_width: f32) {
    let font_size = 400.0;

    let center = get_text_center("OK?", Option::None, font_size as u16, 1.0, 0.0);
    draw_text(
        "OK?",
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        font_size,
        YELLOW,
    );
}

pub fn draw_countdown(count: f32, image_height: f32, image_width: f32) {
    let count_digit = count.ceil() as i32;
    let digit_str = format!("{}", count_digit);
    let font_size = 160.0 * (-(count * 2.0 * PI).sin() + 2.0) / 2.0;
    // let font_size = 260.0 * (count % 1.0 + 0.3);

    let center = get_text_center(&digit_str, Option::None, font_size as u16, 1.0, 0.0);
    draw_text(
        &digit_str,
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        font_size,
        YELLOW,
    );
}

pub fn draw_buttons(image_height: f32, image_width: f32, buttons: &Vec<button::Button>) {
    let button_x = image_width + 100.0;
    let label_x = button_x + 60.0;
    let button_y = 100.0;
    let button_y_separation = 100.0;
    let font_size = 40.0;

    for (i, button) in buttons.into_iter().enumerate() {
        if button.enabled {
            draw_circle(
                button_x,
                button_y + (i as f32) * button_y_separation,
                40.0,
                button.color,
            );
            draw_text(
                button.text.as_str(),
                label_x,
                button_y + (i as f32) * button_y_separation + font_size / 4.0,
                font_size,
                WHITE,
            );
        }
    }
}
