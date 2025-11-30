use crate::endpoint_record::{endpoint::{Endpoint, HttpVerb}, request_decorator::{Decorate, Handler, RequestDecorator}};
use reqwest::{
    Client as ReqClient, Method, Response, Url,
    header::{HeaderName, HeaderValue},
};
use std::{any::Any, borrow::Borrow, collections::HashMap, error::Error, sync::Arc, time::Duration};

use super::get_rate_limiter_duration;
use super::rate_limiter::RateLimiter;
use delegate::delegate;

#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ReqClient>,
    rate_limiter: Arc<RateLimiter>,
}

impl Client {
    pub(crate) fn new() -> Self {
        Self::_new(
            ReqClient::new(),
            RateLimiter::new(get_rate_limiter_duration()),
        )
    }

    #[allow(dead_code)]
    pub(crate) fn new_custom_rate_limit(rate_limit: Duration) -> Self {
        Self::_new(ReqClient::new(), RateLimiter::new(&rate_limit))
    }

    pub(crate) fn _new(client: ReqClient, rate_limiter: RateLimiter) -> Self {
        Self {
            inner: Arc::new(client),
            rate_limiter: Arc::new(rate_limiter),
        }
    }

    pub async fn reqwest_direct<Fut, E>(
        &self,
        f: impl FnOnce(Arc<ReqClient>) -> Result<Fut, E>,
    ) -> Result<Response, E>
    where
        Fut: Future<Output = Result<Response, E>>,
        E: Error,
    {
        self.rate_limiter.wait().await;
        f(self.inner.clone())?.await
    }

    pub fn request(&self, method: Method, url: impl reqwest::IntoUrl) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
            rate_limiter: self.rate_limiter.clone(),
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn execute_reqwest_req(&self, request: reqwest::Request) -> Result<Response, reqwest::Error> {
        self.rate_limiter.wait().await;
        self.inner.execute(request).await
    }

    pub async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        request.rate_limiter.wait().await;
        self.inner.execute(request.inner).await
    }

    pub async fn build_req_builder(
        &self,
        endpoint: &Endpoint,
        parse_values: impl Borrow<HashMap<String, String>>,
        query_values: impl Borrow<Vec<(String, Option<String>)>>
    ) -> Result<RequestBuilder, Box<dyn Error>> {
        let parse_values = parse_values.borrow();
        let url = Url::parse(&endpoint.full_url(parse_values, query_values.borrow()).await)?;
        let request = self.request(endpoint.http_verb().as_method(), url);
        
        let mut request = match endpoint.http_verb() {
            HttpVerb::GET 
                | HttpVerb::DELETE(Option::None) 
                | HttpVerb::OPTIONS 
                | HttpVerb::HEAD => request,
            
            HttpVerb::POST(body) 
                | HttpVerb::PUT(body) 
                | HttpVerb::PATCH(body) 
                | HttpVerb::DELETE(Some(body)) 
            => {
                let request = request.body(body.to_formatted(parse_values).await);
                request
            },
        };

        let capabilities = endpoint.all_capabilities();
        let capabilities = capabilities.0.iter().chain(capabilities.1.iter());

        for capability in capabilities {
            request = capability.apply(request)
        }

        Ok(request)
    }

    pub async fn build_req(
        &self,
        endpoint: &Endpoint,
        parse_values: impl Borrow<HashMap<String, String>>,
        query_values: impl Borrow<Vec<(String, Option<String>)>>
    ) -> Result<Request, Box<dyn Error>> { 
        match self.build_req_builder(endpoint, parse_values, query_values).await {
            Ok(rb) => Ok(rb.build()?),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn endpoint_handler<'a>(&'a self) -> Handler<'a, reqwest::Error> {
        Arc::new(
            move |req: Request| Box::pin(
                async move {
                    self.execute(req).await
                }
            )
        )
    }

    pub fn run_endpoint<'a, 'b>(
        &'a self,
        endpoint: Endpoint,
        parse_values: &'b HashMap<String, String>,
        query_values: &'b Vec<(String, Option<String>)>
    ) -> EndpointRunner<'a, reqwest::Error> 
    where
        'b: 'a,
    {
        let handler = self.endpoint_handler();
        EndpointRunner {
            client: self.clone(),
            handler,
            endpoint,
            parse_values: parse_values,
            query_values: query_values,
        }
    }

    pub fn run_request<'a, E: Error + Send + 'a>(&'a self, request: Request) -> RequestRunner<'a, reqwest::Error> {
        let handler = self.endpoint_handler();
        RequestRunner {
            client: self.clone(),
            handler,
            request,
        }
    }
}

pub struct RequestRunner<'a, E: Error + Send + 'static> {
    client: Client,
    handler: Handler<'a, E>,
    request: Request,
}

impl<'a, E: Error + Send + 'static> RequestRunner<'a, E>
where
    E: Error + Send + 'static,
{
    pub fn decorate<G: Error + Send + 'a>(
        self,
        decorator: &'a dyn RequestDecorator<E, G>,
    ) -> RequestRunner<'a, G> 
    where
        G: 'a,
    {
        let new_handler = self.handler.decorate(decorator);
        RequestRunner {
            client: self.client,
            handler: new_handler,
            request: self.request,
        }
    }

    pub async fn run(self) -> Result<Response, Box<dyn Error>> {
        Ok((self.handler)(self.request).await?)
    }
}

pub struct EndpointRunner<'a, E: Error + Send + 'static> {
    client: Client,
    handler: Handler<'a, E>,
    endpoint: Endpoint,
    parse_values: &'a HashMap<String, String>,
    query_values: &'a Vec<(String, Option<String>)>
}

