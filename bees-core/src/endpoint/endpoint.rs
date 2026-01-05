use crate::record;
use crate::utils::format_string::FormatString;
use crate::{
    capability::Capability,
    net::client::HttpVerb,
    record::{Record, RecordInfo},
    utils::format_string::FormattableStringPart,
};
use core::{fmt, hash};
use dashmap::DashMap;
use reqwest::Response;
use std::{
    any::{Any, TypeId},
    borrow::Borrow,
    collections::HashMap,
    fmt::Debug,
    pin::Pin,
    sync::Arc,
};

#[derive(Debug)]
pub struct Endpoint(Arc<InnerEndpoint>);

impl Endpoint {
    pub fn new(
        record_name: String,
        name: String,
        path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
        endpoint_outputs: EndpointProcessors,
    ) -> Self {
        unsafe {
            Self::new_unchecked_record(
                &record!(&record_name),
                name,
                path,
                http_verb,
                capabilities,
                endpoint_outputs,
            )
        }
    }

    /// # Safety
    /// doesn't check if the record exists before creating the endpoint
    /// callers must ensure that the record exists at the time of calling.
    pub unsafe fn new_unchecked_record(
        record: &Record,
        name: String,
        path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
        endpoint_outputs: EndpointProcessors,
    ) -> Self {
        unsafe {
            Self::new_unchecked(
                record.into(),
                name,
                path,
                http_verb,
                capabilities,
                endpoint_outputs,
            )
        }
    }

    /// # Safety
    /// doesn't check if the record exists before creating the endpoint.
    /// callers must ensure that the record exists at the time of calling.
    pub unsafe fn new_unchecked(
        record_info: RecordInfo,
        name: String,
        path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
        endpoint_outputs: EndpointProcessors,
    ) -> Self {
        Self(Arc::new(unsafe {
            InnerEndpoint::new_direct(
                record_info,
                name,
                path,
                http_verb,
                capabilities,
                endpoint_outputs,
            )
        }))
    }

    pub fn builder(
        record_name: String,
        name: String,
        path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
    ) -> EndpointBuilder {
        EndpointBuilder::new(record_name, name, path, http_verb, capabilities)
    }

    pub fn builder_template(template: EndpointTemplate) -> EndpointBuilder {
        EndpointBuilder::new_template(template)
    }

    // pub fn new_func<F, Fut, T>(func: impl FnOnce() -> EndpointTemplate<F, Fut, T>) -> Self
    // where
    //     F: Fn(Response) -> Fut + Send + Sync + 'static,
    //     Fut: Future<Output = T> + Send + 'static,
    //     T: Any + Send + Sync + 'static,
    // {
    //     Self::new_template(func())
    // }

    pub async fn full_url(
        &self,
        format_values: impl Borrow<HashMap<String, String>>,
        query_params: &[(String, Option<String>)],
    ) -> String {
        self.0.full_url(format_values, query_params).await
    }

    pub fn http_verb(&self) -> &HttpVerb {
        &self.0.http_verb
    }

    pub fn name(&self) -> &String {
        &self.0.name
    }

    pub fn path(&self) -> &FormatString {
        &self.0.path
    }

    /// returns only the capabilities defined on this endpoint, not including the record's capabilities
    pub fn endpoint_capabilities(&self) -> Arc<[Box<dyn Capability>]> {
        Arc::clone(&self.0.capabilities)
    }

    #[allow(clippy::type_complexity)]
    /// returns both the record's capabilities and the endpoint's capabilities, `(record capabilities, endpoint capabilities)`
    pub fn all_capabilities(&self) -> (Arc<[Box<dyn Capability>]>, Arc<[Box<dyn Capability>]>) {
        (
            Arc::clone(self.record_info().capabilities()),
            Arc::clone(&self.endpoint_capabilities()),
        )
    }

    pub fn record_name(&self) -> &String {
        self.0.record_info.record_name()
    }

    pub fn record_constant_url(&self) -> &String {
        self.0.record_info.constant_url()
    }

    pub(crate) fn record_info(&self) -> &RecordInfo {
        &self.0.record_info
    }

    pub async fn endpoint_output_specific<T: Any + Send + Sync + 'static>(
        &self,
        resp: Response,
    ) -> T {
        self.0.endpoint_outputs.run::<T>(resp).await
    }
}

pub struct EndpointTemplate {
    pub record_name: String,
    pub name: String,
    pub path: FormatString,
    pub http_verb: HttpVerb,
    pub capabilities: Arc<[Box<dyn Capability>]>,
}

pub struct EndpointBuilder {
    pub record_name: String,
    pub name: String,
    pub path: FormatString,
    pub http_verb: HttpVerb,
    pub capabilities: Arc<[Box<dyn Capability>]>,
    pub endpoint_output: EndpointProcessors,
}

impl EndpointBuilder {
    pub fn new(
        record_name: String,
        name: String,
        path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
    ) -> Self {
        Self {
            record_name,
            name,
            path,
            http_verb,
            capabilities,
            endpoint_output: EndpointProcessors::new(),
        }
    }

    pub fn new_template(template: EndpointTemplate) -> Self {
        Self {
            record_name: template.record_name,
            name: template.name,
            path: template.path,
            http_verb: template.http_verb,
            capabilities: template.capabilities,
            endpoint_output: EndpointProcessors::new(),
        }
    }

    pub fn push_endpoint_output<F, Fut, O>(&mut self, func: F) -> &mut Self
    where
        F: Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = O> + Send + 'static,
        O: Any + Send + Sync + 'static,
    {
        self.endpoint_output.insert(func);
        self
    }

    pub fn build(self) -> Endpoint {
        Endpoint::new(
            self.record_name,
            self.name,
            self.path,
            self.http_verb,
            self.capabilities,
            self.endpoint_output,
        )
    }

