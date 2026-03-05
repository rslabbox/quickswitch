pub mod app;
pub mod app_state;
pub mod config;
pub mod core;
pub mod logging;
pub mod modes;
pub mod services;
pub mod terminal;
pub mod theme;
pub mod utils;

pub use app::App;
pub use app_state::AppState;
pub use config::get_data_dir;
pub use modes::ModeHandler;
pub use services::FilesystemService;
pub use terminal::run_interactive_mode;
pub use utils::{AppMode, ShellType, is_tty, qs_init, run_non_interactive};

pub type Result<T> = anyhow::Result<T>;
