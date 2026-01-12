use crate::theme::THEME;
use crate::widgets::validators::ValidatorsListState;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Styled},
    text::Text,
    widgets::{Block, BorderType, Borders, Cell, Row, StatefulWidget, Table, Widget},
};
use std::sync::{Arc, RwLock};
use suno_primitives::display::format_planks;

#[derive(Debug, Clone)]
pub struct ValidatorsDetailedListWidget {
    pub state: Arc<RwLock<ValidatorsListState>>,
}

/// Validators detailed list view widget implementation, to be used as an alternative of the grouped view
impl Widget for &ValidatorsDetailedListWidget {
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
            .borders(Borders::NONE)
            .border_type(BorderType::Plain);

        let rows = state.validators_iter().map(|v| {
            let points = match v.delta_points() {
                Some(d) => format!("+{} {}", d, v.total_points()),
                None => v.total_points().to_string(),
            };

            let decimals = v.account.token_decimals();
            Row::new(vec![
                Text::from(v.display_name()).alignment(Alignment::Left),
                Text::from(points).alignment(Alignment::Right),
                Text::from(format_planks(v.stake.total(), decimals, 4)).alignment(Alignment::Right),
                Text::from(format_planks(v.stake.own(), decimals, 4)).alignment(Alignment::Right),
                Text::from(v.stake.nominators_count().to_string()).alignment(Alignment::Right),
                Text::from(v.commission_as_percentage(2)).alignment(Alignment::Right),
            ])
        });

        let widths = [
            Constraint::Length(20),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .header(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(Text::from("points").alignment(Alignment::Right)),
                    Cell::from(Text::from("total").alignment(Alignment::Right)),
                    Cell::from(Text::from("own").alignment(Alignment::Right)),
                    Cell::from(Text::from("nominators").alignment(Alignment::Right)),
                    Cell::from(Text::from("commission").alignment(Alignment::Right)),
                ])
                .set_style(THEME.table.header),
            )
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}
