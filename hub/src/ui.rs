use image::{DynamicImage, ImageEncoder, ImageReader, RgbaImage};
use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::{ProgramState, button::{self, BIG_WHITE_BUTTON, button_text}};

pub struct DisplayTexture {
    pub texture: Texture2D,
    pub width: u32,
    pub height: u32,
}

impl DisplayTexture {
    pub fn new(image: &RgbaImage) -> Self {
        let width = u16::try_from(image.width()).unwrap();
        let height = u16::try_from(image.height()).unwrap();

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
    let font_size = 100.0;

    let center = get_text_center("Loading...", Option::None, font_size as u16, 1.0, 0.0);
    draw_text(
        "Loading...",
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        font_size,
        YELLOW,
    );
}

pub fn draw_score_reveal(image_height: f32, image_width: f32, score: u32, progress: f32, rank: usize) {
    const FONT_SIZE: u16 = 150;
    let rank_text = if rank == 1 {
        String::from("You are number 1!")
    } else {
        format!("Rank: {}", rank)
    };
    let score_text = format!("Score: {}, {}", score, rank_text);
    let scale = (progress * 4.0).min(1.0);
    let center = get_text_center(score_text.as_str(), Option::None, FONT_SIZE, scale, 0.0);

    draw_text_ex(
        score_text.as_str(),
        image_width / 2.0 - center.x / 2.0 - FONT_SIZE as f32,
        image_height / 2.0 - center.y / 2.0,
        TextParams {
            font_size: FONT_SIZE,
            font_scale: scale,
            color: YELLOW,
            ..Default::default()
        },
    );
}

pub fn draw_capture_failed(image_height: f32, image_width: f32) {
    let font_size = 100.0;

    let text = "Failed, try again";

    let center = get_text_center(text, Option::None, font_size as u16, 1.0, 0.0);
    draw_text(
        text,
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        font_size,
        RED,
    );
}

pub fn draw_review_frame(image_height: f32, image_width: f32, current_score: Option<u32>, maybe_rank: Option<usize>) {
    let font_size = 150.0;
    let rank_text = if let Some(rank) = maybe_rank {
        format!("{}. ", rank)
    } else {
        String::from("")
    };

    let score_text = match current_score {
        Some(score) => format!("{}Score: {}", rank_text, score),
        None => String::from("Accept/Reject?"),
    };

    draw_text(score_text.as_str(), 0., image_height + font_size, font_size, YELLOW);
}

pub fn draw_countdown(count: f32, image_height: f32, image_width: f32) {
    const FONT_SIZE: u16 = 160;
    let count_digit = count.ceil() as i32;
    let digit_str = format!("{}", count_digit);
    let font_scale = (-(count * 2.0 * PI).sin() + 2.0) / 2.0;

    let center = get_text_center(&digit_str, None, FONT_SIZE, font_scale, 0.0);
    draw_text_ex(
        &digit_str,
        image_width / 2.0 - center.x / 2.0,
        image_height / 2.0 - center.y / 2.0,
        TextParams {
            font_size: FONT_SIZE,
            font_scale,
            color: YELLOW,
            ..Default::default()
        },
    );
}

pub fn draw_buttons(image_height: f32, image_width: f32, buttons: &Vec<button::Button>, state: &ProgramState) {
    let button_x = image_width + 100.0;
    let label_x = button_x + 60.0;
    let button_y = 100.0;
    let button_y_separation = 100.0;
    let font_size = 40.0;

    let mut i = 0.0;
    let mut separation = button_y_separation;
    for button in buttons {
        if button.enabled {
            let (circle_size, x_offset) = if button.pin == BIG_WHITE_BUTTON {
                (60.0, 20.0)
            } else {
                (40.0, 0.0)
            };

            draw_circle(
                button_x,
                button_y + i * separation,
                circle_size,
                button.color,
            );
            draw_text(
                button_text(button.pin, state),
                label_x + x_offset,
                button_y + i * separation + font_size / 4.0,
                font_size,
                WHITE,
            );

            if button.pin == BIG_WHITE_BUTTON {
                separation += 40.0;
            }

            i += 1.0;
        }
    }
}
