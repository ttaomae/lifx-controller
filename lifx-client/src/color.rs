use super::protocol::message::Hsbk;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    hue: u16,
    saturation: u16,
    brightness: u16,
    kelvin: Option<u16>,
}

impl Color {
    pub const WHITE: Color = Color {
        hue: 0,
        saturation: 0,
        brightness: 0xffff,
        kelvin: Option::None,
    };
    pub const RED: Color = Color::from_hue_u16(0x0000);
    pub const YELLOW: Color = Color::from_hue_u16(0x2aaa);
    pub const GREEN: Color = Color::from_hue_u16(0x5555);
    pub const CYAN: Color = Color::from_hue_u16(0x7fff);
    pub const BLUE: Color = Color::from_hue_u16(0xaaaa);
    pub const MAGENTA: Color = Color::from_hue_u16(0xd555);

    const fn from_hue_u16(hue: u16) -> Color {
        Color {
            hue,
            saturation: 0xffff,
            brightness: 0xffff,
            kelvin: Option::None,
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        let r = normalize(r);
        let g = normalize(g);
        let b = normalize(b);

        let min = f32::min(f32::min(r, g), b);
        let max = f32::max(f32::max(r, g), b);

        let chroma = max - min;
        let brightness = max;

        let saturation = if brightness == 0.0 {
            0.0
        } else {
            chroma / brightness
        };

        // `max` is derived directly from the values of `r`, `g`, and `b`, so it should be exactly
        // equal to at least one of them, making it safe to do exact floating-point comparison.
        #[allow(clippy::float_cmp)]
        #[rustfmt::skip]
        let hue_degrees = if chroma == 0.0 {
            0.0
        } else if max == r {
            60.0 * (0.0 + (g - b) / chroma)
        } else if max == g {
            60.0 * (2.0 + (b - r) / chroma)
        } else { // max == b
            60.0 * (4.0 + (r - g) / chroma)
        };

        Color {
            hue: degrees_to_u16(hue_degrees),
            saturation: denormalize(saturation),
            brightness: denormalize(brightness),
            kelvin: Option::None,
        }
    }

    pub fn plus_degrees(&self, degrees: f32) -> Color {
        let delta_hue = degrees_to_u16(degrees);
        Color {
            hue: self.hue.wrapping_add(delta_hue),
            saturation: self.saturation,
            brightness: self.brightness,
            kelvin: self.kelvin,
        }
    }

    pub fn with_hue(&self, hue: f32) -> Color {
        Color {
            hue: degrees_to_u16(hue),
            saturation: self.saturation,
            brightness: self.brightness,
            kelvin: self.kelvin,
        }
    }

    pub fn with_saturation(&self, saturation: f32) -> Color {
        Color {
            hue: self.hue,
            saturation: denormalize(saturation),
            brightness: self.brightness,
            kelvin: self.kelvin,
        }
    }

    pub fn with_brightness(&self, brightness: f32) -> Color {
        Color {
            hue: self.hue,
            saturation: self.saturation,
            brightness: denormalize(brightness),
            kelvin: self.kelvin,
        }
    }
}

fn normalize(n: u8) -> f32 {
    n as f32 / 0xff as f32
}

fn denormalize(n: f32) -> u16 {
    if n < 0.0 {
        0x00
    } else if n > 1.0 {
        0xffff
    } else {
        (n * 0xffff as f32) as u16
    }
}

fn degrees_to_u16(degrees: f32) -> u16 {
    // Scale between 0.0 and 360.0.
    let scaled_degrees = (degrees % 360.0 + 360.0) % 360.0;
    let normalized_degrees = scaled_degrees / 360.0;
    denormalize(normalized_degrees)
}

impl From<Color> for Hsbk {
    fn from(color: Color) -> Self {
        // Use average of temperature range [2500 - 9000], if none is provided.
        Hsbk::new(
            color.hue,
            color.saturation,
            color.brightness,
            color.kelvin.unwrap_or(5750),
        )
    }
}

impl From<Hsbk> for Color {
    fn from(hsbk: Hsbk) -> Self {
        Color {
            hue: hsbk.hue(),
            saturation: hsbk.saturation(),
            brightness: hsbk.brightness(),
            kelvin: Option::Some(hsbk.kelvin()),
        }
    }
}
