use geswm::daemon::Daemon;
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
    let mut daemon = Daemon::new()?.add_keyboard(None, 200, 200)?;
    daemon.run();
}
