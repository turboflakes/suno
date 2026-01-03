use crate::display::format_millis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Epoch {
    // Epoch index
    index: u64,
    // Block number that the epoch started
    start_bn: u32,
    // Number of blocks each epoch should take
    duration_bn: u64,
    // Expected average block creation in milliseconds
    block_time_ms: u64,
}

impl Epoch {
    pub fn new(index: u64, start_bn: u32, duration_bn: u64, block_time_ms: u64) -> Self {
        Self {
            index,
            start_bn,
            duration_bn,
            block_time_ms,
        }
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn start(&self) -> u64 {
        self.start_bn as u64
    }

    pub fn duration(&self) -> u64 {
        self.duration_bn
    }

    pub fn progress(&self, current_block_number: u64) -> f64 {
        if current_block_number < self.start() {
            return 0.0;
        }
        let diff = current_block_number - self.start();
        diff as f64 / self.duration() as f64
    }

    pub fn countdown_time(&self, current_block_number: u64) -> String {
        if current_block_number < self.start()
            || current_block_number >= self.start() + self.duration()
        {
            return format_millis(0);
        }
        let diff = self.duration() - (current_block_number - self.start());
        format_millis(diff * self.block_time_ms)
    }
}
