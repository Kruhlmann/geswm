use smithay::{
    output::{Mode, Output as SmithayOutput, PhysicalProperties, Scale},
    utils::Transform,
    wayland::output::WlOutputData,
};
use wayland_server::{
    backend::GlobalId, protocol::wl_output::WlOutput, DisplayHandle, GlobalDispatch,
};

use crate::surface::SurfaceLogicalPosition;

pub struct WlOutputAdapter {
    _global_id: GlobalId,
    inner: SmithayOutput,
}

impl WlOutputAdapter {
    pub fn new<D>(
        display_handle: &DisplayHandle,
        name: impl Into<String>,
        physical_properties: PhysicalProperties,
        initial_mode: Mode,
    ) -> Self
    where
        D: GlobalDispatch<WlOutput, WlOutputData>,
        D: 'static,
    {
        let inner = SmithayOutput::new(name.into(), physical_properties);
        inner.change_current_state(
            Some(initial_mode),
            Some(Transform::Normal),
            Some(Scale::Integer(1)),
            Some((0, 0).into()),
        );
        inner.set_preferred(initial_mode);

        let global_id = inner.create_global::<D>(display_handle);

        Self {
            _global_id: global_id,
            inner,
        }
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.set_mode(Mode {
            size: (width, height).into(),
            refresh: 60_000,
        });
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.inner
            .change_current_state(Some(mode), None, None, None);
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.inner
            .change_current_state(None, Some(transform), None, None);
    }

    pub fn set_scale(&mut self, scale: i32) {
        self.inner
            .change_current_state(None, None, Some(Scale::Integer(scale)), None);
    }

    pub fn set_location(&mut self, location: SurfaceLogicalPosition) {
        self.inner
            .change_current_state(None, None, None, Some(location));
    }
}
