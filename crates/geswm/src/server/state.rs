use std::collections::HashMap;

use smithay::{
    input::{Seat, SeatState},
    wayland::{
        compositor::CompositorState,
        selection::data_device::DataDeviceState,
        shell::xdg::{XdgShellState, decoration::XdgDecorationState},
        shm::ShmState,
    },
};
use wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    server::{WaylandSocket, WaylandSocketInitError},
    surface::SurfaceGeometry,
};

pub struct ServerState {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub xdg_decoration_state: XdgDecorationState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub seat: Seat<Self>,
    pub socket: WaylandSocket,
    pub geometries: HashMap<WlSurface, SurfaceGeometry>,
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
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "geswm");
        let socket = WaylandSocket::try_autocreate()?;
        Ok(Self {
            compositor_state,
            xdg_shell_state,
            xdg_decoration_state,
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

    pub fn geometry_for_surface(&self, surface: &WlSurface) -> Option<SurfaceGeometry> {
        self.geometries.get(surface).copied()
    }

    pub fn geometry_for_surface_or_default(&self, surface: &WlSurface) -> SurfaceGeometry {
        self.geometries.get(surface).copied().unwrap_or_else(|| {
            tracing::warn!(?surface, "No surface geometry; using default");
            SurfaceGeometry::default()
        })
    }

    pub fn set_geometry_for_surface(&mut self, surface: WlSurface, geometry: SurfaceGeometry) {
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
