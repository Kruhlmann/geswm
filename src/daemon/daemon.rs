use std::{sync::Arc, time::Instant};

use smithay::{
    backend::{input::InputEvent, renderer::gles::GlesRenderer, winit::WinitEvent},
    input::keyboard::{KeyboardHandle, XkbConfig},
};
use winit::platform::pump_events::PumpStatus;

use crate::{
    backend::WinitBackend,
    client::ClientState,
    daemon::{
        error::{DaemonInitError, DaemonKeyboardInitError},
        keyboard::{KeyboardHandler, NoKeyboard},
    },
    server::{ServerState, WaylandSocket},
};

type NoMouse = ();
pub struct Daemon<Keyboard = NoKeyboard, Mouse = NoMouse> {
    socket: WaylandSocket,
    server_state: ServerState,
    display: wayland_server::Display<ServerState>,
    epoch: Instant,
    backend: WinitBackend<GlesRenderer>,
    clients: Vec<wayland_server::Client>,
    keyboard: Keyboard,
    mouse: Mouse,
}

impl Daemon {
    pub fn new() -> Result<Daemon<(), ()>, DaemonInitError> {
        let display: wayland_server::Display<ServerState> = wayland_server::Display::new()?;
        let server_state = ServerState::from_display(&display);
        let socket = WaylandSocket::try_autocreate()
            .inspect(|s| tracing::info!("listening on socket {}", s.name))?;
        let backend = WinitBackend::new_gles_renderer()?;

        Ok(Self {
            socket,
            server_state,
            display,
            epoch: Instant::now(),
            backend,
            clients: vec![],
            keyboard: (),
            mouse: (),
        })
    }
}

impl<Keyboard, Mouse> Daemon<Keyboard, Mouse>
where
    Daemon<Keyboard, Mouse>: KeyboardHandler,
{
    pub fn run(&mut self) -> ! {
        loop {
            self.handle_new_events();
            self.handle_client_connections();
            self.synchronize_clients();
            self.backend.render(&self.server_state, &self.epoch);
        }
    }

    fn handle_new_events(&mut self) {
        let mut event_queue = Vec::new();
        while {
            let pump_status = self
                .backend
                .event_loop
                .dispatch_new_events(|event| event_queue.push(event));
            matches!(pump_status, PumpStatus::Continue)
        } {}

        for event in &event_queue {
            match event {
                WinitEvent::Resized { size: _, scale_factor: _ } => todo!(),
                WinitEvent::Focus(_) => todo!(),
                WinitEvent::Input(input_event) => match input_event {
                    InputEvent::Keyboard { event } => self.on_keyboard_event(event),
                    _ => todo!(),
                },
                WinitEvent::CloseRequested => todo!(),
                WinitEvent::Redraw => todo!(),
            }
        }
    }

    fn handle_client_connections(&mut self) {
        if let Some(unix_stream) = self.socket.accept().unwrap() {
            tracing::info!("new client");
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

impl<Mouse> Daemon<NoKeyboard, Mouse> {
    pub fn add_keyboard(
        mut self,
        xkb_configuration: Option<XkbConfig>,
        repeat_delay: i32,
        repeat_rate: i32,
    ) -> Result<Daemon<KeyboardHandle<ServerState>, Mouse>, DaemonKeyboardInitError> {
        tracing::info!(
            ?xkb_configuration,
            ?repeat_delay,
            ?repeat_rate,
            "add keyboard"
        );

        let xkb = xkb_configuration.unwrap_or_default();
        let xkb_str = format!("{xkb:?}");
        let handle = self
            .server_state
            .seat
            .add_keyboard(xkb, repeat_delay, repeat_rate)
            .map_err(|e| match e {
                smithay::input::keyboard::Error::BadKeymap => {
                    DaemonKeyboardInitError::InvalidKeyMap(xkb_str)
                }
                smithay::input::keyboard::Error::IoError(e) => DaemonKeyboardInitError::Io(e),
            })?;

        let Daemon {
            socket,
            server_state,
            display,
            epoch,
            backend,
            clients,
            keyboard: (),
            mouse,
        } = self;

        Ok(Daemon {
            socket,
            server_state,
            display,
            epoch,
            backend,
            clients,
            keyboard: handle,
            mouse,
        })
    }
}
