use bees::{
    Endpoint, HandlerStacks, Record, chain,
    endpoint::{EndpointInfo, HandlerStack, HandlerStackError},
    handler,
    handlers::BaseHandler,
    net::{Client, HttpMethod, HttpVerb, net_error::NetError, rate_limiter::RateLimiter},
    pipe,
    provided::{capabilities::add_headers::AddHeaders, handlers::IntoJson, resources::constant_res::ConstRes},
};
use reqwest::{Client as ReqClient, Response};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let rate_per_sec = 5.0;
    let burst = 2;
    // create the bees client:
    //                               specify the reqwest client...    ...and the rate limiter
    let client = Client::new(ReqClient::new(), RateLimiter::new(rate_per_sec, burst));

    // add a Resource to the Client's resource manager, making it
    // available for interpolation into URLs and request bodies
    client.resource_manager.add_resource(ConstRes::new("my_resource", "some_value") );

    // once the Endpoint and its HandlerStacks have been declared, the Endpoint can
    // be easily called by using the .run_endpoint<EndpointName, Output>() method on the Client
    let string_output: String = client
        .run_endpoint::<MyEndpoint, String>()
        .await
        .expect("Couldn't execute request");

    let json_output: JsonOutput = client
        .run_endpoint::<MyEndpoint, JsonOutput>()
        .await
        .expect("Couldn't execute request");
}

// derive impl of a Record:
#[derive(Record)]
#[record(
    // initial part of url shared by all Endpoints using this Record
    path = "https://example.com/api/",

    // Capabilities used by every Endpoint using this Record
    capabilities = [
        AddHeaders(
            vec![
                ("Some-Header".into(), "value".into())
            ])
    ]
)]
struct MyRecord;

#[derive(Debug, Endpoint, HandlerStacks)]
// Endpoint declaration:
#[endpoint(
    // the Record used by this Endpoint
    record = MyRecord,
    // the path to append to the Record's; in this case, the my_resource Resource will
    // be interpolated into the URL, and so the path will be
    // https://example.com/api/my/some_value/endpoint
    path = "my/<my_resource>/endpoint",
    // The HTTP method used by this Endpoint
    http_method = HttpMethod::new_no_body(HttpVerb::GET),
)]
// HandlerStack impl:
#[stacks(
    // This Endpoint will support a String as an output type, and if a String
    // is requested as an Output, BaseHandler will get the Response from the 
    // server and then IntoText will get the String (see below for IntoText impl) 
    String: BaseHandler, IntoText
)]
struct MyEndpoint;

#[handler]
// this Handler takes Result<Response, NetError> as an input (BaseHandler's output),
// and outputs a String.
pub async fn IntoText(#[input] resp: Result<Response, NetError>) -> String {
    resp.expect("Request failed")
        .text()
        .await
        .expect("Couldn't get text out of Response")
}

type JsonOutput = Result<Result<Value, serde_json::Error>, NetError>;

// Another HandlerStack implementation; this means that the Endpoint will also support
// JsonOutput as an Output
impl HandlerStack<JsonOutput> for MyEndpoint {
    // The chain! macro chains together any number of Handlers by using nested Chain<A, B>,
    // which is a Handler that chains the output of Handler A with the input of Handler B.
    // if a Handler returns a Result enum, this can be marked with `~` or `try` to bubble the
    // error variant up, like the question mark operator (?)
    type Handlers = chain!(~BaseHandler, IntoJson);

    async fn handlers(
        _: &mut <Self as EndpointInfo>::CallContext,
    ) -> Result<Self::Handlers, HandlerStackError> {
        // the pipe! macro is like the chain! macro, but for expressions instead of types.
        // like chain!, `~` or `try` can be used to bubble up the error variant of the output type
        // of the expression the `~` or `try` sigil is used on, like the question mark
        // operator in normal expressions; if the question mark operator is used inside the pipe! macro,
        // it will expand and bubble up the error variant of the expression itself, and not of the output type.
        Ok(pipe!(try BaseHandler, IntoJson))
    }
}
