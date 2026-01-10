// use bees::{
//     Endpoint, Record, capability::Capability, endpoint_def::no_op_processor, init_pre_context, net::client, prelude::HttpVerb, re_exports::reqwest::{Response, header::HeaderMap}, record
// };
// #[cfg(predicate)]
// use bees::capability::CapabilityOutput;

// use bees::endpoint;

// struct DummyCap;
// impl Capability for DummyCap {
//     fn apply<'a>(
//         &'a self,
//         request: bees::net::request::RequestBuilder,
//     ) -> CapabilityOutput<'a> {
//         CapabilityOutput::new(async move { Ok(request) })
//     }
// }

// struct DummyCap2 {
//     pub header: HeaderMap,
// }
// impl Capability for DummyCap2 {
//     fn apply<'a>(
//         &'a self,
//         request: bees::net::request::RequestBuilder,
//     ) -> CapabilityOutput<'a> {
//         CapabilityOutput::new(async move { Ok(request.headers(self.header.clone())) })
//     }
// }

// #[derive(Record)]
// #[record(url = "https://my.api.org/api", name = "RecordName")]
// #[record(capabilities = [DummyCap])]
// pub struct MyRecord;

// fn http_verb() -> HttpVerb {
//     HttpVerb::GET
// }

// #[derive(Debug, Endpoint)]
// #[endpoint(
//         record_name = "RecordName",
//         http_verb = http_verb(),
//         processors(no_op_processor, get_body),
//         path = "api/endpoint"
//     )
// ]
// #[endpoint(capabilities = [DummyCap, DummyCap2 { header: HeaderMap::new() }])]
// pub struct MyEndpoint;

// pub async fn get_body(resp: Response) -> String {
//     resp.text().await.unwrap()
// }

// fn main() {
//     init_pre_context!(MyEndpoint, MyRecord);

//     bees::init(2);

//     record!("RecordName2" => "https://api.com/api");
//     endpoint!("RecordName2" => new "MyEndpoint2", "/endpoint", HttpVerb::GET, no_op_processor);

//     let _client = client();

//     let record = record!("RecordName");



//     dbg!(record);

//     // let t = trybuild::TestCases::new();
//     // t.pass("tests/record_ok.rs");
// }

// /*
//     Record: contiene endpoint
// */

// /*
//     http://google.com/api/ricerca?ricerca=the+rust+book
//     http://google.com/api/accedi

//     POST body {"risulatato1": "sito1", ...}


//     GET "passami l'html" 200 <- non ha body 
//     POST "ti passo dati io, te me ritorni altri" <- ha un body 
// */

fn main() {
    
}