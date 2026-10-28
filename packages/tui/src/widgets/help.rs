use crate::widgets::logo::Logo;
use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};
use suno_theme::Theme;

const LOGO_WIDTH: u16 = 46;
const LOGO_HEIGHT: u16 = 25;

/// Scroll state of the help window, whose text can be taller than the window.
#[derive(Debug, Default, Clone, Copy)]
pub struct HelpState {
    scroll: u16,
    /// Last scrollable height, known once rendered.
    max_scroll: u16,
    theme: Theme,
}

impl HelpState {
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1).min(self.max_scroll);
    }
}

/// Rows the `lines` take once wrapped to `width` columns.
fn wrapped_height(lines: &[Line], width: u16) -> u16 {
    let width = width.max(1) as usize;
    lines
        .iter()
        .map(|line| line.width().div_ceil(width).max(1))
        .sum::<usize>()
        .min(u16::MAX as usize) as u16
}

/// A line of the help: what it describes, and its description.
type HelpEntry = (&'static str, &'static str);

const PROJECT_DETAILS: &[HelpEntry] = &[
    ("site ", "https://suno.sh"),
    (
        "description ",
        "Yet another way to manage substrate node operations from your terminal.",
    ),
    ("repo ", "https://github.com/turboflakes/suno"),
];

const VALIDATOR_STATUS: &[HelpEntry] = &[
    ("[A] ", "Authority"),
    ("[P] ", "Para-Authority"),
    ("[W] ", "Waiting"),
    ("[U] ", "Undefined. No Validator intention set."),
];

const VALIDATOR_PROXIES: &[HelpEntry] = &[
    ("[S] ", "Staking. Must be configured on Asset Hub Chain."),
    (
        "[SO] ",
        "StakingOperator. Must be configured on Asset Hub Chain.",
    ),
];

const COLLATOR_STATUS: &[HelpEntry] = &[
    ("[P] ", "Permissionless"),
    ("[I] ", "Invulnerable"),
    ("[C] ", "Candidate"),
    ("[E] ", "Exiting. Signalled its intent to leave."),
    ("[U] ", "Undefined. No Collator intention set."),
];

const COLLATOR_PROXIES: &[HelpEntry] = &[
    (
        "[C] ",
        "Collator. Must be configured on every System Chain.",
    ),
    (
        "[NT] ",
        "NonTransfer. Must be configured on every System Chain.",
    ),
];

const STAKING_EXTRINSICS: &[HelpEntry] = &[
    ("/bond ", "Bond requires `Staking` proxy."),
    ("/bond_extra ", "BondExtra requires `Staking` proxy."),
    ("/unbond ", "Unbond requires `Staking` proxy."),
    ("/rebond ", "Rebond requires `Staking` proxy."),
    (
        "/withdraw_unbonded ",
        "Withdraw unbonded requires `Staking` proxy.",
    ),
    (
        "/validate ",
        "Validate requires one of `Staking` or `StakingOperator` proxy.",
    ),
    (
        "/chill ",
        "Chill requires one of `Staking` or `StakingOperator` proxy.",
    ),
];

const COLLATOR_EXTRINSICS: &[HelpEntry] = &[
    (
        "/register_as_candidate ",
        "Register requires `Collator` proxy.",
    ),
    ("/leave_intent ", "Leave intent requires `Collator` proxy."),
    ("/update_bond ", "Update bond requires `Collator` proxy."),
    (
        "/take_candidate_slot ",
        "Take slot requires `Collator` proxy.",
    ),
];

const KEYS_EXTRINSICS: &[HelpEntry] = &[
    (
        "/set_keys ",
        "Set session keys requires one of `Staking`, `StakingOperator` or `NonTransfer` proxy.",
    ),
    (
        "/purge_keys ",
        "Purge session keys requires one of `Staking`, `StakingOperator` or `NonTransfer` proxy.",
    ),
];

const CUSTOM_COMMANDS: &[HelpEntry] = &[
    (
        "/rotate_and_set_keys ",
        "Rotate and set session keys as a single operation. Requires proxy configured.",
    ),
    (
        "/has_keys ",
        "Check whether the host has the next session keys.",
    ),
    (
        "/has_queued_keys ",
        "Check whether the host has queued session keys.",
    ),
];

const KEYBINDINGS: &[HelpEntry] = &[
    ("/ or ctrl+e ", "Show commands."),
    (
        "tab ",
        "Input focus, Autocomplete or Navigate between pane sections and the main window.",
    ),
    ("esc ", "Close popup or Go back."),
    (
        "space ",
        "Pin the selected validator or collator, or unpin it. Not for the ones listed in the config.",
    ),
    ("ctrl+w ", "Switch to the next window."),
    ("ctrl+m ", "Toggle mask on hosts."),
    ("ctrl+t ", "Show list of available themes."),
    (
        "ctrl+f ",
        "Filter the validators or the collators by identity or address.",
    ),
    ("ctrl+v ", "Toggle between grouped and listed view."),
    (
        "ctrl+h/ctrl+l/left/right ",
        "Focus the window tabs, and then switch window.",
    ),
    (
        "ctrl+j/ctrl+k/up/down ",
        "Select a chain, validator, or extrinsic depending on the highlighted area.",
    ),
    ("shift+ctrl+c ", "Copy to clipboard."),
    ("ctrl+c ", "Quit suno."),
];

