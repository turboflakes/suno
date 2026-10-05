use crate::widgets::chains::ChainsList;
use crate::widgets::collators::CollatorsList;
use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Styled,
    text::{Line, Span, Text},
    widgets::{Block, Cell, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use std::collections::BTreeSet;
use suno_config::SupportedRuntime;
use suno_primitives::{
    collator::Collator,
    display::{create_progress_bar_by_blocks, format_millis},
};
use suno_theme::Theme;

pub const LIST_HEADER_HEIGHT: u16 = 4;
pub const LINES_PER_CHAIN: u16 = 1;
pub const PADDING: u16 = 3;

/// Header height for a group: the fixed lines plus [`LINES_PER_CHAIN`] lines per
/// distinct parachain the group's collators span.
fn group_header_height(collators: &[&Collator]) -> u16 {
    let distinct_chains = collators
        .iter()
        .map(|c| c.runtime())
        .collect::<BTreeSet<_>>()
        .len();
    LIST_HEADER_HEIGHT + distinct_chains as u16 * LINES_PER_CHAIN
}

#[derive(Debug)]
pub struct CollatorsDetailedListWidget<'a> {
    pub chains: &'a ChainsList,
    theme: Theme,
}

impl<'a> CollatorsDetailedListWidget<'a> {
    pub fn new(chains: &'a ChainsList, theme: Theme) -> Self {
        Self { chains, theme }
    }
}

/// Collators list view widget implementation, mostly to be used under the collators main tab view
impl<'a> StatefulWidget for CollatorsDetailedListWidget<'a> {
    type State = CollatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_height(area.height);
        let collators_grouped = state.get_collators_grouped_by_relay_chain();
        let group_count = collators_grouped.len();
        let total_height: u16 = collators_grouped
            .values()
            .map(|c| group_header_height(c) + c.len() as u16 + PADDING)
            .sum();
        let is_scroll_visible = state.is_active() && area.height < total_height;
        let area_width = if is_scroll_visible {
            area.width.saturating_sub(1)
        } else {
            area.width
        };

        // Create a new area to fit all the content as total_height, but as wide as the screen.
        let mut full_content_buf = Buffer::empty(Rect::new(0, 0, area_width, total_height));

        // Track the current height of group in display
        let mut current_y_group = 0;

