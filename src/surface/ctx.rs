use smithay::backend::renderer::Renderer;
use wayland_server::protocol::wl_surface::WlSurface;

use crate::surface::{SurfaceLogicalRectangle, TransformRenderTarget, WindowTransform};

#[derive(Debug, Clone, Copy)]
pub struct ArrangeContext {
    pub focused: bool,
    pub fullscreen: bool,
    pub floating: bool,
    pub output_rect: SurfaceLogicalRectangle,
}

pub struct RenderTransformContext<'ctx, R>
where
    R: Renderer,
{
    pub target: &'ctx mut dyn TransformRenderTarget<Error = R::Error>,
    pub surface: &'ctx WlSurface,
    pub transform: WindowTransform,
    pub focused: bool,
    pub output_rect: SurfaceLogicalRectangle,
}
