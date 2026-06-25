use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::{Physical, Rectangle},
};

pub trait TransformRenderTarget {
    type Error;

    fn clear_rects(
        &mut self,
        color: Color32F,
        rects: &[Rectangle<i32, Physical>],
    ) -> Result<(), Self::Error>;
}

impl<F> TransformRenderTarget for F
where
    F: Frame,
{
    type Error = F::Error;

    fn clear_rects(
        &mut self,
        color: Color32F,
        rects: &[Rectangle<i32, Physical>],
    ) -> Result<(), Self::Error> {
        self.clear(color, rects)
    }
}
