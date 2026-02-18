use std::{borrow::Borrow, fmt::{Debug, Display}, hash::Hash};
#[cfg(not(feature = "async-trait"))]
use std::pin::Pin;

#[cfg(not(feature = "async-trait"))]
pub struct ResourceOutput<'a>(pub Pin<Box<dyn Future<Output = Box<dyn Display>> + Send + 'a>>);

#[cfg(not(feature = "async-trait"))]
impl<'a> ResourceOutput<'a> {
    pub fn new(fut: impl Future<Output = Box<dyn Display>> + Send + 'a) -> Self {
        Self(Box::pin(fut))
    }
}

#[cfg(not(feature = "async-trait"))]
impl<'a> Future for ResourceOutput<'a> {
    type Output = Box<dyn Display>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

#[cfg(not(feature = "async-trait"))]
pub trait Resource: Debug + Send + Sync {
    fn ident(&self) -> &str;
    fn data<'a>(&'a self) -> ResourceOutput<'a>;
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
pub trait Resource: Debug + Send + Sync {
    fn ident(&self) -> &str;
    async fn data(&self) -> Box<dyn Display>;
}

impl PartialEq for dyn Resource {
    fn eq(&self, other: &Self) -> bool {
        self.ident() == other.ident()
    }
}
impl Eq for dyn Resource {}
impl Hash for dyn Resource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident().hash(state);
    }
}

impl Borrow<str> for dyn Resource {
    fn borrow(&self) -> &str {
        self.ident()
    }
}

impl Borrow<str> for Box<dyn Resource> {
    fn borrow(&self) -> &str {
        self.ident()
    }
}