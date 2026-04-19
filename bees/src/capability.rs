use crate::net::request::RequestBuilder;
use crate::{utils::error::Error};

#[cfg(not(feature = "async-trait"))]
use crate::CapabilityOutput;
use crate::CapError;

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
