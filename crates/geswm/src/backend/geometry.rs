use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

pub type BackendGeometryPhysical = Size<i32, Physical>;
pub type BackendGeometryLogical = Size<i32, Logical>;
pub type BackendRectanglePhysical = Rectangle<i32, Physical>;

pub type WindowSize = Size<i32, Logical>;
pub type WindowPosition = Point<i32, Logical>;
pub type WindowRectangle = Rectangle<i32, Logical>;

#[derive(Debug, Clone, Copy)]
pub struct WindowGeometry {
    pub position: WindowPosition,
    pub size: WindowSize,
}
