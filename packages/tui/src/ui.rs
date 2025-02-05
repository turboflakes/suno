use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // Define layouts.
    let container = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
        .split(frame.area());

    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(container[0]);

    // TODO: Set constraints based on the enabled features.
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(4),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Fill(1),
        ])
        .split(outer_layout[0]);

    // Render widgets.

    render_networks_widget(app, frame, left_layout[0]);

    render_validators_widget(app, frame, left_layout[1]);

    render_collators_widget(app, frame, left_layout[2]);

    render_rpcs_widget(app, frame, left_layout[3]);

    render_body_widget(app, frame, outer_layout[1]);

    render_legend_widget(app, frame, container[1]);
}

fn render_networks_widget(_app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(" >> Network List")
            .block(
                Block::new()
                    .title(" Network ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::Blue).bg(Color::Black))
            .left_aligned(),
        rect,
    );
}

fn render_validators_widget(_app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(" >> Validators List")
            .block(
                Block::new()
                    .title(" Validators ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::Blue).bg(Color::Black))
            .left_aligned(),
        rect,
    );
}

fn render_collators_widget(_app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(" >> Collators List")
            .block(
                Block::new()
                    .title(" Collators ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
            )
            .style(Style::default().fg(Color::Blue).bg(Color::Black))
            .left_aligned(),
        rect,
    );
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

fn render_body_widget(app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            app.counter
        ))
        .block(
            Block::bordered()
                .title(" Body Dynamic Content ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .style(Style::default().fg(Color::Blue).bg(Color::Black))
        .centered(),
        rect,
    );
}

fn render_legend_widget(_app: &mut App, frame: &mut Frame, rect: Rect) {
    frame.render_widget(
        Paragraph::new(format!(
            "← → ↑ ↓: navigate | x: menu | q: quit | +/-: increment/decrement"
        ))
        .style(Style::default().fg(Color::Blue).bg(Color::Black))
        .centered(),
        rect,
    );
}
