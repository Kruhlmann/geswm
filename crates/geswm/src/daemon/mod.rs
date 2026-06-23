pub mod bind;
pub mod command;
pub mod error;
pub mod focus;
pub mod keyboard;
pub mod mouse;

use std::{collections::HashMap, sync::Arc, time::Instant};

use smithay::input::{keyboard::KeyboardHandle, pointer::PointerHandle};

use crate::{
    backend::{BackendEvent, BackendPumpStatus, GesWmBackend, InputEvent, NoBackend, WindowSize},
    client::ClientState,
    config::KeyboardConfiguration,
    daemon::{
        bind::KeyBind,
        command::UserCommand,
        error::{DaemonInitError, DaemonKeyboardInitError},
        focus::FocusHandler,
        keyboard::{KeyboardHandler, NoKeyboard},
        mouse::{MouseHandler, NoMouse},
    },
    layout::{Layout, LayoutWindow, NoLayout},
    server::ServerState,
};

pub struct Daemon<Keyboard, Mouse, Backend, L> {
    server_state: ServerState,
    display: wayland_server::Display<ServerState>,
    epoch: Instant,
    clients: Vec<wayland_server::Client>,
    backend: Box<Backend>,
    keyboard: Keyboard,
    mouse: Mouse,
    keybinds: HashMap<KeyBind, UserCommand>,
    layout: L,
}

impl Daemon<NoKeyboard, NoMouse, NoBackend, NoLayout> {
    pub fn new() -> Result<Daemon<NoKeyboard, NoMouse, NoBackend, NoLayout>, DaemonInitError> {
        let display: wayland_server::Display<ServerState> = wayland_server::Display::new()?;
        let server_state = ServerState::from_display(&display)?;

        Ok(Self {
            server_state,
            display,
            epoch: Instant::now(),
            clients: vec![],
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
    pub fn run(&mut self) -> ! {
        loop {
            self.handle_new_events();
            self.handle_client_connections();
            self.arrange_windows();
            self.synchronize_clients();
            self.ensure_focus();
            self.backend.render(&self.server_state, &self.epoch);
        }
    }

    fn arrange_windows(&mut self) {
        let physical_size = self.backend.window_size();

        let output_size = WindowSize::from((physical_size.w, physical_size.h));

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
            self.server_state
                .set_geometry_for_surface(surface.wl_surface().clone(), layout_window.geometry);

            let width = layout_window.geometry.size.w.max(1);
            let height = layout_window.geometry.size.h.max(1);
            let size = (width, height).into();

            let size_changed = surface.with_pending_state(|state| {
                state.size.replace(size).map(|old| old != size).unwrap_or(true)
            });

            if size_changed {
                surface.send_configure();
            }
        }
    }

    fn handle_new_events(&mut self) {
        let mut event_queue = Vec::new();
        match self
            .backend
            .dispatch_new_events(|event| event_queue.push(event))
        {
            BackendPumpStatus::Continue => {}
            BackendPumpStatus::Exit(exit_code) => {
                tracing::info!(?exit_code, "event loop exited");
                std::process::exit(0);
            }
        };

        for event in &event_queue {
            match event {
                BackendEvent::Resize { size, scale } => {
                    tracing::info!("resized event: {size:?} scale: {scale:?}")
                }
                BackendEvent::FocusGained => tracing::info!("focus on"),
                BackendEvent::FocusLost => tracing::info!("focus off"),
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
                    InputEvent::Unimplemented => {}
                },
                BackendEvent::CloseRequested => {
                    tracing::info!("close requested");
                    std::process::exit(0);
                }
                BackendEvent::Redraw => tracing::info!("redraw event"),
            }
        }
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
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            keyboard: self.keyboard,
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
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            keyboard: self.keyboard,
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
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            mouse: self.mouse,
            backend: self.backend,
            layout: self.layout,
            keybinds: self.keybinds,
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
            display: self.display,
            epoch: self.epoch,
            clients: self.clients,
            keyboard: self.keyboard,
            mouse: self.mouse,
            backend: self.backend,
            keybinds: self.keybinds,
            layout,
        }
    }
}

impl<Mouse, Backend, L> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    pub fn bind_key(
        mut self,
        keybind: KeyBind,
        command: UserCommand,
    ) -> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
        if let std::collections::hash_map::Entry::Occupied(entry) = self.keybinds.entry(keybind) {
            tracing::warn!(
                "keybind {keybind} already exists, overwriting: {:?}",
                entry.get()
            );
        } else {
            tracing::info!("binding keybind {keybind} to command: {command:?}");
        }
        self.keybinds.insert(keybind, command);
        self
    }
}
