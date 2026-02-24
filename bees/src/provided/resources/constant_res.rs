use std::{fmt::{Display, Debug}, future::ready, sync::Arc};

use crate::resources::resource::{Resource, ResourceOutput};

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

impl<D: Display + Debug + Send + std::marker::Sync + 'static> Resource for ConstRes<D> {
    fn ident(&self) -> &str {
        &self.ident
    }

    fn data<'a>(&'a self) -> crate::resources::resource::ResourceOutput<'a> {
        ResourceOutput::new(ready(Box::new(self.value.clone()) as Box<dyn Display + Send>))
    }
} 