#[cfg(not(feature = "async-trait"))]
use futures::future::ready;

use crate::net::request::RequestBuilder;
use crate::{utils::error::Error};
use std::error::Error as StdError;

#[cfg(not(feature = "async-trait"))]
use std::pin::Pin;

pub type CapError = Box<dyn StdError + Send>;

#[cfg(not(feature = "async-trait"))]
pub struct CapabilityOutput<'a>(pub Pin<Box<dyn Future<Output = Result<RequestBuilder, CapError>> + Send + 'a>>);

#[cfg(not(feature = "async-trait"))]
impl<'a> CapabilityOutput<'a> {
    pub fn new(fut: impl Future<Output = Result<RequestBuilder, CapError>> + Send + 'a) -> Self {
        Self(Box::pin(fut))
    }
}

#[cfg(not(feature = "async-trait"))]
impl<'a> Future for CapabilityOutput<'a> {
    type Output = Result<RequestBuilder, CapError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

#[cfg(not(feature = "async-trait"))]
impl<'a> From<Result<RequestBuilder, CapError>> for CapabilityOutput<'a> {
    fn from(value: Result<RequestBuilder, CapError>) -> Self {
        Self(Box::pin(ready(value)))
    }
}


#[cfg(not(feature = "async-trait"))]
pub trait Capability: Send + Sync {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a>;
}

#[cfg(not(feature = "async-trait"))]
impl<T, Func, Fut> Capability for Func 
where 
    Fut: Future<Output = T> + Send + Sync,
    T: Into<Result<RequestBuilder, CapError>>,
    Func: Fn(RequestBuilder) -> Fut + Send + Sync,
{
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move { (self)(request).await.into() } )
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
pub trait Capability: Send + Sync {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, CapError>;
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl<T, Func, Fut> Capability for Func 
where 
    Fut: Future<Output = T> + Send + Sync,
    T: Into<Result<RequestBuilder, CapError>>,
    Func: Fn(RequestBuilder) -> Fut + Send + Sync,
{
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, CapError> {
        (self)(request).await.into()
    }
}


impl From<RequestBuilder> for Result<RequestBuilder, Error> {
    fn from(val: RequestBuilder) -> Self {
        Ok(val)
    }
}
