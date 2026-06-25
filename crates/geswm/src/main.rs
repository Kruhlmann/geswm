use geswm::{
    backend::WinitBackend, cmd::WmSessionCommand, config::KeyboardConfiguration, daemon::Daemon,
    input::XkbKeyCode, layout::MasterStackLayout,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    geswm::log::setup_logging();
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
    Daemon::new()?
        .with_mouse()
        .with_backend(backend)
        .with_keyboard(keyboard_config)?
        .with_initial_layout(MasterStackLayout::default())
        .bind(XkbKeyCode::Return.with_shift(), vec!["alacritty"])
        .bind(XkbKeyCode::D.with_shift(), vec!["rofi", "-show", "drun"])
        .bind(XkbKeyCode::Q.with_shift(), WmSessionCommand::CloseFocused)
        .run();
}
