#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use bees::{
    Endpoint, HandlerStacks, Record, chain,
    endpoint::{EndpointInfo, HandlerStack, HandlerStackError},
    handler, handlers::BaseHandler,
    net::{Client, HttpMethod, HttpVerb, net_error::NetError, rate_limiter::RateLimiter},
    pipe, provided::{capabilities::add_headers::AddHeaders, handlers::IntoJson},
};
use reqwest::{Response, Client as ReqClient};
use serde_json::Value;
fn main() {
    let body = async {
        let client = Client::new(ReqClient::new(), RateLimiter::new(5.0, 2));
        let string_output: String = client
            .run_endpoint::<MyEndpoint, String>()
            .await
            .expect("Couldn't execute request");
        let json_output: JsonOutput = client
            .run_endpoint::<MyEndpoint, JsonOutput>()
            .await
            .expect("Couldn't execute request");
    };
    let body = {
        if false {
            let _: &dyn ::core::future::Future<Output = ()> = &body;
        }
        body
    };
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return,
        clippy::unwrap_in_result
    )]
    {
        use tokio::runtime::Builder;
        return Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
#[record(
    path = "https://example.com/api/",
    capabilities = [AddHeaders(vec![("Some-Header".into(), "value".into())])]
)]
struct MyRecord;
#[automatically_derived]
impl ::bees::record::Record for MyRecord {
    const SHARED_URL: &str = "https://example.com/api/";
    fn shared_caps() -> ::std::sync::Arc<[Box<dyn ::bees::capability::Capability>]> {
        ::std::sync::Arc::new([
            ::std::boxed::Box::new(
                AddHeaders(
                    ::alloc::boxed::box_assume_init_into_vec_unsafe(
                        ::alloc::intrinsics::write_box_via_move(
                            ::alloc::boxed::Box::new_uninit(),
                            [("Some-Header".into(), "value".into())],
                        ),
                    ),
                ),
            ) as ::std::boxed::Box<dyn ::bees::capability::Capability>,
        ])
    }
}
#[endpoint(
    record = MyRecord,
    http_method = HttpMethod::new_no_body(HttpVerb::GET),
    path = "hello"
)]
#[stacks(String:BaseHandler, IntoText)]
struct MyEndpoint;
#[automatically_derived]
impl ::core::fmt::Debug for MyEndpoint {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "MyEndpoint")
    }
}
#[automatically_derived]
impl ::bees::endpoint::EndpointInfo for MyEndpoint {
    const PATH: &str = "hello";
    type Record = MyRecord;
    type CallContext = ();
    #[allow(clippy::manual_async_fn)]
    fn http_method(
        _: &mut Self::CallContext,
    ) -> impl Future<Output = HttpMethod> + Send {
        async move { HttpMethod::new_no_body(HttpVerb::GET) }
    }
    fn capabilities(
        _: &mut Self::CallContext,
    ) -> ::std::sync::Arc<[Box<dyn ::bees::capability::Capability>]> {
        ::std::sync::Arc::new([])
    }
    #[allow(clippy::manual_async_fn)]
    fn modify_url(
        ____url___: ::bees::re_exports::url::Url,
        _: &mut Self::CallContext,
    ) -> impl ::std::future::Future<
        Output = ::bees::re_exports::url::Url,
    > + ::std::marker::Send {
        ::std::future::ready(____url___)
    }
}
#[automatically_derived]
impl ::bees::endpoint::HandlerStack<String> for MyEndpoint {
    type Handlers = ::bees::handlers::Chain<BaseHandler, IntoText>;
    async fn handlers(
        ctx: &mut <Self as ::bees::endpoint::EndpointInfo>::CallContext,
    ) -> Result<Self::Handlers, Box<dyn ::std::error::Error + Send + Sync>> {
        Ok({ ::bees::handlers::Chain(BaseHandler, IntoText) })
    }
}
pub struct IntoText;
#[automatically_derived]
impl ::core::fmt::Debug for IntoText {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "IntoText")
    }
}
#[automatically_derived]
impl ::bees::handlers::Handler for IntoText {
    type Input = Result<Response, NetError>;
    type Output = String;
    fn execute(&self, input: Self::Input) -> impl Future<Output = Self::Output> + Send {
        #[allow(non_snake_case)]
        #[doc(hidden)]
        #[inline(always)]
        async fn _IntoText(resp: Result<Response, NetError>) -> String {
            resp.expect("Request failed")
                .text()
                .await
                .expect("Couldn't get text out of Response")
        }
        _IntoText(input)
    }
}
type JsonOutput = Result<Result<Value, serde_json::Error>, NetError>;
impl HandlerStack<JsonOutput> for MyEndpoint {
    type Handlers = ::bees::handlers::TryChain<BaseHandler, IntoJson>;
    async fn handlers(
        _: &mut <Self as EndpointInfo>::CallContext,
    ) -> Result<Self::Handlers, HandlerStackError> {
        Ok(::bees::handlers::TryChain(BaseHandler, IntoJson))
    }
}
