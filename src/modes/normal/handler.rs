use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
};

use crossterm::event::{KeyCode, KeyModifiers};
use crate::{
    app_state::AppState,
    services::PreviewManager,
    utils::{AppMode, DisplayItem, FileItem},
    modes::{
        ModeHandler, Renderer, ModeAction,
        normal::{FileListRenderer, NormalHelpRenderer, data_provider::FileListDataProvider},
        preview::PreviewRenderer,
    },
};

/// Handler for Normal mode (default navigation mode)
pub struct NormalModeHandler {
    file_list_renderer: Box<dyn Renderer>,
    preview_renderer: Box<dyn Renderer>,
    help_renderer: Box<dyn Renderer>,
}

impl Default for NormalModeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalModeHandler {
    pub fn new() -> Self {
        Self {
            file_list_renderer: Box::new(FileListRenderer::new()),
            preview_renderer: Box::new(PreviewRenderer::new()),
            help_renderer: Box::new(NormalHelpRenderer::new()),
        }
    }
}

impl ModeHandler for NormalModeHandler {
    fn handle_key_event(&mut self, state: &mut AppState, key: crossterm::event::KeyEvent) -> anyhow::Result<crate::modes::ModeAction> {
        // Mode switch
        if !state.is_searching {
            if key.code == KeyCode::Char('v') {
                return Ok(ModeAction::Switch(AppMode::History));
            }
            if key.code == KeyCode::Char('/') {
                state.is_searching = true;
                return Ok(ModeAction::Stay);
            }
        }
        
        // Exit keys
        match key.code {
            KeyCode::Esc => {
                if state.is_searching {
                    state.is_searching = false;
                    return Ok(ModeAction::Stay);
                } else if state.get_selected_item().is_none() {
                    return Ok(ModeAction::Exit(None));
                } else {
                    state.file_list_state.select(None);
                    PreviewManager::clear_preview(state);
                    return Ok(ModeAction::Stay);
                }
            }
            KeyCode::Enter
                if key.modifiers != KeyModifiers::CONTROL => {
                    use crate::services::DataProvider;
                    let provider = FileListDataProvider;
                    if let Some(item) = state.get_selected_item() {
                        let _ = provider.navigate_to_selected(state);
                        match item {
                            DisplayItem::File(file) => return Ok(ModeAction::Exit(Some(file))),
                            DisplayItem::History(entry) => {
                                let file_item = FileItem::from_path(&entry.path);
                                return Ok(ModeAction::Exit(Some(file_item)));
                            }
                        }
                    } else {
                        let file_item = FileItem::from_path(&state.current_dir);
                        return Ok(ModeAction::Exit(Some(file_item)));
                    }
                }
            _ => {}
        }
        
        // Delegate shared logic
        let provider = FileListDataProvider;
        if let Some(action) = crate::core::InputDispatcher::handle_key_event(state, key, &provider)? {
            return Ok(action);
        }
        
        Ok(ModeAction::Stay)
    }

    fn handle_mouse_event(&mut self, state: &mut AppState, mouse: crossterm::event::MouseEvent) -> anyhow::Result<crate::modes::ModeAction> {
        let provider = FileListDataProvider;
        crate::core::InputDispatcher::handle_mouse_event(state, mouse, &provider)
    }
    fn render_left_panel(&self, f: &mut Frame, area: Rect, state: &AppState) {
        self.file_list_renderer.render(f, area, state);
    }

    fn render_right_panel(&self, f: &mut Frame, area: Rect, state: &AppState) {
        if self.should_show_help(state) {
            self.help_renderer.render(f, area, state);
        } else {
            self.preview_renderer.render(f, area, state);
        }
    }

    fn get_search_box_config(&self, state: &AppState) -> (String, String, Style) {
        let (info, style) = if state.is_searching {
            if state.search_input.is_empty() {
                (
                    "SEARCH - Type to search, ESC to exit search".to_string(),
                    state.theme.search_box_active,
                )
            } else {
                (
                    format!(
                        "SEARCH - '{}' - {} matches (ESC to exit)",
                        state.search_input,
                        state.filtered_files.len()
                    ),
                    state.theme.search_box_active,
                )
            }
        } else if !state.search_input.is_empty() {
            // Show search results even when not actively searching
            (
                format!(
                    "FILTERED - '{}' - {} matches (/ to search again)",
                    state.search_input,
                    state.filtered_files.len()
                ),
                state.theme.search_box_results,
            )
        } else {
            (
                "NORMAL - hjkl navigate, bf half page, / search, v history, Enter exit"
                    .to_string(),
                state.theme.search_box_normal,
            )
        };
        (info, state.search_input.clone(), style)
    }

    fn should_show_help(&self, state: &AppState) -> bool {
        // Show help if no selection or if searching with no results
        if state.is_searching {
            state.search_input.is_empty() || state.filtered_files.is_empty()
        } else {
            state.file_list_state.selected().is_none() || state.filtered_files.is_empty()
        }
    }
}
