use smithay::{backend::input::ButtonState, input::pointer::PointerHandle};

use crate::{cmd::Cmd, daemon::Daemon, server::ServerState};

pub type NoMouse = ();

pub trait MouseHandler {
    fn on_mouse_input_event(
        &mut self,
        _time: &u64,
        _button: &u32,
        _state: &ButtonState,
    ) -> Option<Cmd> {
        None
    }
    fn on_mouse_moved_event(&mut self, _time: &u64, _x: &f64, _y: &f64) -> Option<Cmd> {
        None
    }
    fn on_mouse_wheel_event(&mut self, _time: &u64) -> Option<Cmd> {
        None
    }
}

impl<Keyboard, Backend, L> MouseHandler
    for Daemon<Keyboard, PointerHandle<ServerState>, Backend, L>
{
    fn on_mouse_input_event(
        &mut self,
        _time: &u64,
        _button: &u32,
        _state: &ButtonState,
    ) -> Option<Cmd> {
        None
    }

    fn on_mouse_moved_event(&mut self, _time: &u64, _x: &f64, _y: &f64) -> Option<Cmd> {
        None
    }

    fn on_mouse_wheel_event(&mut self, _time: &u64) -> Option<Cmd> {
        None
    }
}

impl<Keyboard, Backend, L> MouseHandler for Daemon<Keyboard, NoMouse, Backend, L> {}
