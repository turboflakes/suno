use crate::widgets::chain_header::render_network_header;
use crate::widgets::chains::ChainsList;
use crate::widgets::list_focus::ListFocus;
use crate::widgets::nominators::NominatorsList;
use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::validators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Styled},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table, TableState, Widget},
};
use suno_config::{Features, SupportedRuntime, CONFIG};
use suno_primitives::{display::format_planks, AccountDisplay, Nominator};
use suno_theme::Theme;

#[derive(Debug)]
pub struct NominatorsDetailedGroupWidget<'a> {
    pub chains: &'a ChainsList,
    theme: Theme,
}

impl<'a> NominatorsDetailedGroupWidget<'a> {
    pub fn new(chains: &'a ChainsList, theme: Theme) -> Self {
        Self { chains, theme }
    }
}

/// Nominators grouped view widget implementation, mostly to be used under the nominators main tab view
impl<'a> StatefulWidget for NominatorsDetailedGroupWidget<'a> {
    type State = NominatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_height(area.height);
        let nominators_grouped = state.get_nominators_grouped_by_runtime();
        let group_count = nominators_grouped.len();
        let total_height = state.total_detailed_group_height();
        let is_scroll_visible = state.focus().is_active() && area.height < total_height;
        let area_width = if is_scroll_visible {
            area.width.saturating_sub(1)
        } else {
            area.width
        };

        // Values needed further down to render the table
        let focus = state.focus();
        let is_active = focus.is_highlighted();
        let table_state = state.table_state;

        // Create a new area to fit all the content as total_height, but as wide as the screen.
        let mut full_content_buf = Buffer::empty(Rect::new(0, 0, area_width, total_height));

        // Track the current height of group in display
        let mut current_y_group = 0;

        // Iterate and render each group
        for (group_index, (runtime, nominators)) in nominators_grouped.into_iter().enumerate() {
            let group_height = GROUP_HEADER_HEIGHT + nominators.len() as u16 + PADDING;
            let group_area = Rect::new(0, current_y_group, area_width, group_height);

            // Get selected nominator if one of the nominators in the current section
            let selected_nominator = match state.get_selected() {
                Some(selected)
                    if nominators.iter().any(|n| n.key() == selected.key()) && is_active =>
                {
                    Some(selected)
                }
                _ => None,
            };

            let is_odd = group_index % 2 == 1;
            let padding = if group_count > 1 {
                Padding::proportional(1)
            } else {
                Padding::ZERO
            };
            let block = Block::default()
                .style(self.theme.block.alt(is_odd))
                .padding(padding);

            self.render_group(
                runtime,
                &nominators,
                selected_nominator,
                focus,
                is_odd,
                block,
                group_area,
                &mut full_content_buf,
                &mut table_state.clone(),
            );

            current_y_group += group_height;
        }

        // Copy the visible part from full_content_buf to the actual screen buf
        for y in 0..area.height {
            let virtual_y = y + state.scroll_offset;
            if virtual_y >= total_height {
                break;
            }

            for x in 0..area_width {
                // Get the cell from the large virtual buffer
                let source_cell = &full_content_buf[(x, virtual_y)];

                // Get the destination cell in the actual screen buffer
                // Note: area.x and area.y are the offsets where the widget starts
                let dest_x = area.x + x;
                let dest_y = area.y + y;

                // Safety check to stay within buffer bounds
                if dest_x < buf.area.width && dest_y < buf.area.height {
                    let target_cell = &mut buf[(dest_x, dest_y)];
                    target_cell.set_symbol(source_cell.symbol());
                    target_cell.set_style(source_cell.style());
                }
            }
        }

        // Render scrollbar when active
        if state.focus().is_active() && area.height < total_height {
            let selected_pos = state.table_state.selected().unwrap_or_default();

            let scrollbar_area = Rect {
                x: area.right().saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            render_scrollbar(
                self.theme,
                state.scroll_offset as usize + selected_pos,
                total_height as usize,
                scrollbar_area,
                buf,
            );
        }
    }
}

impl<'a> NominatorsDetailedGroupWidget<'a> {
    #[allow(clippy::too_many_arguments)]
    fn render_group(
        &self,
        runtime: SupportedRuntime,
        nominators: &[&Nominator],
        selected_nominator: Option<&Nominator>,
        focus: ListFocus,
        is_odd: bool,
        block: Block,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let content_area = block.inner(area);
        block.render(area, buf);

        // Split area into header and body
        let [header_area, body_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(GROUP_HEADER_HEIGHT), // Header height
                Constraint::Min(0),                      // Body takes remaining
            ])
            .areas(content_area);

        // Render network header
        self.render_table_header(runtime, nominators, header_area, buf, is_odd);

        // Render network nominators table
        self.render_table_body(
            runtime,
            nominators,
            selected_nominator,
            focus,
            is_odd,
            body_area,
            buf,
            table_state,
        );
    }

    fn render_table_header(
        &self,
        runtime: SupportedRuntime,
        nominators: &[&Nominator],
        area: Rect,
        buf: &mut Buffer,
        is_odd: bool,
    ) {
        let displayed = Line::from(vec![
            Span::raw("Displayed ").style(self.theme.paragraph.label),
            Span::raw(format!(
                "{} nominating, {} idle",
                nominators.iter().filter(|n| n.is_nominating()).count(),
                nominators.iter().filter(|n| !n.is_nominating()).count(),
            )),
        ]);
        render_network_header(
            self.chains,
            self.theme,
            runtime,
            displayed,
            area,
            buf,
            is_odd,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_table_body(
        &self,
        runtime: SupportedRuntime,
        nominators: &[&Nominator],
        selected_nominator: Option<&Nominator>,
        focus: ListFocus,
        is_odd: bool,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let theme = self.theme;

        let symbol = Span::raw(runtime.token_symbol()).style(theme.paragraph.label(false));

        // The era the stake is unlocked in is only known once the era of the chain is
        let era = runtime
            .asset_hub_runtime()
            .and_then(|runtime| self.chains.get_chain_by_runtime(runtime))
            .and_then(|chain| chain.era().as_ref().map(|era| era.index()));

        let columns = Columns::new(nominators, era, CONFIG.features());

        let widths = columns.widths();

        let header = columns.header();

        let rows = self.build_nominator_rows(
            nominators,
            &columns,
            era.unwrap_or_default(),
            theme,
            &symbol,
        );

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we always want to start from the top.
        *table_state.offset_mut() = 0;

        // Every group renders its own table, but the selected index of the state counts the rows
        // of all the groups. Translate it to the position of the selected nominator in this group,
        // or clear it when the selection is in another group, so only the right row is highlighted.
        table_state.select(nominators.iter().position(|item| {
            selected_nominator.is_some_and(|selected| item.key() == selected.key())
        }));

        let block = Block::new().set_style(theme.block.alt(is_odd));
        let table = Table::new(rows, widths)
            .block(block)
            .header(header.set_style(theme.table.header))
            .row_highlight_style(focus.row_highlight(&theme.table));

        StatefulWidget::render(table, area, buf, table_state);
    }

    fn build_nominator_rows(
        &self,
        nominators: &[&Nominator],
        columns: &Columns,
        era: u32,
        theme: Theme,
        symbol: &Span<'static>,
    ) -> Vec<Row<'static>> {
        nominators
            .iter()
            .map(|n| self.nominator_row(n, columns, era, theme, symbol))
            .collect()
    }

    fn nominator_row(
        &self,
        nominator: &Nominator,
        columns: &Columns,
        era: u32,
        theme: Theme,
        symbol: &Span<'static>,
    ) -> Row<'static> {
        let row = Row::new(self.nominator_cells(nominator, columns, era, theme, symbol));
        if nominator.is_nominating() {
            row
        } else {
            row.style(Style::default().add_modifier(Modifier::DIM))
        }
    }

    fn nominator_cells(
        &self,
        nominator: &Nominator,
        columns: &Columns,
        era: u32,
        theme: Theme,
        symbol: &Span<'static>,
    ) -> Vec<Cell<'static>> {
        let decimals = nominator.runtime().token_decimals();

        let status = match &nominator.nominations {
            Some(n) if n.suppressed => "[S]",
            Some(n) if !n.targets.is_empty() => "[N]",
            _ => "[I]",
        };
        let identity = match nominator.identity() {
            Some(identity) => identity.to_string(),
            None => nominator.to_compact_string(6),
        };

        let mut cells = vec![
            Cell::from(Text::from(nominator.origins.symbol()).style(theme.paragraph.label(false))),
            Cell::from(Text::from(status).alignment(Alignment::Left)),
            Cell::from(Text::from(identity).alignment(Alignment::Left)),
            Cell::from(
                Text::from(nominator.targets_count().to_string()).alignment(Alignment::Right),
            ),
        ];

        if columns.own_stake {
            cells.push(amount_cell(
                nominator.ledger.active(),
                decimals,
                symbol.clone(),
                Alignment::Right,
            ));
        }

        if columns.bonded {
            if nominator.ledger.total() != nominator.ledger.active() {
                cells.push(amount_cell(
                    nominator.ledger.total(),
                    decimals,
                    symbol.clone(),
                    Alignment::Left,
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        if columns.unlocking {
            let unlocking = nominator.unlocking(era);
            cells.push(if unlocking > 0 {
                amount_cell(unlocking, decimals, symbol.clone(), Alignment::Right)
            } else {
                Cell::from(Text::from(""))
            });
        }

        if columns.unlocked {
            let unlocked = nominator.unlocked(era);
            cells.push(if unlocked > 0 {
                amount_cell(unlocked, decimals, symbol.clone(), Alignment::Right)
            } else {
                Cell::from(Text::from(""))
            });
        }

        if columns.payee {
            cells.push(Cell::from(
                Text::from(
                    nominator
                        .payee
                        .to_compact_string(nominator.account_format(), 3),
                )
                .alignment(Alignment::Right),
            ));
        }

        cells
    }
}

struct Columns {
    own_stake: bool,
    bonded: bool,
    unlocking: bool,
    unlocked: bool,
    payee: bool,
}

impl Columns {
    fn new(nominators: &[&Nominator], era: Option<u32>, feat: &Features) -> Self {
        Self {
            own_stake: feat.own_stake_visible(),
            bonded: feat.own_stake_visible()
                && nominators
                    .iter()
                    .any(|n| n.ledger.total() != n.ledger.active()),
            unlocking: feat.own_stake_visible()
                && era.is_some_and(|era| nominators.iter().any(|n| n.unlocking(era) > 0)),
            unlocked: feat.own_stake_visible()
                && era.is_some_and(|era| nominators.iter().any(|n| n.unlocked(era) > 0)),
            payee: feat.payee_visible(),
        }
    }

    fn header(&self) -> Row<'static> {
        let mut cells = vec![
            // The origin of the nominator
            Cell::from(""),
            // The status of the nominator
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
            Cell::from(Text::from("identity").alignment(Alignment::Left)),
            Cell::from(Text::from("targets").alignment(Alignment::Right)),
        ];

        if self.own_stake {
            cells.push(Cell::from(
                Text::from("own-stake").alignment(Alignment::Right),
            ));
        }

        if self.bonded {
            cells.push(Cell::from(
                Text::from("(bonded)").alignment(Alignment::Left),
            ));
        }

        if self.unlocking {
            cells.push(Cell::from(
                Text::from("unlocking").alignment(Alignment::Right),
            ));
        }

        if self.unlocked {
            cells.push(Cell::from(
                Text::from("unlocked").alignment(Alignment::Right),
            ));
        }

        if self.payee {
            cells.push(Cell::from(Text::from("payee").alignment(Alignment::Right)));
        }

        Row::new(cells)
    }

    fn widths(&self) -> Vec<Constraint> {
        let mut widths = vec![
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(24),
            // targets
            Constraint::Fill(1),
        ];

        if self.own_stake {
            widths.push(Constraint::Fill(2));
        }

        if self.bonded {
            widths.push(Constraint::Fill(2));
        }

        if self.unlocking {
            widths.push(Constraint::Fill(2));
        }

        if self.unlocked {
            widths.push(Constraint::Fill(2));
        }

        if self.payee {
            widths.push(Constraint::Fill(2));
        }

        widths
    }
}

fn amount_cell(
    value: u128,
    decimals: u32,
    symbol: Span<'static>,
    alignment: Alignment,
) -> Cell<'static> {
    Cell::from(
        Line::from(vec![Span::raw(format_planks(value, decimals, 4)), symbol]).alignment(alignment),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::utils::AccountId32;
    use suno_primitives::staking::{Chunk, StakeLedger};

    fn nominator_unbonding(era: u32) -> Nominator {
        let mut nominator =
            Nominator::new(SupportedRuntime::AssetHubKusama, AccountId32::from([1; 32]));
        nominator.ledger = StakeLedger::new(100, 160, vec![Chunk::new(era, 60)]);
        nominator
    }

    #[test]
    fn the_unlocking_column_is_shown_while_the_stake_is_unbonding() {
        let nominator = nominator_unbonding(12);
        let features = Features::default();

        let columns = Columns::new(&[&nominator], Some(10), &features);
        assert!(columns.own_stake && columns.bonded && columns.unlocking);
        assert!(!columns.unlocked);

        let columns = Columns::new(&[&nominator], Some(12), &features);
        assert!(!columns.unlocking && columns.unlocked);
    }

    #[test]
    fn the_unlocking_columns_wait_for_the_era() {
        let nominator = nominator_unbonding(12);
        let columns = Columns::new(&[&nominator], None, &Features::default());
        assert!(!columns.unlocking && !columns.unlocked);
    }
}
