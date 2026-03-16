use crate::theme::THEME;
use crate::widgets::logo::Logo;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Widget},
};
use strum::{Display, EnumIter, FromRepr};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

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

    fn render_logs(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .style(THEME.block.main)
            .padding(Padding::proportional(1));

        Clear.render(area, buf);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let logger = TuiLoggerWidget::default()
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(THEME.block.main);
        logger.render(inner_area, buf);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let [logo_area, help_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(28), // Large logo height
                Constraint::Fill(1),    // Help area takes remaining
            ])
            .areas(area);

        // Render logo block
        let logo_block = Block::default()
            .style(THEME.logo.base)
            .padding(Padding::proportional(4));
        let logo_inner_area = logo_block.inner(logo_area);
        logo_block.render(logo_area, buf);

        let logo = Logo::original();
        logo.render(logo_inner_area, buf);

        // Render help and legend block
        let help_lines = vec![
            Line::from(Span::raw("GENERAL INFO").style(THEME.paragraph.header)),
            Line::from(vec![
                Span::raw("site ").style(THEME.paragraph.label),
                Span::raw("https://suno.sh"),
            ]),
            Line::from(vec![
                Span::raw("description ").style(THEME.paragraph.label),
                Span::raw("Yet another way to manage substrate node operations from your terminal. Available for Polkadot, Kusama, Paseo and Westend Networks."),
            ]),
            Line::from(vec![
                Span::raw("repo ").style(THEME.paragraph.label),
                Span::raw("https://github.com/turboflakes/suno"),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("SUPPORTED PROXIES").style(THEME.paragraph.header)),
            Line::from(vec![
                Span::raw("[NT] ").style(THEME.paragraph.label),
                Span::raw("NonTransfer, must be set up on the Relay Chain."),
            ]),
            Line::from(vec![
                Span::raw("[S] ").style(THEME.paragraph.label),
                Span::raw("Staking, must be set up on the Asset Hub Chain."),
            ]),
            Line::from(vec![
                Span::raw("[SO] ").style(THEME.paragraph.label),
                Span::raw("StakingOperator, must be set up on the Asset Hub Chain."),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("VALIDATOR STATUS (◈)").style(THEME.paragraph.header)),
            Line::from(vec![
                Span::raw("[A] ").style(THEME.paragraph.label),
                Span::raw("Authority"),
            ]),
            Line::from(vec![
                Span::raw("[P] ").style(THEME.paragraph.label),
                Span::raw("Para-Authority"),
            ]),
            Line::from(vec![
                Span::raw("[W] ").style(THEME.paragraph.label),
                Span::raw("Waiting"),
            ]),
            Line::from(vec![
                Span::raw("[U] ").style(THEME.paragraph.label),
                Span::raw("Undefined. A bond is required to set the account as stash. Next, the operator must set the session keys and trigger the validate intention."),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("SUPPORTED EXTRINSICS").style(THEME.paragraph.header)),
            Line::from(vec![
                Span::raw("/bond ").style(THEME.paragraph.label),
                Span::raw("Bond an amount from your free balance. Only available if the `Staking` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/bond_extra ").style(THEME.paragraph.label),
                Span::raw("Bond additional funds to your existing stash, from your free balance. Only available if the `Staking` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/unbond ").style(THEME.paragraph.label),
                Span::raw("Unbond a portion of your staked balance. Only available if the `Staking` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/rebond ").style(THEME.paragraph.label),
                Span::raw("Rebond a portion that is currently unlocking. Only available if the `Staking` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/withdraw_unbonded ").style(THEME.paragraph.label),
                Span::raw("Withdraw unbonded funds back to your free balance. Only available if the `Staking` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/validate ").style(THEME.paragraph.label),
                Span::raw("Set validate intention, change commission or enable/disable nominations. Only available if the `Staking` or `StakingOperator` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/set_keys ").style(THEME.paragraph.label),
                Span::raw("Set session keys from the output of 'author_rotateKeys' RPC call. Only available if the `NonTransfer` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/purge_keys ").style(THEME.paragraph.label),
                Span::raw("Remove all session keys. Only available is `NonTransfer` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/set_keys_async ").style(THEME.paragraph.label),
                Span::raw("Set session keys from the output of 'author_rotateKeys' RPC call asynchronously. Only available if the `StakingOperator` proxy is set."),
            ]),
            Line::from(vec![
                Span::raw("/purge_keys_async ").style(THEME.paragraph.label),
                Span::raw("Remove all session keys. Only available if the `StakingOperator` proxy is set."),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::raw("KEY BINDINGS").style(THEME.paragraph.header)),
            Line::from(vec![
                Span::raw("ctrl+w ").style(THEME.paragraph.label),
                Span::raw("Switch window."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+c ").style(THEME.paragraph.label),
                Span::raw("Quit suno."),
            ]),
            Line::from(vec![
                Span::raw("esc ").style(THEME.paragraph.label),
                Span::raw("Close or go back."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+h/ctrl+l/left/right ").style(THEME.paragraph.label),
                Span::raw("Navigate between pane sections."),
            ]),
            Line::from(vec![
                Span::raw("tab ").style(THEME.paragraph.label),
                Span::raw("Autocomplete or navigate between pane sections."),
            ]),
            Line::from(vec![
                Span::raw("ctrl+j/ctrl+k/up/down ").style(THEME.paragraph.label),
                Span::raw("Select a chain, validator, or extrinsic depending on the highlighted area."),
            ]),
        ];

        let block = Block::default()
            .style(THEME.block.main)
            .padding(Padding::proportional(1));
        let help = Paragraph::new(help_lines)
            .block(block)
            .style(THEME.paragraph.base);

        help.render(help_area, buf);
    }
}

impl Widget for &Window {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Window::Logs => self.render_logs(area, buf),
            Window::Help => self.render_help(area, buf),
            _ => {}
        }
    }
}
