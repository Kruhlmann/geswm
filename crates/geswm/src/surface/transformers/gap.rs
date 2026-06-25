use smithay::backend::renderer::Renderer;

use crate::surface::{
    ArrangeContext, SurfaceLogicalRectangle, SurfaceTransformer, WindowTransform,
};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceGapTransformer {
    pub gap: i32,
}

impl<R> SurfaceTransformer<R> for SurfaceGapTransformer
where
    R: Renderer,
{
    fn arrange(&self, mut input: WindowTransform, _ctx: &ArrangeContext) -> WindowTransform {
        let g = self.gap;
        let rect = input.client_rect;

        let client_rect = SurfaceLogicalRectangle::new(
            (rect.loc.x + g, rect.loc.y + g).into(),
            (
                rect.size.w.saturating_sub(g * 2),
                rect.size.h.saturating_sub(g * 2),
            )
                .into(),
        );

        input.client_rect = client_rect;
        input.configure_size = client_rect.size;

        input
    }
}
