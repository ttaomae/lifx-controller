use lifx_client::device::Device;
use rocket::request::FromForm;

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

#[derive(Debug, FromForm)]
pub(crate) struct Hsb {
    pub(crate) hue: Option<f32>,
    pub(crate) saturation: Option<f32>,
    pub(crate) brightness: Option<f32>,
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
