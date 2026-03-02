use crate::theme::THEME;
use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::validators::ValidatorsListState;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::Styled,
    text::Text,
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table, Widget},
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

        let block = Block::new()
            .set_style(THEME.block.menu_bottom(state.is_active))
            .padding(Padding::symmetric(0, 1));

        let rows = state.validators_iter();

        let widths = [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ];

        let header_cells = vec![
            Cell::from(""),
            Cell::from(Text::from("validators").alignment(Alignment::Left)),
            Cell::from(""),
            Cell::from(""),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(THEME.table.header(state.is_active)))
            .style(THEME.table.base)
            .row_highlight_style(THEME.table.row_highlight(state.is_active))
            .highlight_symbol(THEME.table.highlight_symbol(state.is_active));

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // Render scrollbar when active
        if state.is_active && state.validators.len() >= area.height.saturating_sub(2) as usize {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };
            if let Some(row_index) = state.table_state.selected() {
                render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
            }
        }
    }
}
