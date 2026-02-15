use ratatui::style::{Color, Modifier, Style};

// Grays
// | Hex       | RGB             | Notes         |
// | --------- | --------------- | ------------- |
// | `#F8F7F7` | (248, 247, 247) | Lightest gray |
// | `#DCDBDB` | (220, 219, 219) |               |
// | `#C1BEBF` | (193, 190, 191) |               |
// | `#A5A1A3` | (165, 161, 163) |               |
// | `#8A8587` | (138, 133, 135) |               |
// | `#6D696B` | (109, 105, 107) |               |
// | `#514E4F` | (81, 78, 79)    |               |
// | `#343233` | (52, 50, 51)    | **Primary**   |
// | `#1F1E1F` | (31, 30, 31)    |               |
// | `#0A0A0A` | (10, 10, 10)    |               |
// | --------- | --------------- | ------------- |
//
// Yellows
// | Hex       | RGB             | Notes          |
// | --------- | --------------- | -------------- |
// | `#FEEB8B` | (254, 235, 139) | Light yellow   |
// | `#FDE253` | (253, 226, 83)  |                |
// | `#FDD91E` | (253, 217, 30)  | **Secondary**  |
// | `#DEBB02` | (222, 187, 2)   |                |
// | `#A78C01` | (167, 140, 1)   | Darkest yellow |
// | --------- | --------------- | -------------- |
//
//
const GRAY_10: Color = Color::Rgb(248, 247, 247);
const GRAY_20: Color = Color::Rgb(220, 219, 219);
const GRAY_30: Color = Color::Rgb(193, 190, 191);
const GRAY_40: Color = Color::Rgb(165, 161, 163);
const GRAY_50: Color = Color::Rgb(138, 133, 135);
const GRAY_60: Color = Color::Rgb(109, 105, 107);
const GRAY_70: Color = Color::Rgb(81, 78, 79);
const GRAY_80: Color = Color::Rgb(52, 50, 51); // Primary
const GRAY_90: Color = Color::Rgb(31, 30, 31);
const GRAY_100: Color = Color::Rgb(10, 10, 10);

const YELLOW_50: Color = Color::Rgb(254, 235, 139);
const YELLOW_60: Color = Color::Rgb(253, 226, 83);
const YELLOW_70: Color = Color::Rgb(253, 217, 30); // Secondary
const YELLOW_80: Color = Color::Rgb(222, 187, 2);
const YELLOW_90: Color = Color::Rgb(167, 140, 1);

pub struct Theme {
    pub block: Block,
    pub table: Table,
    pub paragraph: Paragraph,
    pub scrollbar: Scrollbar,
    pub input: Input,
}

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
    fn get_style(&self, style: Style, active: bool) -> Style {
        if active {
            self.active
        } else {
            style
        }
    }

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

pub struct Scrollbar {
    pub base: Style,
}

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

pub const THEME: Theme = Theme {
    block: Block {
        base: Style::new().bg(GRAY_70).fg(GRAY_20),
        menu_top: Style::new().bg(GRAY_90).fg(GRAY_20),
        menu_bottom: Style::new().bg(GRAY_80).fg(GRAY_20),
        main: Style::new().bg(GRAY_100).fg(GRAY_20),
        footer_left: Style::new().bg(GRAY_80).fg(GRAY_50),
        footer_right: Style::new().bg(GRAY_100).fg(GRAY_50),
        active: Style::new().bg(GRAY_70).fg(GRAY_10),
    },
    table: Table {
        base: Style::new().fg(GRAY_20),
        header: Style::new().fg(YELLOW_60),
        header_active: Style::new().fg(YELLOW_70).add_modifier(Modifier::BOLD),
        row_highlight: Style::new().fg(GRAY_10),
        row_highlight_active: Style::new().bg(GRAY_10).fg(GRAY_100),
    },
    paragraph: Paragraph {
        base: Style::new().fg(GRAY_20),
        base_active: Style::new().fg(GRAY_10),
        header: Style::new().fg(YELLOW_60),
        header_active: Style::new().fg(YELLOW_70).add_modifier(Modifier::BOLD),
        label: Style::new().fg(GRAY_50).add_modifier(Modifier::BOLD),
    },
    scrollbar: Scrollbar {
        base: Style::new().fg(GRAY_20),
    },
    input: Input {
        base: Style::new().bg(GRAY_90).fg(GRAY_40),
        base_active: Style::new()
            .bg(GRAY_90)
            .fg(GRAY_20)
            .add_modifier(Modifier::SLOW_BLINK),
        label: Style::new().fg(YELLOW_60).add_modifier(Modifier::BOLD),
        placeholder: Style::new().fg(GRAY_50),
        prefix: Style::new().fg(YELLOW_90),
        prefix_active: Style::new().fg(YELLOW_70).add_modifier(Modifier::BOLD),
        suffix: Style::new().fg(GRAY_80),
        suffix_active: Style::new().fg(GRAY_40).add_modifier(Modifier::BOLD),
        error: Style::new().fg(YELLOW_90).add_modifier(Modifier::ITALIC),
    },
};
