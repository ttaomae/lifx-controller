use protocol::{message::Message, packet::Packet};
use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

pub(crate) mod device;
pub(crate) mod light;
pub(crate) mod protocol;

fn send_packet(
    socket: &UdpSocket,
    socket_address: SocketAddr,
    packet: Packet,
) -> io::Result<Message> {
    let mut buf = [0u8; 128];
    let broadcast = socket.broadcast()?;
    socket.set_broadcast(false)?;
    socket.send_to(&packet.as_bytes(), socket_address)?;
    let n_bytes = socket.recv(&mut buf)?;
    let response = Packet::from(&buf[..n_bytes]);
    let message = response.message();

    socket.set_broadcast(broadcast)?;
    Result::Ok(message.clone())
}

fn send_packet_no_response(
    socket: &UdpSocket,
    socket_address: SocketAddr,
    packet: Packet,
) -> io::Result<()> {
    let broadcast = socket.broadcast()?;
    socket.set_broadcast(false)?;
    socket.send_to(&packet.as_bytes(), socket_address)?;
    socket.set_broadcast(broadcast)?;
    Result::Ok(())
}
