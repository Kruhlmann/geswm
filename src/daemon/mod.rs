pub mod error;
pub mod focus;
pub mod keyboard;
pub mod mouse;

use std::{collections::HashMap, sync::Arc, time::Instant};

use smithay::input::{keyboard::KeyboardHandle, pointer::PointerHandle};

use crate::{
    backend::{BackendPumpStatus, GesWmBackend, NoBackend},
    client::ClientState,
    cmd::{LayoutCommand, WmSessionCommand},
    config::KeyboardConfiguration,
    daemon::{
        error::{DaemonInitError, DaemonKeyboardInitError},
        focus::FocusHandler,
        keyboard::{KeyboardHandler, NoKeyboard},
        mouse::{MouseHandler, NoMouse},
    },
    input::Key,
    layout::{Layout, LayoutContext, LayoutWindow, NoLayout},
    server::{event::BackendEventHandler, ServerState},
    surface::{ArrangeContext, SurfaceLogicalRectangle, SurfaceLogicalSize},
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

impl<Keyboard, Mouse, Backend: GesWmBackend<ServerState>, L> Daemon<Keyboard, Mouse, Backend, L>
where
    Daemon<Keyboard, Mouse, Backend, L>: KeyboardHandler + MouseHandler + FocusHandler,
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
                && focused_surface == surface {
                    self.clear_focus();
                }
        }

        if !removed_surfaces.is_empty() {
            tracing::debug!(?removed_surfaces, "pruned dead windows");
            self.server_state.mark_layout_dirty();
        }
    }

    fn arrange_windows(&mut self) {
        self.server_state
            .sync_output(self.backend.output_description());

        let physical_size = self.backend.window_size();
        let output_size = SurfaceLogicalSize::from((physical_size.w, physical_size.h));

        let mut layout_windows = self
            .server_state
            .windows
            .iter()
            .map(|window| LayoutWindow {
                geometry: window.geometry,
                focused: self.server_state.is_focused(&window.surface),
            })
            .collect::<Vec<_>>();

        let mut ctx = LayoutContext {
            output_size,
            windows: &mut layout_windows,
        };

        self.layout.arrange(&mut ctx);

        for (window, layout_window) in self
            .server_state
            .windows
            .iter_mut()
            .zip(layout_windows.into_iter())
        {
            let outer_geometry = layout_window.geometry;
            window.geometry = outer_geometry;

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

            let size_changed = window.toplevel.with_pending_state(|state| {
                state
                    .size
                    .replace(size)
                    .map(|old| old != size)
                    .unwrap_or(true)
            });

            if size_changed {
                tracing::debug!(?window, "window size changed");
                window.toplevel.send_configure();
            }
        }

        self.server_state.layout_dirty = false;
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
                std::process::exit(exit_code);
            }
        };

        let mut commands: Vec<Option<WmSessionCommand>> = Vec::new();
        for event in &event_queue {
            commands.push(self.handle_backend_event(event));
        }

        for command in commands.into_iter().flatten() {
            self.exec(&command);
        }
    }

    fn move_focused_window_down(&mut self) {
        let Some(focused_surface) = self.server_state.focused_window.as_ref() else {
            return;
        };

        let Some(index) = self
            .server_state
            .windows
            .iter()
            .position(|window| &window.surface == focused_surface)
        else {
            return;
        };

        if index == 0 {
            return;
        }

        self.server_state.windows.swap(index, index - 1);
        self.server_state.mark_layout_dirty();
    }

    fn move_focused_window_up(&mut self) {
        let Some(focused_surface) = self.server_state.focused_window.as_ref() else {
            return;
        };

        let Some(index) = self
            .server_state
            .windows
            .iter()
            .position(|window| &window.surface == focused_surface)
        else {
            return;
        };

        if index + 1 >= self.server_state.windows.len() {
            return;
        }

        self.server_state.windows.swap(index, index + 1);
        self.server_state.mark_layout_dirty();
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
