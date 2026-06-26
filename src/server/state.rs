use smithay::{
    input::{Seat, SeatState},
    wayland::{
        compositor::CompositorState,
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{XdgShellState, decoration::XdgDecorationState},
        },
        shm::ShmState,
    },
};
use wayland_server::protocol::wl_surface::WlSurface;

use crate::{
    output::{OutputDescription, WlOutputAdapter},
    server::{WaylandSocket, WaylandSocketInitError},
    surface::{SurfaceGeometry, Window},
};

pub struct ServerState {
    pub display_handle: wayland_server::DisplayHandle,
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
    pub output: Option<WlOutputAdapter>,
    pub windows: Vec<Window>,
    pub focused_window: Option<WlSurface>,
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
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let data_device_state = DataDeviceState::new::<Self>(&display_handle);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "geswm");
        Ok(Self {
            display_handle,
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
            output: None,
            windows: Vec::new(),
            focused_window: None,
            layout_dirty: true,
        })
    }

    pub fn prune(&mut self) {
        let mut removed_surfaces = Vec::new();
        self.windows.retain(|window| {
            if window.is_alive() {
                true
            } else {
                removed_surfaces.push(window.surface.clone());
                false
            }
        });

        if !removed_surfaces.is_empty() {
            tracing::debug!(?removed_surfaces, "pruned dead windows");
            self.mark_layout_dirty();
        }
    }

    pub fn socket_name(&self) -> &str {
        self.socket.display_name()
    }

    pub fn add_window(&mut self, window: Window) {
        self.focused_window = Some(window.surface.clone());
        self.windows.push(window);
        self.mark_layout_dirty();
    }

    pub fn remove_window_by_surface(&mut self, surface: &WlSurface) -> Option<WlSurface> {
        let removed_index = self
            .windows
            .iter()
            .position(|window| window.surface() == surface)?;

        let removed_was_focused = self.focused_window.as_ref() == Some(surface);

        self.windows.remove(removed_index);

        let new_focus = if removed_was_focused {
            self.focus_target_after_removal_prefer_next(removed_index)
                .map(|window| window.surface().clone())
        } else {
            self.focused_window.clone()
        };

        self.focused_window = new_focus.clone();
        self.mark_layout_dirty();

        new_focus
    }

    fn focus_target_after_removal_prefer_next(&self, removed_index: usize) -> Option<&Window> {
        if self.windows.is_empty() {
            return None;
        }

        let focus_index = if removed_index < self.windows.len() {
            removed_index
        } else {
            self.windows.len() - 1
        };

        self.windows.get(focus_index)
    }

    pub fn window_for_surface(&self, surface: &WlSurface) -> Option<&Window> {
        self.windows
            .iter()
            .find(|window| window.matches_surface(surface))
    }

    pub fn window_for_surface_mut(&mut self, surface: &WlSurface) -> Option<&mut Window> {
        self.windows
            .iter_mut()
            .find(|window| window.matches_surface(surface))
    }

    pub fn geometry_for_surface(&self, surface: &WlSurface) -> Option<SurfaceGeometry> {
        self.window_for_surface(surface)
            .map(|window| window.geometry)
    }

    pub fn geometry_for_surface_or_default(&self, surface: &WlSurface) -> SurfaceGeometry {
        self.geometry_for_surface(surface).unwrap_or_else(|| {
            tracing::warn!(?surface, "No surface geometry; using default");
            SurfaceGeometry::default()
        })
    }

    pub fn set_geometry_for_surface(&mut self, surface: &WlSurface, geometry: SurfaceGeometry) {
        match self.window_for_surface_mut(surface) {
            Some(window) => {
                window.geometry = geometry;
            }
            None => {
                tracing::warn!(
                    ?surface,
                    ?geometry,
                    "Tried to set geometry for unknown surface"
                );
            }
        }
    }

    pub fn is_focused(&self, surface: &WlSurface) -> bool {
        self.focused_window
            .as_ref()
            .is_some_and(|focused| focused == surface)
    }

    pub fn set_focused_surface(&mut self, surface: Option<WlSurface>) {
        self.focused_window = surface;
    }

    pub fn focus_window(&mut self, surface: &WlSurface) {
        if self.window_for_surface(surface).is_none() {
            tracing::warn!(?surface, "Tried to focus unknown surface");
            return;
        }
        self.focused_window = Some(surface.clone());
        if let Some(index) = self
            .windows
            .iter()
            .position(|window| window.matches_surface(surface))
        {
            let window = self.windows.remove(index);
            self.windows.push(window);
        }

        self.mark_layout_dirty();
    }

    pub fn iter_windows_bottom_to_top(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    pub fn iter_windows_top_to_bottom(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter().rev()
    }

    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
    }

    pub fn sync_output(&mut self, description: OutputDescription) {
        match self.output.as_mut() {
            Some(output) => output.apply_state(description.state),
            None => {
                self.output = Some(WlOutputAdapter::new::<Self>(
                    &self.display_handle,
                    description,
                ));
            }
        }
    }
}
