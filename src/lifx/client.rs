use super::{
    color::Color,
    device::{self, Device},
    light,
    protocol::message::{Power, StatePayload},
};
use std::{cell::Cell, collections::HashSet, io, net::UdpSocket, time::Duration};

const ZERO_DURATION: Duration = Duration::from_secs(0);
const MAX_DURATION: Duration = Duration::from_millis(u32::MAX as u64);

pub(crate) struct Client {
    socket: UdpSocket,
    source: u32,
    sequence: Cell<u8>,
    devices: HashSet<Device>,
}

impl Client {
    pub(crate) fn new(socket: UdpSocket) -> Client {
        Client {
            socket,
            source: rand::random::<u32>(),
            sequence: Cell::new(0),
            devices: HashSet::new(),
        }
    }

    /// Returns information about LIFX devices on the network.
    pub(crate) fn discover(&mut self) -> Result<HashSet<Device>, io::Error> {
        let device_addresses =
            device::get_device_address(&self.socket, self.source, self.sequence())?;

        for address in device_addresses {
            let mac_address = address.mac_address();
            let socket_address = address.socket_address();

            let label = device::get_label(&self.socket, &address, self.source, self.sequence())?;
            let group = device::get_group(&self.socket, &address, self.source, self.sequence())?;
            let location =
                device::get_location(&self.socket, &address, self.source, self.sequence())?;
            let device = Device::new(
                mac_address,
                socket_address,
                trim_trailing_null(label.label),
                trim_trailing_null(group.label),
                trim_trailing_null(location.label),
            );
            self.devices.insert(device);
        }
        Result::Ok(self.devices.clone())
    }

    pub(crate) fn get_state(&self, device: &Device) -> io::Result<StatePayload> {
        let state = light::get_state(&self.socket, device, self.source, self.sequence())?;
        Result::Ok(state)
    }

    pub(crate) fn transition_on(&self, device: &Device, duration: Duration) -> io::Result<()> {
        light::set_power(
            &self.socket,
            device,
            self.source,
            self.sequence(),
            Power::On,
            to_millis(duration),
        )?;
        Result::Ok(())
    }

    pub(crate) fn turn_on(&self, device: &Device) -> io::Result<()> {
        self.transition_on(device, ZERO_DURATION)
    }

    pub(crate) fn transition_off(&self, device: &Device, duration: Duration) -> io::Result<()> {
        light::set_power(
            &self.socket,
            device,
            self.source,
            self.sequence(),
            Power::Off,
            to_millis(duration),
        )?;
        Result::Ok(())
    }

    pub(crate) fn turn_off(&self, device: &Device) -> io::Result<()> {
        self.transition_off(device, ZERO_DURATION)
    }

    pub(crate) fn toggle_power(&self, device: &Device, duration: Duration) -> io::Result<()> {
        match self.get_state(device)?.power() {
            Power::Off => self.transition_on(device, duration),
            Power::On => self.transition_off(device, duration),
            Power::Unknown(n) => Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unknown power {}", n),
            )),
        }
    }

    pub(crate) fn transition_brightness(
        &self,
        device: &Device,
        brightness: f32,
        duration: Duration,
    ) -> io::Result<()> {
        if brightness <= 0.0 {
            self.transition_off(device, duration)?;
        } else {
            let state = self.get_state(device)?;
            let color = state.color();
            let brightness_value = (f32::min(brightness, 1.0) * 0xffff as f32) as u16;

            // Turn on before adjusting brightness, if necessary.
            match state.power() {
                Power::Off | Power::Unknown(_) => self.turn_on(device)?,
                Power::On => (),
            }
            light::set_color(
                &self.socket,
                device,
                self.source,
                self.sequence(),
                color.with_brightness(brightness_value),
                to_millis(duration),
            )?;
        }

        Result::Ok(())
    }

    pub(crate) fn set_brightness(&self, device: &Device, brightness: f32) -> io::Result<()> {
        self.transition_brightness(device, brightness, ZERO_DURATION)
    }

    pub(crate) fn transition_color(
        &self,
        device: &Device,
        color: Color,
        duration: Duration,
    ) -> io::Result<()> {
        light::set_color(
            &self.socket,
            &device,
            self.source,
            self.sequence(),
            color.into(),
            to_millis(duration),
        )?;
        Result::Ok(())
    }

    pub(crate) fn set_color(&self, device: &Device, color: Color) -> io::Result<()> {
        self.transition_color(device, color, ZERO_DURATION)
    }

    pub(crate) fn transition_temperature(
        &self,
        device: &Device,
        temperature: u16,
        duration: Duration,
    ) -> io::Result<()> {
        let hsbk = self.get_state(device)?.color();

        light::set_color(
            &self.socket,
            device,
            self.source,
            self.sequence(),
            hsbk.with_hue(0).with_saturation(0).with_kelvin(temperature),
            to_millis(duration),
        )?;
        Result::Ok(())
    }

    pub(crate) fn set_temperature(&self, device: &Device, temperature: u16) -> io::Result<()> {
        self.transition_temperature(device, temperature, ZERO_DURATION)
    }

    /// Return current sequence value then increment.
    fn sequence(&self) -> u8 {
        let sequence = self.sequence.get();
        self.sequence.set(sequence.wrapping_add(1));
        sequence
    }
}

fn to_millis(duration: Duration) -> u32 {
    if duration < ZERO_DURATION {
        0u32
    } else if duration > MAX_DURATION {
        u32::MAX
    } else {
        duration.as_millis() as u32
    }
}

/// Trim trailing null bytes from a string.
fn trim_trailing_null(s: String) -> String {
    s.trim_end_matches('\0').to_string()
}
