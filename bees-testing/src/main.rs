use std::{future::ready, sync::Arc};

use async_rate_limiter::RateLimiter;
use bees::{
    self,
    capability::Capability,
    endpoint::{self, EndpointInfo, EndpointProcessor, Process},
    handler::{BaseHandler, Handler, Retries, RetriesWrapper, WrapDecorate},
    net::{
        Client, HttpVerb,
        bodies::{Body, TextBody},
    },
    record::Record,
};
use reqwest::Response;
use url::Url;

#[tokio::main]
async fn main() {
    let mut client = Client::new(reqwest::Client::new(), RateLimiter::new(2));

    let endpoint_runner = client.run_endpoint::<Test>(UrlContext(Vec::new()));

    let mut endpoint_runner_2 = endpoint_runner.wrap(RetriesWrapper::<2>);

    // let x = endpoint_runner_2
    //     .run::<Result<std::string::String, reqwest::Error>>()
    //     .await;
    

    println!("{client:?}")
}

pub struct TestR;
impl Record for TestR {
    const SHARED_URL: &str = "https://idk.com/";
    fn shared_caps() -> Arc<[Box<dyn Capability>]> {
        Arc::new([])
    }
}

struct NoOpProcessor;

impl Process for NoOpProcessor {
    type ProcessOutput = Response;

    fn process(resp: Response) -> impl Future<Output = Self::ProcessOutput> {
        ready(resp)
    }
}

struct IntoTextProcessor;

impl Process for IntoTextProcessor {
    type ProcessOutput = String;

    async fn process(resp: Response) -> Self::ProcessOutput {
        resp.text().await.unwrap()
    }
}

impl EndpointProcessor<String> for Test {
    type Process = NoOpProcessor;

    async fn refine(
        proc_output: <Self::Process as Process>::ProcessOutput,
        call_context: &mut Self::CallContext,
    ) -> String {
        proc_output.text().await.unwrap()
    }
}

impl EndpointProcessor<u8> for Test {
    type Process = IntoTextProcessor;

    fn refine(
        proc_output: <Self::Process as Process>::ProcessOutput,
        call_context: &mut Self::CallContext,
    ) -> impl Future<Output = u8> {
        ready(proc_output.as_bytes()[0])
    }
}

struct Test;
impl EndpointInfo for Test {
    type Record = TestR;
    type CallContext = UrlContext;
    type EndpointHandler = Retries<BaseHandler, 3>;

    const PATH: &str = "idk";

    fn caps(_: &Self::CallContext) -> Arc<[Box<dyn Capability>]> {
        Arc::new([])
    }

    fn endpoint_handler(_: &mut Self::CallContext) -> Self::EndpointHandler {
        BaseHandler.wrap(RetriesWrapper::<3>)
    }

    async fn http_verb(_: &mut Self::CallContext) -> HttpVerb {
        HttpVerb::GET
    }

    async fn modify_url(mut url: Url, call: &mut Self::CallContext) -> Url {
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
