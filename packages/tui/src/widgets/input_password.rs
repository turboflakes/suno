use crate::widgets::input_field::InputField;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::Styled,
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph, Widget},
};
use std::sync::{Arc, RwLock};
use suno_config::CONFIG;

#[derive(Debug)]
pub struct InputPasswordWidget {
    pub state: Arc<RwLock<InputField>>,
}

impl Widget for &InputPasswordWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();
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
            Constraint::Fill(1), // InputField
        ];

        // Set area to show hotkey when input is valid
        if state.is_valid() {
            h_constraints.push(Constraint::Length(7))
        }

        // Set area to show spinner when input is busy
        if state.is_busy() {
            h_constraints.push(Constraint::Length(4))
        }

        let input_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_constraints)
            .split(area[0]);

        let block = Block::new()
            .set_style(theme.input.base(state.is_active()))
            .padding(Padding::proportional(1));

        let mut input_spans = vec![];

        // Label
        if let Some(label) = state.label() {
            input_spans.push(Span::styled(format!("{}: ", label), theme.input.label));
        };

        // Placeholder
        if state.is_empty() && state.is_active() {
            input_spans.push(Span::raw("proxy password").style(theme.input.placeholder));
        } else if state.is_locked() {
            input_spans.push(Span::raw("unlocking proxy account..").style(theme.input.placeholder));
        }

        // Input value
        input_spans.push(Span::raw(state.value()));

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

        // Show hotkey when input is valid
        if state.is_valid() {
            let block = Block::new()
                .set_style(theme.input.base(state.is_active()))
                .padding(Padding::new(0, 2, 1, 1));

            let hotkey = Paragraph::new(Line::from(vec![
                Span::raw("enter").style(theme.input.suffix(state.is_valid()))
            ]))
            .block(block);
            hotkey.render(input_area[1], buf);
        }

        // Lock and show spinner when input is busy
        if state.is_busy() {
            let spinner = state.spinner();
            spinner.render(input_area[1], buf);
        }

        // Show invalid message when input is invalid
        if state.is_invalid() {
            Clear.render(area[1], buf);
            let block = Block::new()
                .set_style(theme.input.base(state.is_active()))
                .padding(Padding::new(2, 0, 0, 1));

            let error = Paragraph::new(Line::from(vec![
                Span::raw(state.status()).style(theme.input.error)
            ]))
            .block(block);
            error.render(area[1], buf);
        }
    }
}
