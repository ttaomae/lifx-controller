use super::header::{DeviceMessageType, LightMessageType, MessageType};
use std::convert::TryInto;

/// A LIFX packet message.
#[derive(Debug, Clone)]
pub(crate) enum Message {
    Empty(MessageType),
    Bytes(MessageType, Vec<u8>),
    StateService(StateServicePayload),
    StateLabel(StateLabelPayload),
    StateGroup(StateGroupPayload),
    StateLocation(StateLocationPayload),
    State(StatePayload),
    SetColor(SetColorPayload),
    SetPower(SetPowerPayload),
}

impl Message {
    pub(crate) fn from(message_type: MessageType, bytes: &[u8]) -> Message {
        if bytes.is_empty() {
            return Message::Empty(message_type);
        }

        use DeviceMessageType::*;
        use LightMessageType::*;
        use MessageType::Device;
        use MessageType::Light;
        match message_type {
            Device(StateService) => Message::StateService(StateServicePayload::from_bytes(bytes)),
            Device(StateLabel) => Message::StateLabel(StateLabelPayload::from_bytes(bytes)),
            Device(StateLocation) => {
                Message::StateLocation(StateLocationPayload::from_bytes(bytes))
            }
            Device(StateGroup) => Message::StateGroup(StateGroupPayload::from_bytes(bytes)),
            Light(State) => Message::State(StatePayload::from_bytes(bytes)),
            _ => Message::Bytes(message_type, bytes.to_vec()),
        }
    }

    pub(crate) fn message_type(&self) -> MessageType {
        match self {
            Message::Empty(message_type) => *message_type,
            Message::Bytes(message_type, _) => *message_type,
            Message::StateService(_) => MessageType::Device(DeviceMessageType::StateService),
            Message::StateLabel(_) => MessageType::Device(DeviceMessageType::StateLabel),
            Message::StateLocation(_) => MessageType::Device(DeviceMessageType::StateLocation),
            Message::StateGroup(_) => MessageType::Device(DeviceMessageType::StateGroup),
            Message::State(_) => MessageType::Light(LightMessageType::State),
            Message::SetColor(_) => MessageType::Light(LightMessageType::SetColor),
            Message::SetPower(_) => MessageType::Light(LightMessageType::SetPower),
        }
    }

    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        match self {
            Message::Empty(_) => Vec::new(),
            Message::Bytes(_, bytes) => bytes.clone(),
            Message::SetPower(set_power_payload) => set_power_payload.as_bytes(),
            Message::SetColor(set_color_payload) => set_color_payload.as_bytes(),
            _ => panic!("Unrecognized message"),
        }
    }
}

/// A payload sent by a client. Can be converted to bytes.
trait ClientPayload {
    fn as_bytes(&self) -> Vec<u8>;
}

/// A payload sent by a device. Can be created from bytes.
trait DevicePayload {
    fn from_bytes(bytes: &[u8]) -> Self;
}

/// The payload for a StateService message.
#[derive(Debug, Copy, Clone)]
pub(crate) struct StateServicePayload {
    service: u8,
    port: u16,
}

impl StateServicePayload {
    pub(crate) fn port(&self) -> u16 {
        self.port
    }
}

impl DevicePayload for StateServicePayload {
    fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() != 5 {
            panic!("{:?}", bytes);
        }

        let service = bytes[0];
        let port = u32::from_le_bytes(bytes[1..5].try_into().unwrap());

        if port > u16::MAX as u32 {
            panic!("Unknown port: {}", port);
        }
        StateServicePayload {
            service,
            port: port as u16,
        }
    }
}

/// The payload for a StateLabel message.
#[derive(Debug, Clone)]
pub(crate) struct StateLabelPayload {
    pub(crate) label: String,
}

impl DevicePayload for StateLabelPayload {
    fn from_bytes(bytes: &[u8]) -> Self {
        StateLabelPayload {
            label: String::from_utf8(bytes.to_vec()).unwrap(),
        }
    }
}

/// The payload for a StateLocation message.
#[derive(Debug, Clone)]
pub(crate) struct StateLocationPayload {
    location: [u8; 16],
    pub(crate) label: String,
    updated_at: u64,
}

impl DevicePayload for StateLocationPayload {
    fn from_bytes(bytes: &[u8]) -> Self {
        let location = bytes[0..16].try_into().expect("");
        let label = String::from_utf8(bytes[16..48].to_vec()).unwrap();
        let updated_at = u64::from_le_bytes(bytes[48..56].try_into().expect(""));

        StateLocationPayload {
            location,
            label,
            updated_at,
        }
    }
}

