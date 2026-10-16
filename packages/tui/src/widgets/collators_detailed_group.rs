use crate::widgets::chains::ChainsList;
use crate::widgets::collators::CollatorsList;
use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::spinner::current_frame;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Styled,
    text::{Line, Span, Text},
    widgets::{Block, Cell, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use suno_config::{Features, SupportedRuntime, CONFIG};
use suno_primitives::{
    collator::Collator,
    display::{create_progress_bar_by_blocks, format_millis, format_planks, get_elapsed_millis},
    Aura, NodeHost,
};
use suno_theme::Theme;

pub const GROUP_HEADER_HEIGHT: u16 = 5;
pub const PADDING: u16 = 3;

#[derive(Debug)]
pub struct CollatorsDetailedGroupWidget<'a> {
    pub chains: &'a ChainsList,
    theme: Theme,
    masked: bool,
}

impl<'a> CollatorsDetailedGroupWidget<'a> {
    pub fn new(chains: &'a ChainsList, theme: Theme) -> Self {
        Self {
            chains,
            theme,
            masked: true,
        }
    }

    pub fn with_masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }
}

/// Collators grouped view widget implementation, mostly to be used under the collators main tab view
impl<'a> StatefulWidget for CollatorsDetailedGroupWidget<'a> {
    type State = CollatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_height(area.height);
        let collators_grouped = state.get_collators_grouped_by_runtime();
        let group_count = collators_grouped.len();
        let total_height = state.total_detailed_group_height();
        let is_scroll_visible = state.is_active() && area.height < total_height;
        let area_width = if is_scroll_visible {
            area.width.saturating_sub(1)
        } else {
            area.width
        };

        // Values needed further down to render the table
        let is_active = state.is_active();
        let table_state = state.table_state;

        // Create a new area to fit all the content as total_height, but as wide as the screen.
        let mut full_content_buf = Buffer::empty(Rect::new(0, 0, area_width, total_height));

        // Track the current height of group in display
        let mut current_y_group = 0;

        // Iterate and render each group
        for (group_index, (runtime, collators)) in collators_grouped.into_iter().enumerate() {
            let group_height = GROUP_HEADER_HEIGHT + collators.len() as u16 + PADDING;
            let group_area = Rect::new(0, current_y_group, area_width, group_height);

            // Get selected collator if one of the collators in the current section
            let selected_collator = match state.get_selected_ref() {
                Some(selected) if collators.contains(&selected) && is_active => Some(selected),
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
                &collators,
                selected_collator,
                is_active,
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
            let scrollbar_area = Rect {
                x: area.right().saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            // The thumb follows what is displayed, the scrolled rows out of the ones that don't fit
            let scrollable = total_height.saturating_sub(area.height) as usize;
            render_scrollbar(
                self.theme,
                (state.scroll_offset as usize).min(scrollable),
                scrollable + 1,
                scrollbar_area,
                buf,
            );
        }
    }
}

impl<'a> CollatorsDetailedGroupWidget<'a> {
    #[allow(clippy::too_many_arguments)]
    fn render_group(
        &self,
        runtime: SupportedRuntime,
        collators: &[&Collator],
        selected_collator: Option<&Collator>,
        is_active: bool,
        is_odd: bool,
        block: Block,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let content_area = block.inner(area);
        block.render(area, buf);

        let [header_area, body_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(GROUP_HEADER_HEIGHT), Constraint::Min(0)])
            .areas(content_area);

        self.render_table_header(runtime, collators, header_area, buf, is_odd);
        self.render_table_body(
            runtime,
            collators,
            selected_collator,
            is_active,
            is_odd,
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
        is_odd: bool,
    ) {
        let theme = self.theme;
        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            let block = Block::new().set_style(theme.block.alt(is_odd));
            block.render(area, buf);
            return;
        };

        let [network_area, progress_area, progress_bar_area, countdown_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),    // Network info
                Constraint::Length(20), // Slot info
                Constraint::Length(24), // Slot progress bar
                Constraint::Length(20), // Countdown
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
                Span::raw(format!("{} CHAIN", runtime.to_string().to_uppercase()))
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
        let permissionless_count = collators.iter().filter(|c| c.is_permissionless()).count();
        let registered_count = collators.iter().filter(|c| c.is_candidate()).count();

