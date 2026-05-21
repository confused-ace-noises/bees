use std::convert::Infallible;

use derive_more::{Display, Error, From};

use crate::{capability::CapError, endpoint::HandlerStackError, net::net_error::NetError, utils::resource_string::FormatStringError};

#[derive(Debug, Display, Error, From)]
#[display("`bees::Error`: {_variant}")]
pub enum Error {
    NetError(#[error(source)] NetError),
    
    #[display("No Resource with the specified name `{_0}` was found")]
    StringInterpolationErr(FormatStringError),

    #[display("A Capability threw an error: {_0}")]
    CapabilityError(#[error(source)] CapError),

    #[display("A HandlerStack threw an error: {_0}")]
    #[from(skip)]
    HandlerStackError(#[error(source)] HandlerStackError),
}