        // Iterate and render each relay-chain group
        for (group_index, (relay_runtime, collators)) in collators_grouped.into_iter().enumerate() {
            let header_height = group_header_height(&collators);
            let group_height = header_height + collators.len() as u16 + PADDING;
            let group_area = Rect::new(0, current_y_group, area_width, group_height);

            // Get selected collator if one of the collators in the current section
            let selected_collator = match state.get_selected_ref() {
                Some(selected) if collators.contains(&selected) && state.is_active() => {
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
                relay_runtime,
                &collators,
                selected_collator,
                group_area,
                &mut full_content_buf,
                &mut state.table_state.clone(),
                is_odd,
                block,
                header_height,
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
                let source_cell = &full_content_buf[(x, virtual_y)];

                let dest_x = area.x + x;
                let dest_y = area.y + y;

                if dest_x < buf.area.width && dest_y < buf.area.height {
                    let target_cell = &mut buf[(dest_x, dest_y)];
                    target_cell.set_symbol(source_cell.symbol());
                    target_cell.set_style(source_cell.style());
                }
            }
        }

        // Render scrollbar when active
        if state.is_active() && area.height < total_height {
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

impl<'a> CollatorsDetailedListWidget<'a> {
    #[allow(clippy::too_many_arguments)]
    fn render_group(
        &self,
        relay_runtime: SupportedRuntime,
        collators: &[&Collator],
        selected_collator: Option<&Collator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
        is_odd: bool,
        block: Block,
        header_height: u16,
    ) {
        let content_area = block.inner(area);
        block.render(area, buf);

        let [header_area, body_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .areas(content_area);

        self.render_list_header(relay_runtime, collators, header_area, buf, is_odd);
        self.render_list_body(
            collators,
            selected_collator,
            body_area,
            buf,
            table_state,
            is_odd,
        );
    }

    fn render_list_header(
        &self,
        relay_runtime: SupportedRuntime,
        collators: &[&Collator],
        area: Rect,
        buf: &mut Buffer,
        is_odd: bool,
    ) {
        let theme = self.theme;

        // On-chain totals (invulnerables + permissionless authorities), summed across
        // every distinct parachain this relay-chain group spans.
        let chain_runtimes: BTreeSet<SupportedRuntime> =
            collators.iter().map(|c| c.runtime()).collect();
        let (mut total_invulnerables, mut total_permissionless) = (0usize, 0usize);
        for runtime in &chain_runtimes {
            if let Some(aura) = self
                .chains
                .get_chain_by_runtime(*runtime)
                .and_then(|chain| chain.aura().clone())
            {
                total_invulnerables += aura.invulnerables().len();
                total_permissionless += aura.permissionless().len();
            }
        }

        let total_count = [(total_invulnerables, "inv"), (total_permissionless, "perm")]
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .map(|(count, label)| format!("{} {}", count, label))
            .collect::<Vec<_>>()
            .join(", ");

        let invulnerables_count = collators.iter().filter(|c| c.is_invulnerable()).count();
        let permissionless_count = collators.iter().filter(|c| c.is_permissionless()).count();
        let registered_count = collators.iter().filter(|c| c.is_waiting()).count();

        let displayed = [
            (invulnerables_count, "inv"),
            (permissionless_count, "perm"),
            (registered_count, "reg"),
        ]
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .map(|(count, label)| format!("{} {}", count, label))
        .collect::<Vec<_>>()
        .join(", ");

        let block_style = theme.block.alt(is_odd);

        // Split vertically: title (1 row) + one row per distinct chain + the fixed
        // "Total collators"/"Displayed"/blank block at the bottom.
        let [title_area, chains_area, bottom_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(chain_runtimes.len() as u16),
                Constraint::Length(3),
            ])
            .areas(area);

        let title_block = Block::new().set_style(block_style);
        Paragraph::new(Line::from(
            Span::raw(format!(
                "{} CHAINS",
                relay_runtime.to_string().to_uppercase()
            ))
            .style(theme.paragraph.header_active),
        ))
        .block(title_block)
        .style(theme.paragraph.base)
        .render(title_area, buf);

        // One row per distinct parachain, split left/right so "Avg. block time" stays
        // left-aligned while the session progress (bar + countdown) is right-aligned -
        // each chain has its own independent Aura/session state.
        let row_constraints: Vec<Constraint> = chain_runtimes
            .iter()
            .map(|_| Constraint::Length(1))
            .collect();
        let chain_row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(chains_area);

        for (runtime, row_area) in chain_runtimes.iter().zip(chain_row_areas.iter()) {
            let Some(chain) = self.chains.get_chain_by_runtime(*runtime) else {
                Paragraph::new(Line::from(format!("{} -", runtime)))
                    .block(Block::new().set_style(block_style))
                    .style(theme.paragraph.base)
                    .render(*row_area, buf);
                continue;
            };

            let Some(aura) = chain.aura() else {
                Paragraph::new(Line::from(format!("{} -", runtime)))
                    .block(Block::new().set_style(block_style))
                    .style(theme.paragraph.base)
                    .render(*row_area, buf);
                continue;
            };

            let [network_area, progress_area, progress_bar_area, countdown_area] =
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Fill(1),    // Avg. block time
                        Constraint::Length(26), // Session info
                        Constraint::Length(24), // Session progress bar
                        Constraint::Length(16), // Countdown
                    ])
                    .areas(*row_area);

            let avg_block_time = aura
                .slot_duration_ms()
                .and_then(|slot_duration_ms| chain.average_block_time_ms(slot_duration_ms))
                .map(|ms| format_millis(ms, true, true))
                .unwrap_or_else(|| "-".to_string());

            Paragraph::new(Line::from(vec![
                Span::raw(format!("{} avg. block time ", runtime.system_name()))
                    .style(theme.paragraph.label),
                Span::raw(avg_block_time),
            ]))
            .block(Block::new().set_style(block_style))
            .style(theme.paragraph.base)
            .render(network_area, buf);

            let session_progress =
                aura.session_progress(chain.finalized_block(), runtime.duration_bn());

            Paragraph::new(
                Line::from(format!(
                    "{} session {} {:.0}% ",
                    runtime.as_str_short().to_lowercase(),
                    aura.current_session_index().unwrap_or_default(),
                    session_progress * 100_f64
                ))
                .alignment(Alignment::Right),
            )
            .block(Block::new().set_style(block_style))
            .style(theme.paragraph.base)
            .render(progress_area, buf);

            let session_progress_bar = create_progress_bar_by_blocks(session_progress, 24);

            Paragraph::new(Line::from(session_progress_bar).alignment(Alignment::Right))
                .block(Block::new().set_style(block_style))
                .style(theme.paragraph.base)
                .render(progress_bar_area, buf);

            let session_countdown =
                aura.session_countdown_time(chain.finalized_block(), runtime.duration_bn());

            Paragraph::new(
                Line::from(format!(" {}", session_countdown)).alignment(Alignment::Left),
            )
            .block(Block::new().set_style(block_style))
            .style(theme.paragraph.base)
            .render(countdown_area, buf);
        }

        let bottom_lines = vec![
            Line::from(vec![
                Span::raw("Total collators ").style(theme.paragraph.label),
                Span::raw(total_count),
            ]),
            Line::from(vec![
                Span::raw("Displayed ").style(theme.paragraph.label),
                Span::raw(displayed),
            ]),
            Line::from(""),
        ];

        Paragraph::new(bottom_lines)
            .block(Block::new().set_style(block_style))
            .style(theme.paragraph.base)
            .render(bottom_area, buf);
    }

    fn render_list_body(
        &self,
        collators: &[&Collator],
        selected_collator: Option<&Collator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
        is_odd: bool,
    ) {
        let theme = self.theme;
        let show_next_keys = collators.iter().any(|c| c.is_next_keys_changed());

        let mut header_cells = vec![
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
            Cell::from(Text::from("chain").alignment(Alignment::Left)),
            Cell::from(Text::from("identity").alignment(Alignment::Left)),
            Cell::from(Text::from("last block").alignment(Alignment::Right)),
            Cell::from(Text::from("").alignment(Alignment::Left)),
            Cell::from(Text::from("next slot").alignment(Alignment::Right)),
            Cell::from(Text::from("in").alignment(Alignment::Left)),
            Cell::from(Text::from("keys").alignment(Alignment::Right)),
        ];
        if show_next_keys {
            header_cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }
        let header = Row::new(header_cells);

        let mut widths = vec![
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Length(24),
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ];
        if show_next_keys {
            widths.push(Constraint::Length(10));
        }

        let rows = collators
            .iter()
            .map(|c| self.collator_row(c, selected_collator, theme, show_next_keys))
            .collect::<Vec<_>>();

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we always want to start from the top.
        *table_state.offset_mut() = 0;

        let block = Block::new().set_style(theme.block.alt(is_odd));
        let table = Table::new(rows, widths)
            .block(block)
            .header(header.set_style(theme.table.header));

        StatefulWidget::render(table, area, buf, table_state);
    }

    fn collator_row(
        &self,
        collator: &Collator,
        selected: Option<&Collator>,
        theme: Theme,
        show_next_keys: bool,
    ) -> Row<'static> {
        let (cell_style, _highlight_symbol) = match selected {
            Some(selected) if collator == selected => (theme.paragraph.cell_active, "❯"),
            _ => (theme.paragraph.cell, ""),
        };

        // Each row may belong to a different parachain, so unlike the grouped view, the
        // Aura/slot data has to be looked up per-collator rather than once per section.
        let chain = self.chains.get_chain_by_runtime(collator.runtime());
        let aura = chain.as_ref().and_then(|c| c.aura().as_ref());
        let current_slot = chain
            .as_ref()
            .map(|c| c.current_slot().unwrap_or_default())
            .unwrap_or_default();
        let current_slot_ts = chain
            .as_ref()
            .map(|c| c.current_slot_ts())
            .unwrap_or_default();

        let is_current_author =
            aura.is_some_and(|a| collator.is_current_slot_author(a.authorities(), current_slot));

        let blocks = collator.blocks_in_slot(current_slot);
        let blocks_in_slot_str = match aura.and_then(|a| a.number_blocks_expected()) {
            Some(expected) => format!("{blocks:2}/{expected}"),
            None => format!("{blocks:2}"),
        };

        let last_block_str = collator
            .last_block_authored()
            .map(|b| {
                let prefix = if is_current_author { "> " } else { "" };
                format!("{prefix}#{b}")
            })
            .unwrap_or_default();

        let mut cells = vec![
            Cell::from(Text::from(collator.status().to_string()).alignment(Alignment::Left)),
            Cell::from(Text::from(collator.runtime().system_name()).alignment(Alignment::Left)),
            Cell::from(Text::from(collator.display_identity()).alignment(Alignment::Left))
                .style(cell_style),
            Cell::from(Text::from(last_block_str).alignment(Alignment::Right)),
            Cell::from(Text::from(blocks_in_slot_str).alignment(Alignment::Left)),
        ];

        if is_current_author {
            cells.push(Cell::from(
                Text::from(format!("#{}", current_slot)).alignment(Alignment::Right),
            ));
            cells.push(Cell::from(Text::from("").alignment(Alignment::Left)));
        } else {
            let next_slot_str = aura
                .and_then(|a| collator.next_slot(a.authorities(), current_slot))
                .map(|s| format!("#{}", s))
                .unwrap_or_default();

            let next_slot_countdown_str = aura
                .and_then(|a| {
                    let slot_duration_ms = a.slot_duration_ms()?;
                    collator.next_slot_countdown(
                        a.authorities(),
                        current_slot,
                        slot_duration_ms,
                        current_slot_ts,
                    )
                })
                .unwrap_or_default();

            cells.push(Cell::from(
                Text::from(next_slot_str).alignment(Alignment::Right),
            ));
            cells.push(Cell::from(
                Text::from(next_slot_countdown_str).alignment(Alignment::Left),
            ));
        }

        cells.push(Cell::from(
            Text::from(collator.display_queued_keys(6)).alignment(Alignment::Right),
        ));

        if show_next_keys {
            if collator.is_next_keys_changed() {
                cells.push(Cell::from(
                    Text::from(collator.display_next_keys(6)).alignment(Alignment::Left),
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        Row::new(cells)
    }
}
