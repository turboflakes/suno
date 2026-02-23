use crate::call::{Call, CallError};
use crate::widgets::spinner::Spinner;
use crate::widgets::{input_command::InputCommandWidget, input_password::InputPasswordWidget};
use ratatui::layout::Position;
use std::sync::{Arc, RwLock};
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Default, Clone, PartialEq, Eq, Zeroize)]
pub struct InputField {
    /// Label of the input field
    #[zeroize(skip)]
    label: Option<String>,
    /// Placeholder of the input field
    #[zeroize(skip)]
    placeholder: Option<String>,
    /// Current value of the input box
    input: Zeroizing<String>,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    #[zeroize(skip)]
    mode: Mode,
    /// Current input type
    #[zeroize(skip)]
    r#type: Type,
    /// Track the calculated screen position for the cursor
    #[zeroize(skip)]
    cursor_position: Option<Position>,
    /// Track the calculated screen position for the cursor
    #[zeroize(skip)]
    status: Status,
    /// Set input metadata useful to validate input
    #[zeroize(skip)]
    metadata: Option<Metadata>,
    /// Hold spinner widget state
    #[zeroize(skip)]
    spinner: Spinner,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Editing,
    Locked,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    Command,
    Password,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Status {
    #[default]
    None, // No text yet
    Busy,            // Input is being processed
    Valid,           // Input text is present/valid either password or command
    Invalid(String), // Some invalid text found
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Busy => write!(f, "busy"),
            Self::Valid => write!(f, "valid"),
            Self::Invalid(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Metadata {
    unit: String,
    decimals: u32,
}

impl Metadata {
    pub fn new(unit: String, decimals: u32) -> Self {
        Self { unit, decimals }
    }
}

impl InputField {
    pub fn new() -> Self {
        Self {
            label: None,
            placeholder: None,
            input: Zeroizing::new(String::new()),
            mode: Mode::default(),
            r#type: Type::default(),
            character_index: 0,
            cursor_position: None,
            status: Status::default(),
            metadata: None,
            spinner: Spinner::default(),
        }
    }

    pub fn label(&self) -> Option<String> {
        self.label.clone()
    }

    fn masked_input(&self) -> String {
        "*".repeat(self.input.chars().count())
    }

    pub fn value(&self) -> String {
        if self.r#type == Type::Password {
            self.masked_input()
        } else {
            self.input.trim().to_string()
        }
    }

    pub fn cursor_position(&self) -> Option<Position> {
        self.cursor_position
    }

    pub fn character_index(&self) -> usize {
        self.character_index
    }

    pub fn status(&self) -> String {
        self.status.to_string()
    }

    pub fn spinner(&self) -> &Spinner {
        &self.spinner
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.mode, Mode::Editing)
    }

    pub fn is_locked(&self) -> bool {
        matches!(self.mode, Mode::Locked)
    }

    pub fn is_password(&self) -> bool {
        matches!(self.r#type, Type::Password)
    }

    pub fn is_command(&self) -> bool {
        matches!(self.r#type, Type::Command)
    }

    fn validate(&mut self) {
        match self.r#type {
            Type::Command => {
                let value = self.value();
                let decimals = self.metadata.as_ref().map(|m| m.decimals).unwrap_or(0);
                match Call::parse(&value, decimals) {
                    Ok(_) => self.status = Status::Valid,
                    Err(e) => match e {
                        CallError::InvalidAddress(_)
                        | CallError::InvalidAmount(_)
                        | CallError::InvalidArgument(_)
                        | CallError::UnknownArgument(_)
                        | CallError::UnknownCommand(_)
                        | CallError::UnknownOptional(_)
                        | CallError::MissingArguments(_) => {
                            self.status = Status::Invalid(e.to_string())
                        }
                        _ => self.status = Status::None,
                    },
                }
            }
            Type::Password => {
                let value = self.value();
                if !value.is_empty() {
                    self.status = Status::Valid;
                } else {
                    self.status = Status::None
                }
            }
        }
    }

    fn invalidate(&mut self, msg: &str) -> bool {
        self.status = Status::Invalid(msg.to_string());
        self.mode = Mode::Editing;
        true
    }

    pub fn parsed_call(&self) -> Option<Call> {
        if self.is_command() {
            let value = self.value();
            let decimals = self.metadata.as_ref().map(|m| m.decimals).unwrap_or(0);
            Call::parse(&value, decimals).ok()
        } else {
            None
        }
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
        self.validate();
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

            // validate input text
            self.validate();
        }
    }

    fn clamp_cursor(&self, new_pos: usize) -> usize {
        new_pos.clamp(0, self.input.chars().count())
    }

    pub fn set_cursor_position(&mut self, position: Position) {
        self.cursor_position = Some(position);
    }

    pub fn reset_cursor_position(&mut self) {
        self.cursor_position = None;
    }

    pub fn set_label(&mut self, label: String) {
        self.label = Some(label);
    }

    pub fn lock_input(&mut self) {
        self.mode = Mode::Locked;
        self.status = Status::Busy;
    }

    fn set_focus(&mut self) -> bool {
        if self.is_locked() {
            return false;
        }
        self.mode = Mode::Editing;
        true
    }

    const fn clear_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    fn set_value(&mut self, value: String) {
        self.character_index = value.chars().count();
        if let Some(old_pos) = self.cursor_position {
            let new_pos = Position::new(old_pos.x + self.character_index as u16, old_pos.y + 1);

            self.cursor_position = Some(new_pos);
        }
        self.input = Zeroizing::new(value);
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.status, Status::Busy)
    }

