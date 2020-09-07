use lifx::color::Color;
use std::io;
use std::{net::UdpSocket, time::Duration};

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
    let mut client = lifx::client::Client::new(socket);
    let devices = client.discover()?;
    for device in devices {
        let wait = Duration::from_millis(1000);

        client.set_color(&device, Color::CYAN.with_brightness(0.45))?;
        std::thread::sleep(wait);

        let pink = Color::RED.with_saturation(0.75);
        client.set_color(&device, pink.with_brightness(0.75))?;
        std::thread::sleep(wait);

        let orange = Color::RED.with_hue(30.0);
        // let orange = Color::rgb(255, 127, 0);
        client.set_color(&device, orange.with_brightness(0.45))?;
        std::thread::sleep(wait);

        // let lime_green = Color::GREEN.plus_degrees(-15.0);
        // client.set_color(&device, lime_green.with_brightness(0.60))?;
        // std::thread::sleep(wait);


        // client.set_color(&device, Color::WHITE.with_brightness(0.30))?;
    }
    Result::Ok(())
}
