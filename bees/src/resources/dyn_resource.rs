#[cfg(not(feature = "async-trait"))]
use super::resource::ResourceOutput;

use super::resource::Resource;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::hash::Hash;
use std::borrow::Borrow;


#[derive(Debug)]
pub struct DynResource(pub Arc<dyn Resource + 'static>);

impl DynResource {
    pub fn from_res<R: Resource + 'static>(resource: R) -> Self {
        DynResource(Arc::new(resource) as Arc<dyn Resource>)
    }
}

impl Eq for DynResource{}
impl PartialEq for DynResource {
    fn eq(&self, other: &Self) -> bool {
        self.0.ident() == other.0.ident()
    }
}

impl<R> From<Arc<R>> for DynResource 
where
    R: Resource + 'static,
{
    fn from(value: Arc<R>) -> Self {
        DynResource(value)
    }
}

impl Hash for DynResource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.ident().hash(state);
    }
}

impl Clone for DynResource {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[cfg(not(feature = "async-trait"))]
impl Resource for DynResource {
    fn ident(&self) -> &str {
        self.0.ident()
    }

    fn data<'a>(&'a self) -> ResourceOutput<'a> {
        ResourceOutput::new(async move {Box::new(self.0.data().await) as Box<dyn Display>})
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Resource for DynResource {
    fn ident(&self) -> &str {
        self.0.ident()
    }

    async fn data(&self) -> Box<dyn Display> {
        Box::new(self.0.data().await)
    }
}

impl Deref for DynResource {
    type Target = Arc<dyn Resource>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DynResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Borrow<str> for DynResource {
    fn borrow(&self) -> &str {
        self.ident()
    }
}

// impl Borrow<String> for DynResource {
//     fn borrow(&self) -> &String {
//         self.0.ident().to_string()
//     }
// }
