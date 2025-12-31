#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Epoch {
    // Epoch index
    index: u64,
    // Block number that the epoch started
    start_bn: u32,
    // Number of blocks each epoch should take
    duration_bn: u64,
}

impl Epoch {
    pub fn new(index: u64, start_bn: u32, duration_bn: u64) -> Self {
        Self {
            index,
            start_bn,
            duration_bn,
        }
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn start(&self) -> u32 {
        self.start_bn
    }

    pub fn duration(&self) -> u64 {
        self.duration_bn
    }
}
