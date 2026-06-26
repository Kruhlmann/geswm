use smithay::utils::{Point, Size};

use crate::{
    layout::{Growable, Layout, LayoutContext, LayoutWindow, MasterWindow, Shrinkable},
    surface::SurfaceGeometry,
};

#[derive(Debug, Clone, Copy)]
pub struct MasterStackLayout {
    pub master_percent: i32,
    pub master_count: usize,
}

impl MasterStackLayout {
    pub fn new(master_percent: i32) -> Self {
        Self {
            master_percent,
            master_count: 2,
        }
    }

    pub fn arrange_vertical_stack(
        windows: &mut [LayoutWindow],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        let count = windows.len();

        if count == 0 {
            return;
        }

        let base_h = height / count as i32;

        for (index, window) in windows.iter_mut().enumerate() {
            let window_y = y + index as i32 * base_h;

            let window_h = if index == count - 1 {
                height - (window_y - y)
            } else {
                base_h
            };

            window.geometry = SurfaceGeometry {
                position: Point::from((x, window_y)),
                size: Size::from((width, window_h)),
            };
        }
    }
}

impl MasterWindow for MasterStackLayout {
    fn increase_master_count(&mut self) {
        self.master_count += 1;
    }

    fn decrease_master_count(&mut self) {
        if self.master_count > 0 {
            self.master_count -= 1;
        }
    }
}

impl Shrinkable for MasterStackLayout {
    fn shrink(&mut self) {
        self.master_percent = (self.master_percent - 5).max(10);
    }
}

impl Growable for MasterStackLayout {
    fn grow(&mut self) {
        self.master_percent = (self.master_percent + 5).min(90);
    }
}

impl Layout for MasterStackLayout {
    fn arrange(&mut self, ctx: &mut LayoutContext<'_>) {
        let count = ctx.windows.len();

        if count == 0 {
            return;
        }

        let output_w = ctx.output_size.w;
        let output_h = ctx.output_size.h;

        let master_count = self.master_count.min(count);
        let stack_count = count - master_count;

        if master_count == 0 {
            Self::arrange_vertical_stack(&mut ctx.windows[..], 0, 0, output_w, output_h);
            return;
        }

        if stack_count == 0 {
            Self::arrange_vertical_stack(
                &mut ctx.windows[..master_count],
                0,
                0,
                output_w,
                output_h,
            );
            return;
        }

        let master_w = output_w * self.master_percent / 100;
        let stack_w = output_w - master_w;

        Self::arrange_vertical_stack(&mut ctx.windows[..master_count], 0, 0, master_w, output_h);

        Self::arrange_vertical_stack(
            &mut ctx.windows[master_count..],
            master_w,
            0,
            stack_w,
            output_h,
        );
    }
}
