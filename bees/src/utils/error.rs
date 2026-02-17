use thiserror::Error;

use crate::net::net_error::NetError;
use std::error::Error as StdError;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NetError(#[from] NetError),

    #[error("No resource with name \"{0}\" was found.")]
    NoResFound(String),

    #[error("capability error: {0}")]
    CapabilityError(Box<dyn StdError>),
}