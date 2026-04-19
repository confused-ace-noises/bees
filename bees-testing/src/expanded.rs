#![feature(prelude_import)]
#[macro_use]
// extern crate std;
// #[prelude_import]
use std::prelude::rust_2024::*;
use std::{future::ready, sync::Arc};
use bees::{
    self, Endpoint, EndpointProcessor, Record, capability::Capability,
    endpoint::{EndpointInfo, SupportsOutput, Process},
    handler::{BaseHandler, Retries, RetriesWrapper, WrapDecorate},
    net::{Client, HttpVerb},
    process, provided::capabilities::add_headers::{AddHeaderMap, AddHeaders},
};
use reqwest::{Response, header::HeaderMap};
use url::Url;
// fn main() {
//     // let body = async {
//     //     let client = Client::new(reqwest::Client::new(), RateLimiter::new(2));
//     //     let endpoint_runner = client.run_endpoint_with::<Test>(UrlContext(Vec::new()));
//     //     let endpoint_runner_2 = endpoint_runner.wrap(RetriesWrapper::<2>);
//     //     let _x: Result<String, bees::net::EndpointRunnerError<_>> = endpoint_runner_2
//     //         .run::<String>()
//     //         .await;
//     //     {
//     //         ::std::io::_print(format_args!("{0:?}\n", client));
//     //     }
//     // };
//     #[allow(
//         clippy::expect_used,
//         clippy::diverging_sub_expression,
//         clippy::needless_return,
//         clippy::unwrap_in_result
//     )]
//     {
//         return tokio::runtime::Builder::new_multi_thread()
//             .enable_all()
//             .build()
//             .expect("Failed building the Runtime")
//             .block_on(body);
//     }
// }

pub struct TestRecord;
#[automatically_derived]
impl ::bees::record::Record for TestRecord {
    const SHARED_URL: &str = "https://idk.com/";
    fn shared_caps() -> Arc<[Box<dyn Capability>]> {
        ::std::sync::Arc::new([
            ::std::boxed::Box::new(AddHeaders(Vec::new()))
                as ::std::boxed::Box<dyn ::bees::capability::Capability>,
            ::std::boxed::Box::new(AddHeaderMap(HeaderMap::new()))
                as ::std::boxed::Box<dyn ::bees::capability::Capability>,
        ])
    }
}
struct NoOpProcess;
impl Process for NoOpProcess {
    type ProcessOutput = Response;
    fn process(resp: Response) -> impl Future<Output = Self::ProcessOutput> {
        ready(resp)
    }
}
struct IntoTextProcess;
impl ::bees::endpoint::Process for IntoTextProcess {
    type ProcessOutput = String;
    fn process(
        _IntoTextProcess__: Response,
    ) -> impl Future<Output = Self::ProcessOutput> + Send {
        #[allow(non_snake_case)]
        async fn _IntoTextProcess(resp: Response) -> String {
            resp.text().await.unwrap()
        }
        _IntoTextProcess(_IntoTextProcess__)
    }
}
async fn url_func(url: Url) -> Url {
    url
}

struct Test2;
#[automatically_derived]
impl ::core::fmt::Debug for Test2 {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Test2")
    }
}
#[automatically_derived]
impl ::bees::endpoint::EndpointInfo for Test2 {
    const PATH: &str = "idk";
    type EndpointHandler = BaseHandler;
    type Record = TestRecord;
    type CallContext = ();
    #[allow(clippy::manual_async_fn)]
    fn http_verb(_: &Self::CallContext) -> impl Future<Output = HttpVerb> + Send {
        async move { HttpVerb::GET }
    }
    fn capabilities(_: &Self::CallContext) -> Arc<[Box<dyn Capability>]> {
        ::std::sync::Arc::new([])
    }
    fn endpoint_handler(_: &Self::CallContext) -> Self::EndpointHandler {
        let x = BaseHandler;
        x
    }
    #[allow(clippy::manual_async_fn)]
    fn modify_url(
        ____url___: ::bees::re_exports::url::Url,
        _: &Self::CallContext,
    ) -> impl ::std::future::Future<
        Output = ::bees::re_exports::url::Url,
    > + ::std::marker::Send {
        url_func(____url___)
    }
}
#[automatically_derived]
impl ::bees::endpoint::SupportsOutput<
    <NoOpProcess as ::bees::endpoint::Process>::ProcessOutput,
> for Test2 {
    type Process = NoOpProcess;
}
#[automatically_derived]
impl ::bees::endpoint::SupportsOutput<
    <IntoTextProcess as ::bees::endpoint::Process>::ProcessOutput,
> for Test2 {
    type Process = IntoTextProcess;
}

struct Test;
#[automatically_derived]
impl ::core::fmt::Debug for Test {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Test")
    }
}
#[automatically_derived]
impl ::bees::endpoint::SupportsOutput<
    <NoOpProcess as ::bees::endpoint::Process>::ProcessOutput,
> for Test {
    type Process = NoOpProcess;
}
#[automatically_derived]
impl ::bees::endpoint::SupportsOutput<
    <IntoTextProcess as ::bees::endpoint::Process>::ProcessOutput,
> for Test {
    type Process = IntoTextProcess;
}
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
struct Thing<T>(
    T,
)
where
    T: SupportsOutput<Response>;
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
