use super::protocol::message::Hsbk;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Color {
    hue: u16,
    saturation: u16,
    brightness: u16,
}

impl Color {
    pub(crate) fn rgb(r: u8, g: u8, b: u8) -> Color {
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
        }
    }

    pub(crate) fn add_degrees(&self, degrees: f32) -> Color {
        let delta_hue = degrees_to_u16(degrees);
        Color {
            hue: self.hue.wrapping_add(delta_hue),
            saturation: self.saturation,
            brightness: self.brightness,
        }
    }
}

fn normalize(n: u8) -> f32 {
    n as f32 / 0xff as f32
}

fn denormalize(n: f32) -> u16 {
    assert!(n >= 0.0 && n <= 1.0);
    (n * 0xffff as f32) as u16
}

fn degrees_to_u16(degrees: f32) -> u16 {
    // Scale between 0.0 and 360.0.
    let scaled_degrees = (degrees % 360.0 + 360.0) % 360.0;
    let normalized_degrees = scaled_degrees / 360.0;
    denormalize(normalized_degrees)
}

impl From<Color> for Hsbk {
    fn from(color: Color) -> Self {
        // Use average of temperature range [2500 - 9000].
        Hsbk::new(color.hue, color.saturation, color.brightness, 5750)
    }
}
