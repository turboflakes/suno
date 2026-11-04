use crate::widgets::input_field::InputField;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Widget},
};
use std::sync::{Arc, RwLock};
use suno_theme::Theme;

#[derive(Debug)]
pub struct InputFilterWidget {
    pub state: Arc<RwLock<InputField>>,
    pub theme: Theme,
    pub placeholder: Option<String>,
    pub has_match: bool,
}

impl Widget for &InputFilterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = self.theme;
        let mut state = self.state.write().unwrap();

        Clear.render(area, buf);

        let mut h_constraints = vec![
            Constraint::Length(3), // prefix marker '/'
            Constraint::Fill(1),   // input field
        ];

        // Set area to show hotkey when a theme is highlighted
        if self.has_match {
            h_constraints.push(Constraint::Length(7))
        }

        let input_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_constraints)
            .split(area);

        let block = Block::new()
            .style(theme.input.base(state.is_active()))
            .padding(Padding::new(2, 0, 1, 1));

        let marker = Paragraph::new(Line::from(vec![
            Span::raw("/").style(theme.input.prefix(state.is_active()))
        ]))
        .block(block);
        marker.render(input_area[0], buf);

        let block = Block::new()
            .style(theme.input.base(state.is_active()))
            .padding(Padding::new(0, 2, 1, 1));

        let mut input_spans = vec![];

        // Input value
        input_spans.push(Span::raw(state.value().to_string()));

        // Ghost-complete with the currently highlighted theme's name
        if let Some(placeholder) = &self.placeholder {
            let placeholder: String = placeholder.chars().skip(state.value().len()).collect();
            input_spans.push(Span::raw(placeholder).style(theme.input.placeholder));
        }

        let field = Paragraph::new(Line::from(input_spans)).block(block);
        field.render(input_area[1], buf);

        // Show hotkey when a theme is highlighted
        if self.has_match {
            let block = Block::new()
                .style(theme.input.base(state.is_active()))
                .padding(Padding::new(0, 2, 1, 1));

            let hotkey = Paragraph::new(Line::from(vec![
                Span::raw("enter").style(theme.input.suffix(true))
            ]))
            .block(block);
            hotkey.render(input_area[2], buf);
        }

        if state.is_active() {
            let position = Position::new(
                input_area[1].x + state.character_index() as u16,
                input_area[1].y + 1,
            );
            state.set_cursor_position(position);
        } else {
            state.reset_cursor_position();
        }
    }
}

/// Whether any of the `texts` contains the `query`, ignoring the case. The `query` is expected to
/// be lowercase, and everything matches when it is empty.
pub fn matches_query(query: &str, texts: &[&str]) -> bool {
    query.is_empty() || texts.iter().any(|text| text.to_lowercase().contains(query))
}

/// The filter as a single row, with a row of margin on top, shown on top of a list to filter its
/// rows.
#[derive(Debug)]
pub struct InputInlineFilterWidget {
    pub state: Arc<RwLock<InputField>>,
    pub theme: Theme,
    /// How many rows match, shown on the right
    pub matches: usize,
}

/// Width of the marker cell: the left padding, and the marker.
const INLINE_MARKER_WIDTH: u16 = 3;

impl Widget for &InputInlineFilterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = self.theme;
        let mut state = self.state.write().unwrap();

        let matches = format!("{} matches", self.matches);

        let [margin_area, row_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        Block::new()
            .style(theme.block.main)
            .render(margin_area, buf);

        // The marker, the input field and the number of matches, each one in its own block
        let [marker_area, field_area, matches_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(INLINE_MARKER_WIDTH),
                Constraint::Fill(1),
                Constraint::Length(matches.chars().count() as u16 + 2),
            ])
            .areas(row_area);

        let block = Block::new()
            .style(theme.input.base(state.is_active()))
            .padding(Padding::new(2, 0, 0, 0));

        let marker = Paragraph::new(Line::from(vec![
            Span::raw("/").style(theme.input.prefix(state.is_active()))
        ]))
        .block(block);
        marker.render(marker_area, buf);

        let block = Block::new()
            .style(theme.input.base(state.is_active()))
            .padding(Padding::new(0, 0, 0, 0));

        let field =
            Paragraph::new(Line::from(vec![Span::raw(state.value().to_string())])).block(block);
        field.render(field_area, buf);

        let block = Block::new()
            .style(theme.input.base(state.is_active()))
            .padding(Padding::new(0, 2, 0, 0));

        let count = Paragraph::new(
            Line::from(Span::raw(matches).style(theme.input.suffix(true)))
                .alignment(Alignment::Right),
        )
        .block(block);
        count.render(matches_area, buf);

        if state.is_active() {
            let position =
                Position::new(field_area.x + state.character_index() as u16, field_area.y);
            state.set_cursor_position(position);
        } else {
            state.reset_cursor_position();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_any_text_ignoring_the_case() {
        assert!(matches_query("turbo", &["Turboflakes", "1abc"]));
        assert!(matches_query("turbo", &["nothing", "x/TURBO"]));
        assert!(!matches_query("turbo", &["nothing", "1abc"]));
        assert!(matches_query("", &["anything"]));
    }
}
