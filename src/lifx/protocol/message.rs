use super::header::{DeviceMessageType, MessageType};
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
}

impl Message {
    pub(crate) fn from(message_type: MessageType, bytes: &[u8]) -> Message {
        if bytes.len() == 0 {
            return Message::Empty(message_type);
        }

        use DeviceMessageType::*;
        use MessageType::Device;
        match message_type {
            Device(StateService) => Message::StateService(StateServicePayload::from_bytes(bytes)),
            Device(StateLabel) => Message::StateLabel(StateLabelPayload::from_bytes(bytes)),
            Device(StateLocation) => {
                Message::StateLocation(StateLocationPayload::from_bytes(bytes))
            }
            Device(StateGroup) => Message::StateGroup(StateGroupPayload::from_bytes(bytes)),
            _ => Message::Bytes(message_type, bytes.to_vec()),
            // _ => panic!("Unsupported: {:?}", message_type),
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
        }
    }

    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

trait ClientPayload {
    fn as_bytes(&self) -> &[u8];
}

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
