#[cfg(feature = "async-trait")]
use core::error;
#[cfg(not(feature = "async-trait"))]
use std::error;
use std::{any::Any, fmt::Debug, future::ready, sync::Arc};

use crate::resources::resource::Resource;

#[cfg(feature = "async-trait")]
use crate::resources::resource::{ResourceFuture, ResourceResult};
#[cfg(not(feature = "async-trait"))]
use crate::resources::resource::ResourceOutput;

#[derive(Debug)]
pub struct ConstRes {
    ident: String,
    value: Arc<String>,
}

impl ConstRes {
    pub fn new(ident: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self {
            ident: ident.as_ref().to_string(),
            value: Arc::new(value.as_ref().to_string()),
        }
    }

    pub fn new_arc(ident: impl AsRef<str>, value: Arc<String>) -> Self {
        Self {
            ident: ident.as_ref().to_string(),
            value
        }
    }

    pub fn ident(&self) -> &String {
        &self.ident
    }

    pub fn value(&self) -> Arc<String> {
        self.value.clone()
    } 
}

#[cfg(not(feature = "async-trait"))]
impl Resource for ConstRes {
    fn ident(&self) -> &str {
        &self.ident
    }

    fn data<'a>(&'a self) -> crate::resources::resource::ResourceOutput<'a> {
        ResourceOutput::new(ready(Ok::<_, Arc<dyn error::Error + Send + Sync>>(self.value.clone())))
    }
} 

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Resource for ConstRes {
    fn ident(&self) -> &str {
        &self.ident
    }

    async fn data(&self) -> ResourceResult {
        Ok::<_, Arc<dyn error::Error + Send + Sync>>(self.value.clone())
    }
} 