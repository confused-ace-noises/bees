use crate::{Sealed, endpoint::Endpoint, net::get_rate_limiter, request_decorator::{Decorate, RequestDecorator}};
use async_rate_limiter::RateLimiter;
use reqwest::{
    Client as ReqClient, Method, Response, Url,
};
use std::{
    any::Any, borrow::Borrow, collections::HashMap, error::Error, fmt::{self, Debug}, pin::Pin, sync::Arc
};

use super::request::{Request, RequestBuilder, Body, RequestRunner};

pub type Handler<'a, E> = 
    Arc<
        dyn Fn(Request) -> Pin<
                Box<
                    dyn Future<
                        Output = Result<Response, E>
                > + Send + 'a>
            > + Send + Sync + 'a
    >;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ReqClient>,
    rate_limiter: &'static RateLimiter,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client").field("inner", &self.inner).field("rate_limiter", &"async_rate_limiter internals").finish()
    }
}

impl Client {
    pub(crate) fn new() -> Self {
        Self::_new(
            // ReqClient::builder()
            //     .proxy(Proxy::http("http://127.0.0.1:8080/").unwrap())
            //     .proxy(Proxy::https("https://127.0.0.1:8080/").unwrap())
            //     .build()
            //     .unwrap(),
            ReqClient::new(),
            get_rate_limiter(),
        )
    }

    #[allow(dead_code)]
    pub(crate) fn new_custom_rate_limiter(rate_limiter: &'static RateLimiter) -> Self {
        Self::_new(ReqClient::new(), rate_limiter)
    }

    pub(crate) fn _new(client: ReqClient, rate_limiter: &'static RateLimiter) -> Self {
        Self {
            inner: Arc::new(client),
            rate_limiter,
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
        self.rate_limiter.acquire().await;
        f(self.inner.clone())?.await
    }

    pub fn request(&self, method: Method, url: impl reqwest::IntoUrl) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
            rate_limiter: self.rate_limiter,
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn execute_reqwest_req(
        &self,
        request: reqwest::Request,
    ) -> Result<Response, reqwest::Error> {
        self.rate_limiter.acquire().await;
        self.inner.execute(request).await
    }

    pub async fn execute(&self, request: Request) -> Result<Response, reqwest::Error> {
        request.rate_limiter.acquire().await;
        self.inner.execute(request.inner).await
    }

    pub async fn build_req_builder(
        &self,
        endpoint: &Endpoint,
        parse_values: impl Borrow<HashMap<String, String>>,
        query_values: impl Borrow<Vec<(String, Option<String>)>>,
    ) -> Result<RequestBuilder, Box<dyn Error>> {
        let parse_values = parse_values.borrow();
        let url = Url::parse(&endpoint.full_url(parse_values, query_values.borrow()).await)?;
        let request = self.request(endpoint.http_verb().as_method(), url);

        let mut request = match endpoint.http_verb() {
            HttpVerb::GET | HttpVerb::DELETE(Option::None) | HttpVerb::OPTIONS | HttpVerb::HEAD => {
                request
            }

            HttpVerb::POST(body)
            | HttpVerb::PUT(body)
            | HttpVerb::PATCH(body)
            | HttpVerb::DELETE(Some(body)) => request.body(body.to_formatted(parse_values).await)
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
        query_values: impl Borrow<Vec<(String, Option<String>)>>,
    ) -> Result<Request, Box<dyn Error>> {
        match self
            .build_req_builder(endpoint, parse_values, query_values)
            .await
        {
            Ok(rb) => Ok(rb.build()?),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn endpoint_handler<'a>(&'a self) -> Handler<'a, reqwest::Error> {
        Arc::new(move |req: Request| Box::pin(async move { self.execute(req).await }))
    }
 
    pub fn run_endpoint<'a, 'b>(
        &'a self,
        endpoint: Endpoint,
        parse_values: &'b HashMap<String, String>,
        query_values: &'b Vec<(String, Option<String>)>,
    ) -> EndpointRunner<'a, reqwest::Error>
    where
        'b: 'a,
    {
        let handler = self.endpoint_handler();
        EndpointRunner {
            client: self.clone(),
            handler,
            endpoint,
            parse_values,
            query_values,
        }
    }

    pub fn run_request<'a, E: Error + Send + 'a>(
        &'a self,
        request: Request,
    ) -> RequestRunner<'a, reqwest::Error> {
        let handler = self.endpoint_handler();
        RequestRunner {
            client: self.clone(),
            handler,
            request,
        }
    }
}

pub struct EndpointRunner<'a, E: Send> {
    client: Client,
    handler: Handler<'a, E>,
    endpoint: Endpoint,
    parse_values: &'a HashMap<String, String>,
    query_values: &'a Vec<(String, Option<String>)>,
}

impl<'a, E: Send> Debug for EndpointRunner<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EndpointRunner").field("client", &self.client).field("handler", &"Handler<'a, E> = Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Response, E>> + Send + 'a>> + Send + Sync + 'a>;").field("endpoint", &self.endpoint).field("parse_values", &self.parse_values).field("query_values", &self.query_values).finish()
    }
}

impl<'a, E: Send> Sealed for EndpointRunner<'a, E>{}

impl<'a, E, G> Decorate<'a, E, G> for EndpointRunner<'a, E> 
where
    E: Send + 'a,
    G: Send + 'a,
{
    type Output = EndpointRunner<'a, G>;

    fn decorate<T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T) -> Self::Output
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
}

impl<'a, E: Error + Send + 'static> EndpointRunner<'a, E> {
    pub async fn run_resp(self) -> Result<Response, Box<dyn Error>> {
        let req = self
            .client
            .build_req(&self.endpoint, self.parse_values, self.query_values)
            .await?;

        println!("Running request: {:#?}", req);

        Ok((self.handler)(req).await?)
    }

    pub async fn run<T: Any + Send + Sync + 'static>(self) -> Result<T, Box<dyn Error>> {
        let req = self
            .client
            .build_req(&self.endpoint, self.parse_values, self.query_values)
            .await?;

        println!("Running request: {:#?}", req);

        Ok(self
            .endpoint
            .endpoint_output_specific((self.handler)(req).await?)
            .await)
    }
}


#[derive(Debug)]
pub enum HttpVerb {
    GET,
    POST(Body),
    PUT(Body),
    DELETE(Option<Body>),
    PATCH(Body),
    OPTIONS,
    HEAD,
}
impl HttpVerb {
    pub fn as_method(&self) -> Method {
        match self {
            HttpVerb::GET => Method::GET,
            HttpVerb::POST(_) => Method::POST,
            HttpVerb::PUT(_) => Method::PUT,
            HttpVerb::DELETE(_) => Method::DELETE,
            HttpVerb::PATCH(_) => Method::PATCH,
            HttpVerb::OPTIONS => Method::OPTIONS,
            HttpVerb::HEAD => Method::HEAD,
        }
    }
}