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
use suno_config::CONFIG;

#[derive(Debug, Clone)]
pub struct ValidatorsCompactWidget {
    pub state: Arc<RwLock<ValidatorsListState>>,
}

/// Validators compact view widget implementation, mostly to be used on the left menu
impl Widget for &ValidatorsCompactWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();

        let mut state = self.state.write().unwrap();

        let block = Block::new()
            .set_style(theme.block.pane_body(state.is_active))
            .padding(Padding::symmetric(0, 1));

        let proxies_available = state.proxies_available();

        let rows = state.validators_iter().map(|v| {
            let mut row = vec![
                Text::from(""),
                Text::from(format!("{}/{}", v.runtime(), v.display_name(4),)),
            ];

            if proxies_available {
                row.push(Text::from(v.proxies_as_str()).alignment(Alignment::Right));
            }

            row.push(Text::from(""));
            Row::new(row)
        });

        // Define widths
        let mut widths = vec![Constraint::Length(1), Constraint::Fill(1)];
        if proxies_available {
            widths.push(Constraint::Length(7));
        }
        widths.push(Constraint::Length(1));

        // Define header cells
        let mut header_cells = vec![
            Cell::from(""),
            Cell::from(Text::from("validators").alignment(Alignment::Left)),
        ];
        if proxies_available {
            header_cells.push(Cell::from(
                Text::from("proxies").alignment(Alignment::Right),
            ));
        }
        header_cells.push(Cell::from(""));

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(theme.table.header(state.is_active)))
            .style(theme.table.base)
            .row_highlight_style(theme.table.row_highlight(state.is_active))
            .highlight_symbol(theme.table.highlight_symbol(state.is_active));

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
