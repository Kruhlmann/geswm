use std::time::Instant;

use smithay::{
    backend::{
        SwapBuffersError,
        egl::EGLSurface,
        renderer::{
            Bind, Color32F, Frame, ImportDmaWl, ImportMemWl,
            element::surface::WaylandSurfaceRenderElement, gles::GlesRenderer,
            utils::draw_render_elements,
        },
        winit::{self, Error as WinitError, WinitEventLoop, WinitGraphicsBackend},
    },
    utils::{Rectangle, Transform},
    wayland::compositor::{SurfaceAttributes, TraversalAction},
};

use crate::{
    backend::{BackendEvent, BackendPumpStatus, GesWmBackend},
    server::ServerState,
};

#[derive(Debug, thiserror::Error)]
pub enum WinitBackendInitError {
    #[error("{0}")]
    NotSupported(WinitError),
    #[error("event loop init failure: {0}")]
    EventLoopError(WinitError),
    #[error("renderer init failure: {0}")]
    RendererError(WinitError),
}

impl From<WinitError> for WinitBackendInitError {
    fn from(value: WinitError) -> Self {
        match value {
            WinitError::EventLoopCreation(..) => WinitBackendInitError::EventLoopError(value),
            WinitError::WindowCreation(..)
            | WinitError::Surface(..)
            | WinitError::Egl(..)
            | WinitError::RendererCreationError(..) => WinitBackendInitError::RendererError(value),
            WinitError::NotSupported => WinitBackendInitError::NotSupported(value),
        }
    }
}

pub struct WinitBackend<Renderer>
where
    Renderer: Bind<EGLSurface>,
    SwapBuffersError: From<Renderer::Error>,
{
    pub graphics: WinitGraphicsBackend<Renderer>,
    pub event_loop: WinitEventLoop,
}

impl WinitBackend<GlesRenderer> {
    pub fn new_gles_renderer() -> Result<WinitBackend<GlesRenderer>, WinitBackendInitError> {
        let (graphics, event_loop) = winit::init::<GlesRenderer>()?;
        Ok(WinitBackend {
            graphics,
            event_loop,
        })
    }
}

impl<Renderer> GesWmBackend<ServerState> for WinitBackend<Renderer>
where
    Renderer: Bind<EGLSurface> + ImportDmaWl + ImportMemWl,
    Renderer::TextureId: Clone + 'static,
    SwapBuffersError: From<Renderer::Error>,
{
    fn window_size(&self) -> smithay::utils::Size<i32, smithay::utils::Physical> {
        self.graphics.window_size()
    }

    fn dispatch_new_events<F: FnMut(BackendEvent)>(&mut self, mut f: F) -> BackendPumpStatus {
        match self.event_loop.dispatch_new_events(|io_event| {
            let domain_event: BackendEvent = io_event.into();
            f(domain_event);
        }) {
            ::winit::platform::pump_events::PumpStatus::Continue => BackendPumpStatus::Continue,
            ::winit::platform::pump_events::PumpStatus::Exit(code) => BackendPumpStatus::Exit(code),
        }
    }

    fn render(&mut self, state: &ServerState, epoch: &Instant) {
        let size = self.window_size();
        let damage = Rectangle::from_size(size);

        {
            let (renderer, mut framebuffer) = self.graphics.bind().unwrap();

            let elements = state
                .xdg_shell_state
                .toplevel_surfaces()
                .iter()
                .flat_map(|surface| {
                    let geometry = state
                        .geometry_for_surface(surface.wl_surface())
                        .unwrap_or_else(|| crate::backend::WindowGeometry {
                            position: (0, 0).into(),
                            size: (1, 1).into(),
                        });

                    let render_position =
                        smithay::utils::Point::<i32, smithay::utils::Physical>::from((
                            geometry.position.x,
                            geometry.position.y,
                        ));

                    smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                        renderer,
                        surface.wl_surface(),
                        render_position,
                        1.0,
                        1.0,
                        smithay::backend::renderer::element::Kind::Unspecified,
                    )
                })
                .collect::<Vec<WaylandSurfaceRenderElement<Renderer>>>();

            let mut frame = renderer
                .render(&mut framebuffer, size, Transform::Flipped180)
                .unwrap(); // TODO

            frame
                .clear(Color32F::new(0.05, 0.05, 0.08, 1.0), &[damage])
                .unwrap(); // TODO
            draw_render_elements(&mut frame, 1.0, &elements, &[damage]).unwrap(); // TODO
            let _ = frame.finish().unwrap(); // TODO
        }

        for surface in state.xdg_shell_state.toplevel_surfaces() {
            smithay::wayland::compositor::with_surface_tree_downward(
                surface.wl_surface(),
                (),
                |_, _, &()| TraversalAction::DoChildren(()),
                |_surface, states, &()| {
                    for callback in states
                        .cached_state
                        .get::<SurfaceAttributes>()
                        .current()
                        .frame_callbacks
                        .drain(..)
                    {
                        callback.done(epoch.elapsed().as_millis() as u32);
                    }
                },
                |_, _, &()| true,
            );
        }

        self.graphics.submit(Some(&[damage])).unwrap(); // TODO
    }
}
