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
    surface::{
        ArrangeContext, RenderTransformContext, SurfaceGeometry, SurfacePhysicalPosition,
        SurfacePhysicalSize, SurfaceTransformPipeline, SurfaceTransformer,
    },
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
    pub surface_transform_pipeline: SurfaceTransformPipeline<Renderer>,
}

impl WinitBackend<GlesRenderer> {
    pub fn new_gles_renderer() -> Result<WinitBackend<GlesRenderer>, WinitBackendInitError> {
        let (graphics, event_loop) = winit::init::<GlesRenderer>()?;

        Ok(WinitBackend {
            graphics,
            event_loop,
            surface_transform_pipeline: SurfaceTransformPipeline::new(),
        })
    }
}

impl<Renderer> GesWmBackend<ServerState> for WinitBackend<Renderer>
where
    Renderer: Bind<EGLSurface> + ImportDmaWl + ImportMemWl,
    Renderer::TextureId: Clone + 'static,
    SwapBuffersError: From<Renderer::Error>,
{
    type Renderer = Renderer;

    fn window_size(&self) -> SurfacePhysicalSize {
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

    fn surface_transform_pipeline(&self) -> &SurfaceTransformPipeline<Self::Renderer> {
        &self.surface_transform_pipeline
    }

    fn surface_transform_pipeline_mut(&mut self) -> &mut SurfaceTransformPipeline<Self::Renderer> {
        &mut self.surface_transform_pipeline
    }

    fn render(&mut self, state: &ServerState, epoch: &Instant) {
        let size = self.window_size();
        let damage = Rectangle::from_size(size);

        let output_rect =
            crate::surface::SurfaceLogicalRectangle::new((0, 0).into(), (size.w, size.h).into());

        {
            let (renderer, mut framebuffer) = self.graphics.bind().unwrap();

            struct RenderItem<Renderer>
            where
                Renderer: smithay::backend::renderer::Renderer,
            {
                surface: wayland_server::protocol::wl_surface::WlSurface,
                geometry: SurfaceGeometry,
                arrange_ctx: ArrangeContext,
                elements: Vec<WaylandSurfaceRenderElement<Renderer>>,
            }

            let render_items = state
                .xdg_shell_state
                .toplevel_surfaces()
                .iter()
                .map(|surface| {
                    let wl_surface = surface.wl_surface().clone();

                    let geometry = state
                        .geometry_for_surface(surface.wl_surface())
                        .unwrap_or_else(|| SurfaceGeometry {
                            position: (0, 0).into(),
                            size: (1, 1).into(),
                        });

                    let focused = state.is_focused(surface.wl_surface());

                    let arrange_ctx = ArrangeContext {
                        focused,
                        fullscreen: false,
                        floating: false,
                        output_rect,
                    };

                    let arranged = self
                        .surface_transform_pipeline
                        .begin(geometry, arrange_ctx)
                        .arrange();

                    let transform = arranged.transform();

                    let render_position = SurfacePhysicalPosition::from((
                        transform.client_rect.loc.x,
                        transform.client_rect.loc.y,
                    ));

                    let elements: Vec<WaylandSurfaceRenderElement<Renderer>> =
    smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
        renderer,
        surface.wl_surface(),
        render_position,
        1.0,
        1.0,
        smithay::backend::renderer::element::Kind::Unspecified,
    );

                    RenderItem {
                        surface: wl_surface,
                        geometry,
                        arrange_ctx,
                        elements,
                    }
                })
                .collect::<Vec<_>>();

            let mut frame = renderer
                .render(&mut framebuffer, size, Transform::Flipped180)
                .unwrap();

            frame
                .clear(Color32F::new(0.05, 0.05, 0.08, 1.0), &[damage])
                .unwrap();

            for item in render_items {
                let arranged = self
                    .surface_transform_pipeline
                    .begin(item.geometry, item.arrange_ctx)
                    .arrange();

                let transform = arranged.transform();

                let pre_rendered = {
                    let mut transform_ctx = RenderTransformContext {
                        target: &mut frame,
                        surface: &item.surface,
                        transform,
                        focused: item.arrange_ctx.focused,
                        output_rect,
                    };

                    arranged.render_pre(&mut transform_ctx).unwrap()
                };

                draw_render_elements(&mut frame, 1.0, &item.elements, &[damage]).unwrap();

                {
                    let mut transform_ctx = RenderTransformContext {
                        target: &mut frame,
                        surface: &item.surface,
                        transform,
                        focused: item.arrange_ctx.focused,
                        output_rect,
                    };

                    let _post_rendered = pre_rendered.render_post(&mut transform_ctx).unwrap();
                }
            }

            let _ = frame.finish().unwrap();
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

        self.graphics.submit(Some(&[damage])).unwrap();
    }
}
