/// Rows above and below the viewport of a list whose data is also fetched, so it is there
/// before the row is scrolled to.
pub const VISIBLE_ROWS_MARGIN: u16 = 8;

/// Where the focus is, for a list displayed on the left pane (compact) and on the main window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ListFocus {
    /// The list is not the selected section
    #[default]
    None,
    /// The list is the selected section and the selection is driven from the compact list
    Compact,
    /// The selection is driven from the main window, the compact list loses the focus
    Main,
}

impl ListFocus {
    /// Whether the list has a selection to show, wherever it is driven from.
    pub fn is_active(&self) -> bool {
        *self != Self::None
    }

    pub fn is_compact(&self) -> bool {
        *self == Self::Compact
    }
}
