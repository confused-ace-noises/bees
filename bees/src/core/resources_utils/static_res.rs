use std::fmt::Display;

use async_trait::async_trait;

use crate::core::resource::Resource;

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

#[async_trait]
impl Resource for StaticResource {
    fn ident(&self) ->  &str {
        &self.name
    }

    async fn data(&self) -> Box<dyn Display> {
        Box::new(self.data.clone())
    }
}