use std::error::Error;
use std::io::{Read, Write};
use crate::fuji_controller::ControllerType::Primary;
use crate::fuji_frame::enums::{DestinationAddress, FanMode, FrameACMode, PowerStatus};
use crate::fuji_frame::frame::{ErrorPayload, FujiFrame, FujiPayload, LoginPayload, StatusPayload};

#[derive(Clone, Copy, PartialEq)]
pub enum ControllerType {
    Primary = 32,
    Secondary = 33
}

#[derive(Clone, Copy)]
pub enum ACMode {
    Off = 0,
    Fan = 1,
    Dry = 2,
    Cool = 3,
    Heat = 4,
    Auto = 5
}

pub trait FujiUartDriver<E> where E: Error {
    fn send_frame(&mut self, frame: &[u8; 8]) -> Result<usize, E>;
    fn read_frame(&mut self, buf: &mut [u8]) -> Result<usize, E>;
}

pub struct FujiController<E> {
    controller_type: ControllerType,
    ac_status: StatusPayload,

    setpoint_temperature: u8,
    ac_mode: FrameACMode,
    power_status: PowerStatus,
    fan_mode: FanMode,

    probe_temperature: Option<u8>,
    uart: Box<dyn FujiUartDriver<E>>
}

impl<E> FujiController<E> where E: Error {
    pub fn new(controller_type: ControllerType, uart: Box<dyn FujiUartDriver<E>>) -> FujiController<E> {
        FujiController {
            controller_type,
            ac_status: StatusPayload::default(),
            setpoint_temperature: 20,
            ac_mode: FrameACMode::Auto,
            power_status: PowerStatus::On,
            fan_mode: FanMode::Auto,
            probe_temperature: None,
            uart,
        }
    }

    pub fn set_mode(&mut self, mode: ACMode) {
        match mode {
            ACMode::Off => {
                self.power_status = PowerStatus::Off;
            }
            ACMode::Fan => {
                self.power_status = PowerStatus::On;
                self.ac_mode = FrameACMode::Fan;
            }
            ACMode::Dry => {
                self.power_status = PowerStatus::On;
                self.ac_mode = FrameACMode::Dry;
            }
            ACMode::Cool => {
                self.power_status = PowerStatus::On;
                self.ac_mode = FrameACMode::Cool;
            }
            ACMode::Heat => {
                self.power_status = PowerStatus::On;
                self.ac_mode = FrameACMode::Heat;
            }
            ACMode::Auto => {
                self.power_status = PowerStatus::On;
                self.ac_mode = FrameACMode::Auto;
            }
        }
    }

    pub fn set_fan_mode(&mut self, mode: FanMode) {
        self.fan_mode = mode;
    }

    pub fn set_setpoint_temperature(&mut self, temperature: u8) {
        if temperature >= 16 && temperature <= 29 {
            self.setpoint_temperature = temperature;
        }
    }

    pub fn set_probe_temperature(&mut self, temperature: u8) {
        self.probe_temperature = Some(temperature);
    }

    fn handle_incoming_frame(&mut self, frame: FujiFrame) {
        if (frame.destination as u8) != (self.controller_type as u8) {
            return
        }

        match frame.payload {
            FujiPayload::Status(payload) => {
                self.ac_status = payload;
                if !payload.controller_present {
                    if self.controller_type == Primary {
                        self.send_logged_in_frame()
                    } else {
                        self.send_secondary_frame()
                    }
                } else if payload.has_error {
                    self.send_error_query()
                } else {

                }
            }
            FujiPayload::Login(_) => {}
            FujiPayload::Error(_) => {}
            FujiPayload::Unknown(_) => {}
        }
    }

    fn make_frame(&self) -> FujiFrame {
        FujiFrame {
            source: self.controller_type as u8,
            destination: DestinationAddress::Unit,
            ..Default::default()
        }
    }

    fn send_logged_in_frame(&self) {
        let frame = FujiFrame {
            unknown_bit: true,
            payload: FujiPayload::Login(LoginPayload::default()),
            ..self.make_frame()
        };
    }

    fn send_secondary_frame(&self) {
        let frame = FujiFrame {
            unknown_bit: true,
            payload: FujiPayload::Status(StatusPayload {
                controller_present: true,
                magic_mask: 2,
                ..Default::default()
            }),
            ..self.make_frame()
        };
    }

    fn send_error_query(&self) {
        let frame = FujiFrame {
            payload: FujiPayload::Error(ErrorPayload::default()),
            ..self.make_frame()
        };
    }
}
