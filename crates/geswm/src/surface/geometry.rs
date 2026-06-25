use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

pub type SurfacePhysicalSize = Size<i32, Physical>;
pub type SurfacePhysicalPosition = Point<i32, Physical>;
pub type SurfacePhysicalRectangle = Rectangle<i32, Physical>;

pub type SurfaceLogicalSize = Size<i32, Logical>;
pub type SurfaceLogicalPosition = Point<i32, Logical>;
pub type SurfaceLogicalRectangle = Rectangle<i32, Logical>;

#[derive(Debug, Clone, Copy)]
pub struct SurfaceGeometry {
    pub position: SurfaceLogicalPosition,
    pub size: SurfaceLogicalSize,
}
