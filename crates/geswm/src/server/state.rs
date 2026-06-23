use std::{collections::HashMap, os::unix::io::OwnedFd};

use smithay::{
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::protocol::wl_seat,
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        selection::{
            SelectionHandler,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState},
    },
};
use wayland_protocols::xdg::shell::server::xdg_toplevel;
use wayland_server::{Client, protocol::wl_surface::WlSurface};

use crate::{
    backend::WindowGeometry,
    client::ClientState,
    server::{WaylandSocket, WaylandSocketInitError},
};

pub struct ServerState {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub seat: Seat<Self>,
    pub socket: WaylandSocket,
    pub geometries: HashMap<WlSurface, WindowGeometry>,
    pub focused_surface: Option<WlSurface>,
    pub layout_dirty: bool,
}

impl ServerState {
    pub fn from_display(
        display: &wayland_server::Display<Self>,
    ) -> Result<Self, WaylandSocketInitError> {
        let display_handle = display.handle();
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "geswm");
        let socket = WaylandSocket::try_autocreate()
            .inspect(|s| tracing::info!("listening on socket {}", s.name))?;
        Ok(Self {
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            data_device_state,
            seat,
            socket,
            geometries: HashMap::new(),
            focused_surface: None,
            layout_dirty: true,
        })
    }

    pub fn socket_name(&self) -> &str {
        self.socket.display_name()
    }

    pub fn geometry_for_surface(&self, surface: &WlSurface) -> Option<WindowGeometry> {
        self.geometries.get(surface).copied()
    }

    pub fn geometry_for_surface_or_default(&self, surface: &WlSurface) -> WindowGeometry {
        self.geometries
            .get(surface)
            .copied()
            .unwrap_or_else(|| crate::backend::WindowGeometry {
                position: (0, 0).into(),
                size: (1, 1).into(),
            })
    }

    pub fn set_geometry_for_surface(&mut self, surface: WlSurface, geometry: WindowGeometry) {
        self.geometries.insert(surface, geometry);
    }

    pub fn is_focused(&self, surface: &WlSurface) -> bool {
        self.focused_surface
            .as_ref()
            .is_some_and(|focused| focused == surface)
    }

    pub fn set_focused_surface(&mut self, surface: Option<WlSurface>) {
        self.focused_surface = surface;
    }

    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
    }
}

impl BufferHandler for ServerState {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}

impl CompositorHandler for ServerState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("client data missing")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);
    }
}

impl XdgShellHandler for ServerState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });

        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
    }
}

impl ShmHandler for ServerState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for ServerState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
}

impl SelectionHandler for ServerState {
    type SelectionUserData = ();
}

impl DataDeviceHandler for ServerState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for ServerState {}

impl ServerDndGrabHandler for ServerState {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}

smithay::delegate_compositor!(ServerState);
smithay::delegate_xdg_shell!(ServerState);
smithay::delegate_shm!(ServerState);
smithay::delegate_seat!(ServerState);
smithay::delegate_data_device!(ServerState);