/// The sections of the help, in the order displayed: a title and its entries.
const HELP_SECTIONS: &[(&str, &[HelpEntry])] = &[
    ("PROJECT DETAILS", PROJECT_DETAILS),
    ("VALIDATOR STATUS (◈)", VALIDATOR_STATUS),
    ("VALIDATOR SUPPORTED PROXIES", VALIDATOR_PROXIES),
    ("COLLATOR STATUS (◈)", COLLATOR_STATUS),
    ("COLLATOR SUPPORTED PROXIES", COLLATOR_PROXIES),
    ("STAKING SUPPORTED EXTRINSICS", STAKING_EXTRINSICS),
    ("COLLATOR SUPPORTED EXTRINSICS", COLLATOR_EXTRINSICS),
    ("KEYS SUPPORTED EXTRINSICS", KEYS_EXTRINSICS),
    ("SUPPORTED CUSTOM COMMANDS", CUSTOM_COMMANDS),
    ("KEYBINDINGS", KEYBINDINGS),
];

/// The lines of the help, a header followed by its entries for each section, and a blank
/// line between sections.
fn help_lines(theme: Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (index, (title, entries)) in HELP_SECTIONS.iter().enumerate() {
        if index > 0 {
            lines.push(Line::from(Span::raw("")));
        }
        lines.push(Line::from(
            Span::raw(*title).style(theme.paragraph.header(true)),
        ));
        for (label, description) in entries.iter() {
            lines.push(Line::from(vec![
                Span::raw(*label).style(theme.paragraph.label(true)),
                Span::raw(*description),
            ]));
        }
    }
    lines
}

/// Height of the help content when `width` columns wide: the text, wrapped to the column left
/// by the logo, or the logo, whichever is taller, and at least the `visible_height`.
fn help_content_height(lines: &[Line], visible_height: u16, width: u16) -> u16 {
    let text_width = width.saturating_sub(LOGO_WIDTH);
    wrapped_height(lines, text_width)
        .max(LOGO_HEIGHT)
        .max(visible_height)
}

/// Renders the text and the logo on a buffer `width` by `height`, that starts at the origin,
/// as tall as their content.
fn render_help_content(theme: Theme, lines: Vec<Line<'static>>, width: u16, height: u16) -> Buffer {
    let mut content = Buffer::empty(Rect::new(0, 0, width, height));
    let [details_area, logo_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),            // General project details
            Constraint::Length(LOGO_WIDTH), // Large logo width
        ])
        .areas(content.area);

    // Render help and legend block
    let lines_block = Block::default().style(theme.block.main);
    Paragraph::new(lines)
        .block(lines_block)
        .style(theme.paragraph.base)
        .wrap(Wrap { trim: true })
        .render(details_area, &mut content);

    // Render logo block
    let logo_block = Block::default()
        .style(theme.logo.base)
        .padding(Padding::proportional(2));
    let logo_inner_area = logo_block.inner(logo_area);
    logo_block.render(logo_area, &mut content);

    Logo::original()
        .theme(theme)
        .render(logo_inner_area, &mut content);

    content
}

/// Copies the part of the `content` scrolled to, `scroll` rows down, to the `target` area of
/// the screen `buf`.
fn copy_visible_help(content: &Buffer, target: Rect, scroll: u16, buf: &mut Buffer) {
    for y in 0..target.height {
        let virtual_y = y + scroll;
        if virtual_y >= content.area.height {
            break;
        }

        for x in 0..content.area.width.min(target.width) {
            let source_cell = &content[(x, virtual_y)];

            let dest_x = target.x + x;
            let dest_y = target.y + y;

            if dest_x < buf.area.right() && dest_y < buf.area.bottom() {
                let target_cell = &mut buf[(dest_x, dest_y)];
                target_cell.set_symbol(source_cell.symbol());
                target_cell.set_style(source_cell.style());
            }
        }
    }
}

pub struct HelpWidget<'a> {
    block: Option<Block<'a>>,
}

impl<'a> HelpWidget<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }

    /// The block around the help, the text and the logo go inside of it.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

/// Help view widget implementation, with the text and the logo scrolling together.
impl StatefulWidget for HelpWidget<'_> {
    type State = HelpState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = state.theme;

        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let lines = help_lines(theme);

        // The scrollbar takes a column when the content doesn't fit
        let is_scroll_visible =
            help_content_height(&lines, inner.height, inner.width) > inner.height;
        let width = if is_scroll_visible {
            inner.width.saturating_sub(1)
        } else {
            inner.width
        };
        let height = help_content_height(&lines, inner.height, width);
        state.max_scroll = height.saturating_sub(inner.height);
        state.scroll = state.scroll.min(state.max_scroll);

        let content = render_help_content(theme, lines, width, height);
        copy_visible_help(&content, inner, state.scroll, buf);

        if is_scroll_visible {
            let scrollbar_area = Rect {
                x: inner.x + inner.width.saturating_sub(1),
                y: inner.y + 1,
                width: 1,
                height: inner.height.saturating_sub(2),
            };
            render_scrollbar(
                theme,
                state.scroll as usize,
                state.max_scroll as usize + 1,
                scrollbar_area,
                buf,
            );
        }
    }
}
