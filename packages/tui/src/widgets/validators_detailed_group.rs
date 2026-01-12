use crate::theme::THEME;
use crate::widgets::chains::ChainsListWidget;
use crate::widgets::scrollbar::render_scrollbar;
use crate::widgets::validators::ValidatorsListState;
use log::info;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style, Styled},
    text::{Line, Text},
    widgets::{Cell, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};
use suno_config::SupportedRuntime;
use suno_primitives::{
    display::{create_progress_bar_by_blocks, format_planks},
    validator::Validator,
};

pub const GROUP_HEADER_HEIGHT: u16 = 6;
pub const BOTTOM_PADDING: u16 = 2;

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
            let group_height = GROUP_HEADER_HEIGHT + validators.len() as u16 + BOTTOM_PADDING;
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
                validators,
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
                    target_cell.set_symbol(&source_cell.symbol());
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
                ..area
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
        validators: Vec<&Validator>,
        selected_validator: Option<&Validator>,
        group_area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        // Split area into header and body
        let group_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(GROUP_HEADER_HEIGHT), // Header height
                Constraint::Min(0),                      // Body takes remaining
            ])
            .split(group_area);

        // Render network header
        self.render_table_header(runtime, validators.clone(), group_area[0], buf);

        // Render network validators table
        self.render_table_body(
            validators,
            selected_validator,
            group_area[1],
            buf,
            table_state,
        );
    }

    fn render_table_header(
        &self,
        runtime: SupportedRuntime,
        validators: Vec<&Validator>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        if let Some(chain) = self.chains.get_chain_by_runtime(&runtime) {
            if let Some(ah_chain) = self
                .chains
                .get_chain_by_runtime(&runtime.asset_hub_runtime())
            {
                let header_layout_cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(82), // Network info
                        Constraint::Fill(1),    // Era / Session progress bar
                        Constraint::Length(16), // Countdown
                    ])
                    .split(area);

                let network_info = Paragraph::new(vec![
                    Line::from(format!("# {}", runtime))
                        .style(Style::default().fg(Color::Blue).bold()),
                    Line::from(format!(
                        "Total validators: {} active, {} waiting",
                        ah_chain.active_validators_count(),
                        ah_chain.waiting_validators_count(),
                    )),
                    Line::from(format!(
                        "Total nominators: {} active, {} waiting",
                        ah_chain.active_nominators_count(),
                        ah_chain.waiting_nominators_count()
                    )),
                    Line::from(format!(
                        "Total staked: {}",
                        ah_chain.total_staked_percentage()
                    )),
                    Line::from(format!(
                        "On display: {} active, {} waiting",
                        validators.iter().filter(|v| v.is_active()).count(),
                        validators.iter().filter(|v| v.is_waiting()).count(),
                    )),
                ])
                .style(Style::default().fg(Color::Blue));

                network_info.render(header_layout_cols[0], buf);

                let Some(epoch) = chain.epoch() else {
                    // TODO: Handle epoch not available, maybe render loading indicator
                    return;
                };

                let epoch_progress = epoch.progress(chain.finalized_block());
                let epoch_progress_bar = create_progress_bar_by_blocks(epoch_progress, 24);

                let Some(era) = ah_chain.era() else {
                    // TODO: Handle era not available, maybe render loading indicator
                    return;
                };

                let era_progress = era.progress(epoch.duration(), epoch.block_time_ms());
                let era_progress_bar = create_progress_bar_by_blocks(era_progress, 24);

                let progress_info = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(format!(
                        "era {} {:.0}% {}",
                        era.index(),
                        era_progress * 100 as f64,
                        era_progress_bar
                    ))
                    .alignment(Alignment::Right),
                    Line::from(format!(
                        "epoch {} {:.0}% {}",
                        epoch.index(),
                        epoch_progress * 100 as f64,
                        epoch_progress_bar,
                    ))
                    .alignment(Alignment::Right),
                ])
                .style(Style::default().fg(Color::Blue));

                progress_info.render(header_layout_cols[1], buf);

                let epoch_countdown_time = epoch.countdown_time(chain.finalized_block());
                let era_countdown_time =
                    era.countdown_time(epoch.duration(), epoch.block_time_ms());

                let countdown_info = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(format!(" {}", era_countdown_time,)).alignment(Alignment::Left),
                    Line::from(format!(" {}", epoch_countdown_time,)).alignment(Alignment::Left),
                ])
                .style(Style::default().fg(Color::Blue));

                countdown_info.render(header_layout_cols[2], buf);
            };
        };
    }

    fn render_table_body(
        &self,
        validators: Vec<&Validator>,
        selected_validator: Option<&Validator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let mut rows = Vec::new();

        for v in validators {
            let text_points = match v.delta_points() {
                Some(d) => Text::from(format!("+{}", d)).style(Style::default().fg(Color::White)),
                None => Text::from(v.total_points().to_string()),
            };

            let decimals = v.runtime().token_decimals();
            let staked_total = if v.is_active() { v.stake.total() } else { 0 };
            let staked_own = if v.is_active() {
                v.stake.own()
            } else {
                v.ledger.active()
            };

            let (cell_style, _highlight_symbol) = match selected_validator {
                Some(ref selected) if &v == selected => {
                    (Style::default().fg(Color::Black).bg(Color::White), "❯")
                }
                _ => (Style::default(), ""),
            };

            let validator_cells = vec![
                Cell::from(Text::from(format!("{}", v.status())).alignment(Alignment::Left)),
                Cell::from(Text::from(format!("{}", v.display_name())).alignment(Alignment::Left))
                    .style(cell_style),
                Cell::from(text_points.alignment(Alignment::Right)),
                Cell::from(
                    Text::from(format_planks(staked_total, decimals, 4))
                        .alignment(Alignment::Right),
                ),
                Cell::from(
                    Text::from(format_planks(staked_own, decimals, 4)).alignment(Alignment::Right),
                ),
                Cell::from(
                    Text::from(v.stake.nominators_count().to_string()).alignment(Alignment::Right),
                ),
                Cell::from(Text::from(v.commission_as_percentage(2)).alignment(Alignment::Right)),
            ];
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

        let widths = vec![
            Constraint::Length(3),
            Constraint::Length(28),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];

        let header_cells = vec![
            Cell::from(Text::from("◈").alignment(Alignment::Center)),
            Cell::from(Text::from("identity").alignment(Alignment::Left)),
            Cell::from(Text::from("points").alignment(Alignment::Right)),
            Cell::from(Text::from("total").alignment(Alignment::Right)),
            Cell::from(Text::from("own").alignment(Alignment::Right)),
            Cell::from(Text::from("nominators").alignment(Alignment::Right)),
            Cell::from(Text::from("commission").alignment(Alignment::Right)),
        ];

        // Note: If selected validator is in this group, add a column for the highlight symbol
        // if selected_validator.is_some() {
        //     widths.insert(1, Constraint::Length(1));
        //     header_cells.insert(1, Cell::from(Text::from("")));
        // };
        //

        // Note: Since table_state is being shared with other widgets, it is important to guarantee
        // that table_state offset is ALWAYS 0. Has we alwasy want to start from the top.
        *table_state.offset_mut() = 0;

        let table = Table::new(rows, widths)
            .header(Row::new(header_cells).set_style(THEME.table.header))
            .style(Style::default().fg(Color::Blue));

        StatefulWidget::render(table, area, buf, table_state);
    }
}
