use smithay::wayland::shell::xdg::ToplevelSurface;
use wayland_server::protocol::wl_surface::WlSurface;

use crate::surface::SurfaceGeometry;

#[derive(Debug, Clone)]
pub struct Window {
    pub toplevel: ToplevelSurface,
    pub surface: WlSurface,
    pub geometry: SurfaceGeometry,
    pub fullscreen: bool,
    pub floating: bool,
}

impl Window {
    pub fn new(toplevel: ToplevelSurface) -> Self {
        let surface = toplevel.wl_surface().clone();

        Self {
            toplevel,
            surface,
            geometry: SurfaceGeometry::default(),
            fullscreen: false,
            floating: false,
        }
    }

    pub fn surface(&self) -> &WlSurface {
        &self.surface
    }

    pub fn matches_surface(&self, surface: &WlSurface) -> bool {
        &self.surface == surface
    }

    pub fn is_alive(&self) -> bool {
        self.toplevel.alive()
    }

    pub fn close(&self) {
        self.toplevel.send_close();
    }
}
