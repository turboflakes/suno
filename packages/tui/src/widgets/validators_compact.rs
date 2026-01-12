use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::validators::ValidatorsListState;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, StatefulWidget, Table, Widget},
};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct ValidatorsCompactWidget {
    pub state: Arc<RwLock<ValidatorsListState>>,
}

/// Validators compact view widget implementation, mostly to be used on the left menu
impl Widget for &ValidatorsCompactWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let (table_style, highlight_style, highlight_symbol) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
                "❯",
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Blue),
                "",
            ),
        };

        let block = Block::new()
            .title("Validators")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_type(BorderType::Plain);

        let rows = state.validators_iter();

        let widths = [
            Constraint::Fill(1),    // Network column
            Constraint::Length(14), // Stash column
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style)
            .highlight_symbol(highlight_symbol);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // Render scrollbar when active
        if state.is_active && state.validators.len() >= area.height.saturating_sub(2) as usize {
            let scrollbar_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
                ..area
            };
            if let Some(row_index) = state.table_state.selected() {
                render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
            }
        }
    }
}
