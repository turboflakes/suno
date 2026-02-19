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
    if app.popup.is_visible() {
        match app.section {
            Section::Validators => {
                render_validators_popup(app, frame, container[0]);
            }
            _ => {}
        }
    }
}

fn render_validators_popup(app: &mut App, frame: &mut Frame, area: Rect) {
    let area = match &app.popup.get_mode() {
        PopupMode::Menu => popup_area(
            frame.area(),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Flex::Center,
        ),
        PopupMode::ConfirmAndSign => popup_area(
            frame.area(),
            Constraint::Percentage(40),
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

fn render_legend_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .style(THEME.block.footer_right)
        .padding(Padding::new(0, 2, 1, 1));

    let mut legend = vec![];

    // show how to open popup with extrinsics/commands
    if app.validators.is_active() && !app.popup.is_visible() {
        legend.push(Span::styled(format!("ctrl+e"), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled(
            format!("show extrinsics"),
            THEME.paragraph.label,
        ));
    };

    if app.popup.is_visible() {
        legend.push(Span::raw("   "));
        match app.popup.get_mode() {
            PopupMode::Menu => {
                legend.push(Span::styled(format!("tab"), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(format!("autocomplete"), THEME.paragraph.label));
                legend.push(Span::raw("   "));
                legend.push(Span::styled(format!("enter"), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(format!("confirm"), THEME.paragraph.label));
                legend.push(Span::raw("   "));
            }
            PopupMode::ConfirmAndSign => {
                legend.push(Span::styled(format!("enter"), THEME.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(
                    format!("sign and submit"),
                    THEME.paragraph.label,
                ));
                legend.push(Span::raw("   "));
            }
            _ => {}
        }
        legend.push(Span::styled(format!("esc"), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled(format!("close"), THEME.paragraph.label));
    } else if app.chains.is_active() || app.validators.is_active() {
        legend.push(Span::raw("   "));
        legend.push(Span::styled(format!("tab or ↑ ↓"), THEME.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled(format!("navigate"), THEME.paragraph.label));
    };

    // Always visible
    legend.push(Span::raw("   "));
    legend.push(Span::styled(format!("ctrl+t"), THEME.paragraph.base));
    legend.push(Span::raw(" "));
    legend.push(Span::styled(format!("switch tab"), THEME.paragraph.label));
    legend.push(Span::raw("   "));
    legend.push(Span::styled(format!("ctrl+c"), THEME.paragraph.base));
    legend.push(Span::raw(" "));
    legend.push(Span::styled(format!("quit"), THEME.paragraph.label));

    let footer = Paragraph::new(Line::from(legend))
        .block(block)
        .right_aligned();

    frame.render_widget(footer, area);
}

fn render_logo_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    let logo = Logo::inline();
    frame.render_widget(&logo, area);
}
