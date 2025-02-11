/// Popup variations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Char(char),
    Instruction(String),
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char(c) => write!(f, "{c}"),
            Self::Instruction(s) => write!(f, "{s}"),
        }
    }
}

/// Common struct for shared fields
#[derive(Debug, Clone)]
struct Option {
    command: Command,
    description: String,
}

impl Option {
    fn new(command: Command, description: String) -> Self {
        Self {
            command,
            description,
        }
    }
}

/// Specific types using composition
#[derive(Debug, Clone)]
pub struct Entry {
    entry: Option,
}

impl Entry {
    pub fn new(command: Command, description: String) -> Self {
        Self {
            entry: Option::new(command, description),
        }
    }

    pub fn command(&self) -> String {
        self.entry.command.to_string()
    }

    pub fn description(&self) -> String {
        self.entry.description.to_string()
    }
}
