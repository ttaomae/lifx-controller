use std::{fmt, convert::TryInto, str::FromStr};

// A LIFX packet header frame.
#[derive(Debug, Copy, Clone)]
pub(crate) struct Frame {
    pub(crate) size: u16,
    // protocol - 12 bits, must be 1024 == 0x400
    pub(crate) addressable: bool,
    pub(crate) tagged: bool,
    // origin - 2 bits - must be 0
    pub(crate) source: u32,
}

impl Frame {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.size.to_le_bytes().iter());
        let mut protocol = 1024u16;
        protocol |= (self.addressable as u16) << 12;
        protocol |= (self.tagged as u16) << 13;
        bytes.extend(protocol.to_le_bytes().iter());
        bytes.extend(self.source.to_le_bytes().iter());
        bytes
    }
}

impl From<&[u8]> for Frame {
    fn from(bytes: &[u8]) -> Self {
        if bytes.len() != 8 {
            panic!();
        }
        let size = u16::from_le_bytes(bytes[0..2].try_into().expect(""));
        let protocol = u16::from_le_bytes(bytes[2..4].try_into().expect(""));
        if (protocol & 0x0fff) != 1024 {
            panic!();
        }
        if (protocol & 0b1100_0000_0000_0000) != 0 {
            panic!("{}", protocol);
        }
        let addressable = (protocol & 0b0001_0000_0000_0000) != 0;
        let tagged = (protocol & 0b0010_0000_0000_0000) != 0;
        let source = u32::from_le_bytes(bytes[4..8].try_into().expect(""));

        Frame {
            size,
            addressable,
            tagged,
            source,
        }
    }
}

// A LIFX packet header frame address.
#[derive(Debug, Copy, Clone)]
pub(crate) struct FrameAddress {
    // target consists of a 6 byte MAC address, plus two 0 bytes.
    // `MacAddress.as_bytes()` returns only 6 bytes.
    pub(crate) target: MacAddress,
    // reserved - 48 bits, must all be zero
    pub(crate) res_required: bool,
    pub(crate) ack_required: bool,
    // reserved - 6 bits
    pub(crate) sequence: u8,
}

impl FrameAddress {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.target.as_bytes();
        // Combine two 0 bytes from `target` with following reserved 48 bits (6 bytes).
        bytes.extend([0u8; 8].iter());
        bytes.push(0x00 | (self.res_required as u8) | ((self.ack_required as u8) << 1));
        bytes.push(self.sequence);
        bytes
    }
}

impl From<&[u8]> for FrameAddress {
    fn from(bytes: &[u8]) -> Self {
        if bytes.len() != 16 {
            panic!();
        }

        let target = MacAddress::from(&bytes[0..6]);
        // Bytes 6-7 are last two bytes of target; should be zero.
        // Bytes 8-13 are reserved; must all be zero.
        let res_ack = bytes[14];
        let res_required = (res_ack & 0b0000_0001) != 0;
        let ack_required = (res_ack & 0b0000_0010) != 0;
        let sequence = bytes[15];

        FrameAddress {
            target,
            res_required,
            ack_required,
            sequence,
        }
    }
}

/// A device MAC address.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct MacAddress {
    pub(crate) address: [u8; 6],
}

impl fmt::Display for MacAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{:02x?}:{:02x?}:{:02x?}:{:02x?}:{:02x?}:{:02x?}",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5]
        )
    }
}

impl FromStr for MacAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut address = [0u8; 6];

        for (n, byte) in s.split(':').into_iter().enumerate() {
            if n >= 6 {
                return Result::Err(format!("Could not parse MAC address: {}.", s));
            }
            address[n] = u8::from_str_radix(&byte, 16).map_err(|_| format!("Could not parse MAC Address: {}.", s))?;
        }

        Result::Ok(MacAddress {
            address
        })
    }
}

impl MacAddress {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        self.address.to_vec()
    }
}

impl From<&[u8]> for MacAddress {
    fn from(bytes: &[u8]) -> Self {
        if bytes.len() != 6 {
            panic!();
        }
        MacAddress {
            address: bytes[0..6].try_into().expect(""),
        }
    }
}

/// A LIFX packet protocol header.
#[derive(Debug)]
pub(crate) struct ProtocolHeader {
    pub(crate) message_type: MessageType,
}

