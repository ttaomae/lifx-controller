# LIFX Client
This crate provides a client for communicating with LIFX devices using their LAN protocol.

## Usage
Here is a simple example which finds all devices on the same network, then turns them all on.

```rust
use std::{net::UdpSocket, time::Duration};

fn main() -> io::Result<()> {
    // Create LIFX client.
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
    let mut client = lifx_client::client::Client::new(socket);

    // Find devices.
    let devices = client.discover()?;

    // Turn on all devices.
    for device in devices {
        client.turn_on(&device)?;
    }

    Result::Ok(())
}
```

More examples demonstrating additional features can be found in the `examples/` folder.