pub mod backend;
pub mod client;
pub mod cmd;
pub mod config;
pub mod daemon;
pub mod input;
pub mod layout;
pub mod output;
pub mod server;
pub mod surface;

pub mod prelude {
    pub use crate::backend::{GesWmBackend, WinitBackend};
    pub use crate::cmd::{LayoutCommand, WmSessionCommand};
    pub use crate::config::{KeyboardConfiguration, RgbaColor};
    pub use crate::daemon::Daemon;
    pub use crate::input::Key;
    pub use crate::layout::MasterStackLayout;
    pub use crate::surface::SurfaceBorderTransformer;
}