    /// # Safety
    /// uses [`Endpoint::new_unchecked`], and thus doesn't check
    /// for the existence of the [`Record`] before creating the [`Endpoint`].
    /// Callers must ensure that the [`Record`] exists at the time of calling.
    pub unsafe fn build_unchecked (
        self,
        record_constant_url: String,
        record_shared_capabilities: Arc<[Box<dyn Capability + 'static>]>,
    ) -> Endpoint {
        unsafe {
            Endpoint::new_unchecked(
                RecordInfo {
                    record_name: self.record_name,
                    constant_url: record_constant_url,
                    shared_capabilities: record_shared_capabilities,
                },
                self.name,
                self.path,
                self.http_verb,
                self.capabilities,
                self.endpoint_output,
            )
        }
    }
}

impl Clone for Endpoint {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Endpoint {}

impl hash::Hash for Endpoint {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
    }
}

impl Borrow<str> for Endpoint {
    fn borrow(&self) -> &str {
        &self.0.name
    }
}

pub type Processor = dyn Fn(Response) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send>>
    + Send
    + Sync;

pub async fn no_op_processor(x: Response) -> Response {
    x
}

/// A stored async endpoint with erased output type.
pub struct EndpointProcessors(DashMap<TypeId, Arc<Processor>>);

impl EndpointProcessors {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    /// Creates a new endpoint taking an async function `F: Fn(Response) -> Fut`
    /// whose output type is `T`.
    pub fn insert<F, Fut, O>(&mut self, func: F)
    where
        F: Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = O> + Send + 'static,
        O: Any + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<O>();

        let erased: Arc<Processor> = Arc::new(move |resp: Response| {
            let fut = func(resp);

            // We must erase T into Box<dyn Any>
            let fut = async move {
                let output = fut.await;
                Box::new(output) as Box<dyn Any + Send + Sync>
            };

            Box::pin(fut)
        });

        if self.0.insert(type_id, erased).is_some() {
            panic!("No one endpoint can have multiple output processors that return the same type.")
        }
    }

    /// Run the endpoint and downcast to the requested type.
    pub async fn run<O: Any + Send + Sync + 'static>(&self, resp: Response) -> O {
        let type_id = TypeId::of::<O>();
        let func = self.0.get(&type_id).unwrap_or_else(|| {
            panic!(
                "type id {:?} didn't match any output functions for the selected endpoint",
                type_id
            )
        });

        *func(resp).await.downcast::<O>().unwrap_or_else(|_| {
            panic!("`run` in EndpointOutput could not properly downcast. Report this.")
        })
    }
}

impl Default for EndpointProcessors {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EndpointProcessors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pmap = f.debug_map();

        for r in self.0.iter() {
            let k = r.key();

            pmap.entry(&k as &dyn Debug, &"Arc<dyn Fn(Response) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync + 'static>> + Send + 'static>> + Send + Sync>");
        }

        pmap.finish()
    }
}

#[derive(Debug)]
pub(crate) struct InnerEndpoint {
    pub(crate) name: String,
    pub(crate) path: FormatString,
    pub(crate) http_verb: HttpVerb,
    pub(crate) capabilities: Arc<[Box<dyn Capability>]>, // only endpoint caps
    pub(crate) record_info: RecordInfo,
    pub(crate) endpoint_outputs: EndpointProcessors,
}

impl InnerEndpoint {
    pub(crate) unsafe fn new_direct(
        record_info: RecordInfo,
        name: String,
        mut path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
        endpoint_outputs: EndpointProcessors,
    ) -> Self {
        let vec = path.inner_vec_mut();

        match vec.get_mut(0) {
            Some(FormattableStringPart::Raw(s)) => {
                let trimmed = s.trim_start_matches('/').to_owned();
                s.clear();
                s.push('/');
                s.push_str(&trimmed);
            }

            Some(
                FormattableStringPart::HashMapReplace(_)
                | FormattableStringPart::ResourceReplace(_),
            )
            | None => {
                vec.insert(0, FormattableStringPart::Raw(String::from('/')));
            }
        }

        Self {
            record_info,
            name,
            path,
            http_verb,
            capabilities,
            endpoint_outputs,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new(
        record: &Record,
        name: String,
        path: FormatString,
        http_verb: HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
        endpoint_outputs: EndpointProcessors,
    ) -> Self {
        unsafe {
            Self::new_direct(
                record.into(),
                name,
                path,
                http_verb,
                capabilities,
                endpoint_outputs,
            )
        }
    }

    pub(crate) async fn full_url(
        &self,
        format_values: impl Borrow<HashMap<String, String>>,
        // clippy said to turn this into a [] from a Vec, need to consider if it's actually ergonomic
        query_params: &[(String, Option<String>)],
    ) -> String {
        let mut string = format!("{}{}", self.record_info.constant_url(), self.path.to_formatted_now(format_values).await.expect("TODO: make a decent error system; format values should include all values to be formatted"));

        if !query_params.is_empty() {
            // TODO: check if this actually works
            if string.contains("?") {
                string.push('&');
            } else {
                string.push('?');
            }

            query_params.iter().for_each(|(key, value)| {
                string.push_str(key);
                if let Some(value) = value {
                    string.push_str(value);
                }
                string.push('&');
            });

            string.pop(); // pops the last & 
        }

        string
    }
}

impl Borrow<str> for InnerEndpoint {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for InnerEndpoint {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.record_info == other.record_info
    }
}
impl Eq for InnerEndpoint {}

impl hash::Hash for InnerEndpoint {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[test]
fn test() {
    // let thing = aaa(Box::new(Cap));
    // let mut string = String::from("hello world");

    // string.chars().skip(1).
}
