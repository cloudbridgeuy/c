#[derive(Debug)]
pub enum Anthropic {
    Unknown,
}

impl std::error::Error for Anthropic {}

impl std::fmt::Display for Anthropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown error"),
        }
    }
}
