mod lifx;

use std::io;
use std::{net::UdpSocket, time::Duration};

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_secs(1)))?;
    let devices = lifx::discover(&socket)?;
    println!("{:?}", devices);
    Result::Ok(())
}
