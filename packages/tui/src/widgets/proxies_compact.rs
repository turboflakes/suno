use crate::widgets::proxies::ProxiesList;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Styled},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table},
};
use suno_theme::Theme;

/// The most rows displayed at once, the list scrolls past them.
const MAX_ROWS: usize = 8;

/// The height the compact list needs: the header, the proxy accounts, and the padding.
pub fn compact_height(proxies: &ProxiesList) -> u16 {
    proxies.proxies_iter().count().clamp(1, MAX_ROWS) as u16 + 3
}

#[derive(Debug, Default)]
pub struct ProxiesCompactWidget {
    theme: Theme,
}

impl ProxiesCompactWidget {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}

/// Proxies compact view widget implementation, to be used on the left menu.
impl StatefulWidget for ProxiesCompactWidget {
    type State = ProxiesList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = self.theme;
        let is_active = state.is_active();

        let block = Block::new()
            .set_style(theme.block.pane_body(is_active))
            .padding(Padding::symmetric(0, 1));

        let proxies_available = !state.is_empty();
        let selected = state.table_state.selected();
        let mut rows: Vec<Row> = state
            .proxies_iter()
            .enumerate()
            .map(|(i, proxy)| {
                // Show the origin alongside the chain and address only when active and selected
                let mut name = Vec::new();
                if is_active && selected == Some(i) {
                    name.push(Span::styled(
                        format!("{} ", proxy.origin.symbol()),
                        theme.paragraph.label(is_active),
                    ));
                }
                name.push(Span::raw(format!(
                    "{}/{}",
                    proxy.runtime.as_str_short(),
                    proxy.compact_address(4)
                )));
                Row::new(vec![
                    Text::from(""),
                    Text::from(Line::from(name)),
                    Text::from(proxy.usage.short()).alignment(Alignment::Right),
                    Text::from(""),
                ])
            })
            .collect();

        if rows.is_empty() {
            rows.push(Row::new(vec![
                Text::from(""),
                Text::from(Line::from(vec![
                    Span::styled("-- ", theme.paragraph.label_italic),
                    Span::styled("ctrl+x p", theme.paragraph.base_italic),
                    Span::raw(" "),
                    Span::styled("add a proxy", theme.paragraph.label_italic),
                    Span::styled(" --", theme.paragraph.label_italic),
                ])),
            ]));
        }

        let mut widths = vec![
            Constraint::Length(1),
            Constraint::Fill(1),
        ];
        if proxies_available {
            widths.push(Constraint::Length(10));
        }
        widths.push(Constraint::Length(1));

        let title = "proxies";
        let title_span = if is_active {
            Span::raw(format!(" {} ", title)).style(
                theme
                    .paragraph
                    .header(true)
                    .add_modifier(Modifier::REVERSED),
            )
        } else {
            Span::raw(format!("{} ", title))
        };

        let mut header_cells = vec![Cell::from(""), Cell::from(Line::from(title_span))];
        if proxies_available {
            header_cells.push(Cell::from(
                Text::from("used by").alignment(Alignment::Right),
            ));
        }
        header_cells.push(Cell::from(""));

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(theme.table.header(is_active)))
            .style(theme.table.base)
            .row_highlight_style(theme.table.row_highlight(is_active))
            .highlight_symbol(theme.table.highlight_symbol(is_active));

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}
