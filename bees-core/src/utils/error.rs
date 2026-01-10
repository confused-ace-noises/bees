use std::error::Error as StdError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("Capability emitted error: {0}")]
    CapabilityError(Box<dyn StdError + Send + Sync>),

    #[error("ParseError: {0}")]
    ParseError(#[from] ParseError),

    #[error("Resource threw: {0}")]
    ResourceError(String),
}