use crate::utils::FileItem;
use super::preview::PreviewContent;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

/// Preview state that can be owned directly by AppState
#[derive(Debug, Clone)]
pub struct PreviewState {
    pub content: PreviewContent,
    pub title: String,
    pub scroll_offset: usize,
    pub current_file_item: Option<FileItem>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            content: PreviewContent::text(vec![Line::from(vec![Span::styled(
                "No file selected".to_string(),
                Style::default().fg(Color::Gray),
            )])]),
            title: "Preview".to_string(),
            scroll_offset: 0,
            current_file_item: None,
        }
    }
}

impl PreviewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_current_file_item(&mut self, path: Option<FileItem>) {
        self.current_file_item = path;
    }

    pub fn get_current_file_item(&self) -> Option<FileItem> {
        self.current_file_item.clone()
    }

    pub fn update_preview(
        &mut self,
        title: String,
        content: PreviewContent,
        file_item: Option<FileItem>,
    ) {
        // Now using channel means by the time it arrives, it might be outdated
        if file_item != self.get_current_file_item() {
            return;
        }
        self.title = title;
        self.content = content;
        self.scroll_offset = 0;
    }

    pub fn clear_preview(&mut self) {
        self.title = "Preview".to_string();
        self.content = PreviewContent::text(vec![Line::from(vec![Span::styled(
            "No file selected".to_string(),
            Style::default().fg(Color::Gray),
        )])]);
        self.scroll_offset = 0;
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_content(&self) -> PreviewContent {
        self.content.clone()
    }

    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    pub fn scroll_up(&mut self) -> bool {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            true
        } else {
            false
        }
    }

    pub fn scroll_down(&mut self) -> bool {
        if self.scroll_offset + 1 < self.content.len() {
            self.scroll_offset += 1;
            true
        } else {
            false
        }
    }

    pub fn scroll_page_up(&mut self, visible_height: usize) -> bool {
        let half_screen = (visible_height / 2).max(1);
        let new_offset = self.scroll_offset.saturating_sub(half_screen);
        if new_offset != self.scroll_offset {
            self.scroll_offset = new_offset;
            true
        } else {
            false
        }
    }

    pub fn scroll_page_down(&mut self, visible_height: usize) -> bool {
        let half_screen = (visible_height / 2).max(1);
        let max_offset = self.content.len().saturating_sub(visible_height);
        let new_offset = (self.scroll_offset + half_screen).min(max_offset);
        if new_offset != self.scroll_offset {
            self.scroll_offset = new_offset;
            true
        } else {
            false
        }
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}