/// The payload for a StateGroup message.
#[derive(Debug, Clone)]
pub(crate) struct StateGroupPayload {
    group: [u8; 16],
    pub(crate) label: String,
    updated_at: u64,
}

impl DevicePayload for StateGroupPayload {
    fn from_bytes(bytes: &[u8]) -> Self {
        let group = bytes[0..16].try_into().expect("");
        let label = String::from_utf8(bytes[16..48].to_vec()).unwrap();
        let updated_at = u64::from_le_bytes(bytes[48..56].try_into().expect(""));

        StateGroupPayload {
            group,
            label,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Hsbk {
    hue: u16,
    saturation: u16,
    brightness: u16,
    kelvin: u16,
}

impl Hsbk {
    pub(crate) fn new(hue: u16, saturation: u16, brightness: u16, kelvin: u16) -> Hsbk {
        Hsbk {
            hue,
            saturation,
            brightness,
            kelvin,
        }
    }

    pub(crate) fn hue(&self) -> u16 {
        self.hue
    }

    pub(crate) fn saturation(&self) -> u16 {
        self.saturation
    }

    pub(crate) fn brightness(&self) -> u16 {
        self.brightness
    }

    pub(crate) fn kelvin(&self) -> u16 {
        self.kelvin
    }

    pub(crate) fn with_hue(&self, hue: u16) -> Hsbk {
        Hsbk {
            hue,
            saturation: self.saturation,
            brightness: self.brightness,
            kelvin: self.kelvin,
        }
    }

    pub(crate) fn with_saturation(&self, saturation: u16) -> Hsbk {
        Hsbk {
            hue: self.hue,
            saturation,
            brightness: self.brightness,
            kelvin: self.kelvin,
        }
    }

    pub(crate) fn with_brightness(&self, brightness: u16) -> Hsbk {
        Hsbk {
            hue: self.hue,
            saturation: self.saturation,
            brightness,
            kelvin: self.kelvin,
        }
    }

    pub(crate) fn with_kelvin(&self, kelvin: u16) -> Hsbk {
        Hsbk {
            hue: self.hue,
            saturation: self.saturation,
            brightness: self.brightness,
            kelvin,
        }
    }
}

impl ClientPayload for Hsbk {
    fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.hue.to_le_bytes().iter());
        result.extend(self.saturation.to_le_bytes().iter());
        result.extend(self.brightness.to_le_bytes().iter());
        result.extend(self.kelvin.to_le_bytes().iter());
        result
    }
}

impl DevicePayload for Hsbk {
    fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() != 8 {
            panic!();
        }

        Hsbk {
            hue: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            saturation: u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
            brightness: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            kelvin: u16::from_le_bytes(bytes[6..8].try_into().unwrap()),
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub(crate) struct StatePayload {
    color: Hsbk,
    // reserved - 16 bits
    power: u16,
    label: String, // 32 bytes
    // reserved - 64 bits
}

impl StatePayload {
    pub(crate) fn color(&self) -> Hsbk {
        self.color.clone()
    }

    pub(crate) fn power(&self) -> Power {
        match self.power {
            0 => Power::Off,
            n => Power::On(n),
        }
    }
}

impl DevicePayload for StatePayload {
    fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() != 52 {
            panic!();
        }

        StatePayload {
            color: Hsbk::from_bytes(&bytes[0..8]),
            power: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
            label: String::from_utf8(bytes[12..44].to_vec()).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SetColorPayload {
    // reserved - 8 bits
    color: Hsbk,
    duration: u32,
}

impl SetColorPayload {
    pub(crate) fn new(color: Hsbk, duration: u32) -> SetColorPayload {
        SetColorPayload { color, duration }
    }
}

impl ClientPayload for SetColorPayload {
    fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(0); // reserved - 8 bits
        result.extend(self.color.as_bytes());
        result.extend(self.duration.to_le_bytes().iter());
        result
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Power {
    // Officially the LIFX docs state that only 0 and 0xffff are valid values.
    // However, devices will sometimes responsd with different values, which
    // seem to indicate that they are powered on.
    Off,
    On(u16),
}

#[derive(Debug, Clone)]
pub(crate) struct SetPowerPayload {
    power: Power,
    duration: u32,
}

impl SetPowerPayload {
    pub(crate) fn new(power: Power, duration: u32) -> SetPowerPayload {
        SetPowerPayload { power, duration }
    }
}

impl ClientPayload for SetPowerPayload {
    fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let level = match self.power {
            Power::Off => u16::MIN,
            Power::On(n) => n,
        };
        result.extend(level.to_le_bytes().iter());
        result.extend(self.duration.to_le_bytes().iter());
        result
    }
}
