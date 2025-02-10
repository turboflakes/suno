/// Common struct for shared fields
#[derive(Debug, Clone)]
struct Option {
    key: char,
    description: String,
}

impl Option {
    fn new(key: char, description: String) -> Self {
        Self { key, description }
    }
}

/// Specific types using composition
#[derive(Debug, Clone)]
pub struct Entry {
    entry: Option,
}

impl Entry {
    pub fn new(key: char, description: String) -> Self {
        Self {
            entry: Option::new(key, description),
        }
    }

    // Getter methods if needed
    pub fn key(&self) -> &char {
        &self.entry.key
    }

    pub fn description(&self) -> &str {
        self.entry.description.as_ref()
    }
}
