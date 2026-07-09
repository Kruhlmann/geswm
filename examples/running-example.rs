use geswm::prelude::*;
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_FILTER: &str = "info,backend_winit=warn,smithay=info,wayland_server=warn";

pub fn setup_logging() {
    let env_log_directives = std::env::var("RUST_LOG").unwrap_or(DEFAULT_LOG_FILTER.to_owned());
    let env_filter = EnvFilter::builder().parse_lossy(env_log_directives);
    tracing_subscriber::fmt()
        .compact()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .with_ansi_sanitization(false)
        .init();
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let keyboard_config = KeyboardConfiguration {
        rules: "evdev",
        model: "pc104",
        layout: "us",
        variant: "altgr-intl",
        options: Some("caps:escape".to_string()),
        repeat_delay: 150,
        repeat_rate: 30,
    };
    let border_transformer = SurfaceBorderTransformer::new()
        .width(5)
        .focused_color("#009999ff")
        .unfocused_color("#999999ff");
    let backend = WinitBackend::new_gles_renderer()?
        .add_transformer(border_transformer)
        .set_background_color("#211317")
        .set_refresh_rate(60.0);
    Daemon::new()?
        .with_mouse()
        .with_backend(backend)
        .with_keyboard(keyboard_config)?
        .with_layout(MasterStackLayout::new(50))
        .with_layout(MasterStackLayout::new(10))
        .startup(vec!["wbg", "-s", "examples/bg.png"])
        .startup("mako")
        .bind(Key::Shift | Key::Return, "alacritty")
        .bind(Key::Shift | Key::D, vec!["rofi", "-show", "drun"])
        .bind(Key::Shift | Key::Tab, Cmd::Layout(LayoutCmd::CycleLayout))
        .bind(Key::Shift | Key::H, Cmd::Layout(LayoutCmd::Shrink))
        .bind(Key::Shift | Key::L, Cmd::Layout(LayoutCmd::Grow))
        .bind(Key::Shift | Key::K, Cmd::Layout(LayoutCmd::FocusPrev))
        .bind(Key::Shift | Key::J, Cmd::Layout(LayoutCmd::FocusNext))
        .bind(Key::Shift | Key::I, Cmd::Layout(LayoutCmd::IncrementMaster))
        .bind(Key::Shift | Key::O, Cmd::Layout(LayoutCmd::DecrementMaster))
        .bind(Key::Ctrl | Key::K, Cmd::Layout(LayoutCmd::SendDown))
        .bind(Key::Ctrl | Key::J, Cmd::Layout(LayoutCmd::SendUp))
        .bind(Key::Ctrl | Key::Shift | Key::Q, Cmd::Exit(0))
        .bind(Key::Shift | Key::Q, Cmd::CloseFocused)
        .run();
}
