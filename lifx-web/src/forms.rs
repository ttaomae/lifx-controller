use std::{collections::HashMap, io::Cursor};

use lifx_client::device::Device;
use rocket::{request::FromForm, response::Responder, Response};
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub(crate) struct Duration {
    pub(crate) duration: u32,
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
pub(crate) struct Hsbk {
    pub(crate) hue: Option<f32>,
    pub(crate) saturation: Option<f32>,
    pub(crate) brightness: Option<f32>,
    pub(crate) kelvin: Option<u16>,
    pub(crate) duration: Option<u32>,
}

#[derive(Copy, Clone)]
pub(crate) enum Selector<'a> {
    AllSelector,
    NoneSelector,
    LabelSelector(&'a str),
    GroupSelector(&'a str),
    LocationSelector(&'a str),
}

impl <'a> Selector<'a> {
    pub(crate) fn parse(s: &str) -> Selector {
        if s.starts_with("label:") {
            let label = &s["label:".len()..];
            Selector::LabelSelector(label)
        } else if s.starts_with("group:") {
            let label = &s["group:".len()..];
            Selector::GroupSelector(label)
        } else if s.starts_with("location:") {
            let location = &s["location:".len()..];
            Selector::LocationSelector(location)
        } else if s.eq("all") {
            Selector::AllSelector
        } else {
            Selector::NoneSelector
        }
    }

    pub(crate) fn filter(self, device: &Device) -> bool {
        match self {
            Selector::AllSelector => true,
            Selector::NoneSelector => false,
            Selector::LabelSelector(ref label) => device.label() == label,
            Selector::GroupSelector(ref group) => device.group() == group,
            Selector::LocationSelector(ref location) => device.location() == location,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Preset {
    label: String,
    actions: Vec<LightAction>,
}

impl Preset {
    pub(crate) fn label(&self) -> String {
        self.label.clone()
    }

    pub(crate) fn actions(&self) -> Vec<LightAction> {
        self.actions.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LightAction {
    selector: String,
    hsbk: Hsbk,
}

impl LightAction {
    pub(crate) fn selector(&self) -> Selector {
        Selector::parse(&self.selector)
    }

    pub(crate) fn hsbk(&self) -> Hsbk {
        self.hsbk.clone()
    }
}
