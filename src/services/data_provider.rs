use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::{
    app_state::AppState,
    modes::ModeAction,
    services::PreviewManager,
    utils::{AppMode, DisplayItem},
};

/// Unified data provider trait for different modes
/// This trait provides a consistent interface for all modes to access their data
pub trait DataProvider {
    /// Get items to display for current mode
    fn get_items(&self, state: &AppState) -> Vec<DisplayItem> {
        state
            .filtered_files
            .iter()
            .filter_map(|&index| state.files.get(index))
            .cloned()
            .collect()
    }

    /// Get current selected index
    fn get_selected_index(&self, state: &AppState) -> Option<usize> {
        state.file_list_state.selected()
    }

    /// Set selected index
    fn set_selected_index(&self, state: &mut AppState, index: Option<usize>) {
        state.file_list_state.select(index);
    }

    /// Get total count of items
    fn get_total_count(&self, state: &AppState) -> usize {
        state.filtered_files.len()
    }

    /// Navigate up in the list
    
    fn navigate_up(&self, state: &mut AppState) -> bool {
        let visible_height = state.layout.get_left_content_height() / 2;
        if let Some(selected) = state.file_list_state.selected() {
            if selected > 0 {
                state.file_list_state.select(Some(selected - 1));
                self.update_scroll_offset(state, visible_height);
                PreviewManager::preview_for_selected_item(state);
                return true;
            }
        } else if !state.filtered_files.is_empty() {
            state
                .file_list_state
                .select(Some(state.filtered_files.len() - 1));
            self.update_scroll_offset(state, visible_height);
            PreviewManager::preview_for_selected_item(state);
            return true;
        }
        false
    }

    /// Navigate down in the list
    
    fn navigate_down(&self, state: &mut AppState) -> bool {
        let total = state.filtered_files.len();
        if total == 0 {
            return false;
        }

        let visible_height = state.layout.get_left_content_height() / 2;
        if let Some(selected) = state.file_list_state.selected() {
            if selected + 1 < total {
                state.file_list_state.select(Some(selected + 1));
                self.update_scroll_offset(state, visible_height);
                PreviewManager::preview_for_selected_item(state);
                return true;
            }
        } else {
            state.file_list_state.select(Some(0));
            self.update_scroll_offset(state, visible_height);
            PreviewManager::preview_for_selected_item(state);
            return true;
        }
        false
    }

    /// Navigate half page up in the list
    
    fn navigate_half_page_up(&self, state: &mut AppState) -> bool {
        let total = state.filtered_files.len();
        if total == 0 {
            return false;
        }

        let visible_height = state.layout.get_left_content_height();
        let half_page = (visible_height / 2).max(1);

        if let Some(selected) = state.file_list_state.selected() {
            let new_selected = selected.saturating_sub(half_page);
            state.file_list_state.select(Some(new_selected));
            self.update_scroll_offset(state, visible_height);
            PreviewManager::preview_for_selected_item(state);
            return true;
        } else if !state.filtered_files.is_empty() {
            state
                .file_list_state
                .select(Some(state.filtered_files.len() - 1));
            self.update_scroll_offset(state, visible_height);
            PreviewManager::preview_for_selected_item(state);
            return true;
        }
        false
    }

    /// Navigate half page down in the list
    
    fn navigate_half_page_down(&self, state: &mut AppState) -> bool {
        let total = state.filtered_files.len();
        if total == 0 {
            return false;
        }

        let visible_height = state.layout.get_left_content_height();
        let half_page = (visible_height / 2).max(1);

        if let Some(selected) = state.file_list_state.selected() {
            let new_selected = (selected + half_page).min(total - 1);
            state.file_list_state.select(Some(new_selected));
            self.update_scroll_offset(state, visible_height);
            PreviewManager::preview_for_selected_item(state);
            true
        } else if !state.filtered_files.is_empty() {
            state.file_list_state.select(Some(0));
            self.update_scroll_offset(state, visible_height);
            PreviewManager::preview_for_selected_item(state);
            true
        } else {
            false
        }
    }

    /// Get the file path for preview (unified interface)
    fn get_preview_path(&self, state: &AppState) -> Option<PathBuf> {
        state
            .get_selected_item()
            .map(|item| item.get_path().clone())
    }

    /// Update scroll offset for automatic scrolling
    fn update_scroll_offset(&self, state: &mut AppState, visible_height: usize) {
        if visible_height == 0 {
            return; // Avoid division by zero and overflow
        }

        if let Some(selected) = state.file_list_state.selected() {
            let current_offset = state.file_list_state.offset();
            let new_offset = if selected < current_offset {
                selected
            } else if selected >= current_offset + visible_height
                || selected < current_offset + visible_height - 1
            {
                selected.saturating_sub(visible_height - 1)
            } else {
                current_offset
            };

            if new_offset != current_offset {
                *state.file_list_state.offset_mut() = new_offset;
            }
        }
    }

    /// Navigate to selected item (if applicable)
    /// Returns Some(ModeAction) if mode should change, None if should stay in current mode
    fn navigate_to_selected(&self, _state: &mut AppState) -> Result<bool> {
        Ok(true)
    }

    /// Navigate into the selected directory (if applicable)
    /// Returns Some(ModeAction) if mode should change, None if should stay in current mode
    fn navigate_into_directory(&self, _state: &mut AppState) -> Result<Option<ModeAction>> {
        Ok(Some(ModeAction::Switch(AppMode::Normal)))
    }

    /// Navigate to parent directory (if applicable)
    /// Returns Some(ModeAction) if mode should change, None if should stay in current mode
    fn navigate_to_parent(&self, _state: &mut AppState) -> Result<Option<ModeAction>> {
        Ok(Some(ModeAction::Switch(AppMode::Normal)))
    }

    /// Load initial data for this mode
    fn load_data(&self, state: &mut AppState) -> Result<()>;

    /// Save current position before navigation
    fn save_position(&self, _state: &mut AppState) {}

    /// Restore position after navigation
    fn restore_position(&self, _state: &mut AppState) {}

    /// Handle directory change (called when current_dir changes)
    fn on_directory_changed(&self, _state: &mut AppState, _new_dir: &Path) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}


// Re-added create_data_provider as Box<dyn DataProvider>
pub fn create_data_provider(mode: &crate::utils::AppMode) -> Box<dyn DataProvider> {
    match mode {
        crate::utils::AppMode::Normal => Box::new(crate::modes::normal::FileListDataProvider),
        crate::utils::AppMode::History => Box::new(crate::modes::history::HistoryDataProvider),
    }
}
