use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::{
    app_state::AppState,
    modes::ModeAction,
    services::{DataProvider, FilesystemService, PreviewManager},
    utils::DisplayItem,
};

/// Data provider for file list (Normal and Search modes)
pub struct FileListDataProvider;

impl DataProvider for FileListDataProvider {
    // Use default implementations for most methods
    fn get_preview_path(&self, state: &AppState) -> Option<PathBuf> {
        if let Some(DisplayItem::File(file)) = state.get_selected_item() {
            Some(file.path)
        } else {
            None
        }
    }

    fn navigate_into_directory(&self, state: &mut AppState) -> Result<Option<ModeAction>> {
        if let Some(file) = state.get_selected_item()
            && file.is_directory() {
                // Save current position before changing directory
                self.save_position(state);

                // Change directory
                state.current_dir = file.get_path().to_path_buf();

                // Handle directory change
                self.on_directory_changed(state, &state.current_dir.clone())?;

                return Ok(None); // Stay in current mode
            }
        Ok(None)
    }

    fn navigate_to_parent(&self, state: &mut AppState) -> Result<Option<ModeAction>> {
        // Special handling for DRIVES: view - return to the last drive root
        if state.current_dir.to_string_lossy() == "DRIVES:" {
            return Ok(None);
        }

        if let Some(parent) = state.current_dir.parent() {
            let parent_path = parent.to_path_buf();

            // Save current position before changing directory
            self.save_position(state);

            // Change directory
            state.current_dir = parent_path.clone();

            // Handle directory change
            self.on_directory_changed(state, &parent_path)?;

            Ok(None) // Stay in current mode
        } else {
            // On Windows, if we're at a drive root (like C:\), show drives
            #[cfg(windows)]
            {
                if self.is_windows_drive_root(&state.current_dir) {
                    // Save current position before changing to drives view
                    self.save_position(state);

                    // Set to special drives path
                    state.current_dir = PathBuf::from("DRIVES:");

                    // Handle directory change
                    self.on_directory_changed(state, &state.current_dir.clone())?;

                    return Ok(None);
                }
            }

            Ok(None)
        }
    }

    fn load_data(&self, state: &mut AppState) -> Result<()> {
        let files = FilesystemService::load_directory(&state.current_dir)?;
        state.load_file_items(files);
        state.apply_search_filter();
        Ok(())
    }

    fn save_position(&self, state: &mut AppState) {
        if let Some(selected) = state.file_list_state.selected() {
            state
                .dir_positions
                .insert(state.current_dir.clone(), selected);
        }
    }

    fn restore_position(&self, state: &mut AppState) {
        if let Some(&saved_position) = state.dir_positions.get(&state.current_dir) {
            // 确保保存的位置在当前过滤结果范围内
            if saved_position < state.filtered_files.len() {
                state.file_list_state.select(Some(saved_position));
            } else {
                // 如果保存的位置超出范围，选择最后一个
                if !state.filtered_files.is_empty() {
                    state
                        .file_list_state
                        .select(Some(state.filtered_files.len() - 1));
                } else {
                    state.file_list_state.select(None);
                }
            }
        } else {
            state.file_list_state.select(None);
        }
    }

    fn on_directory_changed(&self, state: &mut AppState, _new_dir: &Path) -> Result<()> {
        // Clear search and exit search mode when changing directory
        state.search_input.clear();
        state.is_searching = false;

        // Load new directory contents
        self.load_data(state)?;

        // Restore position for the new directory
        self.restore_position(state);

        // Clear preview
        PreviewManager::clear_preview(state);

        Ok(())
    }
}

impl FileListDataProvider {
    #[cfg(windows)]
    fn is_windows_drive_root(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        // Check if it's a drive root like "C:\" or "D:\"
        path_str.len() == 3
            && path_str.ends_with(":\\")
            && path_str.chars().next().unwrap().is_ascii_alphabetic()
    }
}
