mod lifx;

use lifx::color::Color;
use std::io;
use std::{net::UdpSocket, time::Duration};

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
    let mut client = lifx::client::Client::new(socket);
    let devices = client.discover()?;
    for device in devices {
        // Set to warm white at 50% brightness.
        client.turn_on(&device, Duration::from_secs(0))?;
        client.set_brightness(&device, 0.5, Duration::from_secs(0))?;
        client.set_temperature(&device, 3000, Duration::from_secs(0))?;

        std::thread::sleep(Duration::from_millis(2000));

        // Cycle through colors.
        let mut color = Color::rgb(255, 0, 0);
        for _ in 0..36 {
            client.set_color(&device, color, Duration::from_millis(500))?;
            std::thread::sleep(Duration::from_millis(500));
            color = color.add_degrees(10.0);
        }

        client.toggle_power(&device, Duration::from_secs(0))?;
    }
    Result::Ok(())
}
