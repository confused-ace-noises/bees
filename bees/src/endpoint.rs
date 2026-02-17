use std::{
    future::ready,
    str::FromStr,
    sync::{Arc, Once, OnceLock},
};

use reqwest::Response;
use url::Url;

use super::net::net_error::NetError;
use crate::utils::error::Error;
use crate::{
    capability::Capability,
    handler::{BaseHandler, Handler},
    net::HttpVerb,
    record::Record,
    utils::format_string::FormatString,
};

pub trait EndpointInfo {
    type Record: Record;
    type CallContext;

    const PATH: &str;

    fn caps(call: &mut Self::CallContext) -> Arc<[Box<dyn Capability>]>;
    fn base_handler(base_handler: BaseHandler, call: &mut Self::CallContext) -> impl Handler;
    fn http_verb(call: &mut Self::CallContext) -> HttpVerb;

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

pub trait Processor<T> {
    fn process(&mut self, resp: Response) -> impl Future<Output = T>;
}

impl<E: EndpointInfo> Processor<Response> for E {
    async fn process(&mut self, resp: Response) -> Response {
        resp
    }
}

pub struct Body(Box<dyn BodyAdder>);
pub trait BodyAdder: Capability + std::fmt::Debug + Send + Sync {}

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

    struct Test;
    impl EndpointInfo for Test {
        type Record = TestR;
        type CallContext = UrlContext;

        const PATH: &str = "idk";

        fn caps(_: &mut Self::CallContext) -> Arc<[Box<dyn Capability>]> {
            Arc::new([])
        }

        fn base_handler(base_handler: BaseHandler, _: &mut Self::CallContext) -> impl Handler {
            base_handler.wrap(RetriesWrapper::<3>)
        }

        fn http_verb(_: &mut Self::CallContext) -> HttpVerb {
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
                    query_pairs.append_pair(&key, &value);
                } else {
                    query_pairs.append_key_only(&key);
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

