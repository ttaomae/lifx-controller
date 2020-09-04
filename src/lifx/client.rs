use super::{
    device::{self, Device},
    light,
    protocol::message::{Power, StatePayload},
};
use std::{cell::Cell, collections::HashSet, io, net::UdpSocket};

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
        // let mut devices = HashSet::new();
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

    pub(crate) fn turn_on(&self, device: &Device, duration: u32) -> io::Result<()> {
        light::set_power(
            &self.socket,
            device,
            self.source,
            self.sequence(),
            Power::On,
            duration,
        )?;
        Result::Ok(())
    }

    pub(crate) fn turn_off(&self, device: &Device, duration: u32) -> io::Result<()> {
        light::set_power(
            &self.socket,
            device,
            self.source,
            self.sequence(),
            Power::Off,
            duration,
        )?;
        Result::Ok(())
    }

    pub(crate) fn toggle_power(&self, device: &Device, duration: u32) -> io::Result<()> {
        match self.get_state(device)?.power() {
            Power::Off => self.turn_on(device, duration),
            Power::On => self.turn_off(device, duration),
            Power::Unknown(n) => Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unknown power {}", n),
            )),
        }
    }

    fn sequence(&self) -> u8 {
        let sequence = self.sequence.get();
        self.sequence.set(sequence.wrapping_add(1));
        sequence
    }
}

/// Trim trailing null bytes from a string.
fn trim_trailing_null(s: String) -> String {
    s.trim_end_matches('\0').to_string()
}
