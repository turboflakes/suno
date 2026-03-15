use crate::theme::THEME;
use crate::widgets::chains::ChainsListWidget;
use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::validators::ValidatorsListState;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled},
    text::{Line, Span, Text},
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};
use suno_config::SupportedRuntime;
use suno_primitives::{
    display::{create_progress_bar_by_blocks, format_planks},
    validator::Validator,
};

pub const GROUP_HEADER_HEIGHT: u16 = 6;
pub const PADDING: u16 = 4;

#[derive(Debug, Clone)]
pub struct ValidatorsDetailedGroupWidget<'a> {
    pub state: Arc<RwLock<ValidatorsListState>>,
    pub chains: &'a ChainsListWidget,
}

/// Validators grouped view widget implementation, mostly to be used under the validators main tab view
impl<'a> Widget for &ValidatorsDetailedGroupWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();
        state.set_viewport_height(area.height);
        let validators_grouped = state.get_validators_grouped_by_runtime();
        let total_height = state.total_detailed_group_height();
        let is_scroll_visible = state.is_active && area.height < total_height;
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
                Some(selected) if validators.contains(&selected) && state.is_active => {
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
        if state.is_active && area.height < total_height {
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
    fn render_group(
        &self,
        runtime: SupportedRuntime,
        validators: &[&Validator],
        selected_validator: Option<&Validator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
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
        );
    }

    fn render_table_header(
        &self,
        runtime: SupportedRuntime,
        validators: &[&Validator],
        area: Rect,
        buf: &mut Buffer,
    ) {
        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };

        let Some(ah_chain) = self
            .chains
            .get_chain_by_runtime(runtime.asset_hub_runtime())
        else {
            return;
        };

        let [network_area, progress_area, progress_bar_area, countdown_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),    // Network info
                Constraint::Fill(2),    // Era / Session info bar
                Constraint::Length(24), // Era / Session progress bar
                Constraint::Length(16), // Countdown
            ])
            .areas(area);

        // Draw and render general network stats

        let network_lines = vec![
            Line::from(
                Span::raw(format!("{} NETWORK", runtime.to_string().to_uppercase()))
                    .style(THEME.paragraph.header),
            ),
            Line::from(vec![
                Span::raw("total validators: ").style(THEME.paragraph.label),
                Span::raw(format!(
                    "{} active, {} waiting",
                    ah_chain.active_validators_count(),
                    ah_chain.waiting_validators_count()
                )),
            ]),
            Line::from(vec![
                Span::raw("total nominators: ").style(THEME.paragraph.label),
                Span::raw(format!(
                    "{} active, {} waiting",
                    ah_chain.active_nominators_count(),
                    ah_chain.waiting_nominators_count()
                )),
            ]),
            Line::from(vec![
                Span::raw("total staked: ").style(THEME.paragraph.label),
                Span::raw(ah_chain.total_staked_percentage()),
            ]),
            Line::from(vec![
                Span::raw("displayed: ").style(THEME.paragraph.label),
                Span::raw(format!(
                    "{} active, {} waiting",
                    validators.iter().filter(|v| v.is_active()).count(),
                    validators.iter().filter(|v| v.is_waiting()).count(),
                )),
            ]),
        ];

        let block = Block::new().set_style(THEME.block.main);
        let network_info = Paragraph::new(network_lines)
            .block(block)
            .style(THEME.paragraph.base);

        network_info.render(network_area, buf);

        // Draw and render Progress Info

        let Some(epoch) = chain.epoch() else {
            // TODO: Handle epoch not available, maybe render loading indicator
            return;
        };

        let epoch_progress = epoch.progress(chain.finalized_block());

        let Some(era) = ah_chain.era() else {
            // TODO: Handle era not available, maybe render loading indicator
            return;
        };

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

        let block = Block::new().set_style(THEME.block.main);
        let progress_info = Paragraph::new(progress_lines)
            .block(block)
            .style(THEME.paragraph.base);

        progress_info.render(progress_area, buf);

        // Draw and render Progress Bars
        let epoch_progress_bar = create_progress_bar_by_blocks(epoch_progress, 24);
        let era_progress_bar = create_progress_bar_by_blocks(era_progress, 24);

        let progress_bar_lines = vec![
            Line::from(""),
            Line::from(era_progress_bar).alignment(Alignment::Right),
            Line::from(epoch_progress_bar).alignment(Alignment::Right),
        ];

        let block = Block::new().set_style(THEME.block.main);
        let progress_bar = Paragraph::new(progress_bar_lines)
            .block(block)
            .style(THEME.paragraph.base);

        progress_bar.render(progress_bar_area, buf);

        // Draw and render Countdowns

        let epoch_countdown_time = epoch.countdown_time(chain.finalized_block());
        let era_countdown_time = era.countdown_time(epoch.duration(), epoch.block_time_ms());

        let countdown_lines = vec![
            Line::from(""),
            Line::from(format!(" {}", era_countdown_time,)).alignment(Alignment::Left),
            Line::from(format!(" {}", epoch_countdown_time,)).alignment(Alignment::Left),
        ];

        let block = Block::new().set_style(THEME.block.main);
        let countdown_info = Paragraph::new(countdown_lines)
            .block(block)
            .style(THEME.paragraph.base);

        countdown_info.render(countdown_area, buf);
    }

    fn render_table_body(
        &self,
        runtime: SupportedRuntime,
        validators: &[&Validator],
        selected_validator: Option<&Validator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let Some(ah_chain) = self
            .chains
            .get_chain_by_runtime(runtime.asset_hub_runtime())
        else {
            return;
        };

        let Some(active_era) = ah_chain.era() else {
            return;
        };

        let show_bonded = validators.iter().any(|v| v.self_stake() != v.bounded());

        let show_unlocking = validators
            .iter()
            .any(|v| v.unlocking(active_era.index()) > 0);

        let show_unlocked = validators
            .iter()
            .any(|v| v.unlocked(active_era.index()) > 0);

        let show_next_commission = validators.iter().any(|v| v.is_commission_changed());

        let show_next_keys = validators.iter().any(|v| v.is_next_keys_changed());

        let mut rows = Vec::new();

        let span_symbol = Span::raw(runtime.token_symbol()).style(THEME.paragraph.label);

        for v in validators {
            let text_points = match v.delta_points() {
                Some(d) => Text::from(format!("+{}", d)).style(Style::default().fg(Color::White)),
                None => Text::from(v.total_points().to_string()),
            };

            let decimals = v.runtime().token_decimals();
            let staked_total = if v.is_active() { v.stake.total() } else { 0 };

            let (cell_style, _highlight_symbol) = match selected_validator {
                Some(ref selected) if v == selected => {
                    (Style::default().fg(Color::Black).bg(Color::White), "❯")
                }
                _ => (Style::default(), ""),
            };

            let mut validator_cells = vec![
                Cell::from(Text::from(format!("{}", v.status())).alignment(Alignment::Left)),
                Cell::from(Text::from(v.display_identity()).alignment(Alignment::Left))
                    .style(cell_style),
                Cell::from(text_points.alignment(Alignment::Right)),
                Cell::from(
                    Line::from(vec![
                        Span::raw(format_planks(staked_total, decimals, 4)),
                        span_symbol.clone(),
                    ])
                    .alignment(Alignment::Right),
                ),
                Cell::from(
                    Line::from(vec![
                        Span::raw(format_planks(v.self_stake(), decimals, 4)),
                        span_symbol.clone(),
                    ])
                    .alignment(Alignment::Right),
                ),
            ];

            if show_bonded {
                if v.self_stake() != v.bounded() {
                    validator_cells.push(Cell::from(
                        Line::from(vec![
                            Span::raw(format_planks(v.bounded(), decimals, 4)),
                            span_symbol.clone(),
                        ])
                        .alignment(Alignment::Left),
                    ));
                } else {
                    validator_cells.push(Cell::from(Text::from("")));
                }
            }

            if show_unlocking {
                let unlocking: u128 = v.unlocking(active_era.index());

                if unlocking > 0 {
                    validator_cells.push(Cell::from(
                        Line::from(vec![
                            Span::raw(format_planks(unlocking, decimals, 4)),
                            span_symbol.clone(),
                        ])
                        .alignment(Alignment::Right),
                    ));
                } else {
                    validator_cells.push(Cell::from(Text::from("")));
                }
            }

            if show_unlocked {
                let unlocked: u128 = v.unlocked(active_era.index());

                if unlocked > 0 {
                    validator_cells.push(Cell::from(
                        Line::from(vec![
                            Span::raw(format_planks(unlocked, decimals, 4)),
                            span_symbol.clone(),
                        ])
                        .alignment(Alignment::Right),
                    ));
                } else {
                    validator_cells.push(Cell::from(Text::from("")));
                }
            }

            validator_cells.push(Cell::from(
                Text::from(v.stake.nominators_count().to_string()).alignment(Alignment::Right),
            ));

            validator_cells.push(Cell::from(
                Text::from(v.commission_as_percentage(2)).alignment(Alignment::Right),
            ));

            if show_next_commission {
                if v.is_commission_changed() {
                    validator_cells.push(Cell::from(
                        Text::from(v.next_commission_as_percentage(2)).alignment(Alignment::Left),
                    ));
                } else {
                    validator_cells.push(Cell::from(Text::from("")));
                }
            }

            validator_cells.push(Cell::from(
                Text::from(v.payee_as_compact(3)).alignment(Alignment::Right),
            ));

            validator_cells.push(Cell::from(
                Text::from(v.display_queued_keys(6)).alignment(Alignment::Right),
            ));

            if show_next_keys && v.is_next_keys_changed() {
                validator_cells.push(Cell::from(
                    Text::from(v.display_next_keys(6)).alignment(Alignment::Left),
                ));
            }

            // if selected_validator.is_some() {
            //     validator_cells.insert(
            //         1,
            //         Cell::from(
            //             Text::from(format!("{}", highlight_symbol)).alignment(Alignment::Left),
            //         )
            //         .style(cell_style),
            //     );
            // }
            rows.push(Row::new(validator_cells));
        }

        let mut widths = vec![
            Constraint::Length(3),
            Constraint::Length(24),
            Constraint::Fill(2),
            Constraint::Fill(2),
            Constraint::Fill(2),
            Constraint::Fill(2),
            Constraint::Fill(2),
        ];

        if show_bonded {
            widths.push(Constraint::Fill(2));
        }

        if show_unlocking {
            widths.push(Constraint::Fill(2));
        }

        if show_unlocked {
            widths.push(Constraint::Fill(2));
        }

        if show_next_commission {
            widths.push(Constraint::Fill(1));
        }

        // payee
        widths.push(Constraint::Fill(2));

        // queued_keys
        widths.push(Constraint::Fill(2));

        if show_next_keys {
            widths.push(Constraint::Length(10));
        }

        let mut header_cells = vec![
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
            Cell::from(Text::from("identity").alignment(Alignment::Left)),
            Cell::from(Text::from("points").alignment(Alignment::Right)),
            Cell::from(Text::from("total").alignment(Alignment::Right)),
            Cell::from(Text::from("own-stake").alignment(Alignment::Right)),
        ];

        if show_bonded {
            header_cells.push(Cell::from(
                Text::from("(bonded)").alignment(Alignment::Left),
            ));
        }

        if show_unlocking {
            header_cells.push(Cell::from(
                Text::from("unlocking").alignment(Alignment::Right),
            ));
        }

        if show_unlocked {
            header_cells.push(Cell::from(
                Text::from("unlocked").alignment(Alignment::Right),
            ));
        }

        header_cells.push(Cell::from(
            Text::from("nominators").alignment(Alignment::Right),
        ));
        header_cells.push(Cell::from(
            Text::from("commission").alignment(Alignment::Right),
        ));

        if show_next_commission {
            header_cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }

        header_cells.push(Cell::from(Text::from("payee").alignment(Alignment::Right)));

        header_cells.push(Cell::from(Text::from("keys").alignment(Alignment::Right)));

        if show_next_keys {
            header_cells.push(Cell::from(Text::from("(next)").alignment(Alignment::Left)));
        }

        // Note: If selected validator is in this group, add a column for the highlight symbol
        // if selected_validator.is_some() {
        //     widths.insert(1, Constraint::Length(1));
        //     header_cells.insert(1, Cell::from(Text::from("")));
        // };
        //

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we alwasy want to start from the top.
        *table_state.offset_mut() = 0;

        let block = Block::new().set_style(THEME.block.main);
        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(THEME.table.header))
            .style(Style::default().fg(Color::Blue));

        StatefulWidget::render(table, area, buf, table_state);
    }
}
