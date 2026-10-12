use crate::app::App;
use crate::section::Section;
use crate::widgets::collators::CollatorsView;
use crate::widgets::collators_compact::CollatorsCompactWidget;
use crate::widgets::collators_detailed_group::CollatorsDetailedGroupWidget;
use crate::widgets::collators_detailed_list::CollatorsDetailedListWidget;
use crate::widgets::logs::LogsWidget;
use crate::widgets::validators_compact::ValidatorsCompactWidget;
use crate::widgets::validators_detailed_group::ValidatorsDetailedGroupWidget;
use crate::widgets::{logo::Logo, popup::Mode as PopupMode, window::render_help, window::Window};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    prelude::Margin,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use strum::IntoEnumIterator;
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

    let validators_enabled = config.features.validators_enabled();
    let collators_enabled = config.features.collators_enabled();

    let validators_constraint = match (validators_enabled, collators_enabled) {
        (true, true) => Constraint::Length(app.validators.total_compact_height()),
        (true, false) => Constraint::Fill(1),
        (false, _) => Constraint::Length(0),
    };
    let collators_constraint = if collators_enabled {
        Constraint::Fill(1)
    } else {
        Constraint::Length(0)
    };

    let constraints = vec![
        Constraint::Length(3 + config.chains.len() as u16),
        validators_constraint,
        collators_constraint,
    ];

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

    render_window(app, frame, outer_layout[1]);

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

fn render_window(app: &mut App, frame: &mut Frame, area: Rect) {
    let [tabs_area, body_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
    render_window_tabs(app, frame, tabs_area);
    // Switch between main body window.
    match app.window {
        Window::Validators | Window::Collators => render_body_widget(app, frame, body_area),
        Window::Logs => render_logs_widget(app, frame, body_area),
        Window::Help => {
            render_help(app.theme, body_area, frame.buffer_mut());
        }
    }
}

fn render_window_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let is_selected = app.section == Section::Main;

    let mut spans = Vec::new();
    for window in Window::iter().filter(|w| w.is_enabled(&CONFIG.features())) {
        let style = if window == app.window {
            let style = theme.paragraph.header(is_selected);
            if is_selected {
                style.add_modifier(Modifier::REVERSED)
            } else {
                style
            }
        } else {
            theme.paragraph.label.add_modifier(Modifier::DIM)
        };
        spans.push(Span::raw(format!(" {} ", window)).style(style));
        spans.push(Span::raw(" "));
    }

    let block = Block::default()
        .style(theme.block.main)
        .padding(Padding::new(2, 2, 1, 0));
    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn render_popup(app: &mut App, frame: &mut Frame, area: Rect) {
    let area = match app.popup.get_mode() {
        PopupMode::Message => flex_area(
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
    frame.render_widget(&mut app.popup, area);
    // Apply the cursor if it was set during render
    if let Some(pos) = app.popup.get_input_cursor_position() {
        frame.set_cursor_position(pos);
    }
}

fn flex_area(area: Rect, horizontal: Constraint, vertical: Constraint, flex: Flex) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(flex).areas(area);
    let [area] = Layout::vertical([vertical]).flex(flex).areas(area);
    area
}

fn render_chains_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_widget(&mut app.chains, area);
}

fn render_validators_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_stateful_widget(
        ValidatorsCompactWidget::new(app.theme),
        area,
        &mut app.validators,
    );
}

fn render_collators_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_stateful_widget(
        CollatorsCompactWidget::new(app.theme),
        area,
        &mut app.collators,
    );
}

fn render_body_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .style(theme.block.main)
        .padding(Padding::proportional(1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);

    match app.window {
        Window::Collators => match app.collators.view() {
            CollatorsView::Group => {
                let widget = CollatorsDetailedGroupWidget::new(&app.chains, theme)
                    .with_masked(app.collators.is_masked());
                frame.render_stateful_widget(widget, block_area, &mut app.collators);
            }
            CollatorsView::List => {
                let widget = CollatorsDetailedListWidget::new(&app.chains, theme)
                    .with_masked(app.collators.is_masked());
                frame.render_stateful_widget(widget, block_area, &mut app.collators);
            }
        },
        _ => {
            let widget = ValidatorsDetailedGroupWidget::new(&app.chains, theme);
            frame.render_stateful_widget(widget, block_area, &mut app.validators);
        }
    }
}

fn render_logs_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .style(theme.block.main)
        .padding(Padding::proportional(1));
    let logs = LogsWidget::new().block(block);
    frame.render_stateful_widget(logs, area, &mut app.logs);
}

fn render_legend_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .style(theme.block.footer_right)
        .padding(Padding::new(0, 2, 1, 1));

    let mut legend = vec![];

    // show how to open popup with available commands depending on the active section
    if ((app.validators.is_active()
        && (app.validators.is_proxy_valid() || app.validators.has_commands_available()))
        || (app.collators.is_active()
            && (app.collators.is_proxy_valid() || app.collators.has_commands_available()))
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
            PopupMode::ThemeMenu => {
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
                if app.popup.get_confirmed_theme().is_some() {
                    legend.push(Span::raw("   "));
                    legend.push(Span::styled("enter".to_string(), theme.paragraph.base));
                    legend.push(Span::raw(" "));
                    legend.push(Span::styled("confirm".to_string(), theme.paragraph.label));
                }
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
    } else if app.chains.is_active() || app.validators.is_active() || app.collators.is_active() {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("↑ ↓".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("select".to_string(), theme.paragraph.label));
        legend.push(Span::raw("   "));
        if app.collators.is_active() {
            legend.push(Span::styled("ctrl+v".to_string(), theme.paragraph.base));
            legend.push(Span::raw(" "));
            legend.push(Span::styled(
                "change view".to_string(),
                theme.paragraph.label,
            ));
            legend.push(Span::raw("   "));
        }
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
    if app.popup.is_hidden() && app.popup.get_mode() != PopupMode::ThemeMenu {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("ctrl+t".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("themes".to_string(), theme.paragraph.label));
    }
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

fn render_logo_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .style(theme.block.pane_body)
        .padding(Padding::new(2, 0, 1, 1));
    let block_area = block.inner(area);
    frame.render_widget(block, area);
    let logo = Logo::inline().theme(theme);
    frame.render_widget(&logo, block_area);
}
