use crate::palette::{Palette, SUNO_DARK_PALETTE};
use ratatui::style::{Modifier, Style};

/// A trait for objects that have an active style.
pub trait Themed {
    fn active_style(&self) -> Style;

    fn get_style(&self, style: Style, is_active: bool) -> Style {
        if is_active {
            self.active_style()
        } else {
            style
        }
    }
}

#[derive(Debug)]
pub struct Theme {
    pub block: Block,
    pub table: Table,
    pub paragraph: Paragraph,
    pub scrollbar: Scrollbar,
    pub input: Input,
    pub logo: Logo,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_palette(&SUNO_DARK_PALETTE)
    }
}

impl Theme {
    pub fn from_palette(p: &Palette) -> Self {
        Theme {
            block: Block {
                base: Style::new().bg(p.color_06).fg(p.color_01),
                menu_top: Style::new().bg(p.color_08).fg(p.color_01),
                menu_bottom: Style::new().bg(p.color_07).fg(p.color_01),
                main: Style::new().bg(p.color_09).fg(p.color_01),
                footer_left: Style::new().bg(p.color_07).fg(p.color_04),
                footer_right: Style::new().bg(p.color_09).fg(p.color_04),
                active: Style::new().bg(p.color_06).fg(p.color_00),
            },
            table: Table {
                base: Style::new().fg(p.color_01),
                header: Style::new().fg(p.color_12),
                header_active: Style::new().fg(p.color_13).add_modifier(Modifier::BOLD),
                row_highlight: Style::new().fg(p.color_00),
                row_highlight_active: Style::new().bg(p.color_00).fg(p.color_09),
            },
            paragraph: Paragraph {
                base: Style::new().fg(p.color_01),
                base_active: Style::new().fg(p.color_00),
                header: Style::new().fg(p.color_12),
                header_active: Style::new().fg(p.color_13).add_modifier(Modifier::BOLD),
                label: Style::new().fg(p.color_04).add_modifier(Modifier::BOLD),
            },
            scrollbar: Scrollbar {
                base: Style::new().fg(p.color_01),
            },
            input: Input {
                base: Style::new().bg(p.color_08).fg(p.color_03),
                base_active: Style::new()
                    .bg(p.color_08)
                    .fg(p.color_01)
                    .add_modifier(Modifier::SLOW_BLINK),
                label: Style::new().fg(p.color_12).add_modifier(Modifier::BOLD),
                placeholder: Style::new().fg(p.color_04),
                prefix: Style::new().fg(p.color_15),
                prefix_active: Style::new().fg(p.color_13).add_modifier(Modifier::BOLD),
                suffix: Style::new().fg(p.color_15),
                suffix_active: Style::new().fg(p.color_13).add_modifier(Modifier::BOLD),
                error: Style::new().fg(p.color_15).add_modifier(Modifier::ITALIC),
            },
            logo: Logo {
                base: Style::new().bg(p.color_09).fg(p.color_01),
                base_dark: Style::new().bg(p.color_01).fg(p.color_09),
                with_shadow: Style::new().bg(p.color_13).fg(p.color_01),
                only_shadow: Style::new().bg(p.color_09).fg(p.color_13),
            },
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub base: Style,
    pub menu_top: Style,
    pub menu_bottom: Style,
    pub footer_left: Style,
    pub footer_right: Style,
    pub main: Style,
    pub active: Style,
}

impl Block {
    pub fn base(&self, active: bool) -> Style {
        self.get_style(self.base, active)
    }

    pub fn menu_top(&self, active: bool) -> Style {
        self.get_style(self.menu_top, active)
    }

    pub fn menu_bottom(&self, active: bool) -> Style {
        self.get_style(self.menu_bottom, active)
    }

    pub fn main(&self, active: bool) -> Style {
        self.get_style(self.main, active)
    }
}

impl Themed for Block {
    fn active_style(&self) -> Style {
        self.active
    }
}

#[derive(Debug)]
pub struct Table {
    pub base: Style,
    pub header: Style,
    pub header_active: Style,
    pub row_highlight: Style,
    pub row_highlight_active: Style,
}

impl Table {
    pub fn header(&self, active: bool) -> Style {
        if active {
            self.header_active
        } else {
            self.header
        }
    }

    pub fn row_highlight(&self, active: bool) -> Style {
        if active {
            self.row_highlight_active
        } else {
            self.row_highlight
        }
    }

    pub fn highlight_symbol(&self, active: bool) -> &str {
        if active {
            "❯"
        } else {
            ""
        }
    }
}

#[derive(Debug)]
pub struct Paragraph {
    pub base: Style,
    pub base_active: Style,
    pub header: Style,
    pub header_active: Style,
    pub label: Style,
}

impl Paragraph {
    pub fn base(&self, active: bool) -> Style {
        if active {
            self.base_active
        } else {
            self.base
        }
    }

    pub fn header(&self, active: bool) -> Style {
        if active {
            self.header_active
        } else {
            self.header
        }
    }
}

#[derive(Debug)]
pub struct Scrollbar {
    pub base: Style,
}

#[derive(Debug)]
pub struct Input {
    pub base: Style,
    pub base_active: Style,
    pub label: Style,
    pub placeholder: Style,
    pub prefix: Style,
    pub prefix_active: Style,
    pub suffix: Style,
    pub suffix_active: Style,
    pub error: Style,
}

impl Input {
    pub fn base(&self, active: bool) -> Style {
        if active {
            self.base_active
        } else {
            self.base
        }
    }

    pub fn prefix(&self, active: bool) -> Style {
        if active {
            self.prefix_active
        } else {
            self.prefix
        }
    }

    pub fn suffix(&self, active: bool) -> Style {
        if active {
            self.suffix_active
        } else {
            self.suffix
        }
    }
}

#[derive(Debug)]
pub struct Logo {
    pub base: Style,
    pub base_dark: Style,
    pub with_shadow: Style,
    pub only_shadow: Style,
}
