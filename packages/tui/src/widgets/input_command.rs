use crate::theme::THEME;
use crate::widgets::input_field::InputField;
use crate::{call::Call, entry::ToPlaceholder};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::Styled,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
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

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3), // prefix_marker /
                Constraint::Fill(1),   // InputField
            ])
            .split(area);

        //
        let block = Block::new()
            .set_style(THEME.input.base(state.is_active()))
            .padding(Padding::new(2, 0, 1, 1));

        let marker = Paragraph::new(Line::from(vec![
            Span::raw("/").style(THEME.input.prefix(state.is_active()))
        ]))
        .block(block);
        marker.render(area[0], buf);

        let block = Block::new()
            .set_style(THEME.input.base(state.is_active()))
            .padding(Padding::new(0, 2, 1, 1));

        let mut input_spans = vec![];

        // Label
        if let Some(label) = state.label() {
            input_spans.push(Span::styled(format!("{}: ", label), THEME.input.label));
        };

        // Input value
        input_spans.push(Span::raw(format!("{}", state.value())));

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
        field.render(area[1], buf);

        // Calculate and save the cursor position into the state
        if state.is_editing() {
            let position = Position::new(area[1].x + state.character_index() as u16, area[1].y + 1);
            state.set_cursor_position(position);
        } else {
            state.reset_cursor_position();
        }
    }
}
