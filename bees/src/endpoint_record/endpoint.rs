use core::{fmt, hash};
use std::{any::{Any, TypeId}, borrow::Borrow, collections::HashMap, pin::Pin, sync::Arc};
use http::Method;
use reqwest::Response;
use std::hash::Hash;
use crate::{endpoint_record::record::Record, net, record, resource};

#[derive(Debug)]
pub struct Endpoint(Arc<InnerEndpoint>);

impl Endpoint {
    pub fn new<F, Fut, T>(
        record_name: String,
        name: String, 
        path: FormatString,
        http_verb:HttpVerb,
        capabilities: Arc<[Box<dyn Capability>]>,
        endpoint_output: F
    ) -> Self 
    where
        F: Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Any + Send + Sync + 'static,
    {
        Self(Arc::new(InnerEndpoint::new(&record!(&record_name), name, path, http_verb, capabilities, EndpointOutput::new(endpoint_output))))
    }

    pub fn new_template<F, Fut, T>(template: EndpointTemplate<F, Fut, T>) -> Self 
    where
        F: Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Any + Send + Sync + 'static,
    {
        Self::new(template.record_name, template.name, template.path, template.http_verb, template.capabilities, template.endpoint_output)
    }

    pub fn new_func<F, Fut, T>(func: impl FnOnce() -> EndpointTemplate<F, Fut, T>) -> Self 
    where
        F: Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Any + Send + Sync + 'static,
    {
        Self::new_template(func())
    }

    pub async fn full_url(&self, format_values: impl Borrow<HashMap<String, String>>, query_params: &Vec<(String, Option<String>)>) -> String {
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

    /// returns both the record's capabilities and the endpoint's capabilities, `(record capabilities, endpoint capabilities)`
    pub fn all_capabilities(&self) -> (Arc<[Box<dyn Capability>]>, Arc<[Box<dyn Capability>]>) {
        (Arc::clone(&self.record().capabilities()), Arc::clone(&self.endpoint_capabilities()))
    }

    pub fn record_name(&self) -> &String {
        &self.0.record.record_name()
    }

    pub fn record_constant_url(&self) -> &String {
        &self.0.record.constant_url()
    }

    pub fn record(&self) -> &Record {
        &self.0.record
    }

    pub async fn endpoint_output_specific<T: Any + Send + Sync + 'static>(&self, resp: Response) -> T {
        self.0.endpoint_output.run_typed(resp).await
    } 

    pub async fn endpoint_output(&self, resp: Response) -> Box<dyn Any + Send + Sync + 'static> {
        self.0.endpoint_output.run(resp).await
    } 
}

pub struct EndpointTemplate<F, Fut, T> 
where
    F: Fn(Response) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Any + Send + Sync + 'static,
{
    pub record_name: String,
    pub name: String, 
    pub path: FormatString,
    pub http_verb:HttpVerb,
    pub capabilities: Arc<[Box<dyn Capability>]>,
    pub endpoint_output: F
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

pub type ErasedAsyncFn =
    dyn Fn(Response) -> Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send>>
        + Send
        + Sync;

/// A stored async endpoint with erased output type.
pub struct EndpointOutput {
    func: Arc<ErasedAsyncFn>, // Arc so Fn can be called many times
    pub type_id: TypeId,
}

impl EndpointOutput {
    /// Creates a new endpoint taking an async function `F: Fn(Response) -> Fut`
    /// whose output type is `T`.
    pub fn new<F, Fut, T>(func: F) -> Self
    where
        F: Fn(Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
        T: Any + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();

        let erased: Arc<ErasedAsyncFn> = Arc::new(move |resp: Response| {
            let fut = func(resp);

            // We must erase T into Box<dyn Any>
            let fut = async move {
                let output = fut.await;
                Box::new(output) as Box<dyn Any + Send + Sync>
            };

            Box::pin(fut)
        });

        EndpointOutput { func: erased, type_id }
    }

    /// Run the endpoint asynchronously and return erased output.
    async fn run(&self, resp: Response) -> Box<dyn Any + Send + Sync> {
        (self.func)(resp).await
    }

    /// Run the endpoint and downcast to the requested type.
    pub async fn run_typed<T: Any + Send + Sync + 'static>(
        &self,
        resp: Response,
    ) -> T {
        if self.type_id != TypeId::of::<T>() {
            panic!("endpoint output function called with wrong output type");
        }

        let boxed = self.run(resp).await;

        boxed
            .downcast::<T>()
            .map(|b| *b)
            .unwrap_or_else(|_| {panic!("endpoint output function called with wrong output type")})
    }
}

pub(crate) struct InnerEndpoint {
    pub(crate) name: String,
    pub(crate) path: FormatString,
    pub(crate) http_verb: HttpVerb,
    pub(crate) capabilities: Arc<[Box<dyn Capability>]>, // only endpoint caps
    pub(crate) record: Record,
    pub(crate) endpoint_output: EndpointOutput,

}

