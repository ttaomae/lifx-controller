mod lifx;

use std::io;
use std::{net::UdpSocket, time::Duration};

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
    let mut client = lifx::client::Client::new(socket);
    let devices = client.discover()?;

    for device in devices {
        client.toggle_power(&device, 0)?;
    }
    Result::Ok(())
}
