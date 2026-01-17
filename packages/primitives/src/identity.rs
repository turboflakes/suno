use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    #[serde(default)]
    name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<String>,
}

impl Identity {
    pub fn with_name(name: String) -> Self {
        Self { name, sub: None }
    }
    pub fn with_name_and_sub(name: String, sub: String) -> Self {
        Self {
            name,
            sub: Some(sub),
        }
    }
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(sub) = &self.sub {
            write!(f, "{}/{}", self.name, sub)
        } else {
            write!(f, "{}", self.name)
        }
    }
}
