use thiserror::Error;

pub mod timezones;

#[derive(Debug, Error)]
pub enum RizzyError {
    #[error("Unknown timezone '{0}', did you mean: {1:?}")]
    InvalidTimezone(String, Vec<String>),
    #[error("{0}")]
    InvalidArg(String),
}
