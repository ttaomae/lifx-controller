use std::collections::HashSet;
use std::io;
use std::net::{SocketAddr, UdpSocket};

use super::protocol::header::*;
use super::protocol::message::*;
use super::protocol::packet::*;
use crate::lifx;

/// Returns information about LIFX devices on the network.
pub(crate) fn discover(socket: &UdpSocket) -> Result<Vec<Device>, io::Error> {
    let device_addresses = get_device_address(&socket)?;

    let mut devices = Vec::new();
    for address in device_addresses {
        let mac_address = address.mac_address;
        let socket_address = address.socket_address;

        let label = get_label(socket, mac_address, socket_address)?;
        let group = get_group(socket, mac_address, socket_address)?;
        let location = get_location(socket, mac_address, socket_address)?;
        let device = Device {
            mac_address: mac_address,
            socket_address: socket_address,
            label: trim_trailing_null(label.label),
            group: trim_trailing_null(group.label),
            location: trim_trailing_null(location.label),
        };
        devices.push(device);
    }
    Result::Ok(devices)
}

/// Trim trailing null bytes from a string.
fn trim_trailing_null(s: String) -> String {
    s.trim_end_matches('\0').to_string()
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct Device {
    mac_address: MacAddress,
    socket_address: SocketAddr,
    label: String,
    group: String,
    location: String,
}

impl Device {
    pub(crate) fn mac_address(&self) -> MacAddress {
        self.mac_address
    }

    pub(crate) fn socket_address(&self) -> SocketAddr {
        self.socket_address
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct DeviceAddress {
    mac_address: MacAddress,
    socket_address: SocketAddr,
}

// Return MAC and socket address of devices by broadcasting a GetService message.
fn get_device_address(socket: &UdpSocket) -> io::Result<HashSet<DeviceAddress>> {
    let get_service = PacketBuilder::with_empty_device_message(DeviceMessageType::GetService)
        .source(1)
        .res_required(true)
        .build();

    let get_service = get_service.as_bytes();
    let broadcast = socket.broadcast()?;

    let mut buf = [0u8; 128];
    let mut device_addresses = HashSet::new();
    socket.set_broadcast(true)?;
    socket.send_to(&get_service, "255.255.255.255:56700")?;
    while let Ok((_, mut addr)) = socket.recv_from(&mut buf) {
        let response = Packet::from(&buf[..]);
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
fn get_label(
    socket: &UdpSocket,
    mac: MacAddress,
    socket_address: SocketAddr,
) -> Result<StateLabelPayload, io::Error> {
    let packet = PacketBuilder::with_empty_device_message(DeviceMessageType::GetLabel)
        .source(1)
        .res_required(true)
        .target(mac)
        .build();

    let message = lifx::send_packet(socket, socket_address, packet)?;
    if let Message::StateLabel(label_payload) = message {
        Result::Ok(label_payload.clone())
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}

/// Return the location for a specific device.
fn get_location(
    socket: &UdpSocket,
    mac: MacAddress,
    socket_address: SocketAddr,
) -> Result<StateLocationPayload, io::Error> {
    let packet = PacketBuilder::with_empty_device_message(DeviceMessageType::GetLocation)
        .source(1)
        .res_required(true)
        .target(mac)
        .build();

    let message = lifx::send_packet(socket, socket_address, packet)?;
    if let Message::StateLocation(location_payload) = &message {
        Result::Ok(location_payload.clone())
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}

/// Return the group for a specific device.
fn get_group(
    socket: &UdpSocket,
    mac: MacAddress,
    socket_address: SocketAddr,
) -> Result<StateGroupPayload, io::Error> {
    let packet = PacketBuilder::with_empty_device_message(DeviceMessageType::GetGroup)
        .source(1)
        .res_required(true)
        .source(123)
        .sequence(111)
        .target(mac)
        .build();

    let message = lifx::send_packet(socket, socket_address, packet)?;
    if let Message::StateGroup(group_payload) = message {
        Result::Ok(group_payload.clone())
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", message),
        ))
    }
}
