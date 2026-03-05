use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, KeyEvent, MouseEvent},
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use std::{env, io};

use crate::{
    App,
    modes::{ModeAction, history::HistoryDataProvider},
    utils::FileItem,
};

/// Main entry point for keyboard event handling
/// Now delegates to the current mode handler
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool> {
    let action = app.mode_manager.current_handler.handle_key_event(&mut app.state, key)?;
    handle_action(app, action)
}

/// Handle mouse events
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> Result<bool> {
    let action = app.mode_manager.current_handler.handle_mouse_event(&mut app.state, mouse)?;
    handle_action(app, action)
}

fn handle_action(app: &mut App, action: ModeAction) -> Result<bool> {
    match action {
        ModeAction::Stay => Ok(true),
        ModeAction::Switch(new_mode) => {
            app.mode_manager.switch_mode(&mut app.state, &new_mode)?;
            Ok(true)
        }
        ModeAction::Exit(file_item) => {
            handle_exit(app, file_item.as_ref())?;
            Ok(false) // This should never be reached due to process::exit in handle_exit
        }
    }
}

fn handle_exit(app: &mut App, file: Option<&FileItem>) -> Result<()> {
    if let Some(file) = file {
        let select_path = if file.is_dir {
            file.path.clone()
        } else {
            app.state.current_dir.clone()
        };
        // Save to history using history data provider
        let history_provider: HistoryDataProvider = HistoryDataProvider;
        history_provider
            .add_to_history(select_path.clone())
            .unwrap_or(());

        // Properly cleanup terminal state before exit
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        )?;

        unsafe { env::set_var("QS_SELECT_PATH", select_path.to_string_lossy().as_ref()) };
        eprintln!("{}", select_path.display());
    } else {
        // If no file is selected, just exit with proper cleanup
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        )?;
    }

    std::process::exit(0);
}
