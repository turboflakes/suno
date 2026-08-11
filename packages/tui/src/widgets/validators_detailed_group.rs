use crate::widgets::chains::ChainsList;
use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::validators::ValidatorsList;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Styled},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use suno_config::{Features, SupportedRuntime, CONFIG};
use suno_primitives::{
    display::{create_progress_bar_by_blocks, format_planks},
    validator::Validator,
};
use suno_theme::Theme;

pub const GROUP_HEADER_HEIGHT: u16 = 6;
pub const PADDING: u16 = 4;

#[derive(Debug)]
pub struct ValidatorsDetailedGroupWidget<'a> {
    pub chains: &'a ChainsList,
}

impl<'a> ValidatorsDetailedGroupWidget<'a> {
    pub fn new(chains: &'a ChainsList) -> Self {
        Self { chains }
    }
}

/// Validators grouped view widget implementation, mostly to be used under the validators main tab view
impl<'a> StatefulWidget for ValidatorsDetailedGroupWidget<'a> {
    type State = ValidatorsList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.set_viewport_height(area.height);
        let validators_grouped = state.get_validators_grouped_by_runtime();
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
        for (runtime, validators) in validators_grouped {
            let group_height = GROUP_HEADER_HEIGHT + validators.len() as u16 + PADDING;
            let group_area = Rect::new(0, current_y_group, area_width, group_height);

            // Get selected validator if one of the validators in the current section
            let selected_validator = match state.get_selected_ref() {
                Some(selected) if validators.contains(&selected) && state.is_active() => {
                    Some(selected)
                }
                _ => None,
            };

            self.render_group(
                runtime,
                &validators,
                selected_validator,
                group_area,
                &mut full_content_buf,
                &mut state.table_state.clone(),
                state.is_masked(),
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
        if state.is_active() && area.height < total_height {
            let selected_pos = state.table_state.selected().unwrap_or_default();

            let scrollbar_area = Rect {
                x: area.right().saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            render_scrollbar(
                state.scroll_offset as usize + selected_pos,
                total_height as usize,
                scrollbar_area,
                buf,
            );
        }
    }
}

impl<'a> ValidatorsDetailedGroupWidget<'a> {
    #[allow(clippy::too_many_arguments)]
    fn render_group(
        &self,
        runtime: SupportedRuntime,
        validators: &[&Validator],
        selected_validator: Option<&Validator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
        is_masked: bool,
    ) {
        // Split area into header and body
        let [header_area, body_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(GROUP_HEADER_HEIGHT), // Header height
                Constraint::Min(0),                      // Body takes remaining
            ])
            .areas(area);

        // Render network header
        self.render_table_header(runtime, validators, header_area, buf);

        // Render network validators table
        self.render_table_body(
            runtime,
            validators,
            selected_validator,
            body_area,
            buf,
            table_state,
            is_masked,
        );
    }

    fn render_table_header(
        &self,
        runtime: SupportedRuntime,
        validators: &[&Validator],
        area: Rect,
        buf: &mut Buffer,
    ) {
        let theme = CONFIG.theme();
        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let Some(ah_chain) = self
            .chains
            .get_chain_by_runtime(runtime.asset_hub_runtime())
        else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let [network_area, progress_area, progress_bar_area, countdown_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),    // Network info
                Constraint::Length(16), // Era / Session info bar
                Constraint::Length(24), // Era / Session progress bar
                Constraint::Length(16), // Countdown
            ])
            .areas(area);

        // Draw and render general network stats

