use super::{
    device::Device,
    protocol::{
        header::LightMessageType,
        message::{Hsbk, Message, Power, SetColorPayload, SetPowerPayload, StatePayload},
        packet::{PacketBuilder,send_packet,send_packet_no_response},
    },
};
use std::{io, net::UdpSocket};

pub(crate) fn get_state(
    socket: &UdpSocket,
    device: &Device,
    source: u32,
    sequence: u8,
) -> io::Result<StatePayload> {
    let packet = PacketBuilder::with_empty_light_message(LightMessageType::Get)
        .target(device.mac_address())
        .source(source)
        .sequence(sequence)
        .res_required(true)
        .build();

    let response = send_packet(socket, device.socket_address(), packet)?;

    if let Message::State(state_payload) = response {
        Result::Ok(state_payload)
    } else {
        Result::Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Unexpected response. {:?}", response),
        ))
    }
}

pub(crate) fn set_power(
    socket: &UdpSocket,
    device: &Device,
    source: u32,
    sequence: u8,
    power: Power,
    duration: u32,
) -> io::Result<()> {
    let packet = PacketBuilder::new(Message::SetPower(SetPowerPayload::new(power, duration)))
        .target(device.mac_address())
        .source(source)
        .sequence(sequence)
        .build();

    send_packet_no_response(socket, device.socket_address(), packet)?;
    Result::Ok(())
}

pub(crate) fn set_color(
    socket: &UdpSocket,
    device: &Device,
    source: u32,
    sequence: u8,
    color: Hsbk,
    duration: u32,
) -> io::Result<()> {
    let packet = PacketBuilder::new(Message::SetColor(SetColorPayload::new(color, duration)))
        .target(device.mac_address())
        .source(source)
        .sequence(sequence)
        .res_required(true)
        .build();

    send_packet(socket, device.socket_address(), packet)?;
    Result::Ok(())
}
