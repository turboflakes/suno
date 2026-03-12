pub type Amount = u128;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Balance {
    pub free: Amount,
    pub reserved: Amount,
    pub frozen: Amount,
}

impl Balance {
    pub fn new(free: Amount, reserved: Amount, frozen: Amount) -> Self {
        Self {
            free,
            reserved,
            frozen,
        }
    }

    pub fn free_balance(&self) -> Amount {
        self.free
    }

    pub fn reserved_balance(&self) -> Amount {
        self.reserved
    }

    pub fn frozen_balance(&self) -> Amount {
        self.frozen
    }

    pub fn total_balance(&self) -> Amount {
        self.free + self.reserved + self.frozen
    }

    pub fn add_free_amount(&mut self, amount: Amount) {
        self.free += amount;
    }
}
