use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use geswm::{
    backend::{GesWmBackend, WinitBackend},
    cmd::WmSessionCommand,
    config::{KeyboardConfiguration, RgbaColor},
    daemon::{Daemon, DaemonExit},
    input::XkbKeyCode,
    layout::MasterStackLayout,
    surface::SurfaceBorderTransformer,
};
use signal_hook::{consts::signal::{SIGINT, SIGTERM}, flag};

pub mod cli;
pub mod log;

pub fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, Arc::clone(&shutdown_requested))?;
    flag::register(SIGTERM, Arc::clone(&shutdown_requested))?;

    let keyboard_config = KeyboardConfiguration {
        rules: "evdev",
        model: "pc104",
        layout: "us",
        variant: "altgr-intl",
        options: Some("caps:escape".to_string()),
        repeat_delay: 150,
        repeat_rate: 30,
    };
    let backend =
        WinitBackend::new_gles_renderer()?.add_transformer(SurfaceBorderTransformer::new(
            5,
            RgbaColor::from_hex("#009999ff"),
            RgbaColor::from_hex("#999999ff"),
        ));
    let mut daemon = Daemon::new()?
        .with_mouse()
        .with_backend(backend)
        .with_keyboard(keyboard_config)?
        .with_initial_layout(MasterStackLayout::default())
        .bind(XkbKeyCode::Return.with_shift(), vec!["alacritty"])
        .bind(XkbKeyCode::D.with_shift(), vec!["rofi", "-show", "drun"])
        .bind(XkbKeyCode::Q.with_shift(), WmSessionCommand::CloseFocused);

    let DaemonExit::Requested(exit_code) =
        daemon.run_until(|| shutdown_requested.load(Ordering::Relaxed));
    tracing::info!(exit_code, "server exited");
    Ok(())
}
