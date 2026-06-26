use crate::{
    backend::GesWmBackend,
    daemon::{Daemon, focus::FocusHandler, keyboard::KeyboardHandler, mouse::MouseHandler},
    layout::{Layout, LayoutContext, LayoutSet, LayoutWindow},
    server::ServerState,
    surface::{ArrangeContext, SurfaceLogicalRectangle, SurfaceLogicalSize},
};

pub trait WindowArranger {
    fn arrange_windows(&mut self);
}

impl<Keyboard, Mouse, Backend> WindowArranger for Daemon<Keyboard, Mouse, Backend, LayoutSet>
where
    Backend: GesWmBackend<ServerState>,
    Daemon<Keyboard, Mouse, Backend, Vec<Box<dyn Layout>>>:
        KeyboardHandler + MouseHandler + FocusHandler,
{
    fn arrange_windows(&mut self) {
        self.server_state
            .sync_output(self.backend.output_description());

        let physical_size = self.backend.window_size();
        let output_size = SurfaceLogicalSize::from((physical_size.w, physical_size.h));

        let mut layout_windows = self
            .server_state
            .windows
            .iter()
            .map(|window| LayoutWindow {
                geometry: window.geometry,
                focused: self.server_state.is_focused(&window.surface),
            })
            .collect::<Vec<_>>();

        let mut ctx = LayoutContext {
            output_size,
            windows: &mut layout_windows,
        };

        self.get_active_layout().arrange(&mut ctx);

        for (window, layout_window) in self
            .server_state
            .windows
            .iter_mut()
            .zip(layout_windows.into_iter())
        {
            let outer_geometry = layout_window.geometry;
            window.geometry = outer_geometry;

            let arrange_ctx = ArrangeContext {
                focused: layout_window.focused,
                fullscreen: false,
                floating: false,
                output_rect: SurfaceLogicalRectangle::new((0, 0).into(), output_size),
            };

            let run = self
                .backend
                .surface_transform_pipeline()
                .begin(outer_geometry, arrange_ctx)
                .arrange();

            let configure_size = run.configure_size();

            let width = configure_size.w.max(1);
            let height = configure_size.h.max(1);
            let size = (width, height).into();

            let size_changed = window.toplevel.with_pending_state(|state| {
                state
                    .size
                    .replace(size)
                    .map(|old| old != size)
                    .unwrap_or(true)
            });

            if size_changed {
                tracing::debug!(?window, "window size changed");
                window.toplevel.send_configure();
            }
        }

        self.server_state.layout_dirty = false;
    }
}