impl From<&[u8]> for ProtocolHeader {
    fn from(bytes: &[u8]) -> Self {
        if bytes.len() != 12 {
            panic!();
        }

        let message_value = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
        ProtocolHeader {
            message_type: MessageType::from_value(message_value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MessageType {
    Device(DeviceMessageType),
    Light(LightMessageType),
}

impl MessageType {
    pub(crate) fn from_value(value: u16) -> Self {
        match value {
            2 => MessageType::Device(DeviceMessageType::GetService),
            3 => MessageType::Device(DeviceMessageType::StateService),
            12 => MessageType::Device(DeviceMessageType::GetHostInfo),
            13 => MessageType::Device(DeviceMessageType::StateHostInfo),
            14 => MessageType::Device(DeviceMessageType::GetHostFirmware),
            15 => MessageType::Device(DeviceMessageType::StateHostFirmware),
            16 => MessageType::Device(DeviceMessageType::GetWifiInfo),
            17 => MessageType::Device(DeviceMessageType::StateWifiInfo),
            18 => MessageType::Device(DeviceMessageType::GetWifiFirmware),
            19 => MessageType::Device(DeviceMessageType::StateWifiFirmware),
            20 => MessageType::Device(DeviceMessageType::GetPower),
            21 => MessageType::Device(DeviceMessageType::SetPower),
            22 => MessageType::Device(DeviceMessageType::StatePower),
            23 => MessageType::Device(DeviceMessageType::GetLabel),
            24 => MessageType::Device(DeviceMessageType::SetLabel),
            25 => MessageType::Device(DeviceMessageType::StateLabel),
            32 => MessageType::Device(DeviceMessageType::GetVersion),
            33 => MessageType::Device(DeviceMessageType::StateVersion),
            34 => MessageType::Device(DeviceMessageType::GetInfo),
            35 => MessageType::Device(DeviceMessageType::StateInfo),
            45 => MessageType::Device(DeviceMessageType::Acknowledgement),
            48 => MessageType::Device(DeviceMessageType::GetLocation),
            49 => MessageType::Device(DeviceMessageType::SetLocation),
            50 => MessageType::Device(DeviceMessageType::StateLocation),
            51 => MessageType::Device(DeviceMessageType::GetGroup),
            52 => MessageType::Device(DeviceMessageType::SetGroup),
            53 => MessageType::Device(DeviceMessageType::StateGroup),
            59 => MessageType::Device(DeviceMessageType::EchoRequest),

            101 => MessageType::Light(LightMessageType::Get),
            102 => MessageType::Light(LightMessageType::SetColor),
            103 => MessageType::Light(LightMessageType::SetWaveform),
            119 => MessageType::Light(LightMessageType::SetWaveformOptional),
            107 => MessageType::Light(LightMessageType::State),
            116 => MessageType::Light(LightMessageType::GetPower),
            117 => MessageType::Light(LightMessageType::SetPower),
            118 => MessageType::Light(LightMessageType::StatePower),
            120 => MessageType::Light(LightMessageType::GetInfrared),
            121 => MessageType::Light(LightMessageType::StateInfrared),
            122 => MessageType::Light(LightMessageType::SetInfrared),
            _ => panic!("Unknown message type: {}", value),
        }
    }

    pub(crate) fn value(&self) -> u16 {
        match self {
            MessageType::Device(device_message_type) => match *device_message_type {
                DeviceMessageType::GetService => 2,
                DeviceMessageType::StateService => 3,
                DeviceMessageType::GetHostInfo => 12,
                DeviceMessageType::StateHostInfo => 13,
                DeviceMessageType::GetHostFirmware => 14,
                DeviceMessageType::StateHostFirmware => 15,
                DeviceMessageType::GetWifiInfo => 16,
                DeviceMessageType::StateWifiInfo => 17,
                DeviceMessageType::GetWifiFirmware => 18,
                DeviceMessageType::StateWifiFirmware => 19,
                DeviceMessageType::GetPower => 20,
                DeviceMessageType::SetPower => 21,
                DeviceMessageType::StatePower => 22,
                DeviceMessageType::GetLabel => 23,
                DeviceMessageType::SetLabel => 24,
                DeviceMessageType::StateLabel => 25,
                DeviceMessageType::GetVersion => 32,
                DeviceMessageType::StateVersion => 33,
                DeviceMessageType::GetInfo => 34,
                DeviceMessageType::StateInfo => 35,
                DeviceMessageType::Acknowledgement => 45,
                DeviceMessageType::GetLocation => 48,
                DeviceMessageType::SetLocation => 49,
                DeviceMessageType::StateLocation => 50,
                DeviceMessageType::GetGroup => 51,
                DeviceMessageType::SetGroup => 52,
                DeviceMessageType::StateGroup => 53,
                DeviceMessageType::EchoRequest => 59,
            },
            MessageType::Light(light_message_type) => match *light_message_type {
                LightMessageType::Get => 101,
                LightMessageType::SetColor => 102,
                LightMessageType::SetWaveform => 103,
                LightMessageType::SetWaveformOptional => 119,
                LightMessageType::State => 107,
                LightMessageType::GetPower => 116,
                LightMessageType::SetPower => 117,
                LightMessageType::StatePower => 118,
                LightMessageType::GetInfrared => 120,
                LightMessageType::StateInfrared => 121,
                LightMessageType::SetInfrared => 122,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DeviceMessageType {
    GetService,
    StateService,
    GetHostInfo,
    StateHostInfo,
    GetHostFirmware,
    StateHostFirmware,
    GetWifiInfo,
    StateWifiInfo,
    GetWifiFirmware,
    StateWifiFirmware,
    GetPower,
    SetPower,
    StatePower,
    GetLabel,
    SetLabel,
    StateLabel,
    GetVersion,
    StateVersion,
    GetInfo,
    StateInfo,
    Acknowledgement,
    GetLocation,
    SetLocation,
    StateLocation,
    GetGroup,
    SetGroup,
    StateGroup,
    EchoRequest,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LightMessageType {
    Get,
    SetColor,
    SetWaveform,
    SetWaveformOptional,
    State,
    GetPower,
    SetPower,
    StatePower,
    GetInfrared,
    StateInfrared,
    SetInfrared,
}
