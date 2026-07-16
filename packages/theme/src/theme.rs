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
    pub qrcode: Qrcode,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_palette(&SUNO_DARK_PALETTE.1)
    }
}

impl Theme {
    pub fn from_palette(p: &Palette) -> Self {
        Theme {
            block: Block {
                base: Style::new().bg(p.color_06).fg(p.color_01),
                main: Style::new().bg(p.background).fg(p.foreground),
                pane_header: Style::new().bg(p.color_08).fg(p.color_01),
                pane_body: Style::new().bg(p.color_07).fg(p.color_01),
                footer_left: Style::new().bg(p.color_07).fg(p.color_04),
                footer_right: Style::new().bg(p.color_09).fg(p.color_04),
                active: Style::new().bg(p.color_06).fg(p.color_00),
            },
            table: Table {
                base: Style::new().fg(p.color_01),
                header: Style::new().fg(p.color_13),
                header_active: Style::new().fg(p.color_14).add_modifier(Modifier::BOLD),
                row_highlight: Style::default(),
                row_highlight_active: Style::new()
                    .bg(p.selection_background)
                    .fg(p.selection_foreground),
            },
            paragraph: Paragraph {
                base: Style::new().fg(p.color_01),
                base_active: Style::new().fg(p.color_00),
                header: Style::new().fg(p.color_13),
                header_active: Style::new().fg(p.color_14).add_modifier(Modifier::BOLD),
                label: Style::new().fg(p.color_04),
                label_active: Style::new().fg(p.color_04).add_modifier(Modifier::BOLD),
                label_inverse: Style::new().fg(p.color_14),
                cell: Style::default(),
                cell_active: Style::new()
                    .bg(p.selection_background)
                    .fg(p.selection_foreground),
            },
            scrollbar: Scrollbar {
                base: Style::new().fg(p.color_01),
            },
            input: Input {
                base: Style::new().bg(p.color_07).fg(p.color_03),
                base_active: Style::new()
                    .bg(p.cursor_color)
                    .fg(p.cursor_text)
                    .add_modifier(Modifier::SLOW_BLINK),
                label: Style::new().fg(p.color_13).add_modifier(Modifier::BOLD),
                placeholder: Style::new().fg(p.color_04),
                prefix: Style::new().fg(p.color_14),
                prefix_active: Style::new().fg(p.color_14).add_modifier(Modifier::BOLD),
                suffix: Style::new().fg(p.color_15),
                suffix_active: Style::new().fg(p.color_14).add_modifier(Modifier::BOLD),
                error: Style::new().fg(p.color_15).add_modifier(Modifier::ITALIC),
                success: Style::new().fg(p.color_13).add_modifier(Modifier::ITALIC),
            },
            logo: Logo {
                base: Style::new().bg(p.background).fg(p.foreground),
                base_dark: Style::new().bg(p.foreground).fg(p.background),
                with_shadow: Style::new().bg(p.color_13).fg(p.foreground),
                only_shadow: Style::new().bg(p.background).fg(p.color_13),
            },
            qrcode: Qrcode {
                base: Style::new().bg(p.foreground).fg(p.background),
                scanner: Style::new().bg(p.color_08),
            },
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub base: Style,
    pub pane_header: Style,
    pub pane_body: Style,
    pub footer_left: Style,
    pub footer_right: Style,
    pub main: Style,
    pub active: Style,
}

impl Block {
    pub fn base(&self, active: bool) -> Style {
        self.get_style(self.base, active)
    }

    pub fn pane_header(&self, active: bool) -> Style {
        self.get_style(self.pane_header, active)
    }

    pub fn pane_body(&self, active: bool) -> Style {
        self.get_style(self.pane_body, active)
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
    pub label_active: Style,
    pub label_inverse: Style,
    pub cell: Style,
    pub cell_active: Style,
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

    pub fn label(&self, active: bool) -> Style {
        if active {
            self.label_active
        } else {
            self.label
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
    pub success: Style,
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

#[derive(Debug)]
pub struct Qrcode {
    pub base: Style,
    pub scanner: Style,
}
