use lifx_client::{client::Client, device::Device};
use rocket::{response::Responder, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Cursor, net::UdpSocket, sync::Mutex, time::Duration};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Devices {
    devices: HashSet<JsonDevice>,
}

impl<'r> Responder<'r> for Devices {
    fn respond_to(self, request: &rocket::Request) -> rocket::response::Result<'r> {
        let body = serde_json::to_string(&self).unwrap();
        Response::build().sized_body(Cursor::new(body)).ok()
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct JsonDevice {
    label: String,
    group: String,
    location: String,
}

impl From<&Device> for JsonDevice {
    fn from(device: &Device) -> Self {
        JsonDevice {
            label: device.label().to_string(),
            group: device.group().to_string(),
            location: device.location().to_string(),
        }
    }
}

pub(crate) struct LifxController {
    client: Mutex<Client>,
}

impl LifxController {
    pub(crate) fn new() -> LifxController {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Option::Some(Duration::from_millis(500)));
        let mut client = Client::new(socket);
        client.discover();
        LifxController {
            client: Mutex::new(client),
        }
    }

    pub(crate) fn discover(&self) -> Devices {
        let mut client = self.client.lock().unwrap();
        let devices = client
            .discover()
            .unwrap()
            .iter()
            .map(|d| d.into())
            .collect();
        Devices { devices }
    }

    pub(crate) fn get_lights(&self) -> Devices {
        let client = self.client.lock().unwrap();
        let devices = client.get_devices().iter().map(|d| d.into()).collect();
        Devices { devices }
    }
}
