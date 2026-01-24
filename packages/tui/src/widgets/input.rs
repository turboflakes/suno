use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    prelude::Margin,
    style::{Color, Style, Styled},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::sync::{Arc, RwLock};
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Default, Clone, PartialEq, Eq, Zeroize)]
pub struct Input {
    /// Current value of the input box
    input: Zeroizing<String>,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    #[zeroize(skip)]
    mode: Mode,
    /// Track the calculated screen position for the cursor
    #[zeroize(skip)]
    cursor_position: Option<Position>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Editing,
}

impl Input {
    pub fn new() -> Self {
        Self {
            input: Zeroizing::new(String::new()),
            mode: Mode::default(),
            character_index: 0,
            cursor_position: None,
        }
    }

    pub fn cursor_position(&self) -> Option<Position> {
        self.cursor_position
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn insert_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index > 0 {
            // Calculate the byte position of the character BEFORE the cursor
            let target_char_index = self.character_index - 1;

            if let Some((byte_idx, _)) = self.input.char_indices().nth(target_char_index) {
                // .remove() works on the inner String thanks to DerefMut
                self.input.remove(byte_idx);
                self.move_cursor_left();
            }
        }
        // let is_not_cursor_leftmost = self.character_index != 0;
        // if is_not_cursor_leftmost {
        //     // Method "remove" is not used on the saved text for deleting the selected char.
        //     // Reason: Using remove on String works on bytes instead of the chars.
        //     // Using remove would require special care because of char boundaries.

        //     let current_index = self.character_index;
        //     let from_left_to_current_index = current_index - 1;

        //     // Getting all characters before the selected character.
        //     let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
        //     // Getting all characters after selected character.
        //     let after_char_to_delete = self.input.chars().skip(current_index);

        //     // Put all characters together except the selected one.
        //     // By leaving the selected one out, it is forgotten and therefore deleted.
        //     self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        //     self.move_cursor_left();
        // }
    }

    fn clamp_cursor(&self, new_pos: usize) -> usize {
        new_pos.clamp(0, self.input.chars().count())
    }

    const fn set_focus(&mut self) {
        self.mode = Mode::Editing;
    }

    const fn clear_focus(&mut self) {
        self.mode = Mode::Normal;
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    /// Wipes the string content immediately.
    pub fn cleanup(&mut self) {
        self.input.zeroize();
        self.input.clear();
        self.character_index = 0;
        self.clear_focus();
    }
}

#[derive(Debug)]
pub struct InputWidget {
    state: Arc<RwLock<Input>>,
}

impl InputWidget {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(Input::default())),
        }
    }

    pub fn get_cursor_position(&self) -> Option<Position> {
        let state = self.state.read().unwrap();
        state.cursor_position()
    }

    pub fn set_focus(&mut self) {
        let mut state = self.state.write().unwrap();
        state.set_focus();
    }

    pub fn clear_focus(&mut self) {
        let mut state = self.state.write().unwrap();
        state.clear_focus();
    }

    pub fn insert_char(&mut self, new_char: char) {
        let mut state = self.state.write().unwrap();
        state.insert_char(new_char);
    }

    pub fn delete_char(&mut self) {
        let mut state = self.state.write().unwrap();
        state.delete_char();
    }

    pub fn move_cursor_left(&mut self) {
        let mut state = self.state.write().unwrap();
        state.move_cursor_left();
    }

    pub fn move_cursor_right(&mut self) {
        let mut state = self.state.write().unwrap();
        state.move_cursor_right();
    }

    pub fn execute_with_password<F, R, E>(&self, action: F) -> Result<R, E>
    where
        F: FnOnce(&str) -> Result<R, E>,
    {
        let mut state = self.state.write().unwrap();
        let result = action(&state.input);
        state.cleanup();

        result
    }
}

impl Widget for &InputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let input_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        let label = Text::from(Line::from("Password:")).patch_style(Style::default());
        label.render(input_rows[1], buf);

        // Note: if input type is password, mask the input
        let masked_input: String = "*".repeat(state.input.chars().count());
        let input = Paragraph::new(masked_input)
            .style(match state.mode {
                Mode::Normal => Style::default(),
                Mode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("password"));
        input.render(input_rows[1], buf);

        // Calculate and save the cursor position into the state
        if state.mode == Mode::Editing {
            state.cursor_position = Some(Position::new(
                input_rows[1].x + state.character_index as u16 + 1,
                input_rows[1].y + 1,
            ));
        } else {
            state.cursor_position = None;
        }
    }
}
