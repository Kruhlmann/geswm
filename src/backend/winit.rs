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
    wayland::{
        compositor::{SurfaceAttributes, TraversalAction},
        shell::wlr_layer::{Layer, LayerSurfaceCachedState},
    },
};

use crate::{
    backend::{BackendEvent, BackendPumpStatus, GesWmBackend},
    config::RgbaColor,
    output::{OutputDescription, OutputDiscovery},
    server::ServerState,
    surface::{
        ArrangeContext, RenderTransformContext, SurfaceGeometry, SurfacePhysicalPosition,
        SurfacePhysicalSize, SurfaceTransformPipeline,
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
    pub background_color: Color32F,
    pub surface_transform_pipeline: SurfaceTransformPipeline<Renderer>,
    pub refresh_rate: i32,
}

impl WinitBackend<GlesRenderer> {
    pub fn new_gles_renderer() -> Result<WinitBackend<GlesRenderer>, WinitBackendInitError> {
        let (graphics, event_loop) = winit::init::<GlesRenderer>()?;

        Ok(WinitBackend {
            graphics,
            event_loop,
            background_color: Color32F::new(0.05, 0.05, 0.08, 1.0),
            surface_transform_pipeline: SurfaceTransformPipeline::new(),
            refresh_rate: 60_000,
        })
    }

    pub fn set_refresh_rate(mut self, hz: f32) -> Self {
        let mhz = (hz.floor() as i32) * 1000;
        tracing::info!("setting refresh rate to {} Hz ({} mhz)", hz, mhz);
        self.refresh_rate = mhz;
        self
    }

    pub fn set_background_color(mut self, color: &str) -> Self {
        self.background_color = RgbaColor::from_hex(color).into();
        self
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
        let output_size = SurfacePhysicalSize::from((size.w, size.h));

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

            let layer_of = |s: &smithay::wayland::shell::wlr_layer::LayerSurface| -> Layer {
                smithay::wayland::compositor::with_states(s.wl_surface(), |states| {
                    states
                        .cached_state
                        .get::<LayerSurfaceCachedState>()
                        .current()
                        .layer
                })
            };

            // Layer-shell surfaces rendered below regular windows (Background, Bottom).
            let mut render_items: Vec<RenderItem<Renderer>> = {
                let mut make_layer_render_item =
                    |surface: &smithay::wayland::shell::wlr_layer::LayerSurface| {
                        let wl_surface = surface.wl_surface().clone();
                        let layer_state = surface.current_state();
                        let layer_size = layer_state.size.unwrap_or_else(|| {
                            // TODO: keep layer-shell geometry in logical space and convert at render boundaries.
                            SurfacePhysicalSize::from((640, 480)).to_logical(1)
                        });

                        let geometry = SurfaceGeometry {
                            position: (
                                ((output_size.w - layer_size.w).max(0)) / 2,
                                ((output_size.h - layer_size.h).max(0)) / 2,
                            )
                                .into(),
                            size: layer_size,
                        };

                        let arrange_ctx = ArrangeContext {
                            focused: state.is_focused(surface.wl_surface()),
                            fullscreen: false,
                            floating: true,
                            output_rect,
                        };

                        let render_position = SurfacePhysicalPosition::from((
                            geometry.position.x,
                            geometry.position.y,
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
                    };

                state
                    .layer_shell_state
                    .layer_surfaces()
                    .filter(|s| {
                        s.alive() && matches!(layer_of(s), Layer::Background | Layer::Bottom)
                    })
                    .map(|s| make_layer_render_item(&s))
                    .collect()
            };

            // Regular XDG-shell windows in the middle.
            render_items.extend(
                state
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
                    }),
            );

            // Layer-shell surfaces rendered above regular windows (Top, Overlay).
            {
                let mut make_layer_render_item =
                    |surface: &smithay::wayland::shell::wlr_layer::LayerSurface| {
                        let wl_surface = surface.wl_surface().clone();
                        let layer_state = surface.current_state();
                        let layer_size = layer_state
                            .size
                            .unwrap_or_else(|| SurfacePhysicalSize::from((640, 480)).to_logical(1));

                        let geometry = SurfaceGeometry {
                            position: (
                                ((output_size.w - layer_size.w).max(0)) / 2,
                                ((output_size.h - layer_size.h).max(0)) / 2,
                            )
                                .into(),
                            size: layer_size,
                        };

                        let arrange_ctx = ArrangeContext {
                            focused: state.is_focused(surface.wl_surface()),
                            fullscreen: false,
                            floating: true,
                            output_rect,
                        };

                        let render_position = SurfacePhysicalPosition::from((
                            geometry.position.x,
                            geometry.position.y,
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
                    };

                render_items.extend(
                    state
                        .layer_shell_state
                        .layer_surfaces()
                        .filter(|s| s.alive() && matches!(layer_of(s), Layer::Top | Layer::Overlay))
                        .map(|s| make_layer_render_item(&s)),
                );
            }

            let mut frame = renderer
                .render(&mut framebuffer, size, Transform::Flipped180)
                .unwrap();

            frame.clear(self.background_color, &[damage]).unwrap();

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

        for surface in state.layer_shell_state.layer_surfaces() {
            if !surface.alive() {
                continue;
            }

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

impl<Renderer> OutputDiscovery for WinitBackend<Renderer>
where
    Renderer: Bind<EGLSurface>,
    SwapBuffersError: From<Renderer::Error>,
{
    fn output_description(&self) -> OutputDescription {
        let size = self.graphics.window_size();
        OutputDescription::virtual_output("geswm-winit-0", size.w, size.h, self.refresh_rate)
    }
}
