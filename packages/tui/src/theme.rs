use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub table: Table,
}

pub struct Table {
    pub base: Style,
    pub base_active: Style,
    pub header: Style,
    pub row_highlight: Style,
    pub row_highlight_active: Style,
}

impl Table {
    pub fn base(&self, active: bool) -> Style {
        if active {
            self.base_active
        } else {
            self.base
        }
    }

    pub fn row_highlight(&self, active: bool) -> Style {
        if active {
            self.row_highlight_active
        } else {
            self.row_highlight
        }
    }
}

pub const THEME: Theme = Theme {
    table: Table {
        base: Style::new().fg(Color::Blue),
        base_active: Style::new().fg(Color::White),
        header: Style::new().add_modifier(Modifier::UNDERLINED),
        row_highlight: Style::new().fg(Color::White),
        row_highlight_active: Style::new().fg(Color::Black).bg(Color::White),
    },
};
