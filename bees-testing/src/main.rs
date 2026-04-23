use std::{future::ready, sync::Arc};

use bees::{
    self, Endpoint, EndpointProcessor, Record, capability::Capability, endpoint::{EndpointInfo, Process, SupportsOutput}, handler::{BaseHandler, Retries, RetriesWrapper, WrapDecorate}, net::{
        Client, HttpMethod, HttpVerb, rate_limiter::RateLimiter
    }, process, provided::capabilities::add_headers::{AddHeaderMap, AddHeaders}
};
use reqwest::{Response, header::HeaderMap};
use url::Url;
pub mod expanded;

#[tokio::main]
async fn main() {
    let client = Client::new(reqwest::Client::new(), RateLimiter::new(2.0, 10));

    let endpoint_runner = client.run_endpoint_with::<Test>(UrlContext(Vec::new()));

    let endpoint_runner_2 = endpoint_runner.wrap(RetriesWrapper::<2>);

    let _x: Result<String, bees::net::EndpointRunnerError<_>> = endpoint_runner_2
        .run::<String>()
        .await;
    

    println!("{client:?}")
}

#[derive(Record)]
#[record(
    path = "https://idk.com/",
    capabilities([AddHeaders(Vec::new()), AddHeaderMap(HeaderMap::new())])
)]
pub struct TestRecord;

struct NoOpProcess;

impl Process for NoOpProcess {
    type ProcessOutput = Response;

    fn process(resp: Response) -> impl Future<Output = Self::ProcessOutput> {
        ready(resp)
    }
}

// struct IntoTextProcessor;

// impl Process for IntoTextProcessor {
//     type ProcessOutput = String;

//     async fn process(resp: Response) -> Self::ProcessOutput {
//         resp.text().await.unwrap()
//     }
// }

#[process]
async fn IntoTextProcess(resp: Response) -> String {
    resp.text().await.unwrap()
}

async fn url_func(url: Url) -> Url {
    url
}

#[derive(Debug, Endpoint, EndpointProcessor)]
#[endpoint(
    record = TestRecord,
    handler =  {let x = BaseHandler; x} -> BaseHandler,
    http_verb = HttpMethod::new_no_body(HttpVerb::GET),
    path = "idk",
    modify_url = url_func,
)]
#[process(NoOpProcess, IntoTextProcess)]
struct Test2;

#[derive(Debug, EndpointProcessor)]
#[process(NoOpProcess)]
#[process(IntoTextProcess)]
struct Test;

impl EndpointInfo for Test {
    type Record = TestRecord;
    type CallContext = UrlContext;
    type EndpointHandler = Retries<BaseHandler, 3>;

    const PATH: &str = "idk";

    fn capabilities(_: &Self::CallContext) -> Arc<[Box<dyn Capability>]> {
        Arc::new([])
    }

    fn endpoint_handler(_: &Self::CallContext) -> Self::EndpointHandler {
        BaseHandler.wrap(RetriesWrapper::<3>)
    }

    async fn http_verb(_: &Self::CallContext) -> HttpMethod {
        HttpMethod::new_no_body(HttpVerb::GET)
    }

    async fn modify_url(mut url: Url, call: &Self::CallContext) -> Url {
        call.append_to_url(&mut url);
        url
    }
}

struct UrlContext(Vec<(String, Option<String>)>);

impl UrlContext {
    pub fn append_to_url(&self, url: &mut Url) {
        let mut query_pairs = url.query_pairs_mut();
        for (key, maybe_value) in self.0.iter() {
            if let Some(value) = maybe_value {
                query_pairs.append_pair(key, value);
            } else {
                query_pairs.append_key_only(key);
            }
        }
    }
}

struct Thing<T>(T) where T: SupportsOutput<Response>;
const _: () = {
    let t = Thing(Test);
};