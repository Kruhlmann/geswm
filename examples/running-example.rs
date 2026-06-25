use geswm::{
    backend::{GesWmBackend, WinitBackend},
    cmd::WmSessionCommand,
    config::{KeyboardConfiguration, RgbaColor},
    daemon::{Daemon, DaemonExit},
    input::XkbKeyCode,
    layout::MasterStackLayout,
    surface::SurfaceBorderTransformer,
};
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
    let border_transformer = SurfaceBorderTransformer::new(
        5,
        RgbaColor::from_hex("#009999ff"),
        RgbaColor::from_hex("#999999ff"),
    );
    let backend = WinitBackend::new_gles_renderer()?
        .add_transformer(border_transformer)
        .set_background_color(RgbaColor::from_hex("#211317"));
    let mut daemon = Daemon::new()?
        .with_mouse()
        .with_backend(backend)
        .with_keyboard(keyboard_config)?
        .with_initial_layout(MasterStackLayout::default())
        .bind(XkbKeyCode::Return.with_shift(), vec!["alacritty"])
        .bind(XkbKeyCode::D.with_shift(), vec!["rofi", "-show", "drun"])
        .bind(XkbKeyCode::Q.with_shift(), WmSessionCommand::CloseFocused);

    let DaemonExit::Requested(exit_code) = daemon.run_with_signal_handlers()?;
    tracing::info!(exit_code, "server exited");
    Ok(())
}
