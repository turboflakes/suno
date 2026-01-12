use crate::section::Section;
use crate::widgets::logo::Logo;
use crate::widgets::popup::Mode;
use crate::{app::App, tab::Tab};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    prelude::Margin,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use suno_config::CONFIG;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let config = CONFIG.clone();

    let area = frame.area().inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    let container = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
        .split(area);

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Max(56), Constraint::Fill(1)])
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

    // Switch between main body widgets.
    match app.tab {
        Tab::Main => render_body_widget(app, frame, outer_layout[1]),
        Tab::Logs => render_logs_widget(app, frame, outer_layout[1]),
    }

    // Display commands legend in footer.
    render_legend_widget(app, frame, container[1]);

    // Display logo version in footer.
    render_logo_widget(app, frame, container[1]);
    frame.render_widget(Logo::inline(), container[1]);

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
    let area = match &app.popup.get_mode() {
        Mode::Menu => popup_area(frame.area(), 40, 30),
        Mode::Confirm => popup_area(frame.area(), 40, 10),
        Mode::Transaction => popup_area(frame.area(), 20, 7),
    };
    frame.render_widget(Clear, area); //this clears out the background
    frame.render_widget(&app.popup, area);
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
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
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let block_area = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(&app.validators.as_detailed_group(&app.chains), block_area);
}

fn render_logs_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(
        TuiLoggerWidget::default()
            .block(Block::bordered().title(" Logs "))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::Blue)),
        area,
    );
}

fn render_legend_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let footer = if app.popup.is_visible() {
        Paragraph::new(format!(": run | ↑ ↓: navigate | x: close"))
            .style(Style::default().fg(Color::Blue))
            .right_aligned()
    } else {
        Paragraph::new(format!(
            "tab or alt+tab or ↑ ↓: navigate | x: menu | q: quit"
        ))
        .style(Style::default().fg(Color::Blue))
        .right_aligned()
    };
    frame.render_widget(footer, area);
}

fn render_logo_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(Logo::inline(), area);
}
