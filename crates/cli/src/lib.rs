use geswm::{
    backend::{GesWmBackend, WinitBackend},
    cmd::WmSessionCommand,
    config::{KeyboardConfiguration, RgbaColor},
    daemon::Daemon,
    input::XkbKeyCode,
    layout::MasterStackLayout,
    surface::SurfaceBorderTransformer,
};

pub mod cli;
pub mod log;

pub fn run_server() -> Result<(), Box<dyn std::error::Error>> {
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
            2,
            RgbaColor::new(0.8, 0.8, 0.8, 1.0),
            RgbaColor::new(0.25, 0.25, 0.25, 1.0),
        ));
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
