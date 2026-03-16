use crate::section::Section;
use crate::theme::THEME;
use crate::widgets::{logo::Logo, popup::Mode as PopupMode};
use crate::{app::App, window::Window};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    prelude::Margin,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};
use suno_config::CONFIG;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let config = CONFIG.clone();

    let area = frame.area().inner(Margin {
        horizontal: 0,
        vertical: 0,
    });

    let container = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
        .split(area);

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Max(56), Constraint::Fill(1)])
        .split(container[0]);

    let mut constraints = vec![Constraint::Length(3 + config.chains.len() as u16)];
    if config.features.enable_validators {
        constraints.push(Constraint::Fill(1));
    } else {
        constraints.push(Constraint::Length(0));
    }

    if config.features.enable_collators {
        constraints.push(Constraint::Fill(1));
    } else {
        constraints.push(Constraint::Length(0));
    }

    if config.features.enable_rpcs {
        constraints.push(Constraint::Fill(1));
    } else {
        constraints.push(Constraint::Length(0));
    }

    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(outer_layout[0]);

    // Display configurable features on the left layout.
    render_chains_widget(app, frame, left_layout[0]);

    if config.features.enable_validators {
        render_validators_widget(app, frame, left_layout[1]);
    }

    if config.features.enable_collators {
        render_collators_widget(app, frame, left_layout[2]);
    }

    if config.features.enable_rpcs {
        render_rpcs_widget(app, frame, left_layout[3]);
    }

    // Switch between main body window.
    match app.window {
        Window::Main => render_body_widget(app, frame, outer_layout[1]),
        Window::Logs => render_logs_widget(app, frame, outer_layout[1]),
        Window::Help => render_help_widget(app, frame, outer_layout[1]),
    }

    let footer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Max(56), Constraint::Fill(1)])
        .split(container[1]);

    // Display logo version in footer, aligned left.
    render_logo_widget(app, frame, footer[0]);

    // Display commands legend in footer, aligned right.
    render_legend_widget(app, frame, footer[1]);

    // Render the frame.
    if app.popup.is_visible() && app.section == Section::Validators {
        render_validators_popup(app, frame, container[0]);
    }
}

fn render_validators_popup(app: &mut App, frame: &mut Frame, area: Rect) {
    let area = match &app.popup.get_mode() {
        PopupMode::Menu => popup_area(
            frame.area(),
            Constraint::Percentage(45),
            Constraint::Percentage(40),
            Flex::Center,
        ),
        PopupMode::Confirm | PopupMode::Locked => popup_area(
            frame.area(),
            Constraint::Percentage(45),
            Constraint::Percentage(40),
            Flex::Center,
        ),
        PopupMode::Transaction => popup_area(
            area,
            Constraint::Percentage(100),
            Constraint::Length(3),
            Flex::End,
        ),
    };
    frame.render_widget(&app.popup, area);
    // Apply the cursor if it was set during render
    let state = app.popup.state.read().unwrap();
    if let Some(pos) = state.get_input_cursor_position() {
        frame.set_cursor_position(pos);
    }
}

fn popup_area(area: Rect, x: Constraint, y: Constraint, flex: Flex) -> Rect {
    let horizontal = Layout::horizontal([x]).flex(flex);
    let vertical = Layout::vertical([y]).flex(flex);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn render_chains_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(&app.chains, area);
}

fn render_validators_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(&app.validators.as_compact(), area);
}

fn render_collators_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(&app.collators, area);
}

fn render_rpcs_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(" >> RPCs List")
            .block(
                Block::new()
                    .title(" RPCs ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::Blue))
            .left_aligned(),
        area,
    );
}

fn render_body_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .style(THEME.block.main)
        .padding(Padding::proportional(1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(&app.validators.as_detailed_group(&app.chains), block_area);
}

fn render_logs_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .style(THEME.block.main)
        .padding(Padding::proportional(1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        TuiLoggerWidget::default()
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(THEME.block.main),
        block_area,
    );
}

fn render_help_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    let [logo_area, help_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(24), // Large logo height
            Constraint::Fill(1),    // Help area takes remaining
        ])
        .areas(area);

    // Render logo block
    let logo_block = Block::default()
        .style(THEME.block.logo)
        .padding(Padding::proportional(1));
    let logo_block_area = logo_block.inner(logo_area);
    frame.render_widget(logo_block, logo_area);
    let logo = Logo::large();
    frame.render_widget(&logo, logo_block_area);

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

    frame.render_widget(help, help_area);
}

fn render_legend_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .style(THEME.block.footer_right)
        .padding(Padding::new(0, 2, 1, 1));

    let mut legend = vec![];

    // show how to open popup with extrinsics/commands
    if app.validators.is_active() && app.validators.is_proxy_valid() && !app.popup.is_visible() {
        legend.push(Span::styled("ctrl+e".to_string(), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled(
            "show extrinsics".to_string(),
            THEME.paragraph.label,
        ));
    };

    if app.popup.is_visible() {
        legend.push(Span::raw("   "));
        match app.popup.get_mode() {
            PopupMode::Menu => {
                legend.push(Span::styled("tab".to_string(), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(
                    "autocomplete".to_string(),
                    THEME.paragraph.label,
                ));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("↑ ↓".to_string(), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("select".to_string(), THEME.paragraph.label));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("enter".to_string(), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("confirm".to_string(), THEME.paragraph.label));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("esc".to_string(), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("close".to_string(), THEME.paragraph.label));
            }
            PopupMode::Confirm => {
                legend.push(Span::styled("enter".to_string(), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(
                    "sign and submit".to_string(),
                    THEME.paragraph.label,
                ));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("esc".to_string(), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("close".to_string(), THEME.paragraph.label));
            }
            _ => {}
        }
    } else if app.chains.is_active() || app.validators.is_active() {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("↑ ↓".to_string(), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("select".to_string(), THEME.paragraph.label));
        legend.push(Span::raw("   "));
        legend.push(Span::styled("tab or ← →".to_string(), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("navigate".to_string(), THEME.paragraph.label));
    } else {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("tab or ← →".to_string(), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("navigate".to_string(), THEME.paragraph.label));
    }

    // Always visible
    legend.push(Span::raw("   "));
    legend.push(Span::styled("ctrl+w".to_string(), THEME.paragraph.base));
    legend.push(Span::raw(" "));
    legend.push(Span::styled(
        "switch window".to_string(),
        THEME.paragraph.label,
    ));
    legend.push(Span::raw("   "));
    legend.push(Span::styled("ctrl+c".to_string(), THEME.paragraph.base));
    legend.push(Span::raw(" "));
    legend.push(Span::styled("quit".to_string(), THEME.paragraph.label));

    let footer = Paragraph::new(Line::from(legend))
        .block(block)
        .right_aligned();

    frame.render_widget(footer, area);
}

fn render_logo_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .style(THEME.block.menu_bottom)
        .padding(Padding::new(2, 0, 1, 1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);
    let logo = Logo::inline();
    frame.render_widget(&logo, block_area);
}
