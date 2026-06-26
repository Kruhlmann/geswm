#[cfg(feature = "winit")]
use smithay::output::Subpixel;
use smithay::{
    output::{Mode, Output as SmithayOutput, PhysicalProperties, Scale},
    utils::Transform,
    wayland::output::WlOutputData,
};
use wayland_server::{
    DisplayHandle, GlobalDispatch, backend::GlobalId, protocol::wl_output::WlOutput,
};

use crate::surface::SurfaceLogicalPosition;

#[derive(Clone, Debug)]
pub struct OutputState {
    pub mode: Mode,
    pub scale: Scale,
    pub transform: Transform,
    pub location: SurfaceLogicalPosition,
}

#[derive(Clone, Debug)]
pub struct OutputDescription {
    pub name: String,
    pub physical_properties: PhysicalProperties,
    pub state: OutputState,
}

impl OutputDescription {
    #[cfg(feature = "winit")]
    pub fn virtual_output(
        name: impl Into<String>,
        width: i32,
        height: i32,
        refresh_rate: i32,
    ) -> Self {
        Self {
            name: name.into(),
            physical_properties: PhysicalProperties {
                size: (width, height).into(),
                subpixel: Subpixel::Unknown,
                make: "geswm".into(),
                model: "virtual".into(),
            },
            state: OutputState {
                mode: Mode {
                    size: (width, height).into(),
                    refresh: refresh_rate,
                },
                scale: Scale::Integer(1),
                transform: Transform::Normal,
                location: (0, 0).into(),
            },
        }
    }
}

pub trait OutputDiscovery {
    fn output_description(&self) -> OutputDescription;
}

pub struct WlOutputAdapter {
    _global_id: GlobalId,
    inner: SmithayOutput,
}

impl WlOutputAdapter {
    pub fn new<D>(display_handle: &DisplayHandle, description: OutputDescription) -> Self
    where
        D: GlobalDispatch<WlOutput, WlOutputData>,
        D: 'static,
    {
        let inner = SmithayOutput::new(description.name, description.physical_properties);
        Self::apply_state_to_inner(&inner, description.state.clone());
        inner.set_preferred(description.state.mode);

        let global_id = inner.create_global::<D>(display_handle);

        Self {
            _global_id: global_id,
            inner,
        }
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

    pub fn apply_state(&mut self, state: OutputState) {
        Self::apply_state_to_inner(&self.inner, state);
    }

    fn apply_state_to_inner(inner: &SmithayOutput, state: OutputState) {
        inner.change_current_state(
            Some(state.mode),
            Some(state.transform),
            Some(state.scale),
            Some(state.location),
        );
    }
}
