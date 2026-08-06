use crate::app::App;
use crate::widgets::logs::LogsWidget;
use crate::widgets::{logo::Logo, popup::Mode as PopupMode, window::Window};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    prelude::Margin,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};
use suno_config::CONFIG;

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
    if config.features.validators_enabled() {
        constraints.push(Constraint::Fill(1));
    } else {
        constraints.push(Constraint::Length(0));
    }

    if config.features.collators_enabled() {
        constraints.push(Constraint::Fill(1));
    } else {
        constraints.push(Constraint::Length(0));
    }

    if config.features.rpcs_enabled() {
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

    if config.features.validators_enabled() {
        render_validators_widget(app, frame, left_layout[1]);
    }

    if config.features.collators_enabled() {
        render_collators_widget(app, frame, left_layout[2]);
    }

    if config.features.rpcs_enabled() {
        render_rpcs_widget(app, frame, left_layout[3]);
    }

    // Switch between main body window.
    match app.window {
        Window::Main => render_body_widget(app, frame, outer_layout[1]),
        Window::Logs => render_logs_widget(app, frame, outer_layout[1]),
        Window::Help => {
            frame.render_widget(&app.window, outer_layout[1]);
        }
    }

    let footer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Max(56), Constraint::Fill(1)])
        .split(container[1]);

    // Display logo version in footer, aligned left.
    render_logo_widget(app, frame, footer[0]);

    // Display commands legend in footer, aligned right.
    render_legend_widget(app, frame, footer[1]);

    // Render popup if it is visible.
    if app.popup.is_visible() {
        render_popup(app, frame, container[0]);
    }
}

fn render_popup(app: &mut App, frame: &mut Frame, area: Rect) {
    let area = match &app.popup.get_mode() {
        PopupMode::Transaction | PopupMode::Update => flex_area(
            area,
            Constraint::Percentage(100),
            Constraint::Length(3),
            Flex::End,
        ),
        _ => flex_area(
            frame.area(),
            Constraint::Percentage(70),
            Constraint::Percentage(90),
            Flex::Center,
        ),
    };
    frame.render_widget(&app.popup, area);
    // Apply the cursor if it was set during render
    let state = app.popup.state.read().unwrap();
    if let Some(pos) = state.get_input_cursor_position() {
        frame.set_cursor_position(pos);
    }
}

fn flex_area(area: Rect, horizontal: Constraint, vertical: Constraint, flex: Flex) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(flex).areas(area);
    let [area] = Layout::vertical([vertical]).flex(flex).areas(area);
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
    let theme = CONFIG.theme();
    let block = Block::default()
        .style(theme.block.main)
        .padding(Padding::proportional(1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(&app.validators.as_detailed_group(&app.chains), block_area);
}

fn render_logs_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = CONFIG.theme();
    let block = Block::default()
        .style(theme.block.main)
        .padding(Padding::proportional(1));
    let logs = LogsWidget::new().block(block);
    frame.render_stateful_widget(logs, area, &mut app.logs);
}

fn render_legend_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = CONFIG.theme();
    let block = Block::default()
        .style(theme.block.footer_right)
        .padding(Padding::new(0, 2, 1, 1));

    let mut legend = vec![];

    // show how to open popup with available commands depending on the active section
    if ((app.validators.is_active()
        && (app.validators.is_proxy_valid() || app.validators.is_commands_available()))
        || app.chains.is_active())
        && !app.popup.is_visible()
    {
        legend.push(Span::styled("ctrl+e".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled(
            "show commands".to_string(),
            theme.paragraph.label,
        ));
    };

    if app.popup.is_visible() {
        legend.push(Span::raw("   "));
        match app.popup.get_mode() {
            PopupMode::Menu => {
                legend.push(Span::styled("tab".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(
                    "autocomplete".to_string(),
                    theme.paragraph.label,
                ));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("↑ ↓".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("select".to_string(), theme.paragraph.label));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("enter".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("confirm".to_string(), theme.paragraph.label));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("esc".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("go back".to_string(), theme.paragraph.label));
            }
            PopupMode::Confirmation => {
                legend.push(Span::styled("enter".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled(
                    "sign and submit".to_string(),
                    theme.paragraph.label,
                ));
                legend.push(Span::raw("   "));
                legend.push(Span::styled("esc".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("go back".to_string(), theme.paragraph.label));
            }
            PopupMode::ChainSpecs | PopupMode::Metadata => {
                legend.push(Span::styled("esc".to_string(), theme.paragraph.base));
                legend.push(Span::raw(" "));
                legend.push(Span::styled("go back".to_string(), theme.paragraph.label));
            }
            _ => {}
        }
    } else if app.chains.is_active() || app.validators.is_active() {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("↑ ↓".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("select".to_string(), theme.paragraph.label));
        legend.push(Span::raw("   "));
        legend.push(Span::styled("tab or ← →".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("navigate".to_string(), theme.paragraph.label));
    } else {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("tab or ← →".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("navigate".to_string(), theme.paragraph.label));
    }

    // Always visible
    legend.push(Span::raw("   "));
    legend.push(Span::styled("ctrl+w".to_string(), theme.paragraph.base));
    legend.push(Span::raw(" "));
    legend.push(Span::styled(
        "switch window".to_string(),
        theme.paragraph.label,
    ));
    legend.push(Span::raw("   "));
    legend.push(Span::styled("ctrl+c".to_string(), theme.paragraph.base));
    legend.push(Span::raw(" "));
    legend.push(Span::styled("quit".to_string(), theme.paragraph.label));

    if let Some(v) = app.new_version.as_deref() {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("ctrl+u".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled(
            format!("update to {}", v),
            theme.paragraph.header,
        ));
    }

    let footer = Paragraph::new(Line::from(legend))
        .block(block)
        .right_aligned();

    frame.render_widget(footer, area);
}

fn render_logo_widget(_app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = CONFIG.theme();
    let block = Block::default()
        .style(theme.block.pane_body)
        .padding(Padding::new(2, 0, 1, 1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);
    let logo = Logo::inline();
    frame.render_widget(&logo, block_area);
}