        let displayed = [
            (invulnerables_count, "inv"),
            (permissionless_count, "perm"),
            (registered_count, "cand"),
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

        let block = Block::new().set_style(theme.block.alt(is_odd));
        let network_info = Paragraph::new(network_lines)
            .block(block)
            .style(theme.paragraph.base);

        network_info.render(network_area, buf);

        // Draw and render session and slot progress

        let Some(aura) = chain.aura() else {
            // TODO: Handle aura not available, maybe render loading indicator
            let block = Block::new().set_style(theme.block.alt(is_odd));
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

        let block = Block::new().set_style(theme.block.alt(is_odd));
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

        let block = Block::new().set_style(theme.block.alt(is_odd));
        let progress_bar = Paragraph::new(progress_bar_lines)
            .block(block)
            .style(theme.paragraph.base);

        progress_bar.render(progress_bar_area, buf);

        let countdown_lines = vec![
            Line::from(""),
            Line::from(format!(
                " {} left",
                aura.session_countdown_time(chain.finalized_block(), chain.runtime().duration_bn()),
            ))
            .alignment(Alignment::Left),
            Line::from(format!(" {}", chain.slot_countdown_time(slot_duration_ms)))
                .alignment(Alignment::Left),
        ];

        let block = Block::new().set_style(theme.block.alt(is_odd));
        let countdown_info = Paragraph::new(countdown_lines)
            .block(block)
            .style(theme.paragraph.base);

        countdown_info.render(countdown_area, buf);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_table_body(
        &self,
        runtime: SupportedRuntime,
        collators: &[&Collator],
        selected_collator: Option<&Collator>,
        is_active: bool,
        is_odd: bool,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let theme = self.theme;

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            let block = Block::new().set_style(theme.block.alt(is_odd));
            block.render(area, buf);
            return;
        };

        let Some(aura) = chain.aura() else {
            let block = Block::new().set_style(theme.block.alt(is_odd));
            block.render(area, buf);
            return;
        };

        let columns = Columns::new(collators, CONFIG.features(), false);

        let widths = columns.widths();

        let header = columns.header();

        let current_slot = chain.current_slot().unwrap_or_default();
        let current_slot_ts = chain.current_slot_ts();

        let rows = collators
            .iter()
            .map(|c| {
                self.collator_row(
                    c,
                    selected_collator,
                    &columns,
                    aura,
                    current_slot,
                    current_slot_ts,
                    theme,
                )
            })
            .collect::<Vec<_>>();

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we always want to start from the top.
        *table_state.offset_mut() = 0;

        // Every group renders its own table, but the selected index of the state counts the rows
        // of all the groups. Translate it to the position of the selected collator in this group,
        // or clear it when the selection is in another group, so only the right row is highlighted.
        table_state.select(
            collators
                .iter()
                .position(|item| Some(*item) == selected_collator),
        );

        let block = Block::new().set_style(theme.block.alt(is_odd));
        let table = Table::new(rows, widths)
            .block(block)
            .row_highlight_style(theme.table.row_highlight(is_active))
            .header(header.set_style(theme.table.header));

        StatefulWidget::render(table, area, buf, table_state);
    }

    #[allow(clippy::too_many_arguments)]
    fn collator_row(
        &self,
        collator: &Collator,
        selected: Option<&Collator>,
        columns: &Columns,
        aura: &Aura,
        current_slot: u64,
        current_slot_ts: u128,
        theme: Theme,
    ) -> Row<'static> {
        let (cell_style, _highlight_symbol) = match selected {
            Some(selected) if collator == selected => (theme.paragraph.cell_active, "❯"),
            _ => (theme.paragraph.cell, ""),
        };

        let authorities = aura.authorities();

        let is_current_author = collator.is_current_slot_author(authorities, current_slot);

        let mut cells = vec![
            Cell::from(Text::from(collator.origin.symbol()).style(theme.paragraph.label(false))),
            Cell::from(Text::from(collator.status().to_string()).alignment(Alignment::Left)),
            Cell::from(Text::from(collator.display_identity()).alignment(Alignment::Left))
                .style(cell_style),
        ];

        if columns.host {
            cells.push(Cell::from(
                Text::from(collator.host(self.masked)).alignment(Alignment::Left),
            ));
        }

        if columns.deposit {
            if collator.is_permissionless_or_candidate() {
                cells.push(Cell::from(
                    Line::from(vec![
                        Span::raw(format_planks(
                            collator.deposit(),
                            collator.account.token_decimals(),
                            4,
                        )),
                        Span::raw(collator.account.token_symbol())
                            .style(theme.paragraph.label(false)),
                    ])
                    .alignment(Alignment::Right),
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        if columns.last_block {
            let last_block_str = collator
                .last_block_authored()
                .map(|b| format!("#{b}"))
                .unwrap_or_default();

            let last_block_ago_str = collator
                .last_block_authored_ts
                .map(|ts| format_millis(get_elapsed_millis(ts), false, false))
                .unwrap_or_default();

            cells.push(Cell::from(
                Text::from(last_block_str).alignment(Alignment::Right),
            ));
            cells.push(Cell::from(
                Text::from(last_block_ago_str).alignment(Alignment::Left),
            ));
        }

        if columns.next_slot {
            if is_current_author {
                cells.push(Cell::from(
                    Text::from(current_frame())
                        .style(theme.paragraph.label(false))
                        .alignment(Alignment::Right),
                ));
            } else {
                let next_slot_countdown_str = match aura.slot_duration_ms() {
                    Some(slot_duration_ms) => collator
                        .next_slot_countdown(
                            authorities,
                            current_slot,
                            slot_duration_ms,
                            current_slot_ts,
                        )
                        .unwrap_or_default(),
                    None => "".to_string(),
                };
                cells.push(Cell::from(
                    Text::from(next_slot_countdown_str).alignment(Alignment::Right),
                ));
            }

            // Blocks in slot is only meaningful while the collator is the current author
            let blocks_in_slot_str = if is_current_author {
                let blocks = collator.blocks_in_slot(current_slot);
                match aura.number_blocks_expected() {
                    Some(expected) => format!("{blocks:2}/{expected}"),
                    None => format!("{blocks:2}"),
                }
            } else {
                String::new()
            };
            cells.push(Cell::from(
                Text::from(blocks_in_slot_str).alignment(Alignment::Left),
            ));
        }

        if columns.keys {
            cells.push(Cell::from(
                Text::from(collator.display_queued_keys(6)).alignment(Alignment::Right),
            ));
        }

        if columns.next_keys {
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

/// The columns of a group of collators, each one visible according to its feature flag.
pub(crate) struct Columns {
    /// The chain of the collator, for views that mix the collators of several chains
    pub(crate) chain: bool,
    pub(crate) host: bool,
    pub(crate) deposit: bool,
    pub(crate) last_block: bool,
    pub(crate) next_slot: bool,
    pub(crate) keys: bool,
    pub(crate) next_keys: bool,
}

impl Columns {
    pub(crate) fn new(collators: &[&Collator], feat: &Features, chain: bool) -> Self {
        Self {
            chain,
            host: collators.iter().any(|c| c.has_commands_available()),
            deposit: feat.deposit_visible(),
            last_block: feat.last_block_visible(),
            next_slot: feat.next_slot_visible(),
            keys: feat.keys_visible(),
            next_keys: feat.keys_visible() && collators.iter().any(|c| c.is_next_keys_changed()),
        }
    }

    pub(crate) fn header(&self) -> Row<'static> {
        let mut cells = vec![
            // The origin of the collator
            Cell::from(""),
            // The status of the collator
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
        ];

        if self.chain {
            cells.push(Cell::from(Text::from("chain").alignment(Alignment::Left)));
        }

        cells.push(Cell::from(
            Text::from("identity").alignment(Alignment::Left),
        ));

        if self.host {
            cells.push(Cell::from(Text::from("host").alignment(Alignment::Left)));
        }

        if self.deposit {
            cells.push(Cell::from(
                Text::from("deposit").alignment(Alignment::Right),
            ));
        }

        if self.last_block {
            cells.push(Cell::from(
                Text::from("last block").alignment(Alignment::Right),
            ));
            cells.push(Cell::from(Text::from("(ago)").alignment(Alignment::Left)));
        }

        if self.next_slot {
            cells.push(Cell::from(
                Text::from("next slot").alignment(Alignment::Right),
            ));
            cells.push(Cell::from(
                Text::from("(blocks)").alignment(Alignment::Left),
            ));
        }

        if self.keys {
            cells.push(Cell::from(Text::from("keys").alignment(Alignment::Right)));
        }

        if self.next_keys {
            cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }

        Row::new(cells)
    }

    pub(crate) fn widths(&self) -> Vec<Constraint> {
        let mut widths = vec![Constraint::Length(1), Constraint::Length(3)];

        if self.chain {
            widths.push(Constraint::Length(12));
        }

        widths.push(Constraint::Length(24));

        if self.host {
            widths.push(Constraint::Fill(2));
        }

        if self.deposit {
            widths.push(Constraint::Fill(1));
        }

        if self.last_block {
            // last block and (ago)
            widths.push(Constraint::Fill(2));
            widths.push(Constraint::Fill(1));
        }

        if self.next_slot {
            // next slot and (blocks)
            widths.push(Constraint::Fill(2));
            widths.push(Constraint::Fill(1));
        }

        if self.keys {
            widths.push(Constraint::Fill(2));
        }

        if self.next_keys {
            widths.push(Constraint::Length(10));
        }

        widths
    }
}
