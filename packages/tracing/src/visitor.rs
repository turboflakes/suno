use std::fmt;
use tracing::field::{Field, Visit};

#[derive(Default)]
pub struct EventVisitor {
    message: String,
    fields: Vec<(String, String)>,
}

impl EventVisitor {
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Visit for EventVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let value = format!("{value:?}");

        if field.name() == "message" {
            self.message = value;
        } else {
            self.fields.push((field.name().into(), value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.fields.push((field.name().into(), value.into()));
        }
    }
}
