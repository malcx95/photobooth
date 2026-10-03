use macroquad::prelude::*;
use serialport::{SerialPort};

use crate::{ProgramState, serial};

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
    Leaderboard,
}

pub struct Button {
    pub pin: u8,
    pub color: Color,
    pub enabled: bool,
}

pub fn init_buttons() -> Vec<Button> {
    vec![
        Button {pin: BIG_WHITE_BUTTON, color: GRAY,     enabled: false},
        Button {pin: RED_BUTTON,       color: RED,      enabled: false},
        Button {pin: BLUE_BUTTON,      color: BLUE,     enabled: false},
        Button {pin: WHITE_BUTTON,     color: WHITE,    enabled: false},
        Button {pin: YELLOW_BUTTON,    color: YELLOW,   enabled: false},
        Button {pin: GREEN_BUTTON,     color: GREEN,    enabled: false},
    ]
}

pub fn button_text(pin: u8, state: &ProgramState) -> &'static str {
    // i don't really know if this can be done in a better way sorry frans if you are reading
    // this please help
    if pin == BIG_WHITE_BUTTON {
        "Capture"
    } else if pin == RED_BUTTON {
        match state {
            ProgramState::Leaderboard => "Previous",
            ProgramState::Countdown => "Cancel",
            _ => "Reject",
        }
    } else if pin == BLUE_BUTTON {
        "Change effect"
    } else if pin == WHITE_BUTTON {
        match state {
            ProgramState::Leaderboard => "Go back",
            _ => "See leaderboard",
        }
    } else if pin == YELLOW_BUTTON {
        "Toggle flash"
    } else if pin == GREEN_BUTTON {
        match state {
            ProgramState::Leaderboard => "Next",
            _ => "Accept",
        }
    } else {
        ""
    }
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
            } else if button == WHITE_BUTTON {
                Some(ButtonPress::Leaderboard)
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