        let network_lines = vec![
            Line::from(
                Span::raw(format!("{} NETWORK", runtime.to_string().to_uppercase()))
                    .style(theme.paragraph.header_active),
            ),
            Line::from(vec![
                Span::raw("Total validators ").style(theme.paragraph.label),
                Span::raw(format!(
                    "{} active, {} waiting",
                    ah_chain.active_validators_count(),
                    ah_chain.waiting_validators_count()
                )),
            ]),
            Line::from(vec![
                Span::raw("Total nominators ").style(theme.paragraph.label),
                Span::raw(format!(
                    "{} active, {} waiting",
                    ah_chain.active_nominators_count(),
                    ah_chain.waiting_nominators_count()
                )),
            ]),
            Line::from(vec![
                Span::raw("Total staked ").style(theme.paragraph.label),
                Span::raw(ah_chain.total_staked_percentage()),
            ]),
            Line::from(vec![
                Span::raw("Displayed ").style(theme.paragraph.label),
                Span::raw(format!(
                    "{} active, {} waiting",
                    validators.iter().filter(|v| v.is_active()).count(),
                    validators.iter().filter(|v| v.is_waiting()).count(),
                )),
            ]),
        ];

        let block = Block::new().set_style(theme.block.main);
        let network_info = Paragraph::new(network_lines)
            .block(block)
            .style(theme.paragraph.base);

        network_info.render(network_area, buf);

        // Draw and render Progress Info

        let Some(epoch) = chain.epoch() else {
            // TODO: Handle epoch not available, maybe render loading indicator
            let block = Block::new().set_style(theme.block.main);
            let area = progress_area.union(progress_bar_area).union(countdown_area);
            block.render(area, buf);
            return;
        };

        let Some(era) = ah_chain.era() else {
            // TODO: Handle era not available, maybe render loading indicator
            let block = Block::new().set_style(theme.block.main);
            let area = progress_area.union(progress_bar_area).union(countdown_area);
            block.render(area, buf);
            return;
        };

        let epoch_progress = epoch.progress(chain.finalized_block());
        let era_progress = era.progress(epoch.duration(), epoch.block_time_ms());

        let progress_lines = vec![
            Line::from(""),
            Line::from(format!(
                "era {} {:.0}% ",
                era.index(),
                era_progress * 100_f64,
            ))
            .alignment(Alignment::Right),
            Line::from(format!(
                "epoch {} {:.0}% ",
                epoch.index(),
                epoch_progress * 100_f64,
            ))
            .alignment(Alignment::Right),
        ];

        let block = Block::new().set_style(theme.block.main);
        let progress_info = Paragraph::new(progress_lines)
            .block(block)
            .style(theme.paragraph.base);

        progress_info.render(progress_area, buf);

        // Draw and render Progress Bars
        let epoch_progress_bar = create_progress_bar_by_blocks(epoch_progress, 24);
        let era_progress_bar = create_progress_bar_by_blocks(era_progress, 24);

        let progress_bar_lines = vec![
            Line::from(""),
            Line::from(era_progress_bar).alignment(Alignment::Right),
            Line::from(epoch_progress_bar).alignment(Alignment::Right),
        ];

        let block = Block::new().set_style(theme.block.main);
        let progress_bar = Paragraph::new(progress_bar_lines)
            .block(block)
            .style(theme.paragraph.base);

        progress_bar.render(progress_bar_area, buf);

        // Draw and render Countdowns

        let epoch_countdown_time = epoch.countdown_time(chain.finalized_block());
        let era_countdown_time = era.countdown_time(epoch.duration(), epoch.block_time_ms());

        let countdown_lines = vec![
            Line::from(""),
            Line::from(format!(" {}", era_countdown_time,)).alignment(Alignment::Left),
            Line::from(format!(" {}", epoch_countdown_time,)).alignment(Alignment::Left),
        ];

        let block = Block::new().set_style(theme.block.main);
        let countdown_info = Paragraph::new(countdown_lines)
            .block(block)
            .style(theme.paragraph.base);

