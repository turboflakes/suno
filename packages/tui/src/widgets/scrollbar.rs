use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};
use suno_theme::Theme;

pub fn render_scrollbar(theme: Theme, position: usize, max: usize, area: Rect, buf: &mut Buffer) {
    let mut state = ScrollbarState::new(max)
        .viewport_content_length(area.height as usize)
        .position(position);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐")
        .style(theme.scrollbar.base)
        .render(area, buf, &mut state);
}
