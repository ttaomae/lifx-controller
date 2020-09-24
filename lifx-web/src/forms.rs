use lifx_client::device::Device;
use rocket::request::FromForm;
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub(crate) struct Duration {
    pub(crate) duration: Option<u32>,
}

#[derive(FromForm)]
pub(crate) struct Brightness {
    pub(crate) brightness: f32,
    pub(crate) duration: Option<u32>,
}

#[derive(FromForm)]
pub(crate) struct Temperature {
    pub(crate) kelvin: u16,
    pub(crate) duration: Option<u32>,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub(crate) struct HsbkDuration {
    pub(crate) hue: Option<f32>,
    pub(crate) saturation: Option<f32>,
    pub(crate) brightness: Option<f32>,
    pub(crate) kelvin: Option<u16>,
    pub(crate) duration: Option<u32>,
}

#[derive(Copy, Clone)]
pub(crate) enum Selector<'a> {
    All,
    None,
    Label(&'a str),
    Group(&'a str),
    Location(&'a str),
}

impl<'a> Selector<'a> {
    pub(crate) fn parse(s: &str) -> Selector {
        if s.starts_with("label:") {
            let label = &s["label:".len()..];
            Selector::Label(label)
        } else if s.starts_with("group:") {
            let label = &s["group:".len()..];
            Selector::Group(label)
        } else if s.starts_with("location:") {
            let location = &s["location:".len()..];
            Selector::Location(location)
        } else if s.eq("all") {
            Selector::All
        } else {
            Selector::None
        }
    }

    pub(crate) fn filter(self, device: &Device) -> bool {
        match self {
            Selector::All => true,
            Selector::None => false,
            Selector::Label(ref label) => device.label() == label,
            Selector::Group(ref group) => device.group() == group,
            Selector::Location(ref location) => device.location() == location,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Preset {
    actions: Vec<LightAction>,
}

impl Preset {
    pub(crate) fn actions(&self) -> Vec<LightAction> {
        self.actions.clone()
    }
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub(crate) struct Hsbk {
    pub(crate) hue: Option<f32>,
    pub(crate) saturation: Option<f32>,
    pub(crate) brightness: Option<f32>,
    pub(crate) kelvin: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LightAction {
    selector: String,
    duration: Option<u32>,
    hsbk: Hsbk,
}

impl LightAction {
    pub(crate) fn selector(&self) -> Selector {
        Selector::parse(&self.selector)
    }

    pub(crate) fn duration(&self) -> Option<u32> {
        self.duration
    }

    pub(crate) fn hsbk(&self) -> Hsbk {
        self.hsbk.clone()
    }
}
