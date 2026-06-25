pub mod error;
pub mod focus;
pub mod keyboard;
pub mod mouse;

use std::{collections::HashMap, sync::Arc, time::Instant};

use smithay::input::{keyboard::KeyboardHandle, pointer::PointerHandle};

use crate::{
    backend::{BackendEvent, BackendPumpStatus, GesWmBackend, InputEvent, NoBackend},
    client::ClientState,
    cmd::{
        executor::{DaemonCommandExecutor, UserCommandExecutor},
        WmSessionCommand,
    },
    config::KeyboardConfiguration,
    daemon::{
        error::{DaemonInitError, DaemonKeyboardInitError},
        focus::FocusHandler,
        keyboard::{KeyboardHandler, NoKeyboard},
        mouse::{MouseHandler, NoMouse},
    },
    input::{KeyBind, UnixSocket},
    layout::{Layout, LayoutWindow, NoLayout},
    server::ServerState,
    surface::{ArrangeContext, SurfaceLogicalRectangle, SurfaceLogicalSize},
};

pub struct Daemon<Keyboard, Mouse, Backend, L> {
    server_state: ServerState,
    comms_socket: UnixSocket,
    display: wayland_server::Display<ServerState>,
    epoch: Instant,
    clients: Vec<wayland_server::Client>,
    executor: DaemonCommandExecutor,
    backend: Box<Backend>,
    keyboard: Keyboard,
    mouse: Mouse,
    keybinds: HashMap<KeyBind, WmSessionCommand>,
    layout: L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonExit {
    Requested(i32),
}

impl Daemon<NoKeyboard, NoMouse, NoBackend, NoLayout> {
    pub fn new() -> Result<Daemon<NoKeyboard, NoMouse, NoBackend, NoLayout>, DaemonInitError> {
        let display: wayland_server::Display<ServerState> = wayland_server::Display::new()?;
        let server_state = ServerState::from_display(&display)?;
        let executor = DaemonCommandExecutor::new(server_state.socket_name().to_string());
        let comms_socket = UnixSocket::try_autocreate("geswm")?;

        Ok(Self {
            server_state,
            comms_socket,
            display,
            epoch: Instant::now(),
            clients: vec![],
            executor,
            backend: Box::new(()),
            keyboard: (),
            mouse: (),
            layout: (),
            keybinds: HashMap::new(),
        })
    }
}

impl<Keyboard, Mouse, Backend: GesWmBackend<ServerState>, L> Daemon<Keyboard, Mouse, Backend, L>
where
    Daemon<Keyboard, Mouse, Backend, L>: KeyboardHandler + MouseHandler + FocusHandler,
    L: Layout,
{
    pub fn run(&mut self) -> DaemonExit {
        self.run_until(|| false)
    }

    pub fn run_until<F>(&mut self, mut should_exit: F) -> DaemonExit
    where
        F: FnMut() -> bool,
    {
        loop {
            if should_exit() {
                tracing::info!("shutdown requested");
                return DaemonExit::Requested(0);
            }

            if let Some(exit) = self.handle_new_events() {
                return exit;
            }
            self.handle_client_connections();
            self.arrange_windows();
            self.synchronize_clients();
            self.ensure_focus();
            self.backend.render(&self.server_state, &self.epoch);
        }
    }

    fn arrange_windows(&mut self) {
        let physical_size = self.backend.window_size();
        self.server_state
            .set_output_size(physical_size.w, physical_size.h);
        let output_size = SurfaceLogicalSize::from((physical_size.w, physical_size.h));
        let surfaces = self
            .server_state
            .xdg_shell_state
            .toplevel_surfaces()
            .to_vec();

        let mut layout_windows = surfaces
            .iter()
            .map(|surface| LayoutWindow {
                geometry: self
                    .server_state
                    .geometry_for_surface_or_default(surface.wl_surface()),
                focused: self.server_state.is_focused(surface.wl_surface()),
            })
            .collect::<Vec<_>>();

        let mut ctx = crate::layout::LayoutContext {
            output_size,
            windows: &mut layout_windows,
        };

        self.layout.arrange(&mut ctx);

        for (surface, layout_window) in surfaces.iter().zip(layout_windows.into_iter()) {
            let outer_geometry = layout_window.geometry;
            self.server_state
                .set_geometry_for_surface(surface.wl_surface().clone(), outer_geometry);

            let arrange_ctx = ArrangeContext {
                focused: layout_window.focused,
                fullscreen: false,
                floating: false,
                output_rect: SurfaceLogicalRectangle::new((0, 0).into(), output_size),
            };

            let run = self
                .backend
                .surface_transform_pipeline()
                .begin(outer_geometry, arrange_ctx)
                .arrange();

            let configure_size = run.configure_size();

            let width = configure_size.w.max(1);
            let height = configure_size.h.max(1);
            let size = (width, height).into();

            let size_changed = surface.with_pending_state(|state| {
                state
                    .size
                    .replace(size)
                    .map(|old| old != size)
                    .unwrap_or(true)
            });

            if size_changed {
                surface.send_configure();
            }
        }
    }

    fn handle_new_events(&mut self) -> Option<DaemonExit> {
        let mut event_queue = Vec::new();
        match self
            .backend
            .dispatch_new_events(|event| event_queue.push(event))
        {
            BackendPumpStatus::Continue => {}
            BackendPumpStatus::Exit(exit_code) => {
                tracing::info!(?exit_code, "event loop exited");
                return Some(DaemonExit::Requested(exit_code));
            }
        };

        let mut commands: Vec<Option<WmSessionCommand>> = Vec::new();
        for event in &event_queue {
            let command = match event {
                BackendEvent::Resize { size, scale } => {
                    tracing::info!("resized event: {size:?} scale: {scale:?}");
                    None
                }
                BackendEvent::FocusGained => None,
                BackendEvent::FocusLost => None,
                BackendEvent::Input(input_event) => match input_event {
                    InputEvent::Keyboard { time, key, state } => {
                        self.on_keyboard_event(*time, key, state)
                    }
                    InputEvent::PointerAxis { time } => self.on_mouse_wheel_event(time),
                    InputEvent::PointerButton {
                        time,
                        button_code,
                        state,
                    } => self.on_mouse_input_event(time, button_code, state),
                    InputEvent::PointerMotionAbsolute { time, x, y } => {
                        self.on_mouse_moved_event(time, x, y)
                    }
                    InputEvent::Unimplemented => None,
                },
                BackendEvent::Redraw => {
                    tracing::info!("redraw event");
                    None
                }
                BackendEvent::CloseRequested => {
                    tracing::info!("close requested");
                    return Some(DaemonExit::Requested(0));
                }
            };
            commands.push(command);
        }

        for command in commands.into_iter().flatten() {
            self.executor
                .execute(&command)
                .inspect_err(|error| tracing::error!(?error, ?command, "user cmd execution"))
                .ok();
        }

        None
    }

    fn handle_client_connections(&mut self) {
        if let Some(unix_stream) = self.server_state.socket.accept().unwrap() {
            let client = self
                .display
                .handle()
                .insert_client(unix_stream, Arc::new(ClientState::default()))
                .unwrap();
            self.clients.push(client);
        }
    }

    fn synchronize_clients(&mut self) {
        self.display
            .dispatch_clients(&mut self.server_state)
            .unwrap();
        self.display.flush_clients().unwrap();
    }
}

impl<Keyboard, Mouse, L> Daemon<Keyboard, Mouse, NoBackend, L> {
    pub fn with_backend<Backend>(self, backend: Backend) -> Daemon<Keyboard, Mouse, Backend, L>
    where
        Backend: GesWmBackend<ServerState>,
    {
        Daemon {
            server_state: self.server_state,
            comms_socket: self.comms_socket,
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            keyboard: self.keyboard,
            executor: self.executor,
            mouse: self.mouse,
            layout: self.layout,
            keybinds: self.keybinds,
            backend: Box::new(backend),
        }
    }
}

impl<Keyboard, Backend, L> Daemon<Keyboard, NoMouse, Backend, L> {
    pub fn with_mouse(mut self) -> Daemon<Keyboard, PointerHandle<ServerState>, Backend, L> {
        tracing::info!("add mouse");
        let mouse = self.server_state.seat.add_pointer();
        Daemon {
            server_state: self.server_state,
            comms_socket: self.comms_socket,
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            keyboard: self.keyboard,
            executor: self.executor,
            backend: self.backend,
            layout: self.layout,
            keybinds: self.keybinds,
            mouse,
        }
    }
}

impl<Mouse, Backend, L> Daemon<NoKeyboard, Mouse, Backend, L> {
    pub fn with_keyboard(
        mut self,
        config: KeyboardConfiguration,
    ) -> Result<Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L>, DaemonKeyboardInitError>
    {
        tracing::info!("add keyboard");

        let repeat_delay = config.repeat_delay;
        let repeat_rate = config.repeat_rate;
        let keyboard =
            self.server_state
                .seat
                .add_keyboard(config.into(), repeat_delay, repeat_rate)?;

        Ok(Daemon {
            server_state: self.server_state,
            comms_socket: self.comms_socket,
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            mouse: self.mouse,
            backend: self.backend,
            layout: self.layout,
            keybinds: self.keybinds,
            executor: self.executor,
            keyboard,
        })
    }
}

impl<Keyboard, Mouse, Backend> Daemon<Keyboard, Mouse, Backend, NoLayout> {
    pub fn with_initial_layout<L>(self, layout: L) -> Daemon<Keyboard, Mouse, Backend, L>
    where
        L: Layout,
    {
        Daemon {
            server_state: self.server_state,
            comms_socket: self.comms_socket,
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            keyboard: self.keyboard,
            executor: self.executor,
            mouse: self.mouse,
            backend: self.backend,
            keybinds: self.keybinds,
            layout,
        }
    }
}

impl<Mouse, Backend, L> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    pub fn bind<C>(
        mut self,
        keybind: KeyBind,
        command: C,
    ) -> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L>
    where
        C: Into<WmSessionCommand>,
    {
        let command = command.into();
        if let std::collections::hash_map::Entry::Occupied(..) = self.keybinds.entry(keybind) {
            tracing::warn!("redefining {keybind}: {command}");
        } else {
            tracing::info!("binding {keybind} to command: {command}");
        }
        self.keybinds.insert(keybind, command);
        self
    }
}
