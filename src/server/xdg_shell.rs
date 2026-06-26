use smithay::{
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

use wayland_server::protocol::wl_seat;

use crate::{server::ServerState, surface::Window};

impl XdgShellHandler for ServerState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, toplevel: ToplevelSurface) {
        self.add_window(Window::new(toplevel));
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        surface.send_configure().unwrap();
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        unreachable!("grab");
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        _positioner: PositionerState,
        token: u32,
    ) {
        surface.send_repositioned(token);
        surface.send_configure().unwrap();
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        self.remove_window_by_surface(surface.wl_surface());
    }
}

smithay::delegate_xdg_shell!(ServerState);
