use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::Level;

#[derive(Debug)]
pub struct LogEntry {
    pub ts: u128,
    pub level: Level,
    pub target: String,
    pub message: String,
}

#[derive(Debug)]
pub struct Log {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    rx: mpsc::UnboundedReceiver<LogEntry>,
}

impl Log {
    pub fn new(rx: mpsc::UnboundedReceiver<LogEntry>, max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            rx,
            max_entries,
        }
    }

    pub fn entries(&self) -> &VecDeque<LogEntry> {
        &self.entries
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn update(&mut self) {
        while let Ok(entry) = self.rx.try_recv() {
            self.push(entry);
        }
    }

    fn push(&mut self, entry: LogEntry) {
        self.entries.push_back(entry);

        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }
}
