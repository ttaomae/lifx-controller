use std::collections::HashSet;
use std::io;
use std::{
    fmt,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    str::FromStr,
};

use super::protocol::header::*;
use super::protocol::message::*;
use super::protocol::packet::*;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Device {
    address: DeviceAddress,
    label: String,
    group: String,
    location: String,
}

impl Device {
    pub(crate) fn new(
        address: DeviceAddress,
        label: String,
        group: String,
        location: String,
    ) -> Device {
        Device {
            address,
            label,
            group,
            location,
        }
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn group(&self) -> &String {
        &self.group
    }

    pub fn location(&self) -> &String {
        &self.location
    }
    pub fn address(&self) -> DeviceAddress {
        self.address
    }

    pub(crate) fn mac_address(&self) -> MacAddress {
        self.address.mac_address
    }

    pub(crate) fn socket_address(&self) -> SocketAddr {
        self.address.socket_address
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct DeviceAddress {
    mac_address: MacAddress,
    socket_address: SocketAddr,
}

impl DeviceAddress {
    pub(crate) fn mac_address(&self) -> MacAddress {
        self.mac_address
    }

    pub(crate) fn socket_address(&self) -> SocketAddr {
        self.socket_address
    }
}

impl fmt::Display for DeviceAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}#{}", self.mac_address, self.socket_address)
    }
}

// Inverse of Display implementation.
impl FromStr for DeviceAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains('#') {
            return Result::Err(String::from(
                "String must be in the format <mac_address>#<socket_address>.",
            ));
        }
        let mut parts = s.split('#');

        // Since we already checked that the input contains a separator, `parts.next()` should return at least two elements, so it is safe to unwrap.
        let mac_string = parts.next().unwrap();
        let socket_string = parts.next().unwrap();

        let mac_address: MacAddress = mac_string
            .parse()
            .map_err(|_| format!("Could not parse MAC address {}.", mac_string))?;
        let socket_address: SocketAddr = socket_string
            .parse()
            .map_err(|_| format!("Could not parse socket address {}", socket_string))?;

        if parts.next().is_some() {
            return Result::Err(String::from(
                "String must be in the format <mac_address>#<socket_address>.",
            ));
        }

        Result::Ok(DeviceAddress {
            mac_address,
            socket_address,
        })
    }
}

// Return MAC and socket address of devices by broadcasting a GetService message.
pub(crate) fn get_device_address(
    socket: &UdpSocket,
    source: u32,
    sequence: u8,
) -> io::Result<HashSet<DeviceAddress>> {
    let get_service = PacketBuilder::with_empty_device_message(DeviceMessageType::GetService)
        .source(source)
        .sequence(sequence)
        .res_required(true)
        .build();

    let get_service = get_service.as_bytes();
    let broadcast = socket.broadcast()?;

    let mut buf = [0u8; 128];
    let mut device_addresses = HashSet::new();
    socket.set_broadcast(true)?;

    socket.send_to(&get_service, (Ipv4Addr::BROADCAST, 56700))?;
    while let Ok((n, mut addr)) = socket.recv_from(&mut buf) {
        let response = Packet::from(&buf[..n]);
        if let Message::StateService(service_payload) = response.message() {
            let port = service_payload.port();
            addr.set_port(port);
        } else {
            return Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unexpected response. {:?}", &buf[..]),
            ));
        }

        let mac_address = response.frame_address.target;
        device_addresses.insert(DeviceAddress {
            mac_address,
            socket_address: addr,
        });
    }

    socket.set_broadcast(broadcast)?;
    io::Result::Ok(device_addresses)
}

/// Return the label for a specific device.
pub(crate) fn get_label(
    socket: &UdpSocket,
    device_address: &DeviceAddress,
    source: u32,
    sequence: u8,
) -> Result<StateLabelPayload, io::Error> {
    let packet = PacketBuilder::with_empty_device_message(DeviceMessageType::GetLabel)
        .source(source)
        .sequence(sequence)
        .res_required(true)
        .target(device_address.mac_address())
        .build();

    let message = send_packet(socket, device_address.socket_address(), packet)?;
    if let Message::StateLabel(label_payload) = message {
        Result::Ok(label_payload)
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}

/// Return the location for a specific device.
pub(crate) fn get_location(
    socket: &UdpSocket,
    device_address: &DeviceAddress,
    source: u32,
    sequence: u8,
) -> Result<StateLocationPayload, io::Error> {
    let packet = PacketBuilder::with_empty_device_message(DeviceMessageType::GetLocation)
        .source(source)
        .sequence(sequence)
        .res_required(true)
        .target(device_address.mac_address())
        .build();

    let message = send_packet(socket, device_address.socket_address(), packet)?;
    if let Message::StateLocation(location_payload) = message {
        Result::Ok(location_payload)
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}

/// Return the group for a specific device.
pub(crate) fn get_group(
    socket: &UdpSocket,
    device_address: &DeviceAddress,
    source: u32,
    sequence: u8,
) -> Result<StateGroupPayload, io::Error> {
    let packet = PacketBuilder::with_empty_device_message(DeviceMessageType::GetGroup)
        .source(1)
        .res_required(true)
        .source(source)
        .sequence(sequence)
        .target(device_address.mac_address())
        .build();

    let message = send_packet(socket, device_address.socket_address(), packet)?;
    if let Message::StateGroup(group_payload) = message {
        Result::Ok(group_payload)
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}