        countdown_info.render(countdown_area, buf);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_table_body(
        &self,
        runtime: SupportedRuntime,
        validators: &[&Validator],
        selected_validator: Option<&Validator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
        is_masked: bool,
    ) {
        let theme = CONFIG.theme();
        let features = CONFIG.features();
        let Some(ah_chain) = self
            .chains
            .get_chain_by_runtime(runtime.asset_hub_runtime())
        else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let Some(era) = ah_chain.era() else {
            let block = Block::new().set_style(theme.block.main);
            block.render(area, buf);
            return;
        };

        let symbol = Span::raw(runtime.token_symbol()).style(theme.paragraph.label(false));

        let columns = Columns::new(validators, era.index(), features);

        let widths = columns.widths();

        let header = columns.header();

        let rows = self.build_validator_rows(
            validators,
            selected_validator,
            &columns,
            era.index(),
            is_masked,
            theme,
            &symbol,
        );

        // Note: If selected validator is in this group, add a column for the highlight symbol
        // if selected_validator.is_some() {
        //     widths.insert(1, Constraint::Length(1));
        //     header_cells.insert(1, Cell::from(Text::from("")));
        // };
        //

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we alwasy want to start from the top.
        *table_state.offset_mut() = 0;

        let block = Block::new().set_style(theme.block.main);
        let table = Table::new(rows, widths)
            .block(block)
            .header(header.set_style(theme.table.header));

        StatefulWidget::render(table, area, buf, table_state);
    }

