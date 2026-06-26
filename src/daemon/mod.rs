pub mod error;
pub mod event;
pub mod focus;
pub mod keyboard;
pub mod layout;
pub mod mouse;

use std::{collections::HashMap, sync::Arc, time::Instant};

use smithay::input::{keyboard::KeyboardHandle, pointer::PointerHandle};

use crate::{
    backend::{GesWmBackend, NoBackend},
    client::ClientState,
    cmd::{LayoutCommand, WmSessionCommand},
    config::KeyboardConfiguration,
    daemon::{
        error::{DaemonInitError, DaemonKeyboardInitError},
        event::EventProcessor,
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
            keybinds: HashMap::new(),
        })
    }
}

impl<Keyboard, Mouse, Backend, L> Daemon<Keyboard, Mouse, Backend, L>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, L>:
        KeyboardHandler + MouseHandler + FocusHandler + WindowArranger + EventProcessor,
    L: Layout,
{
    pub fn run(&mut self) -> ! {
        loop {
            self.prune_dead_windows();
            self.handle_new_events();
            self.handle_client_connections();
            self.arrange_windows();
            self.synchronize_clients();
            self.ensure_a_window_is_focused();
            self.backend.render(&self.server_state, &self.epoch);
        }
    }

    fn prune_dead_windows(&mut self) {
        let mut removed_surfaces = Vec::new();
        self.server_state.windows.retain(|window| {
            if window.is_alive() {
                true
            } else {
                removed_surfaces.push(window.surface.clone());
                false
            }
        });

        for surface in &removed_surfaces {
            if let Some(focused_surface) = &self.server_state.focused_window
                && focused_surface == surface
            {
                self.clear_focus();
            }
        }

        if !removed_surfaces.is_empty() {
            tracing::debug!(?removed_surfaces, "pruned dead windows");
            self.server_state.mark_layout_dirty();
        }
    }

    fn exec(&mut self, command: &WmSessionCommand) {
        match command {
            WmSessionCommand::Spawn(cmd) => {
                command
                    .exec_spawn(cmd, self.server_state.socket_name())
                    .inspect_err(|error| tracing::error!(?error, "spawn failed"))
                    .ok();
            }
            WmSessionCommand::Layout(LayoutCommand::FocusNext) => self.focus_next(),
            WmSessionCommand::Layout(LayoutCommand::FocusPrev) => self.focus_prev(),
            WmSessionCommand::Layout(LayoutCommand::SendDown) => self.move_focused_window_down(),
            WmSessionCommand::Layout(LayoutCommand::SendUp) => self.move_focused_window_up(),
            WmSessionCommand::Layout(_layout_command) => todo!(),
            WmSessionCommand::CloseFocused => self.close_focused_window(),
            WmSessionCommand::ConfirmCommand(prompt, next_command) => {
                if WmSessionCommand::show_prompt(prompt) {
                    self.exec(next_command);
                }
            }
            WmSessionCommand::GoToWorkSpace(_) => todo!(),
            WmSessionCommand::MoveFocusedWindowToWorkSpace(_) => todo!(),
            WmSessionCommand::Exit(..) => {}
        };
    }

    pub fn close_focused_window(&mut self) {
        let Some(focused) = self.server_state.focused_window.as_ref() else {
            return;
        };

        if let Some(window) = self
            .server_state
            .windows
            .iter()
            .find(|window| window.surface() == focused)
        {
            window.close();
        }
    }

    fn handle_client_connections(&mut self) {
        if let Some(unix_stream) = self.server_state.socket.accept().unwrap() {
            self.display
                .handle()
                .insert_client(unix_stream, Arc::new(ClientState::default()))
                .unwrap();
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
        let mouse = self.server_state.seat.add_pointer();
        Daemon {
            server_state: self.server_state,
            display: self.display,
            epoch: self.epoch,
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
            keybinds: self.keybinds,
            layout,
        }
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
