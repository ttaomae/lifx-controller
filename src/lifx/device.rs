use std::collections::HashSet;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use super::protocol::header::*;
use super::protocol::message::*;
use super::protocol::packet::*;
use crate::lifx;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub(crate) struct Device {
    mac_address: MacAddress,
    socket_address: SocketAddr,
    label: String,
    group: String,
    location: String,
}

impl Device {
    pub(crate) fn new(
        mac_address: MacAddress,
        socket_address: SocketAddr,
        label: String,
        group: String,
        location: String,
    ) -> Device {
        Device {
            mac_address,
            socket_address,
            label,
            group,
            location,
        }
    }

    pub(crate) fn mac_address(&self) -> MacAddress {
        self.mac_address
    }

    pub(crate) fn socket_address(&self) -> SocketAddr {
        self.socket_address
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct DeviceAddress {
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

    let message = lifx::send_packet(socket, device_address.socket_address(), packet)?;
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

    let message = lifx::send_packet(socket, device_address.socket_address(), packet)?;
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

    let message = lifx::send_packet(socket, device_address.socket_address(), packet)?;
    if let Message::StateGroup(group_payload) = message {
        Result::Ok(group_payload)
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}
