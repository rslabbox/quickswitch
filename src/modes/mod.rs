use anyhow::Result;
use ratatui::{Frame, layout::Rect, style::Style};

use crate::{
    app_state::AppState,
    utils::{AppMode, FileItem},
};

pub mod history;
pub mod normal;
pub mod preview;

pub trait Renderer {
    /// Render the component in the given area
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState);
}

/// Represents a mode switch request
#[derive(Debug, Clone, PartialEq)]
pub enum ModeAction {
    Stay,
    Switch(AppMode),
    Exit(Option<FileItem>),
}

/// Simplified trait that defines the interface for all application modes
/// Each mode focuses on its core rendering and initialization logic
/// All input handling is now unified through InputDispatcher
pub trait ModeHandler {
    /// Render the left panel (file list or history list)
    fn render_left_panel(&self, f: &mut Frame, area: Rect, state: &AppState);

    /// Render the right panel (preview or help)
    fn render_right_panel(&self, f: &mut Frame, area: Rect, state: &AppState);

    /// Get search box configuration (title, content, style)
    fn get_search_box_config(&self, state: &AppState) -> (String, String, Style);

    /// Determine if help should be shown instead of preview
    fn should_show_help(&self, state: &AppState) -> bool;

    /// Called when entering this mode
    fn on_enter(&mut self, _state: &mut AppState) -> Result<()> {
        Ok(())
    }

    /// Called when exiting this mode
    /// Handle keyboard input 
    fn handle_key_event(&mut self, state: &mut AppState, key: crossterm::event::KeyEvent) -> Result<ModeAction>; 
 
    /// Handle mouse input 
    fn handle_mouse_event(&mut self, state: &mut AppState, mouse: crossterm::event::MouseEvent) -> Result<ModeAction>; 
    fn on_exit(&mut self, _state: &mut AppState) -> Result<()> {
        Ok(())
    }
}

/// Factory function to create mode handlers
pub fn create_mode_handler(mode: &AppMode) -> Box<dyn ModeHandler> {
    match mode {
        AppMode::Normal => Box::new(normal::NormalModeHandler::new()),
        AppMode::History => Box::new(history::HistoryModeHandler::new()),
    }
}

/// Mode manager that coordinates between different modes
pub struct ModeManager {
    pub current_handler: Box<dyn ModeHandler>,
    pub current_mode: AppMode,
}

impl ModeManager {
    pub fn new(initial_mode: &AppMode) -> Self {
        Self {
            current_handler: create_mode_handler(initial_mode),
            current_mode: *initial_mode,
        }
    }

    pub fn switch_mode(&mut self, state: &mut AppState, new_mode: &AppMode) -> Result<()> {
        self.current_handler.on_exit(state)?;

        // Clear search when switching modes
        state.search_input.clear();
        state.is_searching = false;

        // Load appropriate data for the new mode using data provider
        let data_provider = crate::services::create_data_provider(new_mode);
        data_provider.load_data(state)?;

        self.current_handler = create_mode_handler(new_mode);
        self.current_mode = *new_mode;
        self.current_handler.on_enter(state)?;
        Ok(())
    }

    pub fn render_left_panel(&self, f: &mut Frame, area: Rect, state: &AppState) {
        self.current_handler.render_left_panel(f, area, state);
    }

    pub fn render_right_panel(&self, f: &mut Frame, area: Rect, state: &AppState) {
        self.current_handler.render_right_panel(f, area, state);
    }

    pub fn get_search_box_config(&self, state: &AppState) -> (String, String, Style) {
        self.current_handler.get_search_box_config(state)
    }

    pub fn get_current_mode(&self) -> &AppMode {
        &self.current_mode
    }

    pub fn is_mode(&self, mode: &AppMode) -> bool {
        self.current_mode == *mode
    }
}
