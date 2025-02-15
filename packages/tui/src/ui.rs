use crate::app::App;
use crate::section::Section;
use crate::widgets::popup::Mode;
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use snops_common::config::CONFIG;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let config = CONFIG.clone();
    // Define layouts.
    let container = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
        .split(frame.area());

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(container[0]);

    let mut constraints = vec![Constraint::Max(8)];
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

    // Display configurable features.
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

    // TODO: Implement tabs to switch between body widgets.
    // render_body_widget(app, frame, outer_layout[1]);
    render_logs_widget(app, frame, outer_layout[1]);

    // Display footer.
    render_legend_widget(app, frame, container[1]);

    // Render the frame.
    if app.popup.is_visible() {
        match app.section {
            Section::Validators => {
                render_validators_popup(app, frame);
            }
            _ => {}
        }
    }
}

fn render_validators_popup(app: &mut App, frame: &mut Frame) {
    let rect = match &app.popup.get_mode() {
        Mode::Menu => popup_area(frame.area(), 50, 30),
        Mode::Confirm => popup_area(frame.area(), 40, 10),
        Mode::Transaction => popup_area(frame.area(), 20, 7),
    };
    frame.render_widget(Clear, rect); //this clears out the background
    frame.render_widget(&app.popup, rect);
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn render_chains_widget(app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(&app.chains, rect);
}

fn render_validators_widget(app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(&app.validators, rect);
}

fn render_collators_widget(app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(&app.collators, rect);
}

fn render_rpcs_widget(_app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(" >> RPCs List")
            .block(
                Block::new()
                    .title(" RPCs ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::Blue).bg(Color::Black))
            .left_aligned(),
        rect,
    );
}

fn render_body_widget(_app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(format!(" >> Press `Esc`, `Ctrl-C` or `q` to stop running."))
            .block(
                Block::bordered()
                    .title(" App Content ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::Blue).bg(Color::Black))
            .centered(),
        rect,
    );
}

fn render_logs_widget(app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        TuiLoggerWidget::default()
            .block(Block::bordered().title("Logs"))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::Blue)),
        rect,
    );
}

fn render_legend_widget(app: &mut App, frame: &mut Frame, rect: Rect) {
    let footer = if app.popup.is_visible() {
        Paragraph::new(format!("enter: run | ↑ ↓: navigate | x: close"))
            .style(Style::default().fg(Color::Blue))
            .centered()
    } else {
        Paragraph::new(format!("← ↑ → ↓: navigate | x: menu | q: quit"))
            .style(Style::default().fg(Color::Blue))
            .centered()
    };
    frame.render_widget(footer, rect);
}
