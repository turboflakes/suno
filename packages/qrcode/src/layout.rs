use ratatui::layout::{Constraint, Flex, Layout, Rect};

fn flex_area(area: Rect, horizontal: Constraint, vertical: Constraint, flex: Flex) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(flex).areas(area);
    let [area] = Layout::vertical([vertical]).flex(flex).areas(area);
    area
}

/// Center a `w`×`h` region within `area`.
pub fn centered_area(area: Rect, w: u16, h: u16) -> Rect {
    flex_area(
        area,
        Constraint::Length(w),
        Constraint::Length(h),
        Flex::Center,
    )
}
