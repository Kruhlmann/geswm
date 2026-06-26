use crate::daemon::Daemon;

pub trait WindowManager {
    fn close_focused_window(&mut self) {}
    fn cycle_layout(&mut self);
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

    fn cycle_layout(&mut self) {
        let next_index = (self.active_layout + 1) % self.layouts.len();
        self.active_layout = next_index;
        tracing::info!("cycling layout to {:?}", self.layouts[self.active_layout]);
    }
}
