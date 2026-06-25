pub mod master_stack;

pub use master_stack::*;

use crate::surface::{SurfaceGeometry, SurfaceLogicalSize};

pub type NoLayout = ();

pub struct LayoutContext<'a> {
    pub output_size: SurfaceLogicalSize,
    pub windows: &'a mut [LayoutWindow],
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutWindow {
    pub geometry: SurfaceGeometry,
    pub focused: bool,
}

pub trait Layout {
    fn arrange(&mut self, ctx: &mut LayoutContext<'_>);
}
