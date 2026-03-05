use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::{
    AppState,
    modes::Renderer,
    utils::{DisplayItem, FileItem},
};

/// Renderer for file list in Normal mode
#[derive(Default)]
pub struct FileListRenderer;

impl FileListRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for FileListRenderer {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let selected_index = state.file_list_state.selected();
        let files: Vec<ListItem> = state
            .filtered_files
            .iter()
            .enumerate()
            .filter_map(|(idx, &i)| state.files.get(i).map(|item| (idx, item)))
            .map(|(idx, item)| {
                let is_selected = selected_index == Some(idx);
                create_display_item_list_item(item, &state.search_input, &state.theme, is_selected)
            })
            .collect();

        let files_title = format!(
            "Files - {} ({}/{})",
            state.current_dir.display(),
            state.filtered_files.len(),
            state.files.len()
        );

        let files_list = List::new(files)
            .block(Block::default().borders(Borders::ALL).title(files_title))
            .highlight_style(
                ratatui::style::Style::default()
                    .fg(state.theme.selected_fg)
                    .bg(state.theme.selected_bg)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            )
            .highlight_symbol(state.theme.list_highlight_symbol);

        f.render_stateful_widget(files_list, area, &mut state.file_list_state.clone());
    }
}

/// Create a list item for a file with optional search highlighting
fn create_file_list_item<'a>(file: &'a FileItem, search_input: &'a str, theme: &crate::theme::Theme, is_selected: bool) -> ListItem<'a> {
    let icon = if file.is_dir { theme.dir_icon } else { theme.file_icon };
    let mut style = if file.is_dir {
        theme.dir_style
    } else {
        theme.file_style
    };

    // Apply explicit fg/bg colors to override internal span colors
    if is_selected {
        style = ratatui::style::Style::default().fg(theme.selected_fg).bg(theme.selected_bg);
    } else {
        style = style.bg(theme.unselected_bg);
    }

    let display_name = if !search_input.is_empty() {
        crate::utils::highlight_search_term(&file.name, search_input, theme, style)
    } else {
        vec![Span::styled(&file.name, style)]
    };

    let mut spans = vec![Span::styled(icon, style), Span::raw(" ")];
    spans.extend(display_name);

    ListItem::new(Line::from(spans))
}

/// Create a list item for a DisplayItem with optional search highlighting
fn create_display_item_list_item<'a>(item: &'a DisplayItem, search_input: &'a str, theme: &crate::theme::Theme, is_selected: bool) -> ListItem<'a> {
    match item {
        DisplayItem::File(file) => create_file_list_item(file, search_input, theme, is_selected),
        DisplayItem::History(entry) => {
            let icon = theme.dir_icon;
            let mut style = theme.dir_style;
            
            if is_selected {
                style = ratatui::style::Style::default().fg(theme.selected_fg).bg(theme.selected_bg);
            } else {
                style = style.bg(theme.unselected_bg);
            }

            let name = entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            let display_name = if !search_input.is_empty() {
                crate::utils::highlight_search_term(name, search_input, theme, style)
            } else {
                vec![Span::styled(name, style)]
            };

            let mut spans = vec![Span::styled(icon, style), Span::raw(" ")];
            spans.extend(display_name);

            ListItem::new(Line::from(spans))
        }
    }
}