impl fmt::Debug for InnerEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerEndpoint")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("http_verb", &self.http_verb)
            .field("capabilities", &self.capabilities)
            .field("record", &self.record)
            .field("endpoint_output", &self.endpoint_output.type_id())
            .finish()
    }
}

impl InnerEndpoint {

    // fn union_caps(a: Arc<[Box<dyn Capability>]>, b: Arc<[Box<dyn Capability>]>) -> Arc<[Box<dyn Capability>]> {
    //     let mut result: Vec<Box<dyn Capability>> = Vec::new();
        
    //     for (i, _) in a.iter().chain(b.iter()).enumerate() {
    //         let idx = i;
    //         let a_clone = Arc::clone(&a);

    //         let wrapper: Box<dyn Capability> = Box::new(move |request: net::client::RequestBuilder| {
    //             (a_clone[idx]).apply(request)
    //         });

    //         result.push(wrapper);
    //     }

    //     Arc::from(result)
    // }

    pub(crate) fn new(record: &Record, name: String, mut path: FormatString, http_enum: HttpVerb, capabilities: Arc<[Box<dyn Capability>]>, endpoint_output: EndpointOutput) -> Self {
        let vec = path.inner_vec_mut();

        match vec.get_mut(0) {
            Some(FormattableStringPart::Raw(s)) => {
                let trimmed = s.trim_start_matches('/').to_owned();
                s.clear();
                s.push('/');
                s.push_str(&trimmed);
            },

            Some(FormattableStringPart::HashMapReplace(_) 
            | FormattableStringPart::ResourceReplace(_)) 
            | None => {
                vec.insert(0, FormattableStringPart::Raw(String::from('/')));
            }
        }

        Self {
            record: record.clone(),
            name,
            path,
            http_verb: http_enum,
            capabilities,
            endpoint_output
        }
    }

    pub(crate) async fn full_url(&self, format_values: impl Borrow<HashMap<String, String>>, query_params: &Vec<(String, Option<String>)>) -> String {
        let mut string = format!("{}{}", self.record.constant_url(), self.path.to_formatted_now(format_values).await.expect("TODO: make a decent error system; format values should include all values to be formatted"));


        if query_params.len() != 0 {
            // TODO: check if this actually works
            if string.contains("?") {
                string.push_str("&");
            } else {
                string.push_str("?");
            }

            query_params.iter().for_each(|(key, value)| {
                string.push_str(key);
                if let Some(value) = value {
                    string.push_str(value);
                }                 
                string.push_str("&");
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
        self.name == other.name && self.record == other.record
    }
}
impl Eq for InnerEndpoint {}

impl hash::Hash for InnerEndpoint {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

pub trait Capability: Sync + Send {
    fn apply(&self, request: net::client::RequestBuilder) -> net::client::RequestBuilder;
}

impl Capability for dyn Fn(net::client::RequestBuilder) -> net::client::RequestBuilder + Send + Sync {
    fn apply(&self, request: net::client::RequestBuilder) -> net::client::RequestBuilder {
        (self)(request)
    }
}

impl<F: Fn(net::client::RequestBuilder) -> net::client::RequestBuilder + Send + Sync> Capability for F {
    fn apply(&self, request: net::client::RequestBuilder) -> net::client::RequestBuilder {
        (self)(request)
    }
}

impl PartialEq for Box<dyn Capability> {
    fn eq(&self, other: &Self) -> bool {
        (self as &dyn Any).type_id() == (other as &dyn Any).type_id()
    }
}

impl Eq for Box<dyn Capability> {}

impl Hash for Box<dyn Capability> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
    }
}

impl std::fmt::Debug for Box<dyn Capability> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Capability {{ type_id: {:?} }}", self.type_id())
    }
}

#[derive(Clone)]
pub enum Body {
    Text(FormatString),
    #[cfg(feature = "reqwest_json")]
    Json(serde_json::Value),
    #[cfg(feature = "reqwest_multipart")]
    Multipart(Arc<Box<dyn Fn(&HashMap<String, String>) -> reqwest::multipart::Form + Send + Sync>>),
}
impl fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            #[cfg(feature = "reqwest_json")]
            Self::Json(arg0) => f.debug_tuple("Json").field(arg0).finish(),
            #[cfg(feature = "reqwest_multipart")]
            Self::Multipart(_) => f.debug_tuple("Multipart").field(&"Box<dyn Fn(&HashMap<String, String>) -> reqwest::multipart::Form + Send + Sync>").finish(),
        }
    }
}

impl Body {
    pub async fn to_formatted(&self, values: impl Borrow<HashMap<String, String>>) -> reqwest::Body {
        match self {
            Body::Text(format_string) => format_string.to_formatted_now(values).await.expect("TODO: make a decent error system; format values should include all values to be formatted").into(),
    
            #[cfg(feature = "reqwest_json")]
            Body::Json(value) => {
                FormatString::new(value.to_string()).to_formatted_now(values).await.expect("TODO: make a decent error system; format values should include all values to be formatted").into()
            }

            #[cfg(feature = "reqwest_multipart")]
            Body::Multipart(multipart) => {
                ((multipart)(values.borrow())).into()
            }
        }
    }
}

