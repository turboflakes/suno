use crate::widgets::nominators::NominatorsList;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Styled},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table},
};
use sp_arithmetic::traits::Zero;
use suno_primitives::AccountDisplay;
use suno_theme::Theme;

/// The most rows displayed at once, the list scrolls past them.
const MAX_ROWS: usize = 8;

/// The height the compact list needs: the header, the chain names, the nominators, and the padding.
pub fn compact_height(nominators: &NominatorsList) -> u16 {
    let chains = nominators
        .nominators_iter()
        .map(|n| n.runtime().relay_chain())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let rows = nominators.nominators_iter().count() + chains;
    rows.clamp(1, MAX_ROWS) as u16 + 3
}

#[derive(Debug, Default)]
pub struct NominatorsCompactWidget {
    theme: Theme,
}

impl NominatorsCompactWidget {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}

/// Nominators compact view widget implementation, to be used on the left menu.
impl StatefulWidget for NominatorsCompactWidget {
    type State = NominatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let theme = self.theme;
        let is_active = state.is_active();

        let block = Block::new()
            .set_style(theme.block.pane_body(is_active))
            .padding(Padding::symmetric(0, 1));

        // The targets column is only needed when there are nominators listed
        let has_nominators = !state.is_empty();
        let proxies_available = state.proxies_available();

        // Interleave a non-selectable chain-name row before the nominators of each chain, named
        // after the relay chain since the nominators are listed under its asset hub.
        // The underlying `table_state` selection index refers to the nominators listed,
        // so it's translated here to account for the extra header rows.
        let selected = state.table_state.selected();
        let mut display_selected = None;
        let mut rows: Vec<Row> = Vec::new();
        let mut last_chain = None;

        for (i, nominator) in state.nominators_iter().enumerate() {
            let chain = nominator.runtime().relay_chain();
            if last_chain != Some(chain) {
                let mut header_row = vec![
                    Text::from(""),
                    Text::from(chain.to_string()).style(theme.paragraph.label_italic),
                    Text::from(""),
                ];
                if proxies_available {
                    header_row.push(Text::from(""));
                }
                header_row.push(Text::from(""));
                rows.push(Row::new(header_row));
                last_chain = Some(chain);
            }

            if selected == Some(i) {
                display_selected = Some(rows.len());
            }

            // Show the origin alongside the address only when active and selected
            let mut name = Vec::new();
            if is_active && selected == Some(i) {
                name.push(Span::styled(
                    format!("{} ", nominator.origins.symbol()),
                    theme.paragraph.label(is_active),
                ));
            }
            name.push(Span::raw(nominator.to_compact_string(4)));

            // The validators nominated, unknown until read from the chain
            let targets = match &nominator.nominations {
                Some(_) => nominator.targets_count().to_string(),
                None => "-".to_string(),
            };

            let mut row = vec![
                Text::from(""),
                Text::from(Line::from(name)),
                Text::from(targets).alignment(Alignment::Right),
            ];
            if proxies_available {
                row.push(Text::from(nominator.proxies_as_str()).alignment(Alignment::Right));
            }
            row.push(Text::from(""));
            rows.push(Row::new(row));
        }

        if rows.len().is_zero() {
            rows.push(Row::new(vec![
                Text::from(""),
                Text::from(Line::from(vec![
                    Span::styled("-- ", theme.paragraph.label_italic),
                    Span::styled("ctrl+x n", theme.paragraph.base_italic),
                    Span::raw(" "),
                    Span::styled("add a nominator", theme.paragraph.label_italic),
                    Span::styled(" --", theme.paragraph.label_italic),
                ])),
            ]));
        }

        let mut widths = vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(8),
        ];
        if proxies_available {
            widths.push(Constraint::Length(7));
        }
        widths.push(Constraint::Length(1));

        let title = "nominators";
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
        let targets_title = if has_nominators { "targets" } else { "" };
        let mut header_cells = vec![
            Cell::from(""),
            Cell::from(Line::from(title_span)),
            Cell::from(Text::from(targets_title).alignment(Alignment::Right)),
        ];
        if proxies_available {
            header_cells.push(Cell::from(
                Text::from("proxies").alignment(Alignment::Right),
            ));
        }
        header_cells.push(Cell::from(""));

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(theme.table.header(is_active)))
            .style(theme.table.base)
            .row_highlight_style(state.focus().compact_row_highlight(&theme.table))
            .highlight_symbol(theme.table.highlight_symbol(is_active));

        let mut display_table_state = state.table_state;
        display_table_state.select(display_selected);

        StatefulWidget::render(table, area, buf, &mut display_table_state);
    }
}
