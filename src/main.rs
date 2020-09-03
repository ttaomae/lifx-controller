mod lifx;

use lifx::protocol::message::Power;
use std::io;
use std::{net::UdpSocket, time::Duration};

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_secs(1)))?;
    let devices = lifx::device::discover(&socket)?;
    println!("{:?}", devices);

    for device in devices {
        let state = lifx::light::get_state(&socket, &device)?;
        match state.get_power() {
            Power::On => lifx::light::set_power(&socket, &device, Power::Off, 0)?,
            Power::Off => lifx::light::set_power(&socket, &device, Power::On, 0)?,
            Power::Unknown(n) => panic!("Unkown power {}", n),
        }
    }
    Result::Ok(())
}
