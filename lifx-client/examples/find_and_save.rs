use lifx_client::{self, client::Client};
use std::{
    collections::HashSet,
    fs::File,
    io::{self, Write},
    net::UdpSocket,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use thread::JoinHandle;

fn main() {
    let mut args = std::env::args();
    let filename = args.nth(1);
    if filename.is_none() {
        eprintln!("No filename provided.");
        std::process::exit(1);
    }
    let filename = filename.unwrap();
    let path = PathBuf::from(&filename);

    if path.exists() {
        eprintln!("Output file {} already exists.", filename);
        std::process::exit(1)
    }

    println!("Searching for devices. Press [Enter] when all devices have been found.");

    let stop_searching = Arc::new(Mutex::new(false));

    let stop = stop_searching.clone();
    let handle: JoinHandle<io::Result<()>> = thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
        let mut client = Client::new(socket);
        let mut devices = HashSet::new();
        loop {
            let new_devices = client.discover()?;
            for device in new_devices {
                if !devices.contains(&device) {
                    println!("Found {} @ {}. ", device.label(), device.address());
                    devices.insert(device);
                }
            }
            if *stop.lock().unwrap() {
                println!("Saving device addresses to file.");
                let mut file = File::create(path)?;

                for device in devices {
                    let mut address = device.address().to_string();
                    address.push('\n');
                    file.write_all(address.as_bytes())?;
                }
                return io::Result::Ok(());
            }
        }
    });

    // Wait for user to press enter.
    io::stdin().read_line(&mut String::new()).unwrap();
    {
        let mut stop = stop_searching.lock().unwrap();
        *stop = true;
    }

    handle.join().unwrap().unwrap();
}
