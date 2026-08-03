use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};
use suno_config::{NodeConfig, CONFIG};
use suno_primitives::Collator;
use tracing::warn;

#[derive(Debug, Clone, Default)]
pub struct CollatorsListWidget {
    state: Arc<RwLock<CollatorsListState>>,
}

#[derive(Debug, Default)]
pub struct CollatorsListState {
    collators: Vec<Collator>,
    table_state: TableState,
    is_active: bool,
}

impl CollatorsListWidget {
    pub fn on_init(&self) {
        let mut state = self.state.write().unwrap();
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                for collator in &chain_config.collators {
                    match collator {
                        NodeConfig::Address(stash) => {
                            state.collators.push(Collator::new(*chain_name, *stash));
                        }
                        NodeConfig::Detailed { stash, .. } => {
                            state.collators.push(Collator::new(*chain_name, *stash));
                            // if let Some(cmds) = commands {
                            //     for cmd in cmds {
                            //         println!("  Command: {} ({})", cmd.name, cmd.run);
                            //     }
                            // }
                        }
                    }
                }
            }
        }
        // Select the first validator.
        if !state.collators.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn _on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    pub fn move_down(&self) -> Option<Collator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.collators.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.collators[i].clone())
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Collator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.collators.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.collators[i].clone())
        } else {
            None
        }
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Collator> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .map(|i| state.collators[i].clone())
    }
}

impl Widget for &CollatorsListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let (table_style, highlight_style) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Blue),
            ),
        };

        let block = Block::new()
            .title("Collators")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_type(BorderType::Plain);

        let rows = state.collators.iter();
        let widths = [Constraint::Fill(1), Constraint::Length(14)];
        let table = Table::new(rows, widths)
            .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        if state.is_active {
            // Render scrollbar.
            let scrollbar_area = Rect {
                y: area.y + 1,
                height: area.height - 2,
                ..area
            };
            if let Some(row_index) = state.table_state.selected() {
                render_scrollbar(row_index, state.collators.len(), scrollbar_area, buf);
            }
        }
    }
}
