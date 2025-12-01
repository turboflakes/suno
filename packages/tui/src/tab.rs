use strum::{Display, EnumIter, FromRepr};

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Main,
    Logs,
}

impl Tab {
    pub fn next(self) -> Self {
        let current_index = self as usize;
        if Self::from_repr(current_index) == Some(Tab::Logs) {
            return Tab::Main;
        }
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn prev(self) -> Self {
        let current_index = self as usize;
        if Self::from_repr(current_index) == Some(Tab::Main) {
            return Tab::Logs;
        }
        let prev_index = current_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }
}
