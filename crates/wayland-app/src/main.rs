use std::{fs::File, io::Write, os::unix::io::AsFd};

use wayland_client::{
    delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Connection, Dispatch, QueueHandle, WEnum,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const STRIDE: i32 = (WIDTH * 4) as i32;
const SOLID_BLUE_GRAY: [u8; 4] = [0x7d, 0x55, 0x2d, 0xff];

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    conn.display().get_registry(&qh, ());

    let mut state = State {
        running: true,
        configured: false,
        surface: None,
        buffer: None,
        wm_base: None,
        xdg_surface: None,
    };

    eprintln!("wayland-app started");
    while state.running {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

struct State {
    running: bool,
    configured: bool,
    surface: Option<wl_surface::WlSurface>,
    buffer: Option<wl_buffer::WlBuffer>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg_surface: Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
}

impl State {
    fn init_xdg_surface(&mut self, qh: &QueueHandle<Self>) {
        let wm_base = self.wm_base.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();

        let xdg_surface = wm_base.get_xdg_surface(surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("Wayland solid color".into());

        surface.commit();
        self.xdg_surface = Some((xdg_surface, toplevel));
    }

    fn present(&self) {
        if !self.configured {
            return;
        }

        let surface = self.surface.as_ref().unwrap();
        let buffer = self.buffer.as_ref().unwrap();

        surface.attach(Some(buffer), 0, 0);
        surface.damage(0, 0, WIDTH as i32, HEIGHT as i32);
        surface.commit();
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        eprintln!("registry event: {event:?}");
        if let wl_registry::Event::Global { name, interface, .. } = event {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    state.surface = Some(compositor.create_surface(qh, ()));

                    if state.wm_base.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());
                    let mut file = tempfile::tempfile().unwrap();
                    draw_solid(&mut file);
                    let pool = shm.create_pool(file.as_fd(), (WIDTH * HEIGHT * 4) as i32, qh, ());
                    let buffer = pool.create_buffer(
                        0,
                        WIDTH as i32,
                        HEIGHT as i32,
                        STRIDE,
                        wl_shm::Format::Argb8888,
                        qh,
                        (),
                    );
                    state.buffer = Some(buffer);
                    state.present();
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "xdg_wm_base" => {
                    state.wm_base = Some(registry.bind::<xdg_wm_base::XdgWmBase, _, _>(
                        name,
                        1,
                        qh,
                        (),
                    ));

                    if state.surface.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        eprintln!("wm_base event: {event:?}");
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        eprintln!("xdg_surface event: {event:?}");
        if let xdg_surface::Event::Configure { serial } = event {
            xdg_surface.ack_configure(serial);
            state.configured = true;
            state.present();
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        eprintln!("xdg_toplevel event: {event:?}");
        if let xdg_toplevel::Event::Close = event {
            state.running = false;
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        eprintln!("seat event: {event:?}");
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
            && capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        eprintln!("keyboard event: {event:?}");

        if let wl_keyboard::Event::Key { key, .. } = event
            && key == 1 {
                state.running = false;
            }
    }
}

delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

fn draw_solid(file: &mut File) {
    let mut writer = std::io::BufWriter::new(file);
    for _ in 0..(WIDTH * HEIGHT) {
        writer.write_all(&SOLID_BLUE_GRAY).unwrap();
    }
    writer.flush().unwrap();
}
