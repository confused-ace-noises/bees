use std::fmt::Display;
use crate::resource_def::resource::Resource;
#[cfg(not(feature = "async-trait"))]
use crate::resource_def::ResourceOutput;

#[derive(Debug)]
pub struct StaticResource {
    pub name: String, 
    pub data: String,
}

impl StaticResource {
    pub fn new(name: String, data: String) -> Self {
        Self {
            name,
            data    
        }
    }
}

#[cfg(not(feature = "async-trait"))]
impl Resource for StaticResource {
    fn ident(&self) ->  &str {
        &self.name
    }

    fn data<'a>(&'a self) -> ResourceOutput<'a> {
        ResourceOutput::new(async move { Box::new(self.data.clone()) as Box<dyn Display> })
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Resource for StaticResource {
    fn ident(&self) ->  &str {
        &self.name
    }

    async fn data(&self) -> Box<dyn Display> {
        Box::new(self.data.clone()) as Box<dyn Display>
    }
}