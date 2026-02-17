use std::{convert::Infallible, future::{Ready, ready}, pin::Pin};

use crate::{CapabilityOutput, net::request::RequestBuilder};
use crate::{utils::error::Error};

pub trait Capability: Send + Sync {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a>;
}

impl<T, Func, Fut> Capability for Func 
where 
    Fut: Future<Output = T> + Send + Sync,
    T: Into<Result<RequestBuilder, Error>>,
    Func: Fn(RequestBuilder) -> Fut + Send + Sync,
{
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move { (self)(request).await.into() } )
    }
}

impl Into<Result<RequestBuilder, Error>> for RequestBuilder {
    fn into(self) -> Result<RequestBuilder, Error> {
        Ok(self)
    }
}
