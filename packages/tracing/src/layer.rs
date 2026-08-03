use crate::log::LogEntry;
use crate::visitor::EventVisitor;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

pub struct TuiLayer {
    tx: mpsc::UnboundedSender<LogEntry>,
}

impl TuiLayer {
    pub fn new(tx: mpsc::UnboundedSender<LogEntry>) -> Self {
        Self { tx }
    }
}

impl<S> Layer<S> for TuiLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = EventVisitor::default();

        event.record(&mut visitor);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let entry = LogEntry {
            ts,
            level: *event.metadata().level(),
            target: event.metadata().target().to_owned(),
            message: visitor.message().to_owned(),
        };

        let _ = self.tx.send(entry);
    }
}
