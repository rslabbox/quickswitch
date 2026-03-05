use ratatui::widgets::ListState;
use std::{collections::HashMap, path::PathBuf, time::Instant};
use tracing::{debug, instrument, warn};

use crate::{
    core::layout::LayoutManager,
    utils::{DisplayItem, FileItem},
};

#[derive(Clone, Debug)]
pub struct DoubleClickState {
    pub last_click_time: Option<Instant>,
    pub last_click_position: Option<(u16, u16)>,
    pub last_clicked_index: Option<usize>,
}

pub struct AppState {
    pub search_input: String,
    pub is_searching: bool,
    pub show_hidden_files: bool,
    pub current_dir: PathBuf,
    pub files: Vec<DisplayItem>,
    pub filtered_files: Vec<usize>,
    pub file_list_state: ListState,
    pub dir_positions: HashMap<PathBuf, usize>,
    pub double_click_state: DoubleClickState,
    pub layout: LayoutManager,
    pub theme: crate::theme::Theme,
    pub preview: crate::services::global_preview_state::PreviewState,
    pub preview_tx: std::sync::mpsc::Sender<(String, crate::services::preview::PreviewContent, Option<crate::utils::FileItem>)>,
    pub preview_rx: std::sync::mpsc::Receiver<(String, crate::services::preview::PreviewContent, Option<crate::utils::FileItem>)>,
}

impl AppState {
    #[instrument]
    pub fn new() -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let (preview_tx, preview_rx) = std::sync::mpsc::channel();
        debug!(dir = %current_dir.display(), "Build AppState");
        Ok(Self {
            search_input: String::new(),
            is_searching: false,
            show_hidden_files: false,
            current_dir,
            files: Vec::new(),
            filtered_files: Vec::new(),
            file_list_state: ListState::default(),
            dir_positions: HashMap::new(),
            double_click_state: DoubleClickState {
                last_click_time: None,
                last_click_position: None,
                last_clicked_index: None,
            },
            layout: LayoutManager::new(),
            theme: crate::theme::Theme::default(),
            preview: crate::services::global_preview_state::PreviewState::default(),
            preview_tx,
            preview_rx,
        })
    }

    /// Update the layout based on terminal size
    #[instrument(skip(self))]
    pub fn update_layout(&mut self, terminal_size: ratatui::layout::Rect) {
        debug!(
            width = terminal_size.width,
            height = terminal_size.height,
            "Updating layout"
        );
        self.layout.update_layout(terminal_size);
    }

    /// Check if a point is in the left panel area
    #[instrument(skip(self))]
    pub fn is_point_in_left_panel(&self, x: u16, y: u16) -> bool {
        let result = self.layout.is_in_left_area(x, y);
        debug!(x, y, result, "Checking if point is in left panel");
        result
    }

    /// Check if a point is in the right panel area
    #[instrument(skip(self))]
    pub fn is_point_in_right_panel(&self, x: u16, y: u16) -> bool {
        let result = self.layout.is_in_right_area(x, y);
        debug!(x, y, result, "Checking if point is in right panel");
        result
    }

    /// Check if a point is in the search area
    #[instrument(skip(self))]
    pub fn is_point_in_search_area(&self, x: u16, y: u16) -> bool {
        let result = self.layout.is_in_search_area(x, y);
        debug!(x, y, result, "Checking if point is in search area");
        result
    }

    /// Load file items for Normal mode
    #[instrument(skip(self, file_items), fields(item_count = file_items.len()))]
    pub fn load_file_items(&mut self, file_items: Vec<FileItem>) {
        debug!("Loading {} file items", file_items.len());
        self.files = file_items.into_iter().map(DisplayItem::File).collect();
        self.reset_filter();
        debug!("File items loaded successfully");
    }

    /// Reset filter and selection
    #[instrument(skip(self))]
    pub fn reset_filter(&mut self) {
        debug!("Resetting filter");
        self.filtered_files = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, item)| self.should_show_item(item))
            .map(|(i, _)| i)
            .collect();
        self.file_list_state.select(None);
        debug!("Filter reset, {} items visible", self.filtered_files.len());
    }

    /// Apply search filter to current items
    #[instrument(skip(self), fields(search_term = %self.search_input))]
    pub fn apply_search_filter(&mut self) {
        debug!("Applying search filter with term: '{}'", self.search_input);

        if self.search_input.is_empty() {
            self.filtered_files = self
                .files
                .iter()
                .enumerate()
                .filter(|(_, item)| self.should_show_item(item))
                .map(|(i, _)| i)
                .collect();
        } else {
            let search_lower = self.search_input.to_lowercase();
            self.filtered_files = self
                .files
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    self.should_show_item(item)
                        && item
                            .get_display_name()
                            .to_lowercase()
                            .contains(&search_lower)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.file_list_state.select(None);
        debug!(
            "Search filter applied, {} items matched",
            self.filtered_files.len()
        );
    }

    /// Get selected item
    #[instrument(skip(self))]
    pub fn get_selected_item(&self) -> Option<DisplayItem> {
        if let Some(selected) = self.file_list_state.selected()
            && let Some(&file_index) = self.filtered_files.get(selected)
                && let Some(item) = self.files.get(file_index).cloned() {
                    debug!(item_name = %item.get_display_name(), "Selected item retrieved");
                    return Some(item);
                }
        debug!("No item selected");
        None
    }

    /// Check if an item should be shown based on current filter settings
    #[instrument(skip(self, item), fields(item = %item.get_display_name()))]
    fn should_show_item(&self, item: &DisplayItem) -> bool {
        // Always show non-file items (like history entries)
        if !matches!(item, DisplayItem::File(_)) {
            debug!("Showing non-file item");
            return true;
        }

        let name = item.get_display_name();

        // Check if it's a hidden file (starts with '.')
        if name.starts_with('.') {
            // Show hidden files only if show_hidden_files is true
            let should_show = self.show_hidden_files;
            debug!(
                is_hidden = true,
                show_hidden_files = self.show_hidden_files,
                should_show,
                "Hidden file visibility check"
            );
            should_show
        } else {
            // Always show non-hidden files
            debug!(
                is_hidden = false,
                should_show = true,
                "Non-hidden file, showing"
            );
            true
        }
    }

    /// Toggle hidden files visibility and reapply filters
    #[instrument(skip(self))]
    pub fn toggle_hidden_files(&mut self) {
        let old_state = self.show_hidden_files;
        self.show_hidden_files = !self.show_hidden_files;
        debug!(
            old_state,
            new_state = self.show_hidden_files,
            "Toggled hidden files visibility"
        );
        self.apply_search_filter();
    }
}
