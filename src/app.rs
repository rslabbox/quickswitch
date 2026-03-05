use anyhow::Result;
use tracing::instrument;

use crate::{
    app_state::AppState,
    modes::ModeManager,
    services::{PreviewManager, create_data_provider, preview::GLOBAL_PICKER},
    utils::AppMode,
};

pub struct App {
    pub state: AppState,
    pub mode_manager: ModeManager,
}

impl App {
    #[instrument]
    pub fn new(initial_mode: AppMode) -> Result<Self> {
        GLOBAL_PICKER.font_size();
        let mut state = AppState::new()?;

        // Load initial data using data provider
        let data_provider = create_data_provider(&initial_mode);
        data_provider.load_data(&mut state)?;

        let mut app = App {
            state,
            mode_manager: ModeManager::new(&initial_mode),
        };

        // Clear preview
        PreviewManager::clear_preview(&mut app.state);

        Ok(app)
    }
}
