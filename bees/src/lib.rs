use std::{fmt::Display, pin::Pin, sync::LazyLock};

use crate::{net::RequestBuilder, resources::resource_handler::ResourceManager, utils::error::Error};
pub mod endpoint;
pub mod record;
pub mod capability;
pub mod net;
pub mod handler;
pub mod utils;
pub mod resources;

//
//   FIXME: HUGE ARCHITECTURE PROBLEM
//
//    ANY ONE HANDLER DOES NOT HAVE
//    ACCESS TO THE CLIENT, THIS NEEDS
//    TO BE FIXED

pub static RESOURCE_MANAGER: LazyLock<ResourceManager> = LazyLock::new(|| {
    ResourceManager::new()
});

pub fn resource_manager() -> &'static ResourceManager {
    &RESOURCE_MANAGER
}

// pub(crate) trait Sealed{}

pub struct ResourceOutput<'a>(pub Pin<Box<dyn Future<Output = Box<dyn Display>> + Send + 'a>>);

impl<'a> ResourceOutput<'a> {
    pub fn new(fut: impl Future<Output = Box<dyn Display>> + Send + 'a) -> Self {
        Self(Box::pin(fut))
    }
}

impl<'a> Future for ResourceOutput<'a> {
    type Output = Box<dyn Display>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

pub struct CapabilityOutput<'a>(pub Pin<Box<dyn Future<Output = Result<RequestBuilder, Error>> + Send + 'a>>);

impl<'a> CapabilityOutput<'a> {
    pub fn new(fut: impl Future<Output = Result<RequestBuilder, Error>> + Send + 'a) -> Self {
        Self(Box::pin(fut))
    }
}

impl<'a> Future for CapabilityOutput<'a> {
    type Output = Result<RequestBuilder, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}