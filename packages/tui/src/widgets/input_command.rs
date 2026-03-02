use crate::theme::THEME;
use crate::widgets::input_field::InputField;
use crate::{call::Call, entry::ToPlaceholder};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Widget},
};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct InputCommandWidget {
    pub state: Arc<RwLock<InputField>>,
    pub call: Option<Call>,
}

impl Widget for &InputCommandWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        // Split area into two parts vertically for the main input field
        // and a footer to display error message
        let mut v_constraints = vec![Constraint::Length(3)];
        if state.is_invalid() {
            v_constraints.push(Constraint::Length(2))
        }

        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(v_constraints)
            .split(area);

        Clear.render(area[0], buf);

        let mut h_constraints = vec![
            Constraint::Length(3), // prefix_marker '/'
            Constraint::Fill(1),   // InputField
        ];

        // Set area to show hotkey when input is valid
        if state.is_valid() {
            h_constraints.push(Constraint::Length(7))
        }

        let input_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_constraints)
            .split(area[0]);

        let block = Block::new()
            .style(THEME.input.base(state.is_active()))
            .padding(Padding::new(2, 0, 1, 1));

        let marker = Paragraph::new(Line::from(vec![
            Span::raw("/").style(THEME.input.prefix(state.is_active()))
        ]))
        .block(block);
        marker.render(input_area[0], buf);

        let block = Block::new()
            .style(THEME.input.base(state.is_active()))
            .padding(Padding::new(0, 2, 1, 1));

        let mut input_spans = vec![];

        // Label
        if let Some(label) = state.label() {
            input_spans.push(Span::styled(format!("{}: ", label), THEME.input.label));
        };

        // Input value
        input_spans.push(Span::raw(state.value().to_string()));

        // Placeholder
        if let Some(call) = &self.call {
            // NOTE: Only show placeholder if command is not defined
            if state.value().split_once(' ').is_none() {
                let placeholder: String = call
                    .placeholder()
                    .chars()
                    .skip(state.value().len())
                    .collect();
                input_spans.push(Span::raw(placeholder).style(THEME.input.placeholder));
            };
        }

        let field = Paragraph::new(Line::from(input_spans)).block(block);
        field.render(input_area[1], buf);

        // Calculate and save the cursor position into the state
        if state.is_active() {
            let position = Position::new(
                input_area[1].x + state.character_index() as u16,
                input_area[1].y + 1,
            );
            state.set_cursor_position(position);
        } else {
            state.reset_cursor_position();
        }

        // Show hotkey when input is valid
        if state.is_valid() {
            let block = Block::new()
                .style(THEME.input.base(state.is_active()))
                .padding(Padding::new(0, 2, 1, 1));

            let hotkey = Paragraph::new(Line::from(vec![
                Span::raw("enter").style(THEME.input.suffix(state.is_valid()))
            ]))
            .block(block);
            hotkey.render(input_area[2], buf);
        }

        // Show invalid message when input is invalid
        if state.is_invalid() {
            Clear.render(area[1], buf);
            let block = Block::new()
                .style(THEME.input.base(state.is_active()))
                .padding(Padding::new(2, 0, 0, 1));

            let error = Paragraph::new(Line::from(vec![
                Span::raw(state.status()).style(THEME.input.error)
            ]))
            .block(block);
            error.render(area[1], buf);
        }
    }
}
