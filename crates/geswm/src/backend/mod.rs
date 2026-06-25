pub mod event;
pub mod pump_status;
#[cfg(feature = "winit")]
pub mod winit;

pub use event::*;
pub use pump_status::*;
#[cfg(feature = "winit")]
pub use winit::*;

use std::time::Instant;

use crate::{
    output::OutputDiscovery,
    surface::{SurfacePhysicalSize, SurfaceTransformPipeline, SurfaceTransformer},
};

pub type NoBackend = ();

pub trait GesWmBackend<State>: Sized + OutputDiscovery {
    type Renderer: smithay::backend::renderer::Renderer;
    fn render(&mut self, state: &State, epoch: &Instant);
    fn window_size(&self) -> SurfacePhysicalSize;
    fn dispatch_new_events<F: FnMut(BackendEvent)>(&mut self, f: F) -> BackendPumpStatus;
    fn surface_transform_pipeline(&self) -> &SurfaceTransformPipeline<Self::Renderer>;
    fn surface_transform_pipeline_mut(&mut self) -> &mut SurfaceTransformPipeline<Self::Renderer>;
    fn add_transformer<T>(mut self, transformer: T) -> Self
    where
        T: SurfaceTransformer<Self::Renderer> + 'static,
    {
        self.surface_transform_pipeline_mut().push(transformer);
        self
    }
}
