use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::{AppState, modes::Renderer, utils::DisplayItem};

/// Renderer for history list in History mode
#[derive(Default)]
pub struct HistoryListRenderer;

impl HistoryListRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for HistoryListRenderer {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let selected_index = state.file_list_state.selected();
        let history_items: Vec<ListItem> = if state.filtered_files.is_empty() {
            if state.files.is_empty() {
                vec![ListItem::new("No history available")]
            } else {
                vec![ListItem::new("No matching history entries")]
            }
        } else {
            state
                .filtered_files
                .iter()
                .enumerate()
                .filter_map(|(idx, &i)| state.files.get(i).map(|item| (idx, item)))
                .map(|(idx, item)| {
                    let is_selected = selected_index == Some(idx);
                    create_history_list_item(item, &state.search_input, &state.theme, is_selected)
                })
                .collect()
        };

        let history_title = if state.is_searching && !state.search_input.is_empty() {
            format!(
                "History - {} matches ({}/{})",
                state.filtered_files.len(),
                state.filtered_files.len(),
                state.files.len()
            )
        } else {
            format!("History - {} entries", state.files.len())
        };

        let history_list = List::new(history_items)
            .block(Block::default().borders(Borders::ALL).title(history_title))
            .highlight_style(
                ratatui::style::Style::default()
                    .fg(state.theme.selected_fg)
                    .bg(state.theme.selected_bg)
            )
            .highlight_symbol(state.theme.list_highlight_symbol);

        f.render_stateful_widget(history_list, area, &mut state.file_list_state.clone());
    }
}

/// Create a list item for a history entry with directory name and full path
fn create_history_list_item<'a>(item: &'a DisplayItem, search_input: &'a str, theme: &crate::theme::Theme, is_selected: bool) -> ListItem<'a> {
    match item {
        DisplayItem::History(entry) => {
            let icon = theme.dir_icon;
            let dir_name = entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let full_path = entry.path.to_string_lossy();

            let mut dir_style = theme.dir_style;
            let mut freq_style = theme.history_freq_style;
            let mut path_style = theme.history_path_style;
            
            if is_selected {
                dir_style = ratatui::style::Style::default();
                freq_style = ratatui::style::Style::default();
                path_style = ratatui::style::Style::default();
            } else {
                dir_style = dir_style.bg(theme.unselected_bg);
                freq_style = freq_style.bg(theme.unselected_bg);
                path_style = path_style.bg(theme.unselected_bg);
            }

            // Create spans for the display
            let mut spans = vec![
                Span::styled(icon, dir_style),
                Span::raw(" "),
            ];

            // Add directory name with highlighting if searching
            if !search_input.is_empty() {
                let search_lower = search_input.to_lowercase();
                let name_lower = dir_name.to_lowercase();
                if let Some(pos) = name_lower.find(&search_lower) {
                    // Highlight the search term in directory name
                    let before = &dir_name[..pos];
                    let matched = &dir_name[pos..pos + search_input.len()];
                    let after = &dir_name[pos + search_input.len()..];

                    spans.push(Span::styled(before, dir_style));
                    spans.push(Span::styled(
                        matched,
                        theme.search_match_style,
                    ));
                    spans.push(Span::styled(after, dir_style));
                } else {
                    spans.push(Span::styled(dir_name, dir_style));
                }
            } else {
                spans.push(Span::styled(dir_name, dir_style));
            }

            // Add frequency indicator
            spans.push(Span::styled(
                format!(" ({}×)", entry.frequency),
                freq_style,
            ));

            // Add full path in darker color
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("({full_path})"),
                path_style,
            ));

            ListItem::new(Line::from(spans))
        }
        DisplayItem::File(_) => {
            // This shouldn't happen in history mode, but handle it gracefully
            ListItem::new("Invalid history entry")
        }
    }
}
