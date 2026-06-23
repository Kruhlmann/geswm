use smithay::{backend::input::ButtonState, input::pointer::PointerHandle};

use crate::{daemon::Daemon, server::ServerState};

pub type NoMouse = ();

pub trait MouseHandler {
    fn on_mouse_input_event(&mut self, _time: &u64, _button: &u32, _state: &ButtonState) {}
    fn on_mouse_moved_event(&mut self, _time: &u64, _x: &f64, _y: &f64) {}
    fn on_mouse_wheel_event(&mut self, _time: &u64) {}
}

impl<Keyboard, Backend, L> MouseHandler
    for Daemon<Keyboard, PointerHandle<ServerState>, Backend, L>
{
    fn on_mouse_input_event(&mut self, time: &u64, button: &u32, state: &ButtonState) {
        tracing::info!("mouse input: {time:?}, button: {button:?}, state: {state:?}");
    }

    fn on_mouse_moved_event(&mut self, time: &u64, x: &f64, y: &f64) {
        tracing::info!("mouse moved: {time:?}, x: {x:?}, y: {y:?}");
    }

    fn on_mouse_wheel_event(&mut self, time: &u64) {
        tracing::info!("mouse wheel: {time:?}");
    }
}

impl<Keyboard, Backend, L> MouseHandler for Daemon<Keyboard, NoMouse, Backend, L> {}
