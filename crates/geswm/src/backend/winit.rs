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
use wayland_server::protocol::wl_surface::WlSurface;

use crate::server::ServerState;

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

impl<Renderer> WinitBackend<Renderer>
where
    Renderer: Bind<EGLSurface> + ImportDmaWl + ImportMemWl,
    Renderer::TextureId: Clone + 'static,
    SwapBuffersError: From<Renderer::Error>,
{
    pub fn render(&mut self, state: &ServerState, epoch: &Instant) {
        let size = self.graphics.window_size();
        let damage = Rectangle::from_size(size);

        {
            let (renderer, mut framebuffer) = self.graphics.bind().unwrap();

            let elements = state
                .xdg_shell_state
                .toplevel_surfaces()
                .iter()
                .flat_map(|surface| {
                    smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                        renderer,
                        surface.wl_surface(),
                        (0, 0),
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
            self.send_frames_surface_tree(surface.wl_surface(), epoch.elapsed().as_millis() as u32);
        }

        self.graphics.submit(Some(&[damage])).unwrap(); // TODO
    }

    fn send_frames_surface_tree(&self, surface: &WlSurface, time: u32) {
        smithay::wayland::compositor::with_surface_tree_downward(
            surface,
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
                    callback.done(time);
                }
            },
            |_, _, &()| true,
        );
    }
}
