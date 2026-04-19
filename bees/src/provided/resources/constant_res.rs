use std::{any::Any, error::Error, fmt::{Debug, Display}, future::ready, sync::Arc};

use crate::resources::resource::Resource;

#[cfg(feature = "async-trait")]
use crate::resources::resource::{ResourceFuture, ResourceResult};
#[cfg(not(feature = "async-trait"))]
use crate::resources::resource::ResourceOutput;

#[derive(Debug)]
pub struct ConstRes<D> 
where
    D: Display + Debug + Send + 'static
{
    ident: String,
    value: Arc<D>,
}

impl<D: Display + Debug + Send + 'static> ConstRes<D> {
    pub fn new(ident: impl AsRef<str>, value: D) -> Self {
        Self {
            ident: ident.as_ref().to_string(),
            value: Arc::new(value),
        }
    }
}

#[cfg(not(feature = "async-trait"))]
impl<D: Display + Debug + Send + std::marker::Sync + 'static> Resource for ConstRes<D> {
    fn ident(&self) -> &str {
        &self.ident
    }

    fn data<'a>(&'a self) -> crate::resources::resource::ResourceOutput<'a> {
        ResourceOutput::new(ready(Ok::<_, Arc<dyn Any + Send + Sync>>(Box::new(self.value.clone()) as Box<dyn Display + Send>)))
    }
} 

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl<D: Display + Debug + Send + std::marker::Sync + 'static> Resource for ConstRes<D> {
    fn ident(&self) -> &str {
        &self.ident
    }

    async fn data(&self) -> ResourceResult {
        Ok::<_, Arc<dyn Any + Send + Sync>>(Box::new(self.value.clone()) as Box<dyn Display + Send>)
    }
} 