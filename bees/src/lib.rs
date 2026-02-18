use std::{pin::Pin, sync::LazyLock};

use crate::{net::RequestBuilder, resources::resource_handler::ResourceManager, utils::error::Error};
pub mod endpoint;
pub mod record;
pub mod capability;
pub mod net;
pub mod handler;
pub mod utils;
pub mod resources;

pub static RESOURCE_MANAGER: LazyLock<ResourceManager> = LazyLock::new(|| {
    ResourceManager::new()
});

pub fn resource_manager() -> &'static ResourceManager {
    &RESOURCE_MANAGER
}

#[cfg(not(feature = "async-trait"))]
pub struct CapabilityOutput<'a>(pub Pin<Box<dyn Future<Output = Result<RequestBuilder, Error>> + Send + 'a>>);

#[cfg(not(feature = "async-trait"))]
impl<'a> CapabilityOutput<'a> {
    pub fn new(fut: impl Future<Output = Result<RequestBuilder, Error>> + Send + 'a) -> Self {
        Self(Box::pin(fut))
    }
}

#[cfg(not(feature = "async-trait"))]
impl<'a> Future for CapabilityOutput<'a> {
    type Output = Result<RequestBuilder, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

// half impl of a proc macro i'll make sometime
#[allow(unused)]
macro_rules! attach_processor {
    ($name:ident -> $ret:ty: $($endpoints:ty),+) => {
        $(
            impl EndpointProcessor<$ret> for $endpoints {
                fn process(&mut self, resp: Response, _: &Self::CallContext) -> impl Future<Output = $ret> {
                    $ident(resp)
                }
            }
        )+
    };

    ($name:ident -> $ret:ty: all) => {
        impl<E: EndpointInfo> EndpointProcessor<$ret> for E {
            fn process(&mut self, resp: Response, _: &Self::CallContext) -> impl Future<Output = $ret> {
                $ident(resp)
            }
        }
    };
}