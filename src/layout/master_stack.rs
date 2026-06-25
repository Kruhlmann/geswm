use smithay::utils::{Point, Size};

use crate::{
    layout::{Layout, LayoutContext},
    surface::SurfaceGeometry,
};

#[derive(Debug, Clone, Copy)]
pub struct MasterStackLayout {
    pub master_percent: i32,
}

impl Default for MasterStackLayout {
    fn default() -> Self {
        Self { master_percent: 50 }
    }
}

impl Layout for MasterStackLayout {
    fn name(&self) -> &'static str {
        "master-stack"
    }

    fn arrange(&mut self, ctx: &mut LayoutContext<'_>) {
        let count = ctx.windows.len();

        if count == 0 {
            return;
        }

        if count == 1 {
            ctx.windows[0].geometry = SurfaceGeometry {
                position: Point::from((0, 0)),
                size: ctx.output_size,
            };
            return;
        }

        let output_w = ctx.output_size.w;
        let output_h = ctx.output_size.h;

        let master_w = output_w * self.master_percent / 100;
        let stack_w = output_w - master_w;

        ctx.windows[0].geometry = SurfaceGeometry {
            position: Point::from((0, 0)),
            size: Size::from((master_w, output_h)),
        };

        let stack_count = count - 1;
        let stack_h = output_h / stack_count as i32;

        for (index, window) in ctx.windows[1..].iter_mut().enumerate() {
            let y = index as i32 * stack_h;

            let height = if index == stack_count - 1 {
                output_h - y
            } else {
                stack_h
            };

            window.geometry = SurfaceGeometry {
                position: Point::from((master_w, y)),
                size: Size::from((stack_w, height)),
            };
        }
    }
}
