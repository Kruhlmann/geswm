pub mod client;
pub mod error;
pub mod event;
pub mod executor;
pub mod focus;
pub mod keyboard;
pub mod layout;
pub mod mouse;
pub mod window;

use std::{collections::HashMap, time::Instant};

use smithay::input::{keyboard::KeyboardHandle, pointer::PointerHandle};

use crate::{
    backend::{GesWmBackend, NoBackend},
    cmd::WmSessionCommand,
    config::KeyboardConfiguration,
    daemon::{
        client::ClientConnectionManager,
        error::{DaemonInitError, DaemonKeyboardInitError},
        event::EventProcessor,
        executor::CommandExecutor,
        focus::FocusHandler,
        keyboard::{KeyboardHandler, NoKeyboard},
        layout::WindowArranger,
        mouse::{MouseHandler, NoMouse},
    },
    input::Key,
    layout::{Layout, NoLayout},
    server::ServerState,
};

pub struct Daemon<Keyboard, Mouse, Backend, L> {
    server_state: ServerState,
    display: wayland_server::Display<ServerState>,
    epoch: Instant,
    backend: Box<Backend>,
    keyboard: Keyboard,
    mouse: Mouse,
    startup_commands: Vec<WmSessionCommand>,
    keybinds: HashMap<u32, WmSessionCommand>,
    layout: L,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonExit {
    Requested(i32),
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonRunError {
    #[error("signal handler init failure: {0}")]
    SignalInit(#[from] std::io::Error),
}

impl Daemon<NoKeyboard, NoMouse, NoBackend, NoLayout> {
    pub fn new() -> Result<Daemon<NoKeyboard, NoMouse, NoBackend, NoLayout>, DaemonInitError> {
        let display: wayland_server::Display<ServerState> = wayland_server::Display::new()?;
        let server_state = ServerState::from_display(&display)?;

        Ok(Self {
            server_state,
            display,
            epoch: Instant::now(),
            backend: Box::new(()),
            keyboard: (),
            mouse: (),
            layout: (),
            startup_commands: Vec::new(),
            keybinds: HashMap::new(),
        })
    }
}

impl<Keyboard, Mouse, Backend, L> Daemon<Keyboard, Mouse, Backend, L>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, L>: KeyboardHandler
        + MouseHandler
        + FocusHandler
        + WindowArranger
        + EventProcessor
        + CommandExecutor<WmSessionCommand>
        + ClientConnectionManager,
    L: Layout,
{
    pub fn run(&mut self) -> ! {
        self.tick();
        for command in self.startup_commands.clone() {
            tracing::info!("executing startup command: {command}");
            self.execute(&command);
        }
        loop {
            self.tick()
        }
    }

    pub fn tick(&mut self) {
        self.server_state.prune();
        self.handle_new_events();
        self.handle_client_connections();
        self.arrange_windows();
        self.ensure_a_window_is_focused();
        self.display
            .dispatch_clients(&mut self.server_state)
            .unwrap();
        self.display.flush_clients().unwrap();
        self.backend.render(&self.server_state, &self.epoch);
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
            keyboard: self.keyboard,
            mouse: self.mouse,
            layout: self.layout,
            startup_commands: self.startup_commands,
            keybinds: self.keybinds,
            backend: Box::new(backend),
        }
    }
}

impl<Keyboard, Backend, L> Daemon<Keyboard, NoMouse, Backend, L> {
    pub fn with_mouse(mut self) -> Daemon<Keyboard, PointerHandle<ServerState>, Backend, L> {
        let mouse = self.server_state.seat.add_pointer();
        Daemon {
            server_state: self.server_state,
            display: self.display,
            epoch: self.epoch,
            keyboard: self.keyboard,
            backend: self.backend,
            layout: self.layout,
            startup_commands: self.startup_commands,
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
        let repeat_delay = config.repeat_delay;
        let repeat_rate = config.repeat_rate;
        let xkb_config: smithay::input::keyboard::XkbConfig = config.clone().into();
        let keyboard = self
            .server_state
            .seat
            .add_keyboard(xkb_config, repeat_delay, repeat_rate)
            .inspect_err(|error| tracing::error!(?error, ?config, "add keyboard"))?;

        Ok(Daemon {
            server_state: self.server_state,
            display: self.display,
            epoch: self.epoch,
            mouse: self.mouse,
            backend: self.backend,
            layout: self.layout,
            startup_commands: self.startup_commands,
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
            keyboard: self.keyboard,
            mouse: self.mouse,
            backend: self.backend,
            startup_commands: self.startup_commands,
            keybinds: self.keybinds,
            layout,
        }
    }
}

impl<Keyboard, Mouse, Backend, L> Daemon<Keyboard, Mouse, Backend, L> {
    pub fn startup<C>(mut self, command: C) -> Daemon<Keyboard, Mouse, Backend, L>
    where
        C: Into<WmSessionCommand>,
    {
        let command = command.into();
        self.startup_commands.push(command);
        self
    }
}

impl<Mouse, Backend, L> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L> {
    pub fn bind<C>(
        mut self,
        key: u32,
        command: C,
    ) -> Daemon<KeyboardHandle<ServerState>, Mouse, Backend, L>
    where
        C: Into<WmSessionCommand>,
    {
        let command = command.into();
        if let std::collections::hash_map::Entry::Occupied(..) = self.keybinds.entry(key) {
            tracing::warn!("redefining {key}: {command}");
        } else {
            tracing::info!("binding {} to command: {command}", Key::display(key));
        }
        self.keybinds.insert(key, command);
        self
    }
}
