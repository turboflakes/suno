use crate::theme::THEME;
use crate::widgets::input_field::InputField;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::Styled,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct InputPasswordWidget {
    pub state: Arc<RwLock<InputField>>,
}

impl Widget for &InputPasswordWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let mut h_constraints = vec![
            Constraint::Fill(1), // InputField
        ];

        // set area to show hotkey when input is valid
        if state.is_valid() {
            h_constraints.push(Constraint::Length(7))
        }

        let input_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_constraints)
            .split(area);

        let block = Block::new()
            .set_style(THEME.input.base(state.is_active()))
            .padding(Padding::proportional(1));

        let mut input_spans = vec![];

        // Label
        if let Some(label) = state.label() {
            input_spans.push(Span::styled(format!("{}: ", label), THEME.input.label));
        };

        // Placeholder
        if state.is_empty() {
            input_spans
                .push(Span::raw("password for the signer account").style(THEME.input.placeholder));
        }

        // Input value
        input_spans.push(Span::raw(format!("{}", state.value())));

        let field = Paragraph::new(Line::from(input_spans)).block(block);
        field.render(input_area[0], buf);

        // Calculate and save the cursor position into the state
        if state.is_active() {
            let position = Position::new(area.x + 2 + state.character_index() as u16, area.y + 1);
            state.set_cursor_position(position);
        } else {
            state.reset_cursor_position();
        }

        // show hotkey when input is valid
        if state.is_valid() {
            let block = Block::new()
                .set_style(THEME.input.base(state.is_active()))
                .padding(Padding::new(0, 2, 1, 1));

            let hotkey = Paragraph::new(Line::from(vec![
                Span::raw("enter").style(THEME.input.suffix(state.is_valid()))
            ]))
            .block(block);
            hotkey.render(input_area[1], buf);
        }
    }
}
