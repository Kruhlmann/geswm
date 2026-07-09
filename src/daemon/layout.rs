use smithay::wayland::{compositor::with_states, shell::xdg::SurfaceCachedState};

use crate::{
    backend::GesWmBackend,
    daemon::{Daemon, focus::FocusHandler, keyboard::KeyboardHandler, mouse::MouseHandler},
    layout::{Layout, LayoutContext, LayoutSet, LayoutWindow},
    server::ServerState,
    surface::{ArrangeContext, SurfaceGeometry, SurfaceLogicalRectangle, SurfaceLogicalSize},
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

        // Partition windows into tiled (no parent) and floating (has a parent surface).
        let mut tiled_indices: Vec<usize> = Vec::new();
        let mut floating_indices: Vec<usize> = Vec::new();
        for (i, w) in self.server_state.windows.iter().enumerate() {
            if w.toplevel.parent().is_none() {
                tiled_indices.push(i);
            } else {
                floating_indices.push(i);
            }
        }

        let mut layout_windows = tiled_indices
            .iter()
            .map(|&i| LayoutWindow {
                geometry: self.server_state.windows[i].geometry,
                focused: self
                    .server_state
                    .is_focused(&self.server_state.windows[i].surface),
            })
            .collect::<Vec<_>>();

        let mut ctx = LayoutContext {
            output_size,
            windows: &mut layout_windows,
        };

        self.get_active_layout().arrange(&mut ctx);

        // Apply tiled geometry.
        for (&i, layout_window) in tiled_indices.iter().zip(layout_windows.into_iter()) {
            let window = &mut self.server_state.windows[i];
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

        // Floating windows: clear any compositor-imposed size so the client
        // can render at its natural size.  Then read back what the client
        // actually committed and build the outer geometry from that so
        // border/gap transformers wrap around the real content.
        for &i in &floating_indices {
            let window = &mut self.server_state.windows[i];

            // Clear imposed size once.
            let cleared = window.toplevel.with_pending_state(|state| {
                if state.size.is_some() {
                    state.size = None;
                    true
                } else {
                    false
                }
            });
            if cleared {
                window.toplevel.send_configure();
            }

            // Read the content size the client has committed via set_window_geometry
            // or (if unset) the surface's buffer dimensions.
            let content_size = with_states(window.toplevel.wl_surface(), |states| {
                states
                    .cached_state
                    .get::<SurfaceCachedState>()
                    .current()
                    .geometry
                    .map(|r| r.size)
            })
            .unwrap_or_else(|| window.geometry.size);

            // Account for the inset added by border/gap transformers so the
            // outer rect encompasses the border, not just the client content.
            let pipeline = self.backend.surface_transform_pipeline();
            let probe_size = SurfaceLogicalSize::from((1024, 1024));
            let probe_geo = SurfaceGeometry {
                position: (0, 0).into(),
                size: probe_size,
            };
            let probe_ctx = ArrangeContext {
                focused: false,
                fullscreen: false,
                floating: true,
                output_rect: SurfaceLogicalRectangle::new((0, 0).into(), probe_size),
            };
            let inset = {
                let run = pipeline.begin(probe_geo, probe_ctx).arrange();
                let client = run.client_rect();
                // inset = how much the pipeline shrank the rect on each side
                (probe_size.w - client.size.w, probe_size.h - client.size.h)
            };

            let outer_w = content_size.w + inset.0;
            let outer_h = content_size.h + inset.1;

            // Center on the output.
            let cx = (output_size.w - outer_w).max(0) / 2;
            let cy = (output_size.h - outer_h).max(0) / 2;

            window.geometry = SurfaceGeometry {
                position: (cx, cy).into(),
                size: (outer_w, outer_h).into(),
            };
        }

        self.server_state.layout_dirty = false;
    }
}
