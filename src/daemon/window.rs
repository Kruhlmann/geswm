use crate::daemon::Daemon;

pub trait WindowManager {
    fn close_focused_window(&mut self) {}
}

impl<Keyboard, Mouse, Backend, L> WindowManager for Daemon<Keyboard, Mouse, Backend, L> {
    fn close_focused_window(&mut self) {
        let Some(focused) = self.server_state.focused_window.as_ref() else {
            return;
        };

        if let Some(window) = self
            .server_state
            .windows
            .iter()
            .find(|window| window.surface() == focused)
        {
            window.close();
        }
    }
}
