use std::{env, io, sync::Arc, time::Instant};

use geswm::client::ClientState;
use geswm::server::{ServerState, WaylandSocket};
use smithay::reexports::winit::platform::pump_events::PumpStatus;
use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::{
            Color32F, Frame, Renderer,
            element::{
                Kind,
                surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            },
            gles::GlesRenderer,
            utils::draw_render_elements,
        },
        winit::{self, WinitEvent},
    },
    input::keyboard::FilterResult,
    reexports::wayland_server::Display,
    utils::{Rectangle, Transform},
    wayland::{
        compositor::{
            SurfaceAttributes, TraversalAction, with_surface_tree_downward,
        },
        selection::data_device::ServerDndGrabHandler,
        shell::xdg::XdgShellHandler,
    },
};
use tracing::info;
use tracing_subscriber::EnvFilter;
use wayland_server::protocol::wl_surface::{self};

const DEFAULT_LOG_FILTER: &str = "info,backend_winit=warn,smithay=info,wayland_server=warn";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_log_directives = env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_FILTER.to_owned());

    let env_filter = EnvFilter::builder().parse_lossy(env_log_directives);

    tracing_subscriber::fmt()
        .compact()
        .with_writer(io::stderr)
        .with_env_filter(env_filter)
        .with_ansi_sanitization(false)
        .init();

    run_winit()
}

fn run_winit() -> Result<(), Box<dyn std::error::Error>> {
    let mut display: Display<ServerState> = Display::new()?;
    let display_handle = display.handle();
    let mut app = ServerState::from_display_handle(&display_handle);
    let socket = WaylandSocket::try_autocreate()
        .inspect(|s| tracing::info!("listening on socket {}", s.name))?;

    let mut wayland_clients = Vec::new();
    let (mut backend, mut winit) = winit::init::<GlesRenderer>()?;
    let start_time = Instant::now();

    let keyboard = app.seat.add_keyboard(Default::default(), 200, 200)?;

    loop {
        let status = winit.dispatch_new_events(|event| match event {
            WinitEvent::Resized { .. } => {}

            WinitEvent::Input(event) => match event {
                InputEvent::Keyboard { event } => {
                    keyboard.input::<(), _>(
                        &mut app,
                        event.key_code(),
                        event.state(),
                        0.into(),
                        0,
                        |_, _, _| FilterResult::Forward,
                    );
                }

                InputEvent::PointerMotionAbsolute { .. } => {
                    if let Some(surface) = app.xdg_shell_state.toplevel_surfaces().iter().next() {
                        let wl_surface = surface.wl_surface().clone();
                        keyboard.set_focus(&mut app, Some(wl_surface), 0.into());
                    }
                }

                _ => {}
            },

            _ => {}
        });

        match status {
            PumpStatus::Continue => {}
            PumpStatus::Exit(_) => return Ok(()),
        }

        if let Some(stream) = socket.accept()? {
            info!("client connected");

            let client = display
                .handle()
                .insert_client(stream, Arc::new(ClientState::default()))?;

            wayland_clients.push(client);
        }

        display.dispatch_clients(&mut app)?;
        display.flush_clients()?;

        let size = backend.window_size();
        let damage = Rectangle::from_size(size);

        {
            let (renderer, mut framebuffer) = backend.bind()?;

            let elements = app
                .xdg_shell_state
                .toplevel_surfaces()
                .iter()
                .flat_map(|surface| {
                    render_elements_from_surface_tree(
                        renderer,
                        surface.wl_surface(),
                        (0, 0),
                        1.0,
                        1.0,
                        Kind::Unspecified,
                    )
                })
                .collect::<Vec<WaylandSurfaceRenderElement<GlesRenderer>>>();

            let mut frame = renderer.render(&mut framebuffer, size, Transform::Flipped180)?;

            frame.clear(Color32F::new(0.05, 0.05, 0.08, 1.0), &[damage])?;
            draw_render_elements(&mut frame, 1.0, &elements, &[damage])?;
            frame.finish()?;

            for surface in app.xdg_shell_state.toplevel_surfaces() {
                send_frames_surface_tree(
                    surface.wl_surface(),
                    start_time.elapsed().as_millis() as u32,
                );
            }
        }

        backend.submit(Some(&[damage]))?;
    }
}

fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
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
