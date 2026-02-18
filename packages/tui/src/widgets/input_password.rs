use crate::theme::THEME;
use crate::widgets::input_field::InputField;
use crate::widgets::spinner::Spinner;
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

        let mut h_constraints = vec![
            Constraint::Fill(1), // InputField
        ];

        // set area to show hotkey when input is valid
        if state.is_valid() {
            h_constraints.push(Constraint::Length(7))
        }

        // set area to show spinner when input is busy
        if state.is_busy() {
            h_constraints.push(Constraint::Length(4))
        }

        let input_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_constraints)
            .split(area[0]);

        let block = Block::new()
            .set_style(THEME.input.base(state.is_active()))
            .padding(Padding::proportional(1));

        let mut input_spans = vec![];

        // Label
        if let Some(label) = state.label() {
            input_spans.push(Span::styled(format!("{}: ", label), THEME.input.label));
        };

        // Placeholder
        if state.is_empty() && state.is_active() {
            input_spans.push(Span::raw("proxy password").style(THEME.input.placeholder));
        } else if state.is_locked() {
            input_spans.push(Span::raw("verifying password..").style(THEME.input.placeholder));
        }

        // Input value
        input_spans.push(Span::raw(format!("{}", state.value())));

        let field = Paragraph::new(Line::from(input_spans)).block(block);
        field.render(input_area[0], buf);

        // Calculate and save the cursor position into the state
        if state.is_active() {
            let position = Position::new(
                area[0].x + 2 + state.character_index() as u16,
                area[0].y + 1,
            );
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

        // Lock and show spinner when input is busy
        if state.is_busy() {
            let spinner = state.spinner();
            spinner.render(input_area[1], buf);
        }

        // show invalid message when input is invalid
        if state.is_invalid() {
            let block = Block::new()
                .set_style(THEME.input.base(state.is_active()))
                .padding(Padding::new(2, 0, 0, 1));

            let error = Paragraph::new(Line::from(vec![
                Span::raw(state.status()).style(THEME.input.error)
            ]))
            .block(block);
            error.render(area[1], buf);
        }
    }
}
