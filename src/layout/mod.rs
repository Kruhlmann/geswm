pub mod master_stack;

pub use master_stack::*;

use crate::surface::{SurfaceGeometry, SurfaceLogicalSize};

pub struct LayoutSet;
pub struct LayoutUnset;

pub struct LayoutContext<'a> {
    pub output_size: SurfaceLogicalSize,
    pub windows: &'a mut [LayoutWindow],
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutWindow {
    pub geometry: SurfaceGeometry,
    pub focused: bool,
}

pub trait Shrinkable {
    fn shrink(&mut self) {}
}

pub trait Growable {
    fn grow(&mut self) {}
}

pub trait Layout: std::fmt::Debug + Shrinkable + Growable {
    fn arrange(&mut self, ctx: &mut LayoutContext<'_>);
}
