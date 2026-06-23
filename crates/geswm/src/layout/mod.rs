pub mod master_stack;

pub use master_stack::*;

use crate::backend::{WindowGeometry, WindowSize};

pub type NoLayout = ();

pub struct LayoutContext<'a> {
    pub output_size: WindowSize,
    pub windows: &'a mut [LayoutWindow],
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutWindow {
    pub geometry: WindowGeometry,
    pub focused: bool,
}

pub trait Layout {
    fn name(&self) -> &'static str;

    fn arrange(&mut self, ctx: &mut LayoutContext<'_>);
}
