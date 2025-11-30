use core::hash;
use std::{borrow::Borrow, sync::Arc};

use dashmap::{DashSet, setref::one::Ref};

use crate::endpoint_record::endpoint::{Capability, Endpoint};

#[derive(Debug)]
pub struct RecordManager {
    records: DashSet<Record>,
}

impl Default for RecordManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordManager {
    pub fn new() -> Self {
        RecordManager {
            records: DashSet::new(),
        }
    }

    pub fn add_record(&self, record: Record) {
        self.records.insert(record);
    }

    pub fn get_record_ref(&self, record_name: &str) -> Option<Ref<'_, Record>> {
        self.records.get(record_name)
    }

    pub fn get_record(&self, record_name: &str) -> Option<Record> {
        self.get_record_ref(record_name).and_then(|inner| Some(inner.clone()))
    }
}


#[derive(Debug, Clone)]
pub struct Record(pub(crate) Arc<InnerRecord>);

impl Record {
    pub fn new(record_name: String, constant_url: String, shared_capabilities: Arc<[Box<dyn Capability>]>) -> Self {
        Self(Arc::new(InnerRecord::new(record_name, constant_url, shared_capabilities)))
    }

    // a record cant start with endpoints dumbass, because endpoints need a record to be created in the first place!
    // pub fn new_with_endpoints(record_name: String, constant_url: String, shared_capabilities: Arc<[Box<dyn Capability>]>, endpoints: DashSet<Endpoint>) -> Self {
    //     Self(Arc::new(InnerRecord::new_with_endpoints(record_name, constant_url, shared_capabilities, endpoints)))
    // }

    pub fn add_endpoint(&self, endpoint: Endpoint) {
        self.0.endpoints.insert(endpoint);
    }

    pub fn get_endpoint_ref(&self, name: &str) -> Option<Ref<'_, Endpoint>> {
        self.0.endpoints.get(name)
    }

    pub fn get_endpoint(&self, name: &str) -> Option<Endpoint> {
        self.0.endpoints.get(name).and_then(|inner| Some(inner.clone()) )
    }

    pub fn remove_endpoint(&self, name: &str) -> Option<Endpoint> {
        self.0.endpoints.remove(name)
    }

    pub fn remove_endpoint_if(
        &self,
        name: &str,
        f: impl FnOnce(&Endpoint) -> bool,
    ) -> Option<Endpoint> {
        self.0.endpoints.remove_if(name, f)
    }

    pub fn contains_endpoint(&self, name: &str) -> bool {
        self.0.endpoints.contains(name)
    }

    pub fn record_name(&self) -> &String {
        &self.0.record_name
    }

    pub fn constant_url(&self) -> &String {
        &self.0.constant_url
    }

    pub fn capabilities(&self) -> &Arc<[Box<dyn Capability>]> {
        &self.0.shared_capabilities
    }
}

impl Borrow<str> for Record {
    fn borrow(&self) -> &str {
        &self.0.record_name
    }
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Record {}

impl hash::Hash for Record {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.record_name.hash(state);
    }
}

#[derive(Debug)]
pub(crate) struct InnerRecord {
    pub(super) record_name: String,
    pub(super) constant_url: String,
    pub(super) endpoints: DashSet<Endpoint>,
    pub(super) shared_capabilities: Arc<[Box<dyn Capability>]>
}

impl Borrow<str> for InnerRecord {
    fn borrow(&self) -> &str {
        &self.record_name
    }
}

#[allow(dead_code)]
impl InnerRecord {
    pub(crate) fn new(record_name: String, constant_url: String, shared_capabilities: Arc<[Box<dyn Capability>]>) -> Self {
        let sanitized = constant_url.trim_end_matches("/").to_string();
        Self {
            record_name,
            constant_url: sanitized,
            endpoints: DashSet::new(),
            shared_capabilities
        }
    }

    pub(crate) fn new_with_endpoints(record_name: String, constant_url: String, shared_capabilities: Arc<[Box<dyn Capability>]>, endpoints: DashSet<Endpoint>) -> Self {
        Self {
            record_name,
            constant_url,
            endpoints,
            shared_capabilities
        }
    }

    pub(crate) fn add_endpoint(&self, endpoint: Endpoint) {
        self.endpoints.insert(endpoint);
    }

    pub(crate) fn get_endpoint(&self, name: &str) -> Option<Ref<'_, Endpoint>> {
        self.endpoints.get(name)
    }

    pub(crate) fn remove_endpoint(&self, name: &str) -> Option<Endpoint> {
        self.endpoints.remove(name)
    }

    pub(crate) fn remove_endpoint_if(
        &self,
        name: &str,
        f: impl FnOnce(&Endpoint) -> bool,
    ) -> Option<Endpoint> {
        self.endpoints.remove_if(name, f)
    }

    pub(crate) fn contains_endpoint(&self, name: &str) -> bool {
        self.endpoints.contains(name)
    }
}

impl PartialEq for InnerRecord {
    fn eq(&self, other: &Self) -> bool {
        self.record_name == other.record_name
    }
}
impl Eq for InnerRecord {}

impl hash::Hash for InnerRecord {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.record_name.hash(state);
    }
}

#[tokio::test]
async fn test() {
    use crate::endpoint_record::endpoint::FormatString;
    use std::collections::HashMap;
    let ps = FormatString::new("api/<<user>>/<id>/details/<detail<<_id>".to_string());
    assert_eq!(
        ps.to_formatted_now(HashMap::from([
            ("id".to_string(), "something".to_string()),
            ("detail<_id".to_string(), "hii".to_string())
        ]))
        .await
        .unwrap(),
        "api/<user>/something/details/hii"
    );
}
