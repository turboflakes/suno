use crate::widgets::chains::ChainsList;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Styled,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use suno_config::SupportedRuntime;
use suno_primitives::display::{create_progress_bar_by_blocks, format_millis};
use suno_theme::Theme;

/// Renders the network info of a group: the chain stats, and the era and epoch progress.
/// `displayed` is the line telling what the group displays.
pub(crate) fn render_network_header(
    chains: &ChainsList,
    theme: Theme,
    runtime: SupportedRuntime,
    displayed: Line<'static>,
    area: Rect,
    buf: &mut Buffer,
    is_odd: bool,
) {
    let Some(chain) = chains.get_chain_by_runtime(runtime) else {
        let block = Block::new().set_style(theme.block.alt(is_odd));
        block.render(area, buf);
        return;
    };

    let Some(ah_chain) = chains.get_chain_by_runtime(
        runtime
            .asset_hub_runtime()
            .expect("every relay has an AssetHub chain"),
    ) else {
        let block = Block::new().set_style(theme.block.alt(is_odd));
        block.render(area, buf);
        return;
    };

    let [network_area, progress_area, progress_bar_area, countdown_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),    // Network info
            Constraint::Length(16), // Era / Session info bar
            Constraint::Length(24), // Era / Session progress bar
            Constraint::Length(20), // Countdown
        ])
        .areas(area);

    // Draw and render general network stats

    let network_lines = vec![
        Line::from(
            Span::raw(format!("{} NETWORK", runtime.to_string().to_uppercase()))
                .style(theme.paragraph.header_active),
        ),
        Line::from(vec![
            Span::raw("Avg. block time ").style(theme.paragraph.label),
            Span::raw(
                chain
                    .epoch()
                    .as_ref()
                    .and_then(|e| chain.average_block_time_ms(e.block_time_ms()))
                    .map(|ms| format_millis(ms, true, true))
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]),
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
        displayed,
    ];

    let block = Block::new().set_style(theme.block.alt(is_odd));
    let network_info = Paragraph::new(network_lines)
        .block(block)
        .style(theme.paragraph.base);

    network_info.render(network_area, buf);

    // Draw and render Progress Info

    let Some(epoch) = chain.epoch() else {
        // TODO: Handle epoch not available, maybe render loading indicator
        let block = Block::new().set_style(theme.block.alt(is_odd));
        let area = progress_area.union(progress_bar_area).union(countdown_area);
        block.render(area, buf);
        return;
    };

    let Some(era) = ah_chain.era() else {
        // TODO: Handle era not available, maybe render loading indicator
        let block = Block::new().set_style(theme.block.alt(is_odd));
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

    let block = Block::new().set_style(theme.block.alt(is_odd));
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

    let block = Block::new().set_style(theme.block.alt(is_odd));
    let progress_bar = Paragraph::new(progress_bar_lines)
        .block(block)
        .style(theme.paragraph.base);

    progress_bar.render(progress_bar_area, buf);

    // Draw and render Countdowns

    let epoch_countdown_time = epoch.countdown_time(chain.finalized_block());
    let era_countdown_time = era.countdown_time(epoch.duration(), epoch.block_time_ms());

    let countdown_lines = vec![
        Line::from(""),
        Line::from(format!(" {} left", era_countdown_time,)).alignment(Alignment::Left),
        Line::from(format!(" {} left", epoch_countdown_time,)).alignment(Alignment::Left),
    ];

    let block = Block::new().set_style(theme.block.alt(is_odd));
    let countdown_info = Paragraph::new(countdown_lines)
        .block(block)
        .style(theme.paragraph.base);

    countdown_info.render(countdown_area, buf);
}
