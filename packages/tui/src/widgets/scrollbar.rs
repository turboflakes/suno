use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

pub fn render_scrollbar(position: usize, max: usize, area: Rect, buf: &mut Buffer) {
    let mut state = ScrollbarState::new(max)
        .viewport_content_length(area.height as usize)
        .position(position);
    Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐")
        .render(area, buf, &mut state);
}
