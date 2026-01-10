use std::sync::OnceLock;

use crate::resource_def::ResourceManager;
use crate::record_def::RecordManager;

pub(crate) static CONTEXT: OnceLock<Context> = OnceLock::new();

#[derive(Debug)]
pub struct Context {
    pub resources: ResourceManager,
    pub client: crate::net::client::Client,
    pub records: RecordManager,
}

impl Context {
    pub fn new() -> Context {
        Context {
            client: crate::net::client::Client::new(),
            resources: ResourceManager::default(),
            records: RecordManager::default(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}
