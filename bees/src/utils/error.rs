use derive_more::{Display, Error, From};

use crate::net::net_error::NetError;
use std::error::Error as StdError;

#[derive(Debug, Display, Error, From)]
#[display("`bees::Error`: {_variant}")]
pub enum Error {
    NetError(#[error(source)] NetError),
    
    #[from(skip)]
    #[error(ignore)]
    #[display("No Resource with the specified name `{_0}` was found")]
    NoResFound(String),

    #[display("A Capability threw an error: {_0}")]
    CapabilityError(#[error(source)] Box<dyn StdError>),
}