use lifx::{client::Client, device::DeviceAddress};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader, Read},
    net::UdpSocket,
    path::PathBuf,
    time::Duration,
};

fn main() -> io::Result<()> {
    let mut args = std::env::args();
    let filename = args.nth(1);
    if filename.is_none() {
        eprintln!("No filename provided.");
        std::process::exit(1);
    }
    let filename = filename.unwrap();
    let path = PathBuf::from(&filename);

    if !path.exists() {
        eprintln!("Input file {} not found.", filename);
        std::process::exit(1)
    }

    let file = File::open(path);
    if file.is_err() {
        eprintln!("Could not open input file {}.", filename);
        std::process::exit(1)
    }
    let file = file?;
    let reader = BufReader::new(file);
    let mut devices = HashSet::new();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
    let mut client = Client::new(socket);

    for line in reader.lines() {
        let device_address: DeviceAddress = line?.parse().unwrap();
        let device = client.find_device(device_address)?;
        devices.insert(device);
    }

    for device in devices {
        client.toggle_power(&device, Duration::from_secs(0))?;
    }

    Result::Ok(())
}
