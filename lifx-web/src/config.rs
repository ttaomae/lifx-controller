use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::forms::Preset;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct AppConfig {
    devices: Option<Vec<String>>,
    presets: HashMap<String, Preset>,
}

impl AppConfig {
    pub(crate) fn new() -> AppConfig {
        AppConfig {
            devices: Option::None,
            presets: HashMap::new(),
        }
    }

    pub(crate) fn devices(&self) -> Vec<String> {
        if let Some(devices) = &self.devices {
            devices.clone()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn presets(&self) -> HashMap<String, Preset> {
        self.presets.clone()
    }

    pub(crate) fn set_preset(&mut self, label: String, preset: Preset) {
        self.presets.insert(label, preset);
    }

    pub(crate) fn get_preset(&self, label: String) -> Option<&Preset> {
        self.presets.get(&label)
    }
}
