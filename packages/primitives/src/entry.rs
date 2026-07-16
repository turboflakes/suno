use serde::Serialize;
use std::fmt::Display;
use subxt::utils::to_hex;

type Bytes = Vec<u8>;

pub trait ToDescription {
    fn description(&self) -> String;
}

pub trait ToPlaceholder {
    fn placeholder(&self) -> String;
}

pub trait ToJson {
    fn to_json(&self) -> String;
}

pub trait ToMethod {
    fn to_method(&self) -> String;
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}

pub trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Command<T> {
    Text(String),
    Instruction { call: T, bytes: Option<Bytes> },
    Data(Bytes),
}

impl<T: Display> Display for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{}", s),
            Self::Instruction { call, .. } => write!(f, "{}", call),
            Self::Data(d) => write!(f, "{}", to_hex(d)),
        }
    }
}

impl<T: Display + ToPlaceholder> Command<T> {
    pub fn placeholder(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Instruction { call, .. } => call.placeholder(),
            Self::Data(_) => String::new(),
        }
    }
}

impl<T: Display + ToDescription> Command<T> {
    pub fn description(&self) -> String {
        match self {
            Self::Text(_) => String::new(),
            Self::Instruction { call, .. } => call.description(),
            Self::Data(_) => String::new(),
        }
    }
}

impl<T: Display + ToJson> Command<T> {
    pub fn to_json(&self) -> String {
        match self {
            Self::Text(s) => serde_json::to_string_pretty(s).unwrap_or_default(),
            Self::Instruction { call, .. } => call.to_json(),
            Self::Data(_) => String::new(),
        }
    }
}

impl<T: Display + ToMethod> Command<T> {
    pub fn to_method(&self) -> String {
        match self {
            Self::Text(_) => String::new(),
            Self::Instruction { call, .. } => call.to_method(),
            Self::Data(_) => String::new(),
        }
    }
}

impl<T: Display + ToHex> Command<T> {
    pub fn to_hex(&self) -> String {
        match self {
            Self::Text(s) => to_hex(s.as_bytes()),
            Self::Instruction { bytes, .. } => to_hex(bytes.clone().unwrap_or_default()),
            Self::Data(d) => to_hex(d),
        }
    }
}

impl<T: Display + AsBytes> Command<T> {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Text(s) => s.as_bytes().to_vec(),
            Self::Instruction { bytes, .. } => bytes.clone().unwrap_or_default(),
            Self::Data(d) => d.clone(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<T> {
    command: Command<T>,
}

impl<T: Display + ToDescription + ToPlaceholder + ToJson + ToMethod + ToHex + AsBytes + Clone>
    Entry<T>
{
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

    pub fn to_json(&self) -> String {
        self.command.to_json()
    }

    pub fn to_method(&self) -> String {
        self.command.to_method()
    }

    pub fn to_hex(&self) -> String {
        self.command.to_hex()
    }

    pub fn to_hex_truncated(&self, max_length: usize) -> String {
        format!(
            "0x{}..",
            self.command
                .to_hex()
                .chars()
                .take(max_length)
                .collect::<String>()
        )
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.command.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub enum Call {
        Chill,
        Bond { amount: u128 },
    }

    impl ToDescription for Call {
        fn description(&self) -> String {
            match self {
                Self::Chill => "declare no intention to validate".to_string(),
                Self::Bond { .. } => "bond more funds".to_string(),
            }
        }
    }

    impl ToPlaceholder for Call {
        fn placeholder(&self) -> String {
            match self {
                Self::Chill => "chill".to_string(),
                Self::Bond { .. } => "bond more funds".to_string(),
            }
        }
    }

    // impl ToHex for Call {
    //     fn to_hex(&self) -> String {
    //         match self {
    //             Self::Chill => to_hex(bytes),
    //             Self::Bond { .. } => to_hex(bytes),
    //         }
    //     }
    // }

    impl ToJson for Call {
        fn to_json(&self) -> String {
            serde_json::to_string_pretty(&self).unwrap_or_default()
        }
    }

    impl ToMethod for Call {
        fn to_method(&self) -> String {
            self.to_string()
        }
    }

    impl ToHex for Call {
        fn to_hex(&self) -> String {
            self.to_string()
        }
    }

    impl AsBytes for Call {
        fn as_bytes(&self) -> Vec<u8> {
            self.to_string().as_bytes().to_vec()
        }
    }

    impl Display for Call {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Chill => write!(f, "/chill"),
                Self::Bond { .. } => write!(f, "/bond"),
            }
        }
    }

    #[test]
    fn test_entry_functionality() {
        let cmd = Command::Instruction {
            call: Call::Chill,
            bytes: None,
        };
        let entry = Entry::new(cmd);

        assert_eq!(entry.command(), "/chill");
        assert_eq!(entry.description(), "declare no intention to validate");

        let cmd = Command::Instruction {
            call: Call::Bond { amount: 100 },
            bytes: None,
        };
        let entry = Entry::new(cmd);

        assert_eq!(entry.command(), "/bond");
        assert_eq!(entry.description(), "bond more funds");
    }

    // #[test]
    // fn test_command_display() {
    //     let payload: Payload = b"bond".to_vec();
    //     let cmd = Command::Instruction(Call::Bond(payload));
    //     assert_eq!(format!("{}", cmd), "/bond");
    // }

    // #[test]
    // fn test_call_as_char() {
    //     let payload: Payload = b"1".to_vec();
    //     assert_eq!(Call::Chill(payload).as_char(), 'c');
    //     let payload: Payload = b"2".to_vec();
    //     assert_eq!(Call::Bond(payload).as_char(), 'b');
    // }

    // #[test]
    // fn test_call_to_hex() {
    //     let payload: Payload = b"chill".to_vec();
    //     assert_eq!(Call::Chill(payload).to_hex(), "6368696c6c");
    //     let payload: Payload = b"bond".to_vec();
    //     assert_eq!(Call::Bond(payload).to_hex(), "626f6e64");
    // }
}
