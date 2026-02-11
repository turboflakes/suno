use crate::call::Call;
use std::fmt::Display;

pub trait ToDescription {
    fn description(&self) -> String;
}

pub trait ToPlaceholder {
    fn placeholder(&self) -> String;
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait AsChar {
    fn as_char(&self) -> char;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command<T> {
    Text(String),
    Instruction(T),
}

impl<T: Display> Display for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{}", s),
            Self::Instruction(s) => write!(f, "{}", s),
        }
    }
}

impl<T: Display + ToPlaceholder> Command<T> {
    pub fn placeholder(&self) -> String {
        match self {
            Self::Text(_) => String::new(),
            Self::Instruction(s) => s.placeholder(),
        }
    }
}

impl<T: Display + ToDescription> Command<T> {
    pub fn description(&self) -> String {
        match self {
            Self::Text(_) => String::new(),
            Self::Instruction(s) => s.description(),
        }
    }
}

impl<T: Display + ToHex> Command<T> {
    pub fn to_hex(&self) -> String {
        match self {
            Self::Text(_) => String::new(),
            Self::Instruction(s) => s.to_hex(),
        }
    }
}

// impl AsChar for Call {
//     fn as_char(&self) -> char {
//         match self {
//             Self::Chill(_) => 'c',
//             Self::Bond(_) => 'b',
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct Entry<T> {
    command: Command<T>,
}

impl<T: Display + ToDescription + ToPlaceholder + ToHex + Clone> Entry<T> {
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

    pub fn placeholder(&self) -> String {
        self.command.placeholder()
    }

    pub fn to_hex(&self) -> String {
        self.command.to_hex()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::utils::to_hex;

    type Payload = Vec<u8>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Call {
        Chill(Payload),
        Bond(Payload),
    }

    impl AsChar for Call {
        fn as_char(&self) -> char {
            match self {
                Self::Chill(_) => 'c',
                Self::Bond(_) => 'b',
            }
        }
    }

    impl ToDescription for Call {
        fn description(&self) -> String {
            match self {
                Self::Chill(_) => "declare no intention to validate".to_string(),
                Self::Bond(_) => "bond more funds".to_string(),
            }
        }
    }

    impl ToPlaceholder for Call {
        fn placeholder(&self) -> String {
            match self {
                Self::Chill(_) => "chill".to_string(),
                Self::Bond(_) => "bond more funds".to_string(),
            }
        }
    }

    impl ToHex for Call {
        fn to_hex(&self) -> String {
            match self {
                Self::Chill(bytes) => to_hex(bytes),
                Self::Bond(bytes) => to_hex(bytes),
            }
        }
    }

    impl Display for Call {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Chill(_) => write!(f, "/chill"),
                Self::Bond(_) => write!(f, "/bond"),
            }
        }
    }

    #[test]
    fn test_entry_functionality() {
        let payload: Payload = b"chill".to_vec();
        let cmd = Command::Instruction(Call::Chill(payload));
        let entry = Entry::new(cmd);

        assert_eq!(entry.command(), "/chill");
        assert_eq!(entry.description(), "declare no intention to validate");
    }

    #[test]
    fn test_command_display() {
        let payload: Payload = b"bond".to_vec();
        let cmd = Command::Instruction(Call::Bond(payload));
        assert_eq!(format!("{}", cmd), "/bond");
    }

    #[test]
    fn test_call_as_char() {
        let payload: Payload = b"1".to_vec();
        assert_eq!(Call::Chill(payload).as_char(), 'c');
        let payload: Payload = b"2".to_vec();
        assert_eq!(Call::Bond(payload).as_char(), 'b');
    }

    #[test]
    fn test_call_to_hex() {
        let payload: Payload = b"chill".to_vec();
        assert_eq!(Call::Chill(payload).to_hex(), "6368696c6c");
        let payload: Payload = b"bond".to_vec();
        assert_eq!(Call::Bond(payload).to_hex(), "626f6e64");
    }
}
