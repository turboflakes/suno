use crate::app::{App, Focus};
use crate::section::Section;
use crate::widgets::collators::CollatorsView;
use crate::widgets::collators_compact::{
    compact_height as collators_compact_height, CollatorsCompactWidget,
};
use crate::widgets::collators_detailed_group::CollatorsDetailedGroupWidget;
use crate::widgets::collators_detailed_list::CollatorsDetailedListWidget;
use crate::widgets::logs::LogsWidget;
use crate::widgets::nominators_compact::{
    compact_height as nominators_compact_height, NominatorsCompactWidget,
};
use crate::widgets::proxies_compact::{
    compact_height as proxies_compact_height, ProxiesCompactWidget,
};
use crate::widgets::validators_compact::{
    compact_height as validators_compact_height, ValidatorsCompactWidget,
};
use crate::widgets::validators_detailed_group::ValidatorsDetailedGroupWidget;
use crate::widgets::{help::HelpWidget, logo::Logo, popup::Mode as PopupMode, window::Window};
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

    let constraints = left_menu_constraints(app);

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

    if config.features.nominators_enabled() {
        render_nominators_widget(app, frame, left_layout[3]);
    }

    if config.features.proxies_enabled() {
        render_proxies_widget(app, frame, left_layout[4]);
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

/// The heights of the left menu: the chains, then the panes from top to bottom, each with the
/// height its rows need. The ones not enabled take no space, and the expanded one takes the
/// space left.
fn left_menu_constraints(app: &App) -> Vec<Constraint> {
    let config = CONFIG.clone();
    let features = &config.features;
    let panes = [
        (
            Section::Validators,
            features.validators_enabled(),
            validators_compact_height(&app.validators),
        ),
        (
            Section::Collators,
            features.collators_enabled(),
            collators_compact_height(&app.collators),
        ),
        (
            Section::Nominators,
            features.nominators_enabled(),
            nominators_compact_height(&app.nominators),
        ),
        (
            Section::Proxies,
            features.proxies_enabled(),
            proxies_compact_height(&app.proxies),
        ),
    ];

    // The expanded pane falls back to the first enabled one
    let expanded = app.expanded_section();
    let expanded = if panes
        .iter()
        .any(|(section, enabled, _)| *enabled && *section == expanded)
    {
        expanded
    } else {
        panes
            .iter()
            .find(|(_, enabled, _)| *enabled)
            .map(|(section, _, _)| section.clone())
            .unwrap_or_default()
    };

    let mut constraints = vec![Constraint::Length(3 + config.chains.len() as u16)];
    constraints.extend(panes.iter().map(|(section, enabled, height)| {
        match (enabled, *section == expanded) {
            (false, _) => Constraint::Length(0),
            (true, true) => Constraint::Fill(1),
            (true, false) => Constraint::Length(*height),
        }
    }));
    constraints
}

fn render_window(app: &mut App, frame: &mut Frame, area: Rect) {
    let is_filter_visible = app.focus == Focus::Filter || app.is_filtered();
    let (tabs_area, filter_area, body_area) = if is_filter_visible {
        let [tabs_area, filter_area, body_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .areas(area);
        (tabs_area, Some(filter_area), body_area)
    } else {
        let [tabs_area, body_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
        (tabs_area, None, body_area)
    };

    render_window_tabs(app, frame, tabs_area);
    if let Some(filter_area) = filter_area {
        render_filter_widget(app, frame, filter_area);
    }
    // Switch between main body window.
    match app.window {
        Window::Validators | Window::Collators => render_body_widget(app, frame, body_area),
        Window::Logs => render_logs_widget(app, frame, body_area),
        Window::Help => render_help_widget(app, frame, body_area),
    }
}

fn render_filter_widget(app: &App, frame: &mut Frame, area: Rect) {
    let (input, matches) = match app.window {
        Window::Validators => (app.validators.filter(), app.validators.count_matches()),
        Window::Collators => (app.collators.filter(), app.collators.count_matches()),
        _ => return,
    };

    frame.render_widget(&input.as_inline_filter(app.theme, matches), area);
    if app.focus == Focus::Filter {
        if let Some(position) = input.get_cursor_position() {
            frame.set_cursor_position(position);
        }
    }
}

fn render_window_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let is_selected = app.section.is_none();

    let mut spans = Vec::new();
    for window in Window::iter().filter(|w| w.is_enabled(CONFIG.features())) {
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

fn render_nominators_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_stateful_widget(
        NominatorsCompactWidget::new(app.theme),
        area,
        &mut app.nominators,
    );
}

fn render_proxies_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    frame.render_stateful_widget(ProxiesCompactWidget::new(app.theme), area, &mut app.proxies);
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

fn render_help_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .style(theme.block.main)
        .padding(Padding::proportional(1));
    let help = HelpWidget::new().block(block);
    frame.render_stateful_widget(help, area, &mut app.help);
}

fn render_legend_widget(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let block = Block::default()
        .style(theme.block.footer_right)
        .padding(Padding::new(0, 2, 1, 1));

    let mut legend = vec![];

    // The keys that complete the chord started with `ctrl+x`
    if app.focus == Focus::Chord {
        let features = CONFIG.features();
        let mut keys = Vec::new();
        if features.validators_enabled() {
            keys.push(("v", "validator"));
        }
        if features.nominators_enabled() {
            keys.push(("n", "nominator"));
        }
        if features.collators_enabled() {
            keys.push(("c", "collator"));
        }
        keys.push(("p", "proxy"));
        keys.push(("esc", "cancel"));
        for (i, (key, label)) in keys.into_iter().enumerate() {
            if i > 0 {
                legend.push(Span::raw("   "));
            }
            legend.push(Span::styled(key.to_string(), theme.paragraph.base));
            legend.push(Span::raw(" "));
            legend.push(Span::styled(label.to_string(), theme.paragraph.label));
        }
        frame.render_widget(
            Paragraph::new(Line::from(legend))
                .block(block)
                .right_aligned(),
            area,
        );
        return;
    }

    // Esc clears the selection without changing the section, so ask the lists, not the section
    let has_selection = app.chains.is_active()
        || app.proxies.is_active()
        || app.nominators.is_active()
        || app.validators.is_active()
        || app.collators.is_active()
        || (app.section.is_none()
            && (app.window.is_logs_selected() || app.window.is_help_selected()));

    // show how to open popup with available commands depending on the active section, or the
    // ones of the first chain when nothing is selected
    if (app.focus == Focus::None
        || (app.validators.is_active()
            && (app.validators.is_proxy_valid()
                || app.validators.has_commands_available()
                || app
                    .validators
                    .get_selected_ref()
                    .is_some_and(|v| v.origins.is_pinned())))
        || (app.collators.is_active()
            && (app.collators.is_proxy_valid()
                || app.collators.has_commands_available()
                || app
                    .collators
                    .get_selected_ref()
                    .is_some_and(|c| c.origin.is_pinned())))
        || (app.nominators.is_active()
            && app
                .nominators
                .get_selected()
                .is_some_and(|n| n.origin.is_pinned()))
        || app.chains.is_active()
        || (app.proxies.is_active()
            && app.proxies.get_selected().is_some_and(|p| p.is_removable())))
        && !app.popup.is_visible()
        && !app.window.is_logs_selected()
        && !app.window.is_help_selected()
    {
        legend.push(Span::styled("/".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("commands".to_string(), theme.paragraph.label));
    };

    if app.popup.is_visible() {
        // The popups show their keybindings in the popup header
    } else if app.focus == Focus::Filter {
        legend.push(Span::raw("   "));
        legend.push(Span::styled(
            "↑ ↓ or enter".to_string(),
            theme.paragraph.base,
        ));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("select".to_string(), theme.paragraph.label));
        legend.push(Span::raw("   "));
        legend.push(Span::styled("esc".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("clear".to_string(), theme.paragraph.label));
    } else if has_selection {
        legend.push(Span::raw("   "));
        if let Some(hint) = app.toggle_hint() {
            legend.push(Span::styled("space".to_string(), theme.paragraph.base));
            legend.push(Span::raw(" "));
            legend.push(Span::styled(hint.to_string(), theme.paragraph.label));
            legend.push(Span::raw("   "));
        }
        let has_items = (app.validators.is_active() || app.collators.is_active())
            && (app.window.is_validators_selected() || app.window.is_collators_selected())
            && app.can_filter();
        if has_items {
            legend.push(Span::styled("↑ ↓".to_string(), theme.paragraph.base));
            legend.push(Span::raw(" "));
            let up_down = if app.window.is_help_selected() {
                "scroll"
            } else {
                "select"
            };
            legend.push(Span::styled(up_down.to_string(), theme.paragraph.label));
            legend.push(Span::raw("   "));
        }
        if has_items && app.collators.is_active() && app.window.is_collators_selected() {
            legend.push(Span::styled("ctrl+v".to_string(), theme.paragraph.base));
            legend.push(Span::raw(" "));
            legend.push(Span::styled(
                "switch view".to_string(),
                theme.paragraph.label,
            ));
            legend.push(Span::raw("   "));
        }
        if app.section.is_none() && app.window.is_logs_selected() {
            legend.push(Span::styled("shift+↑/↓".to_string(), theme.paragraph.base));
            legend.push(Span::raw(" "));
            legend.push(Span::styled(
                "select multiple".to_string(),
                theme.paragraph.label,
            ));
            legend.push(Span::raw("   "));
            legend.push(Span::styled(
                "shift+ctrl+c".to_string(),
                theme.paragraph.base,
            ));
            legend.push(Span::raw(" "));
            legend.push(Span::styled(
                "copy to clipboard".to_string(),
                theme.paragraph.label,
            ));
            legend.push(Span::raw("   "));
        }
    }

    // The hints above are followed by their separator, so drop the last one
    if legend.last().is_some_and(|span| span.content == "   ") {
        legend.pop();
    }

    let can_filter = app.can_filter();
    if app.popup.is_hidden() && app.focus != Focus::Filter && can_filter {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("ctrl+f".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("filter".to_string(), theme.paragraph.label));
    }
    if app.popup.is_hidden() && app.popup.get_mode() != PopupMode::ThemeMenu {
        legend.push(Span::raw("   "));
        legend.push(Span::styled("ctrl+t".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("themes".to_string(), theme.paragraph.label));
        legend.push(Span::raw("   "));
        legend.push(Span::styled("ctrl+o".to_string(), theme.paragraph.base));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("options".to_string(), theme.paragraph.label));
    }
    // The navigation hints go together, `tab` between the sections and `ctrl+w` between the windows
    if app.popup.is_hidden() && app.focus != Focus::Filter {
        legend.push(Span::raw("   "));
        legend.push(Span::styled(
            "tab or ctrl+w or ← →".to_string(),
            theme.paragraph.base,
        ));
        legend.push(Span::raw(" "));
        legend.push(Span::styled("navigate".to_string(), theme.paragraph.label));
    }

    // Always visible
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
