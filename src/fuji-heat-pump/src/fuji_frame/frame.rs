
/*
8 bytes:
    byte 0: message source
    byte 1: message destination
    byte 2:
        xxyywxxx
            - yy: message type
            - w: write bit, set to 1 when we have updates to send to the unit
    byte 3:
        efffmmmo
            - e: error flag
            - fff: fan mode
            - mmm: AC mode
            - o: enabled flag (on/off)
    byte 4:
        ettttttt
            - e: economy flag
            - ttttttt: temperature (setpoint temperature)
    byte 5:
        mmmmxstx
            - mmmm: magic mask: 0 if primary controller, else 2
            - s: swing bit
            - t: step bit
    byte 6:
        xttttttc
            - tttttt: controller temperature (temp probe)
            - c: controller present bit
    byte 7:
        ????????
            set to all zeroes
 */
use crate::fuji_frame::enums::{FrameACMode, DestinationAddress, FanMode, MessageType, PowerStatus};

trait FrameBinaryRepr {
    fn encode(&self) -> [u8; 8];
    fn decode(data: &[u8; 8]) -> Self;
}

trait PayloadBinaryRepr {
    fn encode(&self) -> [u8; 5];
    fn decode(data: &[u8; 5]) -> Self;
}

#[derive(Clone, Copy)]
pub struct StatusPayload {
    pub has_error: bool,
    pub fan_mode: FanMode,
    pub ac_mode: FrameACMode,
    pub power_status: PowerStatus,
    pub economy_mode: bool,
    pub temperature: u8,
    pub swing: bool,
    pub swing_step: bool,
    pub controller_temperature: u8,
    pub controller_present: bool,
    pub magic_mask: u8,
}

impl PayloadBinaryRepr for StatusPayload {
    fn encode(&self) -> [u8; 5] {
        let mut data = [0; 5];
        data[0] = (data[0] & 0b01111111) | ((self.has_error as u8) << 7);
        data[0] = (data[0] & 0b10001111) | ((self.fan_mode as u8) << 4);
        data[0] = (data[0] & 0b11110001) | ((self.ac_mode as u8) << 1);
        data[0] = (data[0] & 0b11111110) | self.power_status as u8;
        data[1] = (data[1] & 0b01111111) | (self.economy_mode as u8) << 7;
        data[1] = (data[1] & 0b10000000) | self.temperature;
        data[2] = (data[2] & 0b00001111) | (self.magic_mask << 4);
        data[2] = (data[2] & 0b11111011) | (self.swing as u8) << 2;
        data[2] = (data[2] & 0b11111101) | (self.swing_step as u8) << 1;
        data[3] = (data[3] & 0b10000001) | (self.controller_temperature << 1);
        data[3] = (data[3] & 0b11111110) | (self.controller_present as u8);

        data
    }

    fn decode(data: &[u8; 5]) -> StatusPayload {
        StatusPayload {
            has_error: data[0] & 0b10000000 != 0,
            fan_mode: FanMode::from((data[0] & 0b01110000) >> 4),
            ac_mode: FrameACMode::from((data[0] & 0b00001110) >> 1),
            power_status: if data[0] & 0b00000001 != 0 {PowerStatus::On} else {PowerStatus::Off},
            economy_mode: data[1] & 0b10000000 != 0,
            temperature: data[1] & 0b01111111,
            magic_mask: (data[2] & 0b11110000) >> 4,
            swing: data[2] & 0b00000100 != 0,
            swing_step: data[2] & 0b00000010 != 0,
            controller_temperature: (data[3] & 0b01111110) >> 1,
            controller_present: data[3] & 0b00000001 != 0,
        }
    }
}

impl Default for StatusPayload {
    fn default() -> Self {
        StatusPayload {
            has_error: false,
            fan_mode: FanMode::Auto,
            ac_mode: FrameACMode::Unknown,
            power_status: PowerStatus::Off,
            economy_mode: false,
            temperature: 0,
            swing: false,
            swing_step: false,
            controller_present: false,
            controller_temperature: 0,
            magic_mask: 0
        }
    }
}

#[derive(Clone, Copy)]
pub struct LoginPayload {
    data: [u8; 5]
}

