# Bees
![MIT or Apache 2.0 Licensed](https://img.shields.io/badge/license-MIT_OR_Apache%202.0-blue?style=for-the-badge)

Bees is a library for building typed, async REST/HTTP API clients in Rust.
It is:

- **Structured**: endpoints are represented as types, and everything about them is implemented on top of the type. 
- **Composable**: Cross-cutting concerns like auth, headers, and retries are types that stack onto any endpoint type without modifying its core logic.
- **Batteries included**: Common patterns such as JSON or multipart requests, retrying with backoff, automatically refreshing auth tokens and header injection are already included.

## Overview
At its core, Bees is a wrapper around [`reqwest`](https://crates.io/crates/reqwest). As such, it requires that users rely on [`tokio`](https://crates.io/crates/tokio) as an async runtime.


A Bees API client is built around a few concepts:

**Records** define a shared base for a group of endpoints. Namely, a common URL prefix and shared `Capabilities` that apply to all of them.

**Endpoints** represent individual API operations. Each endpoint declares its `Record`, HTTP method, path, and any `Capabilities` specific to that operation. The `#[derive(Endpoint)]` macro covers the common case, but every endpoint is ultimately a manual trait implementation under the hood.

**Capabilities and Handlers** are the composable layer: `Capabilities` modify `reqwest`'s `RequestBuilder`, enabling the automatic addition of headers and in general anything pertaining to the content of the request itself, meanwhile `Handlers` are chained one after another or one into another to modify how the `Endpoint` behaves; retry logic and custom return types from `Endpoint`s are made this way.

**Resources** are named values stored on the client that can be automatically interpolated into `ResourceString`s by using the `"<...>"` syntax. These represent credentials, tokens, and similar ambient states.

### Example

```toml
# Cargo.toml

[dependencies]
bees = { git = "https://github.com/confused-ace-noises/bees", features = ["derive", "reqwest-json"]}
reqwest = "0.13.4"
tokio = { version = "1.52.3", features = ["full"] }
serde = "1.0.228"
serde_json = "1.0.150"
```

```rs
use bees::{
    Endpoint, Record, chain, endpoint::{EndpointInfo, HandlerStack, HandlerStackError}, handler, handlers::BaseHandler, net::{Client, HttpMethod, HttpVerb, net_error::NetError, rate_limiter::RateLimiter}, pipe, provided::{capabilities::add_headers::AddHeaders, handlers::IntoJson}
};
use reqwest::{Response, Client as ReqClient};

use serde_json::Value;

#[tokio::main]
async fn main() {
    let client = Client::new(ReqClient::new(), RateLimiter::new(5.0, 2));

    let string_output: String = client.run_endpoint::<MyEndpoint, String>().await.expect("Couldn't execute request");
    let json_output: JsonOutput = client.run_endpoint::<MyEndpoint, JsonOutput>().await.expect("Couldn't execute request");
}

#[derive(Record)]
#[record(
    path = "https://example.com/api/",
    capabilities = [
        AddHeaders(
            vec![
                ("Some-Header".into(), "value".into())
            ])
    ]
)]
struct MyRecord;

#[derive(Debug, Endpoint)]
#[endpoint(
    path = "my/endpoint/path",
    record = MyRecord,
    http_method = HttpMethod::new_no_body(HttpVerb::GET),
)]
struct MyEndpoint;

#[handler]
pub async fn IntoText(#[input] resp: Result<Response, NetError>) -> String {
    resp.expect("Request failed").text().await.expect("Couldn't get text out of Response")
}

impl HandlerStack<String> for MyEndpoint {
    type Handlers = chain!(BaseHandler, IntoText);

    async fn handlers(_: &mut <Self as EndpointInfo>::CallContext) -> Result<Self::Handlers, HandlerStackError> {
        Ok(pipe!(BaseHandler, IntoText))
    }
}

type JsonOutput = Result<Result<Value, serde_json::Error>, NetError>;

impl HandlerStack<JsonOutput> for MyEndpoint {
    type Handlers = chain!(~BaseHandler, IntoJson);

    async fn handlers(_: &mut <Self as EndpointInfo>::CallContext) -> Result<Self::Handlers, HandlerStackError> {
        Ok(pipe!(try BaseHandler, IntoJson))
    }
}
```


#### License
<small>
Licensed under either of [MIT](/LICENSE-MIT) or [Apache License, Version 2.0](/LICENSE-APACHE) license at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions. 
</small>