#[derive(Debug)]
pub enum HttpVerb {
    GET,
    POST(Body),
    PUT(Body),
    DELETE(Option<Body>),
    PATCH(Body),
    OPTIONS,
    HEAD,
}
impl HttpVerb {
    pub fn as_method(&self) -> Method {
        match self {
            HttpVerb::GET => Method::GET,
            HttpVerb::POST(_) => Method::POST,
            HttpVerb::PUT(_) => Method::PUT,
            HttpVerb::DELETE(_) => Method::DELETE,
            HttpVerb::PATCH(_) => Method::PATCH,
            HttpVerb::OPTIONS => Method::OPTIONS,
            HttpVerb::HEAD => Method::HEAD,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatString {
    parts: Vec<FormattableStringPart>
}

impl FormatString {
    pub fn new(raw: impl AsRef<str>) -> Self {
        let raw = raw.as_ref();
        let mut chars = raw.chars().peekable();
        let mut raw_sting_buffer = String::new();
        let mut parts: Vec<FormattableStringPart> = Vec::new();

        'outer: while let Some(c) = chars.next() {
            match c {
                '<' => {
                    if let Some(&'<') = chars.peek() {
                        let _ = chars.next();
                        raw_sting_buffer.push('<');
                        continue 'outer;
                    } else {
                        parts.push(FormattableStringPart::Raw(raw_sting_buffer));
                        raw_sting_buffer = String::new();
                        
                        let mut part = String::new();
                        
                        'inner: while let Some(c_part) = chars.next() {
                            match c_part {
                                '>' => {
                                    if let Some(&'>') = chars.peek() {
                                        let _ = chars.next();
                                        part.push('>');
                                        continue 'inner;
                                    } else {
                                        break 'inner;
                                    }
                                }

                                '<' => {
                                    if let Some(&'<') = chars.peek() {
                                        let _ = chars.next();
                                        part.push('<');
                                        continue 'inner;
                                    } else {
                                        panic!("invalid formattable string in FormatString: lone \'<\' inside formattable section (did you mean \'<<\'?)")
                                    }
                                }

                                a @ _ => part.push(a),
                            }
                        }

                        if part.starts_with('?') {
                            let part = part.chars().skip(1).collect::<String>();
                            parts.push(FormattableStringPart::ResourceReplace(part));
                        } else {
                            parts.push(FormattableStringPart::HashMapReplace(part));
                        }
                    }
                },

                '>' => {
                    if let Some(&'>') = chars.peek() {
                        let _ = chars.next();
                        raw_sting_buffer.push('>')
                    } else {
                        panic!("invalid formattable string in FormatString: unpaired \'>\' inside raw section (did you mean \'>>\'?)")
                    }
                }

                c @ _ => raw_sting_buffer.push(c),
            }
        }

        if !raw_sting_buffer.is_empty() {
            parts.push(FormattableStringPart::Raw(raw_sting_buffer));
        }

        Self {
            parts
        }
    }

    pub async fn to_formatted_now(&self, values: impl std::borrow::Borrow<std::collections::HashMap<String, String>>) -> Result<String, &'static str> {
        let values = values.borrow();
        let mut result = String::new();

        for part in self.parts.iter() {
            match part {
                FormattableStringPart::Raw(raw) => result.push_str(&raw),
                FormattableStringPart::HashMapReplace(replace_name) => result.push_str(values.get(replace_name).ok_or("hashmap replace field not specified in values hashmap.")?),
                FormattableStringPart::ResourceReplace(resource_replace) => {
                    let resource = resource!(option resource_replace);
                    // println!("{:#?}", resource);
                    // println!("{}", resource_replace);
                    let resource = resource.ok_or("No resource with that name registered. Did you spell it right?")?;

                    let data = resource.data().await;
                    result.push_str(&data.to_string());
                },
            }
        }
        
        print!("{result}");

        Ok(result)
    }

    #[allow(dead_code)]
    pub(crate) fn inner_vec(&self) -> &Vec<FormattableStringPart> {
        &self.parts
    }

    pub(crate) fn inner_vec_mut(&mut self) -> &mut Vec<FormattableStringPart> {
        &mut self.parts
    }
}

impl<S: Into<String>> From<S> for FormatString {
    fn from(value: S) -> Self {
        let string = value.into();
        Self::new(string)
    }
}

#[derive(Debug, Clone)]
pub enum FormattableStringPart {
    Raw(String),
    HashMapReplace(String),
    ResourceReplace(String),
}

#[test]
fn test() {
    // let thing = aaa(Box::new(Cap));
    // let mut string = String::from("hello world");

    // string.chars().skip(1).
}
