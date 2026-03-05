use crate::{
    AppState,
    services::{PreviewGenerator, preview::PreviewContent},
    utils::{DisplayItem, FileItem},
};
use ratatui::text::{Line, Span};

/// Unified preview manager for handling all preview functionality
pub struct PreviewManager;

impl PreviewManager {
    pub fn preview_for_selected_item(state: &mut AppState) {
        if let Some(item) = state.get_selected_item() {
            let file_item = match item {
                DisplayItem::File(file) => file.clone(),
                DisplayItem::History(entry) => FileItem::from_path(&entry.path),
            };
            Self::update_preview_for_item_async(state, &file_item);
        }
    }

    fn update_preview_for_item_async(state: &mut AppState, file_item: &FileItem) {
        let placeholder_title = format!("📄 {}", file_item.name);
        let placeholder_content = PreviewContent::text(vec![
            Line::from(vec![Span::styled(
                "Loading preview...".to_string(),
                state.theme.preview_placeholder_style,
            )]),
            Line::from(vec![Span::raw("".to_string())]),
            Line::from(vec![Span::styled(
                "Please wait while content is being processed.".to_string(),
                state.theme.preview_info_style,
            )]),
        ]);
        
        state.preview.set_current_file_item(Some(file_item.clone()));
        state.preview.update_preview(
            placeholder_title,
            placeholder_content,
            Some(file_item.clone()),
        );

        let file_path = file_item.path.clone();
        let file_item_clone = file_item.clone();
        let tx = state.preview_tx.clone();
        let theme_clone = state.theme.clone();

        tokio::spawn(async move {
            let file_item_thread = FileItem::from_path(&file_path);
            let (title, content) = PreviewGenerator::generate_preview_content(&file_item_thread, &theme_clone).await;
            let _ = tx.send((title, content, Some(file_item_clone)));
        });
    }

    pub fn clear_preview(state: &mut AppState) {
        state.preview.clear_preview();
    }

    pub fn scroll_preview_up(state: &mut AppState) -> bool {
        state.preview.scroll_up()
    }

    pub fn scroll_preview_down(state: &mut AppState) -> bool {
        state.preview.scroll_down()
    }

    pub fn scroll_preview_page_up(state: &mut AppState, visible_height: usize) -> bool {
        state.preview.scroll_page_up(visible_height)
    }

    pub fn scroll_preview_page_down(state: &mut AppState, visible_height: usize) -> bool {
        state.preview.scroll_page_down(visible_height)
    }

    pub fn reset_preview_scroll(state: &mut AppState) {
        state.preview.reset_scroll();
    }
}
