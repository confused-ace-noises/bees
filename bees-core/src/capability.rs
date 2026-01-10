use std::any::Any;

#[cfg(not(feature = "async-trait"))]
use std::pin::Pin;

use crate::net::request::RequestBuilder;
#[cfg(not(feature = "async-trait"))]
use crate::utils::Error;
use core::{fmt, hash::Hash};

#[cfg(not(feature = "async-trait"))]
pub struct CapabilityOutput<'a>(Pin<Box<dyn Future<Output = Result<RequestBuilder, Error>> + Send + 'a>>, );

#[cfg(not(feature = "async-trait"))]
impl<'a> CapabilityOutput<'a> {
    pub fn new(future: impl Future<Output = Result<RequestBuilder, Error>> + Send + 'a) -> Self {
        Self(Box::pin(future))
    }
}

#[cfg(not(feature = "async-trait"))]
impl Future for CapabilityOutput<'_> {
    type Output = Result<RequestBuilder, Error>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.get_mut().0.as_mut().poll(cx)
    }
}

#[cfg(not(feature = "async-trait"))]
pub trait Capability: Sync + Send + Any {   
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a>;
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
pub trait Capability: Sync + Send {   
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::utils::Error>;
}

#[cfg(not(feature = "async-trait"))]
impl Capability for dyn Fn(RequestBuilder) -> RequestBuilder + Send + Sync {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move {
            Ok((self)(request))
        })
    }
}

#[cfg(not(feature = "async-trait"))]
impl<F: Fn(RequestBuilder) -> RequestBuilder + Send + Sync + Any> Capability for F {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move {
            Ok((self)(request))
        })
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Capability for dyn Fn(RequestBuilder) -> RequestBuilder + Send + Sync {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::utils::Error> {
        Ok((self)(request))
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl<F: Fn(RequestBuilder) -> RequestBuilder + Send + Sync + Any> Capability for F {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::utils::Error> {
        Ok((self)(request))
    }
}

impl PartialEq for Box<dyn Capability> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().type_id() == (other as &dyn Any).type_id()
    }
}

impl Eq for Box<dyn Capability> {}

impl Hash for Box<dyn Capability> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().type_id().hash(state);
    }
}

impl std::fmt::Debug for dyn Capability{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // no as_ref here because it's not Box<dyn Capability>
        write!(f, "Capability {{ type_id: {:?} }}", self.type_id())
    }
}

pub trait IntoBoxedCapability {
    fn into_boxed_capability(self) -> Box<dyn Capability>;
}

impl IntoBoxedCapability for Box<dyn Capability> {
    fn into_boxed_capability(self) -> Box<dyn Capability> {
        self
    }
}

impl<T: Capability + 'static> IntoBoxedCapability for T {    
    fn into_boxed_capability(self) -> Box<dyn Capability> {
        Box::new(self) as Box<dyn Capability>
    }
}
