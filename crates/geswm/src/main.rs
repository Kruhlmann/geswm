use geswm::{
    backend::WinitBackend,
    daemon::{Daemon, bind::KeyBind},
    layout::MasterStackLayout,
};
use smithay::input::keyboard::XkbConfig;
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
    let keyboard_config = XkbConfig {
        rules: "evdev",
        model: "pc104",
        layout: "us",
        variant: "altgr-intl",
        options: Some("caps:escape".to_string()),
    };
    let backend = WinitBackend::new_gles_renderer()?;
    let mut daemon = Daemon::new()?
        .with_mouse()
        .with_backend(backend)
        .with_keyboard(Some(keyboard_config), 200, 20)?
        .with_initial_layout(MasterStackLayout::default())
        .bind_key(
            KeyBind::new(36u32.into()).with_shift(),
            vec!["alacritty"].into(),
        );
    daemon.run();
}