    pub fn is_valid(&self) -> bool {
        matches!(self.status, Status::Valid)
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self.status, Status::Invalid(_))
    }

    const fn as_password(&mut self) {
        self.r#type = Type::Password;
    }

    fn as_command(&mut self, metadata: Option<Metadata>) {
        self.r#type = Type::Command;
        self.metadata = metadata;
    }

    pub fn reset_as_password(&mut self) {
        self.reset();
        self.as_password();
    }

    pub fn reset_as_command(&mut self, metadata: Option<Metadata>) {
        self.reset();
        self.as_command(metadata);
    }

    pub fn reset(&mut self) {
        self.input.zeroize();
        self.input.clear();
        self.character_index = 0;
        self.status = Status::None;
        self.clear_mode();
    }
}

#[derive(Debug)]
pub struct InputFieldWidget {
    state: Arc<RwLock<InputField>>,
}

impl InputFieldWidget {
    pub fn as_password(&self) -> InputPasswordWidget {
        let mut state = self.state.write().unwrap();
        state.as_password();
        InputPasswordWidget {
            state: self.state.clone(),
        }
    }

    pub fn as_command(&self, call: Option<Call>) -> InputCommandWidget {
        InputCommandWidget {
            state: self.state.clone(),
            call,
        }
    }
}

impl InputFieldWidget {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(InputField::new())),
        }
    }

    pub fn value(&self) -> String {
        let state = self.state.read().unwrap();
        state.value()
    }

    pub fn get_cursor_position(&self) -> Option<Position> {
        let state = self.state.read().unwrap();
        state.cursor_position()
    }

    pub fn get_parsed_call(&self) -> Option<Call> {
        let state = self.state.read().unwrap();
        state.parsed_call()
    }

    //
    pub fn reset_as_command(&mut self, metadata: Option<Metadata>) {
        let mut state = self.state.write().unwrap();
        state.reset_as_command(metadata);
    }

    pub fn reset_as_password(&mut self) {
        let mut state = self.state.write().unwrap();
        state.reset_as_password();
    }

    pub fn set_value(&mut self, value: String) {
        let mut state = self.state.write().unwrap();
        state.set_value(value);
    }

    pub fn set_label(&mut self, label: String) {
        let mut state = self.state.write().unwrap();
        state.set_label(label);
    }

    pub fn set_focus(&mut self) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_focus()
    }

    pub fn clear_focus(&mut self) {
        let mut state = self.state.write().unwrap();
        state.clear_mode();
    }

    pub fn lock_input(&mut self) {
        let mut state = self.state.write().unwrap();
        state.lock_input();
    }

    pub fn invalidate(&mut self, msg: &str) -> bool {
        let mut state = self.state.write().unwrap();
        state.invalidate(msg)
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
        state.reset();

        result
    }
}
