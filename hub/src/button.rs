use macroquad::prelude::*;
use serialport::{SerialPort};

use crate::serial;

pub const BIG_WHITE_BUTTON: u8 = 6;
pub const RED_BUTTON: u8 = 7;
pub const BLUE_BUTTON: u8 = 8;
pub const WHITE_BUTTON: u8 = 9;
pub const YELLOW_BUTTON: u8 = 20;
pub const GREEN_BUTTON: u8 = 21;

pub enum ButtonPress {
    TakePhoto,
    CycleEffect,
    Accept,
    Reject,
}

pub struct Button {
    pub pin: u8,
    pub color: Color,
    pub text: String,
    pub enabled: bool,
}

pub fn init_buttons() -> Vec<Button> {
    vec![
        Button {pin: BIG_WHITE_BUTTON, color: GRAY, text: String::from("Capture"), enabled: false},
        Button {pin: RED_BUTTON,       color: RED, text: String::from("Reject"), enabled: false},
        Button {pin: BLUE_BUTTON,      color: BLUE, text: String::from("Change effect"), enabled: false},
        Button {pin: WHITE_BUTTON,     color: WHITE, text: String::from("White"), enabled: false},
        Button {pin: YELLOW_BUTTON,    color: YELLOW, text: String::from("Yellow"), enabled: false},
        Button {pin: GREEN_BUTTON,     color: GREEN, text: String::from("Accept"), enabled: false},
    ]
}

pub fn read_buttons(port: &mut Box<dyn SerialPort>) -> Option<ButtonPress> {
    let press_opt = match serial::read_serial(port) {
        serial::RXMessage::Nothing => None,
        serial::RXMessage::ButtonPress(button) => {
            if button == BIG_WHITE_BUTTON {
                Some(ButtonPress::TakePhoto)
            } else if button == RED_BUTTON {
                Some(ButtonPress::Reject)
            } else if button == GREEN_BUTTON {
                Some(ButtonPress::Accept)
            } else if button == BLUE_BUTTON {
                Some(ButtonPress::CycleEffect)
            } else {
                None
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
