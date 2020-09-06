use lifx::{self, client::Client};
use std::{
    collections::HashSet,
    io,
    net::UdpSocket,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use thread::JoinHandle;

fn main() {
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
                println!("{:?}", devices);
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
