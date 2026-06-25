use smithay::backend::renderer::Color32F;

pub struct RgbaColor([f32; 4]);

impl RgbaColor {
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

impl Into<Color32F> for RgbaColor {
    fn into(self) -> Color32F {
        Color32F::from(self.0)
    }
}
