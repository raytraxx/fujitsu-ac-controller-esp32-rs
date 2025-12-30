#[derive(Clone, Copy)]
pub enum DestinationAddress {
    Unknown = 0,
    Unit = 1,
    PrimaryController = 32,
    SecondaryController = 33,
}

impl From<u8> for DestinationAddress {
    fn from(value: u8) -> Self {
        match value {
            0 => DestinationAddress::Unknown,
            1 => DestinationAddress::Unit,
            32 => DestinationAddress::PrimaryController,
            33 => DestinationAddress::SecondaryController,
            _ => DestinationAddress::Unknown
        }
    }
}

#[derive(Clone, Copy)]
pub enum MessageType {
    Status = 0,
    Error = 1,
    Login = 2,
    Unknown = 3
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::Status,
            1 => MessageType::Error,
            2 => MessageType::Login,
            3 => MessageType::Unknown,
            _ => MessageType::Unknown
        }
    }
}

#[derive(Clone, Copy)]
pub enum FrameACMode {
    Unknown = 0,
    Fan = 1,
    Dry = 2,
    Cool = 3,
    Heat = 4,
    Auto = 5
}

impl From<u8> for FrameACMode {
    fn from(value: u8) -> Self {
        match value {
            0 => FrameACMode::Unknown,
            1 => FrameACMode::Fan,
            2 => FrameACMode::Dry,
            3 => FrameACMode::Cool,
            4 => FrameACMode::Heat,
            5 => FrameACMode::Auto,
            _ => FrameACMode::Unknown
        }
    }
}

#[derive(Clone, Copy)]
pub enum FanMode {
    Auto = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Max = 4,
    Unknown = 5
}

impl From<u8> for FanMode {
    fn from(value: u8) -> FanMode {
        match value {
            0 => FanMode::Auto,
            1 => FanMode::Low,
            2 => FanMode::Medium,
            3 => FanMode::High,
            4 => FanMode::Max,
            _ => FanMode::Unknown
        }
    }
}

#[derive(Clone, Copy)]
pub enum PowerStatus {
    Off = 0,
    On = 1
}
