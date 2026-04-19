use derive_more::{Display, Error, From};

use crate::{CapError, net::net_error::NetError, utils::format_string::FormatStringError};
use std::error::Error as StdError;

#[derive(Debug, Display, Error, From)]
#[display("`bees::Error`: {_variant}")]
pub enum Error {
    NetError(#[error(source)] NetError),
    
    #[display("No Resource with the specified name `{_0}` was found")]
    StringInterpolationErr(FormatStringError),

    #[display("A Capability threw an error: {_0}")]
    CapabilityError(#[error(source)] CapError),
}