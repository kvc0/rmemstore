#[derive(Debug, Clone, thiserror::Error)]
pub enum ValueError {
    #[error("Missing attribute: {0}")]
    MissingAttribute(&'static str),
}
