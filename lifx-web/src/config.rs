use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct AppConfig {
    devices: Option<Vec<String>>,
}

impl AppConfig {
    pub(crate) fn new() -> AppConfig {
        AppConfig {
            devices: Option::None,
        }
    }

    pub(crate) fn devices(&self) -> Vec<String> {
        if let Some(devices) = &self.devices {
            devices.clone()
        } else {
            Vec::new()
        }
    }
}
