use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Scroll Up on `Option-Up`
        KeyCode::Up => {
            if key_event.modifiers == KeyModifiers::ALT {
                app.chains.scroll_up();
            }
        }
        // Scroll Down on `Option-Down`
        KeyCode::Down => {
            if key_event.modifiers == KeyModifiers::ALT {
                app.chains.scroll_down();
            }
        }
        _ => {}
    }
    Ok(())
}
