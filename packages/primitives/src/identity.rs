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

    pub fn truncate(&self, size: usize) -> String {
        self.to_string().chars().take(size).collect()
    }
}

/// Removes the characters that only change how a glyph is presented: the variation selectors
/// (U+FE0E, U+FE0F) and the zero width joiner (U+200D).
fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|c| !matches!(c, '\u{FE0E}' | '\u{FE0F}' | '\u{200D}'))
        .collect()
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(sub) = &self.sub {
            write!(f, "{}/{}", normalize(&self.name), normalize(sub))
        } else {
            write!(f, "{}", normalize(&self.name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_removes_variation_selectors_and_joiners() {
        let identity =
            Identity::with_name_and_sub("someothername".into(), "\u{26D3}\u{FE0F} sys".into());
        assert_eq!(identity.to_string(), "someothername/\u{26D3} sys");

        let identity = Identity::with_name("a\u{200D}b".into());
        assert_eq!(identity.to_string(), "ab");
    }

    #[test]
    fn truncate_counts_the_normalized_characters() {
        let identity = Identity::with_name("ab\u{26D3}\u{FE0F}cd".into());
        assert_eq!(identity.truncate(4), "ab\u{26D3}c");
    }
}
