use crate::widgets::logo::Logo;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
};
use strum::{Display, EnumIter, FromRepr};
use suno_config::CONFIG;

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum Window {
    #[default]
    Main,
    Logs,
    Help,
}

impl Window {
    pub fn next(self) -> Self {
        let current_index = self as usize;
        if Self::from_repr(current_index) == Some(Window::Help) {
            return Window::Main;
        }
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn prev(self) -> Self {
        let current_index = self as usize;
        if Self::from_repr(current_index) == Some(Window::Main) {
            return Window::Help;
        }
        let prev_index = current_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();
        let [details_area, logo_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),    // General project details
                Constraint::Length(46), // Large logo width
            ])
            .areas(area);

        // Render help and legend block
        let details_lines = vec![
            Line::from(Span::raw("PROJECT DETAILS").style(theme.paragraph.header(true))),
            Line::from(vec![
                Span::raw("site ").style(theme.paragraph.label(true)),
                Span::raw("https://suno.sh"),
            ]),
            Line::from(vec![
                Span::raw("description ").style(theme.paragraph.label(true)),
                Span::raw(
                    "Yet another way to manage substrate node operations from your terminal.",
                ),
            ]),
            Line::from(vec![
                Span::raw("repo ").style(theme.paragraph.label(true)),
                Span::raw("https://github.com/turboflakes/suno"),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("VALIDATOR STATUS (◈)").style(theme.paragraph.header(true))),
            Line::from(vec![
                Span::raw("[A] ").style(theme.paragraph.label(true)),
                Span::raw("Authority"),
            ]),
            Line::from(vec![
                Span::raw("[P] ").style(theme.paragraph.label(true)),
                Span::raw("Para-Authority"),
            ]),
            Line::from(vec![
                Span::raw("[W] ").style(theme.paragraph.label(true)),
                Span::raw("Waiting"),
            ]),
            Line::from(vec![
                Span::raw("[U] ").style(theme.paragraph.label(true)),
                Span::raw("Undefined. No Validator intention set."),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("SUPPORTED PROXIES").style(theme.paragraph.header(true))),
            Line::from(vec![
                Span::raw("[S] ").style(theme.paragraph.label(true)),
                Span::raw("Staking. Must be configured on Asset Hub Chain."),
            ]),
            Line::from(vec![
                Span::raw("[SO] ").style(theme.paragraph.label(true)),
                Span::raw("StakingOperator. Must be configured on Asset Hub Chain."),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("SUPPORTED EXTRINSICS").style(theme.paragraph.header(true))),
            Line::from(vec![
                Span::raw("/bond ").style(theme.paragraph.label(true)),
                Span::raw("Bond requires `Staking` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/bond_extra ").style(theme.paragraph.label(true)),
                Span::raw("BondExtra requires `Staking` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/unbond ").style(theme.paragraph.label(true)),
                Span::raw("Unbond requires `Staking` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/rebond ").style(theme.paragraph.label(true)),
                Span::raw("Rebond requires `Staking` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/withdraw_unbonded ").style(theme.paragraph.label(true)),
                Span::raw("Withdraw unbonded requires `Staking` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/validate ").style(theme.paragraph.label(true)),
                Span::raw("Validate requires one of `Staking` or `StakingOperator` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/chill ").style(theme.paragraph.label(true)),
                Span::raw("Chill requires one of `Staking` or `StakingOperator` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/set_keys ").style(theme.paragraph.label(true)),
                Span::raw("Set session keys requires one of `Staking` or `StakingOperator` proxy."),
            ]),
            Line::from(vec![
                Span::raw("/purge_keys ").style(theme.paragraph.label(true)),
                Span::raw(
                    "Purge session keys requires one of `Staking` or `StakingOperator` proxy.",
                ),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("SUPPORTED CUSTOM COMMANDS").style(theme.paragraph.header(true))),
            Line::from(vec![
                Span::raw("/rotate_and_set_keys ").style(theme.paragraph.label(true)),
                Span::raw(
                    "Rotate and set session keys as a single operation. Requires proxy configured.",
                ),
            ]),
            Line::from(vec![
                Span::raw("/has_keys ").style(theme.paragraph.label(true)),
                Span::raw("Check whether the host has the next session keys."),
            ]),
            Line::from(vec![
                Span::raw("/has_queued_keys ").style(theme.paragraph.label(true)),
                Span::raw("Check whether the host has queued session keys."),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("KEYBINDINGS").style(theme.paragraph.header(true))),
            Line::from(vec![
                Span::raw("tab ").style(theme.paragraph.label(true)),
                Span::raw("Input focus, Autocomplete or Navigate between pane sections."),
            ]),
            Line::from(vec![
                Span::raw("esc ").style(theme.paragraph.label(true)),
                Span::raw("Close popup or Go back."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+e ").style(theme.paragraph.label(true)),
                Span::raw("Show commands."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+w ").style(theme.paragraph.label(true)),
                Span::raw("Switch window."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+m ").style(theme.paragraph.label(true)),
                Span::raw("Toggle mask on hosts."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+h/ctrl+l/left/right ").style(theme.paragraph.label(true)),
                Span::raw("Navigate between pane sections."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+j/ctrl+k/up/down ").style(theme.paragraph.label(true)),
                Span::raw(
                    "Select a chain, validator, or extrinsic depending on the highlighted area.",
                ),
            ]),
            Line::from(vec![
                Span::raw("ctrl+c ").style(theme.paragraph.label(true)),
                Span::raw("Quit suno."),
            ]),
        ];

        let block = Block::default()
            .style(theme.block.main)
            .padding(Padding::proportional(1));
        let details = Paragraph::new(details_lines)
            .block(block)
            .style(theme.paragraph.base)
            .wrap(Wrap { trim: true });

        details.render(details_area, buf);

        // Render logo block
        let logo_block = Block::default()
            .style(theme.logo.base)
            .padding(Padding::proportional(2));
        let logo_inner_area = logo_block.inner(logo_area);
        logo_block.render(logo_area, buf);

        let logo = Logo::original();
        logo.render(logo_inner_area, buf);
    }
}

impl Widget for &Window {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Window::Help => self.render_help(area, buf),
            _ => {}
        }
    }
}
