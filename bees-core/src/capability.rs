use std::any::Any;
use crate::net::request::RequestBuilder;
use core::{fmt, hash::Hash};

pub trait Capability: Sync + Send + Any {
    fn apply(&self, request: RequestBuilder) -> RequestBuilder;
}

impl Capability for dyn Fn(RequestBuilder) -> RequestBuilder + Send + Sync {
    fn apply(&self, request: RequestBuilder) -> RequestBuilder {
        (self)(request)
    }
}

impl<F: Fn(RequestBuilder) -> RequestBuilder + Send + Sync + Any> Capability for F {
    fn apply(&self, request: RequestBuilder) -> RequestBuilder {
        (self)(request)
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

impl std::fmt::Debug for Box<dyn Capability> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Capability {{ type_id: {:?} }}", self.as_ref().type_id())
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
