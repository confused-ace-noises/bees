use bees::{Endpoint, Record, capability::Capability, endpoint::no_op_processor, prelude::HttpVerb, re_exports::reqwest::{Response, header::HeaderMap}, record};

struct DummyCap;

impl Capability for DummyCap {
    fn apply(&self, request: bees::net::request::RequestBuilder) -> bees::net::request::RequestBuilder {
        request
    }
}

struct DummyCap2 {
    pub header: HeaderMap
}

impl Capability for DummyCap2 {
    fn apply(&self, request: bees::net::request::RequestBuilder) -> bees::net::request::RequestBuilder {
        request.headers(self.header.clone())
    }
}

#[derive(Record)]
#[record(url = "https://my.api.org/api", name = "RecordName")]
#[record(capabilities = [DummyCap])]
pub struct MyRecord;

fn http_verb() -> HttpVerb {
    HttpVerb::GET
}

#[derive(Debug, Endpoint)]
#[endpoint(
        record_name = "RecordName",
        http_verb = http_verb(),
        processors(no_op_processor, get_body),
        path = "api/endpoint"
    )
]
#[endpoint(capabilities = [DummyCap, DummyCap2 { header: HeaderMap::new() }])]
pub struct MyEndpoint;

pub async fn get_body(resp: Response) -> String {
    resp.text().await.unwrap()
}

fn main () {
    unsafe { 
        bees::context::pre_context::push_pre_context(MyEndpoint); 
        bees::context::pre_context::push_pre_context(MyRecord); 
    };
    bees::init_default();

    let record = record!("RecordName");
    
    dbg!(record);

    // let t = trybuild::TestCases::new();
    // t.pass("tests/record_ok.rs");
}
