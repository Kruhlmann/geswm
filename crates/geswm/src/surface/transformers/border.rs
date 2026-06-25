use smithay::backend::renderer::{Color32F, Renderer};

use crate::surface::{
    ArrangeContext, RenderTransformContext, SurfaceLogicalRectangle, SurfacePhysicalRectangle,
    SurfaceTransformer, WindowTransform,
};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceBorderTransformer {
    pub width: i32,
    pub focused_color: Color32F,
    pub unfocused_color: Color32F,
}

impl SurfaceBorderTransformer {
    pub fn new<C>(width: i32, focused_color: C, unfocused_color: C) -> Self
    where
        C: Into<Color32F>,
    {
        Self {
            width,
            focused_color: focused_color.into(),
            unfocused_color: unfocused_color.into(),
        }
    }
}

impl<R> SurfaceTransformer<R> for SurfaceBorderTransformer
where
    R: Renderer,
{
    fn arrange(&self, mut input: WindowTransform, _ctx: &ArrangeContext) -> WindowTransform {
        let b = self.width.max(0);
        let rect = input.client_rect;

        let client_width = (rect.size.w - b * 2).max(1);
        let client_height = (rect.size.h - b * 2).max(1);

        let client_rect = SurfaceLogicalRectangle::from_loc_and_size(
            (rect.loc.x + b, rect.loc.y + b),
            (client_width, client_height),
        );

        input.client_rect = client_rect;
        input.configure_size = client_rect.size;

        input
    }

    fn render_pre(&self, ctx: &mut RenderTransformContext<'_, R>) -> Result<(), R::Error> {
        let color = if ctx.focused {
            self.focused_color
        } else {
            self.unfocused_color
        };

        let rects = border_rects(ctx.transform.outer_rect, ctx.transform.client_rect);

        ctx.target.clear_rects(color, &rects)?;

        Ok(())
    }
}

fn border_rects(
    outer: SurfaceLogicalRectangle,
    client: SurfaceLogicalRectangle,
) -> Vec<SurfacePhysicalRectangle> {
    let left = outer.loc.x;
    let top = outer.loc.y;
    let right = outer.loc.x + outer.size.w;
    let bottom = outer.loc.y + outer.size.h;

    let client_left = client.loc.x.clamp(left, right);
    let client_top = client.loc.y.clamp(top, bottom);
    let client_right = (client.loc.x + client.size.w).clamp(left, right);
    let client_bottom = (client.loc.y + client.size.h).clamp(top, bottom);

    [
        (left, top, outer.size.w, client_top - top),
        (left, client_bottom, outer.size.w, bottom - client_bottom),
        (
            left,
            client_top,
            client_left - left,
            client_bottom - client_top,
        ),
        (
            client_right,
            client_top,
            right - client_right,
            client_bottom - client_top,
        ),
    ]
    .into_iter()
    .filter(|(_, _, w, h)| *w > 0 && *h > 0)
    .map(|(x, y, w, h)| SurfacePhysicalRectangle::from_loc_and_size((x, y), (w, h)))
    .collect()
}