    #[allow(clippy::too_many_arguments)]
    fn build_validator_rows(
        &self,
        validators: &[&Validator],
        selected: Option<&Validator>,
        columns: &Columns,
        era: u32,
        masked: bool,
        theme: &Theme,
        symbol: &Span<'static>,
    ) -> Vec<Row<'static>> {
        validators
            .iter()
            .map(|v| self.validator_row(v, selected, columns, era, masked, theme, symbol))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn validator_row(
        &self,
        validator: &Validator,
        selected: Option<&Validator>,
        columns: &Columns,
        era: u32,
        masked: bool,
        theme: &Theme,
        symbol: &Span<'static>,
    ) -> Row<'static> {
        Row::new(self.validator_cells(validator, selected, columns, era, masked, theme, symbol))
    }

    #[allow(clippy::too_many_arguments)]
    fn validator_cells(
        &self,
        validator: &Validator,
        selected: Option<&Validator>,
        columns: &Columns,
        era: u32,
        masked: bool,
        theme: &Theme,
        symbol: &Span<'static>,
    ) -> Vec<Cell<'static>> {
        let decimals = validator.runtime().token_decimals();

        let (cell_style, _highlight_symbol) = match selected {
            Some(selected) if validator == selected => (theme.paragraph.cell_active, "❯"),
            _ => (theme.paragraph.cell, ""),
        };

        let mut cells = vec![
            Cell::from(Text::from(validator.status().to_string()).alignment(Alignment::Left)),
            Cell::from(Text::from(validator.display_identity()).alignment(Alignment::Left))
                .style(cell_style),
        ];

        if columns.host {
            cells.push(Cell::from(
                Text::from(validator.host(masked)).alignment(Alignment::Left),
            ));
        }

        let points = match validator.delta_points() {
            Some(d) => {
                Text::from(format!("+{}", d)).style(Style::default().add_modifier(Modifier::BOLD))
            }
            None => Text::from(validator.total_points().to_string()),
        };
        cells.push(Cell::from(points.alignment(Alignment::Right)));

        if columns.nominators_counter {
            cells.push(Cell::from(
                Text::from(validator.stake.nominators_count().to_string())
                    .alignment(Alignment::Right),
            ));
        }

        if columns.nominators_stake {
            let staked_total = if validator.is_active() {
                validator.stake.total()
            } else {
                0
            };
            cells.push(amount_cell(
                staked_total,
                decimals,
                symbol.clone(),
                Alignment::Right,
            ));
        }

        if columns.own_stake {
            cells.push(amount_cell(
                validator.self_stake(),
                decimals,
                symbol.clone(),
                Alignment::Right,
            ));
        }

        if columns.bonded {
            if validator.self_stake() != validator.bounded() {
                cells.push(amount_cell(
                    validator.bounded(),
                    decimals,
                    symbol.clone(),
                    Alignment::Left,
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        if columns.unlocking {
            let unlocking: u128 = validator.unlocking(era);

            if unlocking > 0 {
                cells.push(amount_cell(
                    unlocking,
                    decimals,
                    symbol.clone(),
                    Alignment::Right,
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        if columns.unlocked {
            let unlocked: u128 = validator.unlocked(era);

            if unlocked > 0 {
                cells.push(amount_cell(
                    unlocked,
                    decimals,
                    symbol.clone(),
                    Alignment::Right,
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        if columns.commission {
            cells.push(Cell::from(
                Text::from(validator.commission_as_percentage(2)).alignment(Alignment::Right),
            ));
        }

        if columns.next_commission {
            if validator.is_commission_changed() {
                cells.push(Cell::from(
                    Text::from(validator.next_commission_as_percentage(2))
                        .alignment(Alignment::Left),
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        if columns.payee {
            cells.push(Cell::from(
                Text::from(validator.payee_as_compact(3)).alignment(Alignment::Right),
            ));
        }

        if columns.keys {
            cells.push(Cell::from(
                Text::from(validator.display_queued_keys(6)).alignment(Alignment::Right),
            ));
        }

        if columns.next_keys {
            if validator.is_next_keys_changed() {
                cells.push(Cell::from(
                    Text::from(validator.display_next_keys(6)).alignment(Alignment::Left),
                ));
            } else {
                cells.push(Cell::from(Text::from("")));
            }
        }

        cells
    }
}

struct Columns {
    host: bool,
    nominators_counter: bool,
    nominators_stake: bool,
    own_stake: bool,
    bonded: bool,
    unlocking: bool,
    unlocked: bool,
    commission: bool,
    next_commission: bool,
    payee: bool,
    keys: bool,
    next_keys: bool,
}

impl Columns {
    fn new(val: &[&Validator], era: u32, feat: &Features) -> Self {
        Self {
            host: val.iter().any(|v| !v.commands.is_empty()),
            nominators_counter: feat.nominators_counter_visible(),
            nominators_stake: feat.nominators_stake_visible(),
            own_stake: feat.own_stake_visible(),
            bonded: feat.own_stake_visible() && val.iter().any(|v| v.self_stake() != v.bounded()),
            unlocking: feat.own_stake_visible() && val.iter().any(|v| v.unlocking(era) > 0),
            unlocked: feat.own_stake_visible() && val.iter().any(|v| v.unlocked(era) > 0),
            commission: feat.commission_visible(),
            next_commission: feat.commission_visible()
                && val.iter().any(|v| v.is_commission_changed()),
            payee: feat.payee_visible(),
            keys: feat.keys_visible(),
            next_keys: feat.keys_visible() && val.iter().any(|v| v.is_next_keys_changed()),
        }
    }

    fn header(&self) -> Row<'static> {
        let mut cells = vec![
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
            Cell::from(Text::from("identity").alignment(Alignment::Left)),
        ];

        if self.host {
            cells.push(Cell::from(Text::from("host").alignment(Alignment::Left)));
        }

        cells.push(Cell::from(Text::from("points").alignment(Alignment::Right)));

        if self.nominators_counter {
            cells.push(Cell::from(Text::from("noms").alignment(Alignment::Right)));
        }

        if self.nominators_stake {
            cells.push(Cell::from(
                Text::from("noms-stake").alignment(Alignment::Right),
            ));
        }

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

        if self.commission {
            cells.push(Cell::from(
                Text::from("commission").alignment(Alignment::Right),
            ));
        }

        if self.next_commission {
            cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }

        if self.payee {
            cells.push(Cell::from(Text::from("payee").alignment(Alignment::Right)));
        }

        if self.keys {
            cells.push(Cell::from(Text::from("keys").alignment(Alignment::Right)));
        }

        if self.next_keys {
            cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }

        Row::new(cells)
    }

    fn widths(&self) -> Vec<Constraint> {
        let mut widths = vec![Constraint::Length(3), Constraint::Length(24)];

        if self.host {
            widths.push(Constraint::Fill(2));
        }

        // points
        widths.push(Constraint::Fill(2));

        if self.nominators_counter {
            widths.push(Constraint::Fill(2));
        }

        if self.nominators_stake {
            widths.push(Constraint::Fill(2));
        }

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

        if self.commission {
            widths.push(Constraint::Fill(2));
        }

        if self.next_commission {
            widths.push(Constraint::Fill(1));
        }

        if self.payee {
            widths.push(Constraint::Fill(2));
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
