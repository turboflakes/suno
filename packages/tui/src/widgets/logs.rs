use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Styled},
    widgets::{Block, Cell, Row, StatefulWidget, Table, TableState, Widget},
};
use std::collections::VecDeque;
use suno_config::CONFIG;
use suno_primitives::display::format_date;
use suno_theme::Theme;
use suno_tracing::Log;
use suno_tracing::LogEntry;
use tokio::sync::mpsc;

pub struct LogsState {
    logs: Log,
    table_state: TableState,
    theme: Theme,
    /// Keeps the last entry selected, so the newest logs stay in view, until the user
    /// scrolls up.
    follow: bool,
    /// Row the selection was extended from with `Shift+Up/Down`, the rows between it and
    /// the selected row are all highlighted.
    anchor: Option<usize>,
}

impl LogsState {
    pub fn new(rx: mpsc::UnboundedReceiver<LogEntry>) -> Self {
        let config = CONFIG.clone();
        Self {
            logs: Log::new(rx, config.logs_max_entries()),
            table_state: TableState::default(),
            theme: Theme::default(),
            follow: true,
            anchor: None,
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn entries(&self) -> &VecDeque<LogEntry> {
        self.logs.entries()
    }

    pub fn update(&mut self) {
        self.logs.update();

        let last = self.logs.total_entries().saturating_sub(1);
        if self.follow {
            self.table_state.select(Some(last));
        } else if let Some(selected) = self.table_state.selected() {
            // Entries are capped, so the selection can end up past the last one
            self.table_state.select(Some(selected.min(last)));
        }
    }

    /// Selects the previous entry, and stops following the newest logs.
    pub fn move_up(&mut self) {
        let last = self.logs.total_entries().saturating_sub(1);
        let selected = self.table_state.selected().unwrap_or(last);
        self.table_state.select(Some(selected.saturating_sub(1)));
        self.follow = false;
        self.anchor = None;
    }

    /// Selects the next entry, and follows the newest logs again once the last one is reached.
    pub fn move_down(&mut self) {
        let last = self.logs.total_entries().saturating_sub(1);
        let selected = self.table_state.selected().unwrap_or(last);
        let next = (selected + 1).min(last);
        self.table_state.select(Some(next));
        self.follow = next == last;
        self.anchor = None;
    }

    /// Extends the selection one entry up, keeping the rows in between highlighted.
    pub fn extend_up(&mut self) {
        let last = self.logs.total_entries().saturating_sub(1);
        let selected = self.table_state.selected().unwrap_or(last);
        self.anchor.get_or_insert(selected);
        self.table_state.select(Some(selected.saturating_sub(1)));
        self.follow = false;
    }

    /// Extends the selection one entry down, keeping the rows in between highlighted.
    pub fn extend_down(&mut self) {
        let last = self.logs.total_entries().saturating_sub(1);
        let selected = self.table_state.selected().unwrap_or(last);
        self.anchor.get_or_insert(selected);
        self.table_state.select(Some((selected + 1).min(last)));
        self.follow = false;
    }

    /// Drops the extended selection.
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Text of the selected entries, one per line, or of the entry under the cursor when the
    /// selection was not extended.
    pub fn selected_text(&self) -> Option<String> {
        let selected = self.table_state.selected()?;
        let range = self.selected_range().unwrap_or(selected..=selected);
        let lines: Vec<String> = self
            .entries()
            .iter()
            .enumerate()
            .filter(|(i, _)| range.contains(i))
            .map(|(_, log)| format!("{} {} {}", format_date(log.ts), log.level, log.message))
            .collect();

        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }

    /// Rows highlighted by an extended selection, if any.
    fn selected_range(&self) -> Option<std::ops::RangeInclusive<usize>> {
        let anchor = self.anchor?;
        let selected = self.table_state.selected()?;
        Some(anchor.min(selected)..=anchor.max(selected))
    }
}

pub struct LogsWidget<'a> {
    block: Option<Block<'a>>,
}

impl<'a> LogsWidget<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for LogsWidget<'_> {
    type State = LogsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = state.theme;

        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        // The first row of the table is its header
        let visible_rows = inner.height.saturating_sub(1) as usize;
        let total_entries = state.entries().len();
        let is_scroll_visible = total_entries > visible_rows;

        // Leave room for the scrollbar on the right when the logs don't fit
        let table_area = if is_scroll_visible {
            Rect {
                width: inner.width.saturating_sub(1),
                ..inner
            }
        } else {
            inner
        };

        let header = Row::new([
            Cell::from("date"),
            Cell::from("level"),
            Cell::from("message"),
        ]);

        let selected_range = state.selected_range();
        let rows = state.entries().iter().enumerate().map(|(i, log)| {
            let level_style = match log.level {
                tracing::Level::TRACE => Style::default().fg(Color::DarkGray),
                tracing::Level::DEBUG => Style::default().fg(Color::Blue),
                tracing::Level::INFO => Style::default().fg(Color::Green),
                tracing::Level::WARN => Style::default().fg(Color::Yellow),
                tracing::Level::ERROR => Style::default().fg(Color::Red),
            };

            let row = Row::new([
                Cell::from(format_date(log.ts)),
                Cell::from(log.level.to_string()).style(level_style),
                Cell::from(log.message.clone()),
            ]);

            match &selected_range {
                Some(range) if range.contains(&i) => row.style(theme.table.row_highlight(true)),
                _ => row,
            }
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(20),
                Constraint::Length(6),
                Constraint::Min(10),
            ],
        )
        .header(header.set_style(theme.table.header))
        .row_highlight_style(theme.table.row_highlight(!state.follow));

        StatefulWidget::render(table, table_area, buf, &mut state.table_state);

        if is_scroll_visible {
            let scrollbar_area = Rect {
                x: inner.right().saturating_sub(1),
                y: inner.y + 1,
                width: 1,
                height: inner.height.saturating_sub(1),
            };
            let position = state.table_state.selected().unwrap_or_default();
            render_scrollbar(theme, position, total_entries, scrollbar_area, buf);
        }
    }
}
