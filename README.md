# geswm

A tiling wayland compositor

## Usage

`geswm` is configured with source code:

```sh
$ cargo new --bin my-wm
$ cargo add geswm --features winit
```

Take a look at the [examples](./examples) directory, or configure your own:

```rs
use geswm::{
    backend::{GesWmBackend, WinitBackend},
    cmd::WmSessionCommand,
    config::{KeyboardConfiguration, RgbaColor},
    daemon::{Daemon, DaemonExit},
    input::XkbKeyCode,
    layout::MasterStackLayout,
    surface::SurfaceBorderTransformer,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let DaemonExit::Requested(exit_code) = daemon.run_with_signal_handlers()?;
    tracing::info!(exit_code, "server exited");
    Ok(())
}
```
