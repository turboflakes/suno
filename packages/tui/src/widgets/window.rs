use crate::widgets::logo::Logo;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use suno_config::Features;
use suno_theme::Theme;

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Window {
    #[default]
    Validators,
    Collators,
    Logs,
    Help,
}

impl Window {
    /// Windows backed by a feature can be switched off in the config.
    pub fn is_enabled(self, features: &Features) -> bool {
        match self {
            Self::Validators => features.validators_enabled(),
            Self::Collators => features.collators_enabled(),
            Self::Logs | Self::Help => true,
        }
    }

    pub fn is_collators_selected(self) -> bool {
        matches!(self, Self::Collators)
    }

    pub fn is_validators_selected(self) -> bool {
        matches!(self, Self::Validators)
    }

    pub fn is_logs_selected(self) -> bool {
        matches!(self, Self::Logs)
    }

    pub fn is_help_selected(self) -> bool {
        matches!(self, Self::Help)
    }

    /// First window enabled, the one displayed on start.
    pub fn first(features: &Features) -> Self {
        Self::iter()
            .find(|w| w.is_enabled(features))
            .unwrap_or_default()
    }

    /// Next enabled window, wrapping around.
    pub fn next(self, features: &Features) -> Self {
        let count = Self::iter().count();
        (1..=count)
            .filter_map(|step| Self::from_repr((self as usize + step) % count))
            .find(|w| w.is_enabled(features))
            .unwrap_or(self)
    }

    /// Previous enabled window, wrapping around.
    pub fn prev(self, features: &Features) -> Self {
        let count = Self::iter().count();
        (1..=count)
            .filter_map(|step| Self::from_repr((self as usize + count - step) % count))
            .find(|w| w.is_enabled(features))
            .unwrap_or(self)
    }
}

pub fn render_help(theme: Theme, area: Rect, buf: &mut Buffer) {
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
            Span::raw("Yet another way to manage substrate node operations from your terminal."),
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
            Span::raw("Purge session keys requires one of `Staking` or `StakingOperator` proxy."),
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
            Span::raw(
                "Input focus, Autocomplete or Navigate between pane sections and the main window.",
            ),
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
            Span::raw("ctrl+t ").style(theme.paragraph.label(true)),
            Span::raw("Show list of available themes."),
        ]),
        Line::from(vec![
            Span::raw("ctrl+v ").style(theme.paragraph.label(true)),
            Span::raw("Toggle between grouped and listed view."),
        ]),
        Line::from(vec![
            Span::raw("ctrl+h/ctrl+l/left/right ").style(theme.paragraph.label(true)),
            Span::raw("Navigate between pane sections. On the main window, switch window."),
        ]),
        Line::from(vec![
            Span::raw("ctrl+j/ctrl+k/up/down ").style(theme.paragraph.label(true)),
            Span::raw("Select a chain, validator, or extrinsic depending on the highlighted area."),
        ]),
        Line::from(vec![
            Span::raw("shift+ctrl+c ").style(theme.paragraph.label(true)),
            Span::raw("Copy to clipboard."),
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

    let logo = Logo::original().theme(theme);
    logo.render(logo_inner_area, buf);
}
