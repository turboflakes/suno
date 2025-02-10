use crate::app::Action;
use crate::config::CONFIG;
use crate::menu::Entry;
use log::{info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct ValidatorsPopupWidget {
    state: Arc<RwLock<ListState>>,
}

#[derive(Debug, Default)]
struct ListState {
    options: Vec<Entry>,
    table_state: TableState,
    is_active: bool,
}

impl ValidatorsPopupWidget {
    pub fn on_init(&self) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        state
            .options
            .push(Entry::new('c', "chill validator".to_string()));
        state
            .options
            .push(Entry::new('b', "bond more funds".to_string()));
        state
            .options
            .push(Entry::new('r', "change reward destination".to_string()));
        state
            .options
            .push(Entry::new('f', "change commission".to_string()));
        state
            .options
            .push(Entry::new('k', "kick nominators".to_string()));
        state
            .options
            .push(Entry::new('s', "change session keys".to_string()));

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    pub fn move_down(&self) -> Option<Entry> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.options.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.options[i].clone())
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Entry> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.options.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.options[i].clone())
        } else {
            None
        }
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Entry> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .map(|i| state.options[i].clone())
    }
}

impl Widget for &ValidatorsPopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        if !state.is_active {
            return; // Do not render if popup is not active.
        }

        let (table_style, highlight_style) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::White),
            ),
        };

        let block = Block::new()
            .title(" Menu ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let rows = state.options.iter();
        let widths = [Constraint::Length(6), Constraint::Fill(1)];
        let table = Table::new(rows, widths)
            .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // if state.is_active {
        //     // Render scrollbar.
        //     let scrollbar_area = Rect {
        //         y: area.y + 1,
        //         height: area.height - 2,
        //         ..area
        //     };
        //     let row_index = state.table_state.selected().unwrap();
        //     render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
        // }
    }
}

impl From<&Entry> for Row<'_> {
    fn from(o: &Entry) -> Self {
        let o = o.clone();
        Row::new(vec![o.key().to_string(), o.description().to_string()])
    }
}
