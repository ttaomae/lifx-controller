use lifx_client::{client::Client, device::Device};
use rocket::{http::Status, response::Responder, Response};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, collections::HashSet, fmt, io, io::Cursor, net::UdpSocket, result,
    sync::Mutex, sync::MutexGuard, time::Duration,
};

use crate::{config::AppConfig, forms::Preset, forms::Selector};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Devices {
    devices: HashSet<JsonDevice>,
}

impl<'r> Responder<'r> for Devices {
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
        if let Ok(body) = serde_json::to_string(&self) {
            Response::build().sized_body(Cursor::new(body)).ok()
        } else {
            std::result::Result::Err(Status::InternalServerError)
        }
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
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
        if let Ok(body) = serde_json::to_string(&self) {
            Response::build().sized_body(Cursor::new(body)).ok()
        } else {
            std::result::Result::Err(Status::InternalServerError)
        }
    }
}

pub(crate) struct LifxController {
    client: Mutex<Client>,
    config: Mutex<AppConfig>,
}

impl LifxController {
    pub(crate) fn new() -> Result<LifxController> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
        let client = Client::new(socket);

        let controller = LifxController {
            client: Mutex::new(client),
            config: Mutex::new(AppConfig::new()),
        };

        controller.update()?;

        Result::Ok(controller)
    }

    pub(crate) fn from_config(config: AppConfig) -> Result<LifxController> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Option::Some(Duration::from_millis(500)))?;
        let mut client = Client::new(socket);

        for device in config.devices() {
            if let Ok(address) = device.parse() {
                client.find_device(address)?;
            } else {
                return Result::Err(Error("Could not find device(s)".to_string()));
            }
        }

        Result::Ok(LifxController {
            client: Mutex::new(client),
            config: Mutex::new(config),
        })
    }

    pub(crate) fn update(&self) -> Result<Devices> {
        let devices = self
            .client()?
            .discover()?
            .iter()
            .map(|d| d.into())
            .collect();
        Result::Ok(Devices { devices })
    }

    pub(crate) fn get_lights(&self) -> Result<Devices> {
        let devices = self
            .client()?
            .get_devices()
            .iter()
            .map(|d| d.into())
            .collect();
        Result::Ok(Devices { devices })
    }

    pub(crate) fn delete_lights(&self) -> Result<()> {
        self.client()?.forget_devices();
        Result::Ok(())
    }

    pub(crate) fn toggle(&self, selector: Selector, duration: u32) -> Result<()> {
        let client = self.client()?;
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_toggle(device, Duration::from_millis(duration as u64))?;
        }

        Result::Ok(())
    }

    pub(crate) fn on(&self, selector: Selector, duration: u32) -> Result<()> {
        let client = self.client()?;
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_on(device, Duration::from_millis(duration as u64))?;
        }

        Result::Ok(())
    }

    pub(crate) fn off(&self, selector: Selector, duration: u32) -> Result<()> {
        let client = self.client()?;
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_off(device, Duration::from_millis(duration as u64))?;
        }
        Result::Ok(())
    }

    pub(crate) fn set_brightness(
        &self,
        selector: Selector,
        brightness: f32,
        duration: u32,
    ) -> Result<()> {
        let client = self.client()?;
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_brightness(
                device,
                brightness,
                Duration::from_millis(duration as u64),
            )?;
        }

        Result::Ok(())
    }

    pub(crate) fn set_temperature(
        &self,
        selector: Selector,
        temperature: u16,
        duration: u32,
    ) -> Result<()> {
        let client = self.client()?;
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            client.transition_temperature(
                device,
                temperature,
                Duration::from_millis(duration as u64),
            )?;
        }

        Result::Ok(())
    }

    pub(crate) fn update_lights(
        &self,
        selector: Selector,
        hue: Option<f32>,
        saturation: Option<f32>,
        brightness: Option<f32>,
        kelvin: Option<u16>,
        duration: u32,
    ) -> Result<()> {
        let client = self.client()?;
        for device in client.get_devices().iter().filter(|d| selector.filter(d)) {
            let mut color = client.get_color(device)?;

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
                        client.transition_on(device, duration)?;
                    }
                }
                client.transition_color(device, color, duration)?;
            } else if let Some(kelvin) = kelvin {
                if let Some(brightness) = brightness {
                    client
                        .transition_temperature_brightness(device, kelvin, brightness, duration)?;
                    if brightness > 0.0 {
                        client.transition_on(device, duration)?;
                    }
                }
                client.transition_temperature(device, kelvin, duration)?;
            }
        }

        Result::Ok(())
    }

    pub(crate) fn presets(&self) -> Result<Presets> {
        let presets = self.config()?.presets();
        Result::Ok(Presets { presets })
    }

    pub(crate) fn set_preset(&self, label: String, preset: Preset) -> Result<()> {
        self.config()?.set_preset(label, preset);
        Result::Ok(())
    }

    pub(crate) fn execute_preset(&self, label: String) -> Result<()> {
        if let Some(preset) = self.config()?.get_preset(label) {
            for action in preset.actions() {
                let selector = action.selector();
                let hsbk = action.hsbk();
                let duration = action.duration().unwrap_or(0);
                self.update_lights(
                    selector,
                    hsbk.hue,
                    hsbk.saturation,
                    hsbk.brightness,
                    hsbk.kelvin,
                    duration,
                )?;
            }
        }

        Result::Ok(())
    }

    fn client(&self) -> Result<MutexGuard<Client>> {
        self.client
            .lock()
            .map_err(|_| Error("Could not acquire client lock".to_string()))
    }

    fn config(&self) -> Result<MutexGuard<AppConfig>> {
        self.config
            .lock()
            .map_err(|_| Error("Could not acquire config lock".to_string()))
    }
}

pub(crate) type Result<T> = result::Result<T, Error>;
#[derive(Debug)]
pub(crate) struct Error(pub(crate) String);

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error(e.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error(s)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
