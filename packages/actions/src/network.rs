use subxt::utils::H256;

pub type BlockNumber = u32;
pub type BlockHash = H256;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Idle,
    Connecting,
    Connected(BlockNumber, BlockHash),
    Error(String),
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "-"),
            Self::Connecting => write!(f, "↺"),
            Self::Connected(block_number, _) => write!(f, "#{}", block_number),
            Self::Error(_) => write!(f, "✗"),
        }
    }
}
