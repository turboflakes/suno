use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Styled},
    widgets::{Block, Cell, Row, StatefulWidget, Table, TableState, Widget},
};
use std::collections::VecDeque;
use suno_config::CONFIG;
use suno_primitives::display::format_date;
use suno_tracing::Log;
use suno_tracing::LogEntry;
use tokio::sync::mpsc;

pub struct LogsState {
    logs: Log,
    table_state: TableState,
}

impl LogsState {
    pub fn new(rx: mpsc::UnboundedReceiver<LogEntry>) -> Self {
        let config = CONFIG.clone();
        Self {
            logs: Log::new(rx, config.logs_max_entries()),
            table_state: TableState::default(),
        }
    }

    pub fn entries(&self) -> &VecDeque<LogEntry> {
        &self.logs.entries()
    }

    pub fn update(&mut self) {
        self.logs.update();

        let last = self.logs.total_entries().saturating_sub(1);
        self.table_state.select(Some(last));
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
        let theme = CONFIG.theme();

        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let header = Row::new([
            Cell::from("Date"),
            Cell::from("Level"),
            Cell::from("Message"),
        ]);

        let rows = state.entries().iter().map(|log| {
            let level_style = match log.level {
                tracing::Level::TRACE => Style::default().fg(Color::DarkGray),
                tracing::Level::DEBUG => Style::default().fg(Color::Blue),
                tracing::Level::INFO => Style::default().fg(Color::Green),
                tracing::Level::WARN => Style::default().fg(Color::Yellow),
                tracing::Level::ERROR => Style::default().fg(Color::Red),
            };

            Row::new([
                Cell::from(format_date(log.ts)),
                Cell::from(log.level.to_string()).style(level_style),
                Cell::from(log.message.clone()),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(20),
                Constraint::Length(6),
                Constraint::Min(10),
            ],
        )
        .header(header.set_style(theme.table.header));

        StatefulWidget::render(table, inner, buf, &mut state.table_state);
    }
}
