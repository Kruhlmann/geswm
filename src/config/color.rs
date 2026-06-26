use smithay::backend::renderer::Color32F;

pub struct RgbaColor([f32; 4]);

impl RgbaColor {
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

impl From<RgbaColor> for Color32F {
    fn from(val: RgbaColor) -> Self {
        Color32F::from(val.0)
    }
}

impl std::fmt::Display for RgbaColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:02X}{:02X}{:02X}{:02X}",
            (self.0[0] * 255.0) as u8,
            (self.0[1] * 255.0) as u8,
            (self.0[2] * 255.0) as u8,
            (self.0[3] * 255.0) as u8
        )
    }
}

impl RgbaColor {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let (r, g, b, a) = match hex.len() {
            6 => (
                u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
                u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
                u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
                255,
            ),
            8 => (
                u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
                u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
                u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
                u8::from_str_radix(&hex[6..8], 16).unwrap_or(255),
            ),
            _ => (0, 0, 0, 255),
        };
        Self([
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ])
    }
}
