use super::{header::*, message::*};

/// A LIFX packet.
#[derive(Debug)]
pub(crate) struct Packet {
    frame: Frame,
    pub(crate) frame_address: FrameAddress,
    protocol_header: ProtocolHeader,
    message: Message,
}

impl Packet {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.frame.as_bytes());
        bytes.extend(self.frame_address.as_bytes());
        // protocol header: 8 bytes reserved, 2 byte type, 2 bytes reserved
        bytes.extend(&[0u8; 8]);
        bytes.extend(&self.message.message_type().value().to_le_bytes());
        bytes.extend(&[0u8; 2]);
        bytes.extend(self.message.as_bytes());
        bytes
    }

    pub(crate) fn message<'a>(&'a self) -> &'a Message {
        &self.message
    }
}

impl From<&[u8]> for Packet {
    fn from(bytes: &[u8]) -> Self {
        let frame = Frame::from(&bytes[0..8]);
        let frame_address = FrameAddress::from(&bytes[8..24]);
        let protocol_header = ProtocolHeader::from(&bytes[24..36]);
        let payload_size = frame.size as usize - 36;
        let message = Message::from(protocol_header.message_type, &bytes[36..36 + payload_size]);
        Packet {
            frame,
            frame_address,
            protocol_header,
            message,
        }
    }
}

/// A LIFX packet builder.
pub(crate) struct PacketBuilder {
    tagged: bool,
    source: u32,
    target: MacAddress,
    res_required: bool,
    ack_required: bool,
    sequence: u8,
    message: Message,
}

impl PacketBuilder {
    pub(crate) fn new(message: Message) -> PacketBuilder {
        PacketBuilder {
            tagged: true,
            source: 0u32,
            target: MacAddress { address: [0u8; 6] },
            res_required: false,
            ack_required: false,
            sequence: 0u8,
            message,
        }
    }

    pub(crate) fn with_empty_device_message(message_type: DeviceMessageType) -> PacketBuilder {
        Self::new(Message::Empty(MessageType::Device(message_type)))
    }

    pub(crate) fn with_empty_light_message(message_type: LightMessageType) -> PacketBuilder {
        Self::new(Message::Empty(MessageType::Light(message_type)))
    }

    pub(crate) fn source(mut self, source: u32) -> Self {
        self.source = source;
        self
    }

    pub(crate) fn target(mut self, target: MacAddress) -> Self {
        self.target = target;
        self.tagged = false;
        self
    }

    pub(crate) fn res_required(mut self, res_required: bool) -> Self {
        self.res_required = res_required;
        self
    }

    pub(crate) fn ack_required(mut self, ack_required: bool) -> Self {
        self.ack_required = ack_required;
        self
    }

    pub(crate) fn sequence(mut self, sequence: u8) -> Self {
        self.sequence = sequence;
        self
    }

    pub(crate) fn build(self) -> Packet {
        // header == 36 bytes.
        let size = 36 + self.message.as_bytes().len();
        Packet {
            frame: Frame {
                size: size as u16,
                addressable: true,
                tagged: self.tagged,
                source: self.source,
            },
            frame_address: FrameAddress {
                target: self.target,
                res_required: self.res_required,
                ack_required: self.ack_required,
                sequence: self.sequence,
            },
            protocol_header: ProtocolHeader {
                message_type: self.message.message_type(),
            },
            message: self.message,
        }
    }
}