impl PayloadBinaryRepr for LoginPayload {
    fn encode(&self) -> [u8; 5] { self.data.clone() }
    fn decode(data: &[u8; 5]) -> LoginPayload { LoginPayload { data: *data } }
}

impl Default for LoginPayload {
    fn default() -> Self { LoginPayload { data: [0; 5] } }
}

#[derive(Clone, Copy)]
pub struct ErrorPayload {
    data: [u8; 5]
}

impl PayloadBinaryRepr for ErrorPayload {
    fn encode(&self) -> [u8; 5] { self.data.clone() }
    fn decode(data: &[u8; 5]) -> ErrorPayload { ErrorPayload { data: *data } }
}

impl Default for ErrorPayload {
    fn default() -> Self { ErrorPayload { data: [0; 5] } }
}

#[derive(Clone, Copy)]
pub struct UnknownPayload {
    data: [u8; 5]
}

impl PayloadBinaryRepr for UnknownPayload {
    fn encode(&self) -> [u8; 5] {
        self.data.clone()
    }

    fn decode(data: &[u8; 5]) -> UnknownPayload {
        UnknownPayload { data: *data }
    }
}

#[derive(Clone, Copy)]
pub enum FujiPayload {
    Status(StatusPayload),
    Login(LoginPayload),
    Error(ErrorPayload),
    Unknown(UnknownPayload)
}


pub struct FujiFrame {
    pub source: u8,
    pub destination: DestinationAddress,
    pub write_bit: bool,
    pub unknown_bit: bool,
    pub payload: FujiPayload
}

impl Default for FujiFrame {
    fn default() -> Self {
        FujiFrame {
            source: 0,
            destination: DestinationAddress::Unknown,
            write_bit: false,
            unknown_bit: false,
            payload: FujiPayload::Status(StatusPayload::default())
        }
    }
}

impl FrameBinaryRepr for FujiFrame {
    fn encode(&self) -> [u8; 8] {
        let mut data = [0u8; 8];
        data[0] = self.source;
        data[1] = self.destination as u8;
        data[1] = (data[1] & 0b01111111) | ((self.unknown_bit as u8) << 7);
        data[2] = (data[2] & 0b11110111) | ((self.write_bit as u8) << 3);

        let message_type;
        let payload_data;

        match self.payload {
            FujiPayload::Status(payload) => {
                message_type = MessageType::Status;
                payload_data = payload.encode();
            },
            FujiPayload::Login(payload) => {
                message_type = MessageType::Login;
                payload_data = payload.encode();
            },
            FujiPayload::Error(payload) => {
                message_type = MessageType::Error;
                payload_data = payload.encode();
            },
            FujiPayload::Unknown(payload) => {
                message_type = MessageType::Unknown;
                payload_data = payload.encode();
            },
        }

        data[2] = (data[2] & 0b11001111) | ((message_type as u8) << 4);
        data[3] = payload_data[0];
        data[4] = payload_data[1];
        data[5] = payload_data[2];
        data[6] = payload_data[3];
        data[7] = payload_data[4];

        data
    }

    fn decode(data: &[u8; 8]) -> Self {
        let payload: FujiPayload;
        let message_type = MessageType::from((data[2] & 0b00110000) >> 4);

        match message_type {
            MessageType::Status => {
                payload = FujiPayload::Status(StatusPayload::decode(&data[3 .. 8].try_into().unwrap()));
            },
            MessageType::Error => {
                payload = FujiPayload::Error(ErrorPayload::decode(&data[3 .. 8].try_into().unwrap()))
            }
            MessageType::Login => {
                payload = FujiPayload::Login(LoginPayload::decode(&data[3 .. 8].try_into().unwrap()))
            }
            MessageType::Unknown => {
                payload = FujiPayload::Unknown(UnknownPayload::decode(&data[3 .. 8].try_into().unwrap()));
            }
        }

        FujiFrame {
            source: data[0],
            destination: DestinationAddress::from(data[1] & 0b01111111),
            write_bit: data[2] & 0b00001000 != 0,
            unknown_bit: data[1] & 0b10000000 != 0,
            payload,
        }
    }
}
