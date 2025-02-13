use std::fmt::Display;

pub trait ToDescription {
    fn description(&self) -> String;
}

pub trait AsChar {
    fn as_char(&self) -> char;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command<T> {
    Text(String),
    Instruction(T),
}

impl<T: Display + ToDescription> Command<T> {
    pub fn description(&self) -> String {
        match self {
            Self::Text(_) => String::new(),
            Self::Instruction(s) => s.description(),
        }
    }
}

impl<T: Display> Display for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{}", s),
            Self::Instruction(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry<T> {
    command: Command<T>,
}

impl<T: Display + ToDescription + Clone> Entry<T> {
    pub fn new(command: Command<T>) -> Self {
        Self { command }
    }

    pub fn get_command(&self) -> Command<T> {
        self.command.clone()
    }

    pub fn command(&self) -> String {
        self.command.to_string()
    }

    pub fn description(&self) -> String {
        self.command.description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Call {
        Chill,
        Bond,
    }

    impl AsChar for Call {
        fn as_char(&self) -> char {
            match self {
                Self::Chill => 'c',
                Self::Bond => 'b',
            }
        }
    }

    impl ToDescription for Call {
        fn description(&self) -> String {
            match self {
                Self::Chill => "declare no intention to validate".to_string(),
                Self::Bond => "bond more funds".to_string(),
            }
        }
    }

    impl Display for Call {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Chill => write!(f, "staking.chill"),
                Self::Bond => write!(f, "staking.bond"),
            }
        }
    }

    #[test]
    fn test_entry_functionality() {
        let cmd = Command::Instruction(Call::Chill);
        let entry = Entry::new(cmd);

        assert_eq!(entry.command(), "staking.chill");
        assert_eq!(entry.description(), "declare no intention to validate");
    }

    #[test]
    fn test_command_display() {
        let cmd = Command::Instruction(Call::Bond);
        assert_eq!(format!("{}", cmd), "staking.bond");
    }

    #[test]
    fn test_call_as_char() {
        assert_eq!(Call::Chill.as_char(), 'c');
        assert_eq!(Call::Bond.as_char(), 'b');
    }
}
