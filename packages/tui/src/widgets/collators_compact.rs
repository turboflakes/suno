use crate::widgets::collators::CollatorsList;
use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::Styled,
    text::Text,
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table},
};
use suno_theme::Theme;

#[derive(Debug, Default)]
pub struct CollatorsCompactWidget {
    theme: Theme,
}

impl CollatorsCompactWidget {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}

/// Collators compact view widget implementation, mostly to be used on the left menu
impl StatefulWidget for CollatorsCompactWidget {
    type State = CollatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = self.theme;

        let block = Block::new()
            .set_style(theme.block.pane_body(state.is_active()))
            .padding(Padding::symmetric(0, 1));

        let rows = state.collators_iter().map(|c| {
            Row::new(vec![
                Text::from(""),
                Text::from(format!("{}/{}", c.runtime(), c.display_name(4))),
                Text::from(""),
                Text::from(""),
            ])
        });

        let widths = [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(7),
            Constraint::Length(1),
        ];

        let header_cells = vec![
            Cell::from(""),
            Cell::from(Text::from("collators").alignment(Alignment::Left)),
            Cell::from(""),
            Cell::from(""),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(theme.table.header(state.is_active())))
            .style(theme.table.base)
            .row_highlight_style(theme.table.row_highlight(state.is_active()))
            .highlight_symbol(theme.table.highlight_symbol(state.is_active()));

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // Render scrollbar when active
        if state.is_active() && state.collators.len() >= area.height.saturating_sub(2) as usize {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };
            if let Some(row_index) = state.table_state.selected() {
                render_scrollbar(theme, row_index, state.collators.len(), scrollbar_area, buf);
            }
        }
    }
}
