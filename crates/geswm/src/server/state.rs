use std::collections::HashMap;

use smithay::{
    input::{Seat, SeatState},
    output::{Mode, PhysicalProperties, Subpixel},
    wayland::{
        compositor::CompositorState,
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{decoration::XdgDecorationState, XdgShellState},
        },
        shm::ShmState,
    },
};
use wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    output::WlOutputAdapter,
    server::{WaylandSocket, WaylandSocketInitError},
    surface::SurfaceGeometry,
};

lazy_static::lazy_static! {
    pub static ref INITIAL_MODE: Mode = Mode { size: (800, 600).into(), refresh: 60_000 };
    pub static ref PHYSICAL_PROPS: PhysicalProperties = PhysicalProperties {
        size: (340, 190).into(),
        subpixel: Subpixel::Unknown,
        make: "geswm".into(),
        model: "winit".into(),
    };
}

pub struct ServerState {
    pub compositor_state: CompositorState,
    pub output_manager_state: OutputManagerState,
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState,
    pub xdg_decoration_state: XdgDecorationState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub seat: Seat<Self>,
    pub socket: WaylandSocket,
    pub output: WlOutputAdapter,
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
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let socket = WaylandSocket::try_autocreate()?;
        let output = WlOutputAdapter::new::<Self>(
            &display_handle,
            socket.socket_name().unwrap().display().to_string(),
            PHYSICAL_PROPS.clone(),
            *INITIAL_MODE,
        );
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "geswm");
        Ok(Self {
            compositor_state,
            output_manager_state,
            xdg_shell_state,
            layer_shell_state,
            xdg_decoration_state,
            shm_state,
            seat_state,
            data_device_state,
            seat,
            socket,
            output,
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

    pub fn set_output_size(&mut self, width: i32, height: i32) {
        self.output.set_size(width, height);
    }
}
