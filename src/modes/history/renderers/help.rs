use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, List, ListItem},
};

use crate::{AppState, modes::Renderer};

/// Renderer for History mode help
#[derive(Default)]
pub struct HistoryHelpRenderer;

impl HistoryHelpRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for HistoryHelpRenderer {
    fn render(&self, f: &mut Frame, area: Rect, _state: &AppState) {
        let help_content = vec![
            Line::from("History Mode Navigation:"),
            Line::from(""),
            Line::from("jk or ↑↓  - Navigate history"),
            Line::from("l→        - Enter directory"),
            Line::from("b          - Move up half page"),
            Line::from("f          - Move down half page"),
            Line::from("/          - Search history"),
            Line::from("ESC        - Exit search (when searching)"),
            Line::from("Enter      - Select directory & exit app"),
            Line::from("ESC        - Return to normal mode"),
            Line::from(""),
            Line::from("Note: Selected directory will be"),
            Line::from("      moved to top of history"),
        ];

        let help_items: Vec<ListItem> = help_content.into_iter().map(ListItem::new).collect();

        let help_widget = List::new(help_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help - History Mode"),
        );

        f.render_widget(help_widget, area);
    }
}
