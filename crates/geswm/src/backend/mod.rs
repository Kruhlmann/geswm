pub mod event;
pub mod geometry;
pub mod pump_status;
pub mod winit;

pub use event::*;
pub use geometry::*;
pub use pump_status::*;
pub use winit::*;

use std::time::Instant;

pub type NoBackend = ();

pub trait GesWmBackend<State> {
    fn render(&mut self, state: &State, epoch: &Instant);
    fn window_size(&self) -> BackendGeometryPhysical;
    fn dispatch_new_events<F: FnMut(BackendEvent)>(&mut self, f: F) -> BackendPumpStatus;
}
