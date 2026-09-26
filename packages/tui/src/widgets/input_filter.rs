use crate::widgets::input_field::InputField;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
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
