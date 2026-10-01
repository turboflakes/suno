use crate::widgets::chains::ChainsList;
use crate::widgets::collators::CollatorsList;
use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Styled,
    text::{Line, Span, Text},
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use suno_config::SupportedRuntime;
use suno_primitives::{
    collator::Collator,
    display::{create_progress_bar_by_blocks, format_millis},
    Aura,
};
use suno_theme::Theme;

pub const GROUP_HEADER_HEIGHT: u16 = 5;
pub const PADDING: u16 = 3;

#[derive(Debug)]
pub struct CollatorsDetailedGroupWidget<'a> {
    pub chains: &'a ChainsList,
    theme: Theme,
}

impl<'a> CollatorsDetailedGroupWidget<'a> {
    pub fn new(chains: &'a ChainsList, theme: Theme) -> Self {
        Self { chains, theme }
    }
}

/// Collators grouped view widget implementation, mostly to be used under the collators main tab view
impl<'a> StatefulWidget for CollatorsDetailedGroupWidget<'a> {
    type State = CollatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_height(area.height);
        let collators_grouped = state.get_collators_grouped_by_runtime();
        let total_height = state.total_detailed_group_height();
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

        // Iterate and render each group
        for (runtime, collators) in collators_grouped {
            let group_height = GROUP_HEADER_HEIGHT + collators.len() as u16 + PADDING;
            let group_area = Rect::new(0, current_y_group, area_width, group_height);

            // Get selected collator if one of the collators in the current section
            let selected_collator = match state.get_selected_ref() {
                Some(selected) if collators.contains(&selected) && state.is_active() => {
                    Some(selected)
                }
                _ => None,
            };

            self.render_group(
                runtime,
                &collators,
                selected_collator,
                group_area,
                &mut full_content_buf,
                &mut state.table_state.clone(),
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

impl<'a> CollatorsDetailedGroupWidget<'a> {
    fn render_group(
        &self,
        runtime: SupportedRuntime,
        collators: &[&Collator],
        selected_collator: Option<&Collator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let [header_area, body_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(GROUP_HEADER_HEIGHT), Constraint::Min(0)])
            .areas(area);

        self.render_table_header(runtime, collators, header_area, buf);
        self.render_table_body(
            runtime,
            collators,
            selected_collator,
            body_area,
            buf,
            table_state,
        );
    }

    fn render_table_header(
        &self,
        runtime: SupportedRuntime,
        collators: &[&Collator],
        area: Rect,
        buf: &mut Buffer,
    ) {
        let theme = self.theme;

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let [network_area, progress_area, progress_bar_area, countdown_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),    // Network info
                Constraint::Length(20), // Slot info
                Constraint::Length(24), // Slot progress bar
                Constraint::Length(16), // Countdown
            ])
            .areas(area);

        let invulnerables_count = chain.aura().as_ref().map_or(0, |a| a.invulnerables().len());
        let permissionless_count = chain
            .aura()
            .as_ref()
            .map_or(0, |a| a.permissionless().len());

        let total_count = [(invulnerables_count, "inv"), (permissionless_count, "perm")]
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .map(|(count, label)| format!("{} {}", count, label))
            .collect::<Vec<_>>()
            .join(", ");

        let mut network_lines = vec![
            Line::from(
                Span::raw(format!("{} NETWORK", runtime.to_string().to_uppercase()))
                    .style(theme.paragraph.header_active),
            ),
            Line::from(vec![
                Span::raw("Avg. block time ").style(theme.paragraph.label),
                Span::raw(
                    chain
                        .aura()
                        .as_ref()
                        .and_then(|a| a.slot_duration_ms())
                        .and_then(|slot_duration_ms| chain.average_block_time_ms(slot_duration_ms))
                        .map(|ms| format_millis(ms, true, true))
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ]),
            Line::from(vec![
                Span::raw("Total collators ").style(theme.paragraph.label),
                Span::raw(total_count),
            ]),
        ];

        let invulnerables_count = collators.iter().filter(|c| c.is_invulnerable()).count();
        let permissionless_count = collators.iter().filter(|c| c.is_authority()).count();
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

        network_lines.push(Line::from(vec![
            Span::raw("Displayed ").style(theme.paragraph.label),
            Span::raw(displayed),
        ]));
        network_lines.push(Line::from(""));

        let block = Block::new().set_style(theme.block.main);
        let network_info = Paragraph::new(network_lines)
            .block(block)
            .style(theme.paragraph.base);

        network_info.render(network_area, buf);

        // Draw and render session and slot progress

        let Some(aura) = chain.aura() else {
            // TODO: Handle aura not available, maybe render loading indicator
            let block = Block::new().set_style(theme.block.main);
            let area = progress_area.union(progress_bar_area).union(countdown_area);
            block.render(area, buf);
            return;
        };

        let slot_duration_ms = aura.slot_duration_ms().unwrap_or(0);
        let session_progress =
            aura.session_progress(chain.finalized_block(), chain.runtime().duration_bn());
        let slot_progress = chain.slot_progress(slot_duration_ms);

        let progress_lines = vec![
            Line::from(""),
            Line::from(format!(
                "session {} {:3.0}% ",
                aura.current_session_index().unwrap_or_default(),
                session_progress * 100_f64
            ))
            .alignment(Alignment::Right),
            Line::from(format!(
                "slot {} {:3.0}% ",
                chain.current_slot().unwrap_or_default(),
                slot_progress * 100_f64
            ))
            .alignment(Alignment::Right),
        ];

        let block = Block::new().set_style(theme.block.main);
        let progress_info = Paragraph::new(progress_lines)
            .block(block)
            .style(theme.paragraph.base);

        progress_info.render(progress_area, buf);

        let session_progress_bar = create_progress_bar_by_blocks(session_progress, 24);
        let slot_progress_bar = create_progress_bar_by_blocks(slot_progress, 24);

        let progress_bar_lines = vec![
            Line::from(""),
            Line::from(session_progress_bar).alignment(Alignment::Right),
            Line::from(slot_progress_bar).alignment(Alignment::Right),
        ];

        let block = Block::new().set_style(theme.block.main);
        let progress_bar = Paragraph::new(progress_bar_lines)
            .block(block)
            .style(theme.paragraph.base);

        progress_bar.render(progress_bar_area, buf);

        let countdown_lines = vec![
            Line::from(""),
            Line::from(format!(
                " {}",
                aura.session_countdown_time(chain.finalized_block(), chain.runtime().duration_bn()),
            ))
            .alignment(Alignment::Left),
            Line::from(format!(" {}", chain.slot_countdown_time(slot_duration_ms)))
                .alignment(Alignment::Left),
        ];

        let block = Block::new().set_style(theme.block.main);
        let countdown_info = Paragraph::new(countdown_lines)
            .block(block)
            .style(theme.paragraph.base);

        countdown_info.render(countdown_area, buf);
    }

    fn render_table_body(
        &self,
        runtime: SupportedRuntime,
        collators: &[&Collator],
        selected_collator: Option<&Collator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let theme = self.theme;
        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let Some(aura) = chain.aura() else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let show_next_keys = collators.iter().any(|c| c.is_next_keys_changed());

        let mut header_cells = vec![
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
            Cell::from(Text::from("identity").alignment(Alignment::Left)),
            Cell::from(Text::from("authored/expected").alignment(Alignment::Right)),
            Cell::from(Text::from("(last block)").alignment(Alignment::Left)),
            Cell::from(Text::from("next slot in").alignment(Alignment::Right)),
            Cell::from(Text::from("(slot)").alignment(Alignment::Left)),
            Cell::from(Text::from("keys").alignment(Alignment::Right)),
        ];
        if show_next_keys {
            header_cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }
        let header = Row::new(header_cells);

        let mut widths = vec![
            Constraint::Length(3),
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

        let current_slot = chain.current_slot().unwrap_or_default();
        let current_slot_ts = chain.current_slot_ts();

        let rows = collators
            .iter()
            .map(|c| {
                self.collator_row(
                    c,
                    selected_collator,
                    aura,
                    current_slot,
                    current_slot_ts,
                    theme,
                    show_next_keys,
                )
            })
            .collect::<Vec<_>>();

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we always want to start from the top.
        *table_state.offset_mut() = 0;

        let block = Block::new().set_style(theme.block.main);
        let table = Table::new(rows, widths)
            .block(block)
            .header(header.set_style(theme.table.header));

        StatefulWidget::render(table, area, buf, table_state);
    }

    #[allow(clippy::too_many_arguments)]
    fn collator_row(
        &self,
        collator: &Collator,
        selected: Option<&Collator>,
        aura: &Aura,
        current_slot: u64,
        current_slot_ts: u128,
        theme: Theme,
        show_next_keys: bool,
    ) -> Row<'static> {
        let (cell_style, _highlight_symbol) = match selected {
            Some(selected) if collator == selected => (theme.paragraph.cell_active, "❯"),
            _ => (theme.paragraph.cell, ""),
        };

        let authorities = aura.authorities();

        let blocks_in_slot_str = match aura.number_blocks_expected() {
            Some(expected) => {
                if collator.is_current_slot_author(authorities, current_slot) {
                    format!("> {:2}/{}", collator.blocks_in_slot(current_slot), expected)
                } else {
                    format!("{:2}/{}", collator.blocks_in_slot(current_slot), expected)
                }
            }
            None => format!("{:2}", collator.blocks_in_slot(current_slot)),
        };

        let last_block_str = collator
            .last_block_authored()
            .map(|b| format!("#{}", b))
            .unwrap_or_default();

        let next_slot_str = collator
            .next_slot(authorities, current_slot)
            .map(|s| format!("#{}", s))
            .unwrap_or_default();

        let next_slot_countdown_str = match aura.slot_duration_ms() {
            Some(slot_duration_ms) => collator
                .next_slot_countdown(authorities, current_slot, slot_duration_ms, current_slot_ts)
                .unwrap_or_default(),
            None => "".to_string(),
        };

        let mut cells = vec![
            Cell::from(Text::from(collator.status().to_string()).alignment(Alignment::Left)),
            Cell::from(Text::from(collator.display_identity()).alignment(Alignment::Left))
                .style(cell_style),
            Cell::from(Text::from(blocks_in_slot_str).alignment(Alignment::Right)),
            Cell::from(Text::from(last_block_str).alignment(Alignment::Left)),
            Cell::from(Text::from(next_slot_countdown_str).alignment(Alignment::Right)),
            Cell::from(Text::from(next_slot_str).alignment(Alignment::Left)),
            Cell::from(Text::from(collator.display_queued_keys(6)).alignment(Alignment::Right)),
        ];

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
