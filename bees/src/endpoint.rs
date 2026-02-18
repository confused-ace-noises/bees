use std::{
    future::ready,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use reqwest::Response;
use url::Url;

use super::net::net_error::NetError;
use crate::utils::error::Error;
use crate::{
    capability::Capability,
    handler::Handler,
    net::HttpVerb,
    record::Record,
    utils::format_string::FormatString,
};

pub trait EndpointInfo {
    type Record: Record;
    type CallContext;
    type EndpointHandler: Handler;

    const PATH: &str;

    fn caps(call: &Self::CallContext) -> Arc<[Box<dyn Capability>]>;
    fn endpoint_handler(call: &mut Self::CallContext) -> Self::EndpointHandler;
    fn http_verb(call: &mut Self::CallContext) -> impl Future<Output = HttpVerb>;

    #[allow(unused_variables)]
    fn modify_url(url: Url, call: &mut Self::CallContext) -> impl Future<Output = Url> {
        ready(url)
    }
}

pub trait EndpointExt: EndpointInfo {
    fn parsed_path() -> &'static FormatString;
    fn record_capabilities() -> Arc<[Box<dyn Capability>]>;
    fn full_url(
        call: &mut <Self as EndpointInfo>::CallContext,
    ) -> impl Future<Output = Result<Url, Error>>;
}

impl<E: EndpointInfo> EndpointExt for E {
    fn parsed_path() -> &'static FormatString {
        static PARSED: OnceLock<FormatString> = OnceLock::new();
        PARSED.get_or_init(|| FormatString::new(E::PATH))
    }

    async fn full_url(call: &mut <Self as EndpointInfo>::CallContext) -> Result<Url, Error> {
        let parsed = Self::parsed_path();
        let formatted = &parsed.to_formatted_now().await?;
        Ok(Self::modify_url(
            Url::from_str(formatted).map_err(NetError::NotAValidUrl)?,
            call,
        )
        .await)
    }

    fn record_capabilities() -> Arc<[Box<dyn Capability>]> {
        <<Self as EndpointInfo>::Record as Record>::shared_caps()
    }
}

pub trait EndpointProcessor<O>: EndpointInfo {
    type Process: Process;

    fn refine(proc_output: <Self::Process as Process>::ProcessOutput, call_context: &mut Self::CallContext) -> impl Future<Output = O>;
}

pub trait Process {
    type ProcessOutput;
    
    fn process(resp: Response) -> impl Future<Output = Self::ProcessOutput>;
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use super::*;
    use crate::handler::{BaseHandler, Retries, RetriesWrapper, WrapDecorate};
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
    
        async fn refine(proc_output: <Self::Process as Process>::ProcessOutput, call_context: &mut Self::CallContext) -> String {
            proc_output.text().await.unwrap()
        }
    }

    impl EndpointProcessor<u8> for Test {
        type Process = IntoTextProcessor;
    
        fn refine(proc_output: <Self::Process as Process>::ProcessOutput, call_context: &mut Self::CallContext) -> impl Future<Output = u8> {
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

    #[test]
    fn test_static_thing() {
        staticy_thing();
        staticy_thing();
        staticy_thing();
        staticy_thing();
    }

    fn staticy_thing() {
        static THING: OnceLock<usize> = OnceLock::new();
        if let Some(some) = THING.get() {
            println!("hit some: {some}")
        } else {
            println!("hit init");
            THING.set(3).unwrap();
        }
    }
}

