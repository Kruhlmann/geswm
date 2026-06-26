use smithay::backend::renderer::{Color32F, Renderer};

use crate::{
    config::RgbaColor,
    surface::{
        ArrangeContext, RenderTransformContext, SurfaceLogicalRectangle, SurfacePhysicalRectangle,
        SurfaceTransformer, WindowTransform,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceBorderTransformer {
    pub width: i32,
    pub focused_color: Color32F,
    pub unfocused_color: Color32F,
}

impl Default for SurfaceBorderTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceBorderTransformer {
    pub fn new() -> Self {
        Self {
            width: 2,
            focused_color: RgbaColor::from_hex("#009999ff").into(),
            unfocused_color: RgbaColor::from_hex("#999999ff").into(),
        }
    }

    pub fn focused_color(mut self, color: &str) -> Self {
        self.focused_color = RgbaColor::from_hex(color).into();
        self
    }

    pub fn unfocused_color(mut self, color: &str) -> Self {
        self.unfocused_color = RgbaColor::from_hex(color).into();
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.width = width;
        self
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

        let client_rect = SurfaceLogicalRectangle::new(
            (rect.loc.x + b, rect.loc.y + b).into(),
            (client_width, client_height).into(),
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
    .map(|(x, y, w, h)| SurfacePhysicalRectangle::new((x, y).into(), (w, h).into()))
    .collect()
}
