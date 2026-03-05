use ratatui::style::{Color, Style};

/// Centralized theme configuration for the application UI
#[derive(Debug, Clone)]
pub struct Theme {
    // Selection state colors
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub unselected_fg: Color,
    pub unselected_bg: Color,

    pub list_highlight_symbol: &'static str,

    // File Types
    pub dir_icon: &'static str,
    pub dir_style: Style,
    pub file_icon: &'static str,
    pub file_style: Style,

    // Search Box
    pub search_box_normal: Style,
    pub search_box_active: Style,
    pub search_box_results: Style,

    // Search Results Highlights
    pub search_match_style: Style,

    // History specifics
    pub history_freq_style: Style,
    pub history_path_style: Style,

    // Preview
    pub preview_placeholder_style: Style,
    pub preview_info_style: Style,
    pub preview_error_style: Style,
    pub preview_text_style: Style,
    pub preview_line_number_style: Style,
    pub preview_border_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Selection state colors
            selected_fg: Color::White,    // set to white for better contrast with cyan background
            selected_bg: Color::Cyan,     // use cyan background for selection to ensure visibility of both file and directory names
            unselected_fg: Color::Reset,  // Inherit terminal fg for unspecified items
            unselected_bg: Color::Reset,  // Inherit terminal bg normally

            list_highlight_symbol: "",

            // File items
            dir_icon: "📁",
            dir_style: Style::default().fg(Color::Cyan),
            file_icon: "📄",
            file_style: Style::default(),

            // Search box status colors
            search_box_normal: Style::default().fg(Color::Cyan),
            search_box_active: Style::default().fg(Color::Black).bg(Color::Yellow),
            search_box_results: Style::default().fg(Color::Black).bg(Color::Green),

            // In-text matching (for search)
            search_match_style: Style::default().fg(Color::Yellow).bg(Color::DarkGray),

            // History mode specifics
            history_freq_style: Style::default().fg(Color::Yellow),
            history_path_style: Style::default().fg(Color::DarkGray),

            // Preview panel text colors
            preview_placeholder_style: Style::default().fg(Color::Yellow),
            preview_info_style: Style::default().fg(Color::Gray),
            preview_error_style: Style::default().fg(Color::Red),
            preview_text_style: Style::default(),
            preview_line_number_style: Style::default().fg(Color::DarkGray),
            preview_border_style: Style::default(),
        }
    }
}
