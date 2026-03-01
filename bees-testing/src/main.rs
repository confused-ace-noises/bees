use std::{future::ready, sync::Arc};

use async_rate_limiter::RateLimiter;
use bees::{
    self, Endpoint, EndpointProcessor, Record, capability::Capability, endpoint::{self, EndpointInfo, EndpointProcessor, Process}, handler::{BaseHandler, Handler, Retries, RetriesWrapper, WrapDecorate}, net::{
        Client, HttpVerb,
        bodies::{Body, TextBody},
    }, process, provided::capabilities::add_headers::{AddHeaderMap, AddHeaders}, record::Record
};
use reqwest::{Response, header::HeaderMap};
use url::Url;

#[tokio::main]
async fn main() {
    let client = Client::new(reqwest::Client::new(), RateLimiter::new(2));

    let endpoint_runner = client.run_endpoint_with::<Test>(UrlContext(Vec::new()));

    let endpoint_runner_2 = endpoint_runner.wrap(RetriesWrapper::<2>);

    let x: Result<String, bees::net::EndpointRunnerError<_>> = endpoint_runner_2
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

struct NoOpProcessor;

impl Process for NoOpProcessor {
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
async fn IntoTextProcessor(resp: Response) -> String {
    resp.text().await.unwrap()
}

// impl EndpointProcessor<String> for Test {
//     type Process = NoOpProcessor;

//     async fn refine(
//         proc_output: <Self::Process as Process>::ProcessOutput,
//         call_context: &Self::CallContext,
//     ) -> String {
//         proc_output.text().await.unwrap()
//     }
// }

impl EndpointProcessor<u8> for Test {
    type Process = IntoTextProcessor;

    fn refine(
        proc_output: <Self::Process as Process>::ProcessOutput,
        _: &Self::CallContext,
    ) -> impl Future<Output = u8> {
        ready(proc_output.as_bytes()[0])
    }
}

async fn url_func(url: Url) -> Url {
    url
}

#[derive(Debug, Endpoint)]
#[endpoint(
    record = TestRecord,
    handler(BaseHandler, BaseHandler),
    http_verb = HttpVerb::GET,
    path = "idk",
    modify_url = url_func,
    processors(NoOpProcessor, IntoTextProcessor)
)]
struct Test2;

#[derive(Debug, EndpointProcessor)]
#[process(NoOpProcessor)]
#[process(IntoTextProcessor)]
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

    async fn http_verb(_: &Self::CallContext) -> HttpVerb {
        HttpVerb::GET
    }

    async fn modify_url(mut url: Url, call: &Self::CallContext) -> Url {
        call.append_to_url(&mut url);
        url
    }
}

struct Thing<T>(T) where T: EndpointProcessor<Response>;
const _: () = {
    let t = Thing(Test);
};

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
