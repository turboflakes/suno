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

        let proxies_available = state.proxies_available();

        // In group view, interleave a non-selectable chain-name row before each group's collators.
        // The underlying `table_state` selection index refers to `state.collators`,
        // so it's translated here to account for the extra header rows.
        // In list view, collators are regrouped by relay chain.
        let mut indexed: Vec<(usize, _)> = state.collators_iter().enumerate().collect();
        if state.is_list_view() {
            indexed.sort_by_key(|(_, c)| c.runtime().relay_chain());
        }

        let selected = state.table_state.selected();
        let mut display_selected = None;
        let mut rows: Vec<Row> = Vec::new();
        let mut last_group = None;

        for (i, c) in indexed {
            let group_key = if state.is_list_view() {
                c.runtime().relay_chain()
            } else {
                c.runtime()
            };

            if last_group != Some(group_key) {
                let mut row = vec![
                    Text::from(""),
                    Text::from(group_key.to_string()).style(theme.paragraph.label_italic),
                ];

                if proxies_available {
                    row.push(Text::from(""));
                }

                row.push(Text::from(""));
                rows.push(Row::new(row));

                last_group = Some(group_key);
            }

            if selected == Some(i) {
                display_selected = Some(rows.len());
            }

            let chain_label = if state.is_list_view() {
                format!("{}/", c.runtime().as_str_short())
            } else {
                String::new()
            };

            let mut row = vec![
                Text::from(""),
                Text::from(format!("{}{}", chain_label, c.display_name(4))),
            ];

            if proxies_available {
                row.push(Text::from(c.proxies_as_str()).alignment(Alignment::Right));
            }

            row.push(Text::from(""));
            rows.push(Row::new(row));
        }

        // Define widths
        let mut widths = vec![Constraint::Length(1), Constraint::Fill(1)];

        if proxies_available {
            widths.push(Constraint::Length(7));
        }
        widths.push(Constraint::Length(1));

        // Define header cells
        let mut header_cells = vec![
            Cell::from(""),
            Cell::from(Text::from("collators").alignment(Alignment::Left)),
        ];

        if proxies_available {
            header_cells.push(Cell::from(
                Text::from("proxies").alignment(Alignment::Right),
            ));
        }

        header_cells.push(Cell::from(""));

        let rows_len = rows.len();
        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(theme.table.header(state.is_active())))
            .style(theme.table.base)
            .row_highlight_style(theme.table.row_highlight(state.is_active()))
            .highlight_symbol(theme.table.highlight_symbol(state.is_active()));

        let mut display_table_state = state.table_state;
        display_table_state.select(display_selected);

        StatefulWidget::render(table, area, buf, &mut display_table_state);

        // Render scrollbar when active
        if state.is_active() && rows_len >= area.height.saturating_sub(2) as usize {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };
            if let Some(row_index) = display_selected {
                render_scrollbar(theme, row_index, rows_len, scrollbar_area, buf);
            }
        }
    }
}
