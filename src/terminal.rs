use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use std::io;

use crate::{App, core::events, utils::AppMode};

pub async fn run_interactive_mode(mode: AppMode) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new(mode)?;
    let result = run_app_loop(&mut terminal, &mut app);
    cleanup_terminal(&mut terminal)?;
    result
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn run_app_loop<W>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
    app: &mut App,
) -> Result<()>
where
    W: std::io::Write,
{
    loop {
        // Update layout if terminal size changed
        let terminal_size = terminal.size()?;
        let terminal_area = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        if app.state.layout.needs_update(terminal_area) {
            app.state.update_layout(terminal_area);
        }

        // Poll preview updates
        while let Ok((title, content, file_item)) = app.state.preview_rx.try_recv() {
            app.state.preview.update_preview(title, content, file_item);
        }

        terminal.draw(|f| render_ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key)
                    if key.kind == KeyEventKind::Press
                        && !events::handle_key_event(app, key)?
                    => {
                        break;
                    }
                Event::Mouse(mouse)
                    if !events::handle_mouse_event(app, mouse)? => {
                        break;
                    }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Simple UI rendering function that delegates to mode manager
fn render_ui(f: &mut Frame, app: &App) {
    // Use the layout manager from app state
    let layout = &app.state.layout;

    // Render search box
    let (title, content, style) = app.mode_manager.get_search_box_config(&app.state);
    let search_box = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(style);
    f.render_widget(search_box, layout.get_search_area());

    // Delegate rendering to app using layout areas
    app.mode_manager
        .render_left_panel(f, layout.get_left_area(), &app.state);
    app.mode_manager
        .render_right_panel(f, layout.get_right_area(), &app.state);

    // Set cursor position when searching
    if app.state.is_searching {
        let search_area = layout.get_search_area();
        f.set_cursor_position((
            search_area.x + app.state.search_input.len() as u16 + 1,
            search_area.y + 1,
        ));
    }
}
