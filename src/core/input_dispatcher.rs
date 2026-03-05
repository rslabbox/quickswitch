use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

use crate::{
    AppState,
    modes::ModeAction,
    services::PreviewManager,
    utils::{DisplayItem, FileItem},
};

/// Unified input dispatcher for handling all user interactions
/// This centralizes key and mouse event handling, reducing duplication across modes
pub struct InputDispatcher;

impl InputDispatcher {
    /// Handle keyboard input uniformly across all modes
    pub fn handle_key_event(
        state: &mut AppState,
        key: KeyEvent,
        provider: &dyn crate::services::DataProvider,
    ) -> Result<Option<ModeAction>> {
        // Handle navigation keys (unified for all modes)
        if let Some(action) = Self::handle_navigation_keys(state, key, provider)? {
            return Ok(Some(action));
        }

        // Handle mode-specific keys
        Ok(Some(Self::handle_mode_specific_keys(state, key)?))
    }

    /// Handle mouse input uniformly across all modes
    pub fn handle_mouse_event(
        state: &mut AppState,
        mouse: MouseEvent,
        provider: &dyn crate::services::DataProvider,
    ) -> Result<ModeAction> {
        match mouse.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                Self::handle_scroll_navigation(state, mouse, provider)
            }
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                Self::handle_left_click(state, mouse, provider)
            }
            _ => Ok(ModeAction::Stay),
        }
    }

    /// Handle navigation keys - unified using data providers
    pub fn handle_navigation_keys(
        state: &mut AppState,
        key: KeyEvent,
        provider: &dyn crate::services::DataProvider,
    ) -> Result<Option<ModeAction>> {

        match key.code {
            KeyCode::Up => {
                provider.navigate_up(state);
                Ok(Some(ModeAction::Stay))
            }
            KeyCode::Down => {
                provider.navigate_down(state);
                Ok(Some(ModeAction::Stay))
            }
            KeyCode::Right => {
                // Use provider's navigation method
                if let Some(action) = provider.navigate_into_directory(state)? {
                    Ok(Some(action))
                } else {
                    Ok(Some(ModeAction::Stay))
                }
            }
            KeyCode::Left => {
                // Use provider's navigation method
                if let Some(action) = provider.navigate_to_parent(state)? {
                    Ok(Some(action))
                } else {
                    Ok(Some(ModeAction::Stay))
                }
            }
            // hjkl keys only work when not searching
            KeyCode::Char('k') if !state.is_searching => {
                provider.navigate_up(state);
                Ok(Some(ModeAction::Stay))
            }
            KeyCode::Char('j') if !state.is_searching => {
                provider.navigate_down(state);
                Ok(Some(ModeAction::Stay))
            }
            KeyCode::Char('l') if !state.is_searching => {
                // Use provider's navigation method
                if let Some(action) = provider.navigate_into_directory(state)? {
                    Ok(Some(action))
                } else {
                    Ok(Some(ModeAction::Stay))
                }
            }
            KeyCode::Char('h') if !state.is_searching => {
                // Use provider's navigation method
                if let Some(action) = provider.navigate_to_parent(state)? {
                    Ok(Some(action))
                } else {
                    Ok(Some(ModeAction::Stay))
                }
            }
            KeyCode::PageUp | KeyCode::PageDown => {
                Self::handle_preview_navigation(state, key);
                Ok(Some(ModeAction::Stay))
            }
            // Half-page navigation keys (only work when not searching)
            KeyCode::Char('b') if !state.is_searching => {
                provider.navigate_half_page_down(state);
                Ok(Some(ModeAction::Stay))
            }
            KeyCode::Char('f') if !state.is_searching => {
                provider.navigate_half_page_up(state);
                Ok(Some(ModeAction::Stay))
            }
            _ => Ok(None),
        }
    }

    /// Handle mode-specific keys that don't fit into common patterns
    pub fn handle_mode_specific_keys(
        state: &mut AppState,
        key: KeyEvent,
    ) -> Result<ModeAction> {
        // Handle search input when in search mode
        if state.is_searching {
            Self::handle_search_keys(state, key)
        } else {
            // Handle non-search mode keys
            match key.code {
                KeyCode::Char('.') => {
                    // Toggle hidden files visibility
                    state.toggle_hidden_files();
                    Ok(ModeAction::Stay)
                }
                _ => Ok(ModeAction::Stay),
            }
        }
    }

    /// Handle search mode specific keys
    fn handle_search_keys(state: &mut AppState, key: KeyEvent) -> Result<ModeAction> {
        match key.code {
            KeyCode::Char(c) => {
                state.search_input.push(c);
                state.apply_search_filter();
                Ok(ModeAction::Stay)
            }
            KeyCode::Backspace => {
                state.search_input.pop();
                state.apply_search_filter();
                Ok(ModeAction::Stay)
            }
            _ => Ok(ModeAction::Stay),
        }
    }

    /// Handle preview navigation (Page Up/Down)
    fn handle_preview_navigation(state: &mut AppState, key: KeyEvent) {
        // Use the actual right panel content height from layout manager
        let visible_height = state.layout.get_right_content_height();
        match key.code {
            KeyCode::PageUp => {
                PreviewManager::scroll_preview_page_up(state, visible_height);
            }
            KeyCode::PageDown => {
                PreviewManager::scroll_preview_page_down(state, visible_height);
            }
            _ => {}
        }
    }

    /// Handle scroll navigation using unified data providers
    fn handle_scroll_navigation(
        state: &mut AppState,
        mouse: MouseEvent,
        provider: &dyn crate::services::DataProvider,
    ) -> Result<ModeAction> {
        let is_scroll_up = matches!(mouse.kind, MouseEventKind::ScrollUp);

        // Check if mouse is in left area (file/history list) or right area (preview)
        if state.is_point_in_left_panel(mouse.column, mouse.row) {
            // Mouse is in left panel - scroll list using unified provider
            if is_scroll_up {
                provider.navigate_up(state);
            } else {
                provider.navigate_down(state);
            }
            PreviewManager::preview_for_selected_item(state);
        } else if state.is_point_in_right_panel(mouse.column, mouse.row) {
            // Mouse is in right panel - scroll preview content
            if is_scroll_up {
                PreviewManager::scroll_preview_up(state);
            } else {
                PreviewManager::scroll_preview_down(state);
            }
        }

        Ok(ModeAction::Stay)
    }

    /// Handle left mouse click using unified data providers
    fn handle_left_click(
        state: &mut AppState,
        mouse: MouseEvent,
        provider: &dyn crate::services::DataProvider,
    ) -> Result<ModeAction> {
        // Only handle clicks in the left panel (file/history list)
        if !state.is_point_in_left_panel(mouse.column, mouse.row) {
            return Ok(ModeAction::Stay);
        }

        let left_area = state.layout.get_left_area();

        // Calculate the actual clicked index considering scroll offset
        let visible_row = (mouse.row - left_area.y - 1) as usize; // Row relative to the visible area
        let scroll_offset = Self::get_scroll_offset(state);
        let clicked_index = visible_row + scroll_offset;

        // Check bounds
        if clicked_index >= provider.get_total_count(state) {
            return Ok(ModeAction::Stay);
        }

        // Check for double-click
        let mouse_position = (mouse.column, mouse.row);
        let is_double_click = Self::is_double_click(state, mouse_position, clicked_index);

        // Update selection
        provider.set_selected_index(state, Some(clicked_index));
        PreviewManager::preview_for_selected_item(state);

        // Update double-click state
        Self::update_double_click_state(state, mouse_position, clicked_index);

        // Handle double-click action
        if is_double_click
            && let Some(item) = state.get_selected_item() {
                match item {
                    DisplayItem::File(_) => {
                        if let Some(action) = provider.navigate_into_directory(state)? {
                            return Ok(action);
                        } else {
                            return Ok(ModeAction::Stay);
                        }
                    }
                    DisplayItem::History(entry) => {
                        let file_item = FileItem::from_path(&entry.path);
                        return Ok(ModeAction::Exit(Some(file_item)));
                    }
                }
            }

        Ok(ModeAction::Stay)
    }

    /// Check if current click is a double-click
    fn is_double_click(
        state: &mut AppState,
        mouse_position: (u16, u16),
        clicked_index: usize,
    ) -> bool {
        use std::time::Instant;
        const DOUBLE_CLICK_INTERVAL_MS: u64 = 150;

        let current_time = Instant::now();
        if let (Some(last_time), Some(last_pos), Some(last_idx)) = (
            state.double_click_state.last_click_time,
            state.double_click_state.last_click_position,
            state.double_click_state.last_clicked_index,
        ) {
            let elapsed = current_time.duration_since(last_time);
            elapsed.as_millis() <= DOUBLE_CLICK_INTERVAL_MS as u128
                && last_pos == mouse_position
                && last_idx == clicked_index
        } else {
            false
        }
    }

    /// Update double-click state
    fn update_double_click_state(
        state: &mut AppState,
        mouse_position: (u16, u16),
        clicked_index: usize,
    ) {
        use std::time::Instant;

        state.double_click_state.last_click_time = Some(Instant::now());
        state.double_click_state.last_click_position = Some(mouse_position);
        state.double_click_state.last_clicked_index = Some(clicked_index);
    }

    /// Get the current scroll offset
    fn get_scroll_offset(state: &mut AppState) -> usize {
        // All modes now use the unified file_list_state
        state.file_list_state.offset()
    }
}
