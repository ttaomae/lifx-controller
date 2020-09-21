use lifx_client::{client::Client, device::Device};
use rocket::{response::Responder, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io::Cursor, net::UdpSocket, sync::Mutex, time::Duration, collections::HashMap};

use crate::{config::AppConfig, forms::Brightness, forms::Selector, forms::Preset};

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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Presets {
    presets: HashMap<String, Preset>,
}

impl<'r> Responder<'r> for Presets {
    fn respond_to(self, request: &rocket::Request) -> rocket::response::Result<'r> {
        let body = serde_json::to_string(&self).unwrap();
        Response::build().sized_body(Cursor::new(body)).ok()
    }
}

pub(crate) struct LifxController {
    client: Mutex<Client>,
    config: Mutex<AppConfig>,
}

impl LifxController {
    pub(crate) fn new() -> LifxController {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Option::Some(Duration::from_millis(500)));
        let mut client = Client::new(socket);

        let controller = LifxController {
            client: Mutex::new(client),
            config: Mutex::new(AppConfig::new()),
        };

        controller.update();

        controller
    }

    pub(crate) fn from_config(config: AppConfig) -> LifxController {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_read_timeout(Option::Some(Duration::from_millis(500)));
        let mut client = Client::new(socket);

        for device in config.devices() {
            client.find_device(device.parse().unwrap());
        }

        LifxController {
            client: Mutex::new(client),
            config: Mutex::new(config),
        }
    }

    pub(crate) fn update(&self) -> Devices {
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

    pub(crate) fn delete_lights(&self) {
        let mut client = self.client.lock().unwrap();
        client.forget_devices();
    }

    pub(crate) fn toggle(&self, selector: Selector, duration: u32) {
        let client = self.client.lock().unwrap();
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_toggle(device, Duration::from_millis(duration as u64));
        }
    }

    pub(crate) fn on(&self, selector: Selector, duration: u32) {
        let client = self.client.lock().unwrap();
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_on(device, Duration::from_millis(duration as u64));
        }
    }

    pub(crate) fn off(&self, selector: Selector, duration: u32) {
        let client = self.client.lock().unwrap();
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_off(device, Duration::from_millis(duration as u64));
        }
    }

    pub(crate) fn set_brightness(&self,  selector: Selector, brightness: f32, duration: u32) {
        let client = self.client.lock().unwrap();
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_brightness(
                device,
                brightness,
                Duration::from_millis(duration as u64),
            );
        }
    }

    pub(crate) fn set_temperature(&self,  selector: Selector, temperature: u16, duration: u32) {
        let client = self.client.lock().unwrap();
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_temperature(
                device,
                temperature,
                Duration::from_millis(duration as u64),
            );
        }
    }

    pub(crate) fn update_lights(
        &self,
        selector: Selector,
        hue: Option<f32>,
        saturation: Option<f32>,
        brightness: Option<f32>,
        kelvin: Option<u16>,
        duration: u32,
    ) {
        let client = self.client.lock().unwrap();
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            let mut color = client.get_color(device).unwrap();

            let mut set_color = false;
            if let Some(hue) = hue {
                color = color.with_hue(hue);
                set_color = true;
            }

            if let Some(saturation) = saturation {
                color = color.with_saturation(saturation);
                set_color = true;
            }

            if let Some(brightness) = brightness {
                color = color.with_brightness(brightness);
                set_color = true;
            }

            let duration = Duration::from_millis(duration as u64);

            if set_color {
                if let Some(brightness) = brightness {
                    if brightness > 0.0 {
                        client.transition_on(device, duration);
                    }
                }
                client.transition_color(device, color, duration);
            } else if let Some(kelvin) = kelvin {
                if let Some(brightness) = brightness {
                    client.transition_temperature_brightness(device, kelvin, brightness, duration);
                    if brightness > 0.0 {
                        client.transition_on(device, duration);
                    }
                }
                client.transition_temperature(device, kelvin, duration);
            }
        }
    }

    pub(crate) fn presets(&self) -> Presets {
        let config = self.config.lock().unwrap();
        let presets = config.presets().clone();
        Presets { presets }
    }

    pub(crate) fn set_preset(&self, label: String, preset: Preset) {
        let mut config = self.config.lock().unwrap();
        config.set_preset(label, preset);
    }

    pub(crate) fn execute_preset(&self, label: String) {
        let config = self.config.lock().unwrap();
        if let Some(preset) = config.get_preset(label) {
            for action in preset.actions() {
                let selector = action.selector();
                let hsbk = action.hsbk();
                let duration = action.duration().unwrap_or(0);
                self.update_lights(selector, hsbk.hue, hsbk.saturation, hsbk.brightness, hsbk.kelvin, duration);
            }
        }
    }
}
