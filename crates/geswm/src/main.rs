use geswm::{
    backend::WinitBackend, cmd::UserCommand, config::KeyboardConfiguration, daemon::Daemon,
    input::XkbKeyCode, layout::MasterStackLayout,
};
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_FILTER: &str = "info,backend_winit=warn,smithay=info,wayland_server=warn";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_log_directives =
        std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_FILTER.to_owned());
    let env_filter = EnvFilter::builder().parse_lossy(env_log_directives);
    tracing_subscriber::fmt()
        .compact()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter)
        .with_ansi_sanitization(false)
        .init();
    let keyboard_config = KeyboardConfiguration {
        rules: "evdev",
        model: "pc104",
        layout: "us",
        variant: "altgr-intl",
        options: Some("caps:escape".to_string()),
        repeat_delay: 150,
        repeat_rate: 30,
    };
    let backend = WinitBackend::new_gles_renderer()?;
    let mut daemon = Daemon::new()?
        .with_mouse()
        .with_backend(backend)
        .with_keyboard(keyboard_config)?
        .with_initial_layout(MasterStackLayout::default())
        .bind_key(XkbKeyCode::Return.with_shift(), vec!["alacritty"].into())
        .bind_key(XkbKeyCode::T.with_shift(), UserCommand::CloseFocused);
    daemon.run();
}
