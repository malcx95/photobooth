use serialport::{SerialPort, SerialPortInfo};
use std::time::{self, Duration, Instant};

const CONNECT_MSG_TYPE: u8 = 0xDE;
const CONNECT_ACK_MSG_TYPE: u8 = 0xAE;
const SET_LIGHT_STATE_MSG_TYPE: u8 = 0x4A;
const SET_ENABLED_BUTTONS_MSG_TYPE: u8 = 0x4B;

const MSG_TIMEOUT: u64 = 400;

const CONTROLLER_START_WORD: u8 = 0xEE;
const BUTTON_PRESS_MSG_TYPE: u8 = 0xBB;
const HUB_START_WORD: u8 = 0x41;
const CONTROLLER_BAUD_RATE: u32 = 9600;

pub enum TXMessage {
    Connect,
    SetLightState(u8),
    SetEnabledButtons(u8),
}

pub enum RXMessage {
    Nothing,
    ButtonPress(u8),
}

pub fn read_serial(port: &mut Box<dyn SerialPort>) -> RXMessage {
    if port.bytes_to_read().unwrap() < 3 {
        return RXMessage::Nothing;
    }

    let mut serial_buf = [0; 3];
    match port.read_exact(serial_buf.as_mut_slice()) {
        Ok(b) => b,
        Err(_) => {
            println!("Failed to read message");
            return RXMessage::Nothing;
        }
    };

    if serial_buf[0] == CONTROLLER_START_WORD {
        if serial_buf[1] == BUTTON_PRESS_MSG_TYPE {
            return RXMessage::ButtonPress(serial_buf[2]);
        }
    }
    RXMessage::Nothing
}

pub fn find_port(ignore_controller: bool) -> Box<dyn SerialPort> {
    let ports = serialport::available_ports().expect("No serial ports found!");
    for p in ports {
        let mut port = match serialport::new(&p.port_name, CONTROLLER_BAUD_RATE)
            .timeout(Duration::from_millis(MSG_TIMEOUT))
            .open()
        {
            Ok(opened) => opened,
            Err(_) => continue,
        };

        if ignore_controller {
            return port;
        }

        println!("Trying port {}", &p.port_name);

        send_serial_message(&mut port, TXMessage::Connect);
        println!("This port might respond");

        let mut serial_buf = [0; 3];
        match port.read_exact(serial_buf.as_mut_slice()) {
            Ok(b) => b,
            Err(_) => {
                println!("This port did not respond properly");
                continue;
            }
        };

        if serial_buf[1] == CONNECT_ACK_MSG_TYPE {
            return port;
        }
    }
    panic!("Found no ports!");
}

pub fn send_serial_message(port: &mut Box<dyn SerialPort>, message: TXMessage) {
    let message = match message {
        TXMessage::Connect => [1, 1, 1, HUB_START_WORD, CONNECT_MSG_TYPE, 0],
        TXMessage::SetLightState(state) => [1, 1, 1, HUB_START_WORD, SET_LIGHT_STATE_MSG_TYPE, state],
        TXMessage::SetEnabledButtons(state) => [1, 1, 1, HUB_START_WORD, SET_ENABLED_BUTTONS_MSG_TYPE, state],
    };
    let _ = port.write(&message);
}