impl<'a, E: Error + Send + 'static> EndpointRunner<'a, E> {
    pub fn decorate<G: Error + Send + 'a>(
        self,
        decorator: &'a dyn RequestDecorator<E, G>,
    ) -> EndpointRunner<'a, G> 
    where
        G: 'a,
    {
        let new_handler = self.handler.decorate(decorator);
        EndpointRunner {
            client: self.client,
            handler: new_handler,
            endpoint: self.endpoint,
            parse_values: self.parse_values,
            query_values: self.query_values,
        }
    }

    pub async fn run_resp(self) -> Result<Response, Box<dyn Error>> {
        let req = self.client.build_req(
            &self.endpoint,
            self.parse_values,
            self.query_values
        ).await?;

        println!("Running request: {:#?}", req);

        Ok((self.handler)(req).await?)
    }

    pub async fn run_specific<T: Any + Send + Sync + 'static>(self) -> Result<T, Box<dyn Error>> {
        let req = self.client.build_req(
            &self.endpoint,
            self.parse_values,
            self.query_values
        ).await?;

        println!("Running request: {:#?}", req);

        Ok(self.endpoint.endpoint_output_specific((self.handler)(req).await?).await)
    }

    pub async fn run(self) -> Result<Box<dyn Any + Send + Sync + 'static>, Box<dyn Error>> {
        let req = self.client.build_req(
            &self.endpoint,
            self.parse_values,
            self.query_values
        ).await?;

        println!("Running request: {:#?}", req);

        Ok(self.endpoint.endpoint_output((self.handler)(req).await?).await)
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
    rate_limiter: Arc<RateLimiter>,
}

impl RequestBuilder {
    delegate! {
        to self.inner {
            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> RequestBuilder
            where
                U: std::fmt::Display,
                P: std::fmt::Display;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn bearer_auth<T>(self, token: T) -> RequestBuilder
            where
                T: std::fmt::Display;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn body<T>(self, body: T) -> RequestBuilder
            where
                T: Into<reqwest::Body>;


            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn header<K, V>(self, key: K, value: V) -> RequestBuilder
            where
                HeaderName: TryFrom<K>,
                <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
                HeaderValue: TryFrom<V>,
                <HeaderValue as TryFrom<V>>::Error: Into<http::Error>;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn headers(self, headers: reqwest::header::HeaderMap) -> RequestBuilder;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn query<T: ?Sized + serde::Serialize>(self, query: &T) -> RequestBuilder;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn form<T: ?Sized + serde::Serialize>(self, form: &T) -> RequestBuilder;

            #[cfg(feature = "reqwest_json")]
            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn json<T: ?Sized + serde::Serialize>(self, json: &T) -> RequestBuilder;

            #[cfg(feature = "reqwest_multipart")]
            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn multipart(self, form: reqwest::multipart::Form) -> RequestBuilder;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn version(self, version: reqwest::Version) -> RequestBuilder;

            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn timeout(self, timeout: std::time::Duration) -> RequestBuilder;

        }
    }

    pub fn build(self) -> Result<Request, reqwest::Error> {
        Ok(Request {
            inner: self.inner.build()?,
            rate_limiter: self.rate_limiter,
        })
    }

    pub async fn send(self) -> Result<Response, reqwest::Error> {
        self.rate_limiter.wait().await;
        self.inner.send().await
    }
}

#[derive(Debug)]
pub struct Request {
    inner: reqwest::Request,
    rate_limiter: Arc<RateLimiter>,
}

impl Request {
    delegate! {
        to self.inner {
            // pub fn try_clone(&self) -> Option<Request>;
            pub fn body(&self) -> Option<&reqwest::Body>;
            pub fn body_mut(&mut self) -> &mut Option<reqwest::Body>;

            pub fn headers(&self) -> &reqwest::header::HeaderMap;
            pub fn headers_mut(&mut self) -> &mut reqwest::header::HeaderMap;

            pub fn method(&self) -> &reqwest::Method;
            pub fn method_mut(&mut self) -> &mut reqwest::Method;

            pub fn timeout(&self) -> Option<&Duration>;
            pub fn timeout_mut(&mut self) -> &mut Option<Duration>;

            pub fn url(&self) -> &reqwest::Url;
            pub fn url_mut(&mut self) -> &mut reqwest::Url;

            pub fn version(&self) -> reqwest::Version;
            pub fn version_mut(&mut self) -> &mut reqwest::Version;

            #[expr(Some(Self { inner: $?, rate_limiter: self.rate_limiter.clone() }))]
            pub fn try_clone(&self) -> Option<Request>;
        }
    }
}

impl<T> TryFrom<http::Request<T>> for Request
where
    T: Into<reqwest::Body>,
{
    type Error = reqwest::Error;

    fn try_from(value: http::Request<T>) -> Result<Self, Self::Error> {
        let reqwest_request = reqwest::Request::try_from(value)?;
        Ok(Self {
            inner: reqwest_request,
            rate_limiter: Arc::new(RateLimiter::new(get_rate_limiter_duration())), // default no rate limit
        })
    }
}

impl TryFrom<Request> for http::Request<reqwest::Body> {
    type Error = reqwest::Error;

    fn try_from(value: Request) -> Result<Self, Self::Error> {
        let http_request = http::Request::try_from(value.inner)?;
        Ok(http_request)
    }
}

#[tokio::test]
async fn test() {
    use crate::core::client;
    let client = client();
    let _ = client
        .reqwest_direct(|reqclient| {
            let req: reqwest::Request = reqclient.get("https://www.fanton.com/").build()?;
            Ok(reqclient.execute(req))
        })
        .await;
}
