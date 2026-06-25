use smithay::backend::renderer::Renderer;

use crate::surface::{
    ArrangeContext, RenderTransformContext, SurfaceGeometry, SurfaceLogicalRectangle,
    SurfaceLogicalSize,
};

pub trait SurfaceTransformer<R>
where
    R: Renderer,
{
    fn arrange(&self, input: WindowTransform, _ctx: &ArrangeContext) -> WindowTransform {
        input
    }

    fn render_pre(&self, _ctx: &mut RenderTransformContext<'_, R>) -> Result<(), R::Error> {
        Ok(())
    }

    fn render_post(&self, _ctx: &mut RenderTransformContext<'_, R>) -> Result<(), R::Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowTransform {
    pub outer_rect: SurfaceLogicalRectangle,
    pub client_rect: SurfaceLogicalRectangle,
    pub configure_size: SurfaceLogicalSize,
}

impl From<SurfaceLogicalRectangle> for WindowTransform {
    fn from(outer_rect: SurfaceLogicalRectangle) -> Self {
        Self {
            outer_rect,
            client_rect: outer_rect,
            configure_size: outer_rect.size,
        }
    }
}

impl From<SurfaceGeometry> for SurfaceLogicalRectangle {
    fn from(geometry: SurfaceGeometry) -> Self {
        SurfaceLogicalRectangle::new(geometry.position, geometry.size)
    }
}

impl From<SurfaceGeometry> for WindowTransform {
    fn from(geometry: SurfaceGeometry) -> Self {
        let outer_rect: SurfaceLogicalRectangle = geometry.into();
        outer_rect.into()
    }
}
