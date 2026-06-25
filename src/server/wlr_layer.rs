use smithay::wayland::shell::wlr_layer::{Layer, LayerSurface};
use wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface};

use crate::surface::SurfaceGeometry;

#[derive(Debug, Clone)]
pub struct LayerWindow {
    pub surface: LayerSurface,
    pub wl_surface: WlSurface,
    pub output: Option<WlOutput>,
    pub layer: Layer,
    pub namespace: String,
    pub geometry: SurfaceGeometry,
}

impl LayerWindow {
    pub fn new(
        surface: LayerSurface,
        output: Option<WlOutput>,
        layer: Layer,
        namespace: String,
    ) -> Self {
        let wl_surface = surface.wl_surface().clone();

        Self {
            surface,
            wl_surface,
            output,
            layer,
            namespace,
            geometry: SurfaceGeometry::default(),
        }
    }

    pub fn matches_surface(&self, surface: &WlSurface) -> bool {
        &self.wl_surface == surface
    }
}
