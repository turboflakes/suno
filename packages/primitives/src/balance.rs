type Amount = u128;

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
}
