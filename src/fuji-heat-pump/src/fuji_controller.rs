use std::error::Error;
use std::sync::Mutex;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
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
    fn send_frame(&self, frame: &[u8; 8]) -> Result<usize, E>;
    fn read_frame(&self, buf: &mut [u8]) -> Result<usize, E>;
}

pub struct FujiController<E> {
    controller_type: ControllerType,

    setpoint_temperature: u8,
    ac_mode: FrameACMode,
    power_status: PowerStatus,
    fan_mode: FanMode,
    economy_mode: bool,

    probe_temperature: Option<u8>,
    uart: Box<dyn FujiUartDriver<E> + Send + Sync>,
}

impl<E> FujiController<E> where E: Error {
    pub fn new(controller_type: ControllerType, uart: Box<dyn FujiUartDriver<E> + Send + Sync>) -> FujiController<E> {
        FujiController {
            controller_type,
            setpoint_temperature: 20,
            ac_mode: FrameACMode::Auto,
            power_status: PowerStatus::On,
            fan_mode: FanMode::Auto,
            economy_mode: false,
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

    pub fn set_economy_mode(&mut self, mode: bool) {
        self.economy_mode = mode;
    }

    pub fn set_setpoint_temperature(&mut self, temperature: u8) {
        if temperature >= 16 && temperature <= 29 {
            self.setpoint_temperature = temperature;
        }
    }

    pub fn set_probe_temperature(&mut self, temperature: u8) {
        self.probe_temperature = Some(temperature);
    }

    fn wait(&self) {
        sleep(Duration::from_millis(60))
    }

    pub fn spawn_thread(&self) -> Mutex<&FujiController<E>> {
        let mutex = Mutex::new(self);

        thread::spawn(|| {
            let mut buffer = [0u8; 8];

            loop {
                if let Ok(res) = self.uart.read_frame(&mut buffer) && res == 8 {
                    let frame = FujiFrame::decode(buffer);
                    let response = self.handle_incoming_frame(frame);
                    match response {
                        None => {}
                        Some(r) => {
                            self.wait();
                            _ = self.uart.send_frame(&r.encode());
                        }
                    }
                }

                self.wait();
            }
        });

        mutex
    }

    fn handle_incoming_frame(&self, frame: FujiFrame) -> Option<FujiFrame> {
        if (frame.destination as u8) != (self.controller_type as u8) {
            return None
        }

        match frame.payload {
            FujiPayload::Status(payload) => {
                if !payload.controller_present {
                    if self.controller_type == Primary {
                        Some(self.make_logged_in_frame())
                    } else {
                        Some(self.make_secondary_frame())
                    }
                } else if payload.has_error {
                    Some(self.make_error_query())
                } else {
                    Some(self.make_status_frame())
                }
            }
            FujiPayload::Login(_) => {None}
            FujiPayload::Error(_) => {None}
            FujiPayload::Unknown(_) => {None}
        }
    }

    fn make_frame(&self) -> FujiFrame {
        FujiFrame {
            source: self.controller_type as u8,
            destination: DestinationAddress::Unit,
            ..Default::default()
        }
    }

    fn make_status_frame(&self) -> FujiFrame {
        FujiFrame {
            unknown_bit: false,
            write_bit: true,
            payload: FujiPayload::Status(StatusPayload {
                controller_present: true,
                magic_mask: 0,
                ac_mode: self.ac_mode,
                power_status: self.power_status,
                fan_mode: self.fan_mode,
                setpoint_temperature: self.setpoint_temperature,
                probe_temperature: self.probe_temperature.unwrap_or_default(),
                has_error: false,
                swing_step: false,
                economy_mode: self.economy_mode,
                swing: false
            }),
            ..self.make_frame()
        }
    }

    fn make_logged_in_frame(&self) -> FujiFrame {
        FujiFrame {
            unknown_bit: true,
            payload: FujiPayload::Login(LoginPayload::default()),
            ..self.make_frame()
        }
    }

    fn make_secondary_frame(&self) -> FujiFrame {
        FujiFrame {
            unknown_bit: true,
            payload: FujiPayload::Status(StatusPayload {
                controller_present: true,
                magic_mask: 2,
                ..Default::default()
            }),
            ..self.make_frame()
        }
    }

    fn make_error_query(&self) -> FujiFrame {
        FujiFrame {
            payload: FujiPayload::Error(ErrorPayload::default()),
            ..self.make_frame()
        }
    }
}
