#[cfg(feature = "winit")]
use smithay::backend::input::{AbsolutePositionEvent, Event, KeyboardKeyEvent, PointerButtonEvent};
use smithay::backend::input::{ButtonState, KeyState, Keycode};
#[cfg(feature = "winit")]
use smithay::backend::winit::WinitEvent;

use crate::surface::SurfacePhysicalSize;

#[derive(Debug)]
pub enum InputEvent {
    Keyboard {
        time: u64,
        key: Keycode,
        state: KeyState,
    },
    PointerMotionAbsolute {
        time: u64,
        x: f64,
        y: f64,
    },
    PointerButton {
        time: u64,
        button_code: u32, // TODO
        state: ButtonState,
    },
    PointerAxis {
        time: u64,
    },
    Unimplemented,
}

#[cfg(feature = "winit")]
impl From<smithay::backend::input::InputEvent<smithay::backend::winit::WinitInput>> for InputEvent {
    fn from(
        event: smithay::backend::input::InputEvent<smithay::backend::winit::WinitInput>,
    ) -> Self {
        match event {
            smithay::backend::input::InputEvent::Keyboard { event } => InputEvent::Keyboard {
                time: event.time(),
                key: event.key_code(),
                state: event.state(),
            },
            smithay::backend::input::InputEvent::PointerMotionAbsolute { event } => {
                InputEvent::PointerMotionAbsolute {
                    time: event.time(),
                    x: event.x(),
                    y: event.y(),
                }
            }
            smithay::backend::input::InputEvent::PointerButton { event } => {
                InputEvent::PointerButton {
                    time: event.time(),
                    button_code: event.button_code(),
                    state: event.state(),
                }
            }
            smithay::backend::input::InputEvent::PointerAxis { event } => {
                InputEvent::PointerAxis { time: event.time() } // TODO
            }
            smithay::backend::input::InputEvent::DeviceAdded { device } => {
                tracing::debug!(?device, "new device");
                InputEvent::Unimplemented
            }
            e => unimplemented!("Unsupported input event: {:?}", e),
        }
    }
}

#[derive(Debug)]
pub enum BackendEvent {
    CloseRequested,
    FocusGained,
    FocusLost,
    Input(InputEvent),
    Redraw,
    Resize {
        size: SurfacePhysicalSize,
        scale: f64,
    },
}

#[cfg(feature = "winit")]
impl From<WinitEvent> for BackendEvent {
    fn from(event: WinitEvent) -> Self {
        match event {
            WinitEvent::CloseRequested => BackendEvent::CloseRequested,
            WinitEvent::Focus(true) => BackendEvent::FocusGained,
            WinitEvent::Focus(false) => BackendEvent::FocusLost,
            WinitEvent::Resized { size, scale_factor } => BackendEvent::Resize {
                size,
                scale: scale_factor,
            },
            WinitEvent::Input(e) => BackendEvent::Input(e.into()),
            WinitEvent::Redraw => BackendEvent::Redraw,
        }
    }
}
