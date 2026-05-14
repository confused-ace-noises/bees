use crate::resources::resource::AsAny;
#[cfg(feature = "async-trait")]
use crate::resources::resource::ResourceResult;

#[cfg(not(feature = "async-trait"))]
use super::resource::ResourceOutput;

use super::resource::Resource;
use std::any::Any;
use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DynResource {
    pub inner: Arc<dyn Resource + 'static>,
    pub(crate) any: Arc<dyn Any + Send + Sync + 'static>,
}

impl DynResource {
    pub fn from_res<R: Resource + 'static>(resource: R) -> Self {
        Self::from_res_arc(Arc::new(resource))
    }

    pub fn from_res_arc<R: Resource + 'static>(arc: Arc<R>) -> Self {
        DynResource {
            any: arc.clone(),
            inner: arc as Arc<dyn Resource>,
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.any.downcast_ref::<T>()
    }
}

impl Eq for DynResource {}
impl PartialEq for DynResource {
    fn eq(&self, other: &Self) -> bool {
        self.inner.ident() == other.inner.ident()
    }
}

impl<R> From<Arc<R>> for DynResource
where
    R: Resource + 'static,
{
    fn from(value: Arc<R>) -> Self {
        Self::from_res_arc(value)
    }
}

impl Hash for DynResource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.ident().hash(state);
    }
}

#[cfg(not(feature = "async-trait"))]
impl Resource for DynResource {
    fn ident(&self) -> &str {
        self.inner.ident()
    }

    fn data<'a>(&'a self) -> ResourceOutput<'a> {
        ResourceOutput::new(self.inner.data())
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Resource for DynResource {
    fn ident(&self) -> &str {
        self.inner.ident()
    }

    async fn data(&self) -> ResourceResult {
        self.inner.data().await
    }
}

impl Deref for DynResource {
    type Target = Arc<dyn Resource>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for DynResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Borrow<str> for DynResource {
    fn borrow(&self) -> &str {
        self.ident()
    }
}