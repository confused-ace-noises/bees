use crate::{Sealed, endpoint_def::Endpoint, net::get_rate_limiter, request_decorator::{Decorate, RequestDecorator}, utils::Error};
use async_rate_limiter::RateLimiter;
use reqwest::{
    Client as ReqClient, Method, Response, Url,
};
use std::{
    any::Any, borrow::Borrow, error::Error as StdError, fmt::{self, Debug}, pin::Pin, sync::Arc
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
        E: StdError,
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
    ) -> Result<Response, Error> {
        self.rate_limiter.acquire().await;
        Ok(self.inner.execute(request).await?)
    }

    pub async fn execute(&self, request: Request) -> Result<Response, Error> {
        request.rate_limiter.acquire().await;
        Ok(self.inner.execute(request.inner).await?)
    }

    pub async fn build_req_builder(
        &self,
        endpoint: &Endpoint,
        url_query_values: impl Borrow<Vec<(String, Option<String>)>>,
    ) -> Result<RequestBuilder, Error> {
        let query_values = url_query_values.borrow();
        
        // TODO: fix errors; remove .unwrap()
        let url = Url::parse(&endpoint.full_url(query_values).await)?;
        let request = self.request(endpoint.http_verb().as_method(), url);

        let mut request = match endpoint.http_verb() {
            HttpVerb::GET | HttpVerb::DELETE(Option::None) | HttpVerb::OPTIONS | HttpVerb::HEAD => {
                request
            }

            HttpVerb::POST(body)
            | HttpVerb::PUT(body)
            | HttpVerb::PATCH(body)
            | HttpVerb::DELETE(Some(body)) => body.add_body(request).await?,
        };

        let capabilities = endpoint.all_capabilities();
        let capabilities = capabilities.0.iter().chain(capabilities.1.iter());

        for capability in capabilities {
            request = capability.apply(request).await?
        }

        Ok(request)
    }

    pub async fn build_req(
        &self,
        endpoint: &Endpoint,
        query_values: impl Borrow<Vec<(String, Option<String>)>>,
    ) -> Result<Request, Error> {
        match self
            .build_req_builder(endpoint, query_values)
            .await
        {
            Ok(rb) => Ok(rb.build()?),
            Err(e) => Err(e),
        }
    }

    pub(crate) fn endpoint_handler<'a>(&'a self) -> Handler<'a, Error> {
        Arc::new(move |req: Request| Box::pin(async move { self.execute(req).await }))
    }
 
    pub fn run_endpoint<'a, 'b>(
        &'a self,
        endpoint: Endpoint,
        query_values: &'b Vec<(String, Option<String>)>,
    ) -> EndpointRunner<'a, Error>
    where
        'b: 'a,
    {
        let handler = self.endpoint_handler();
        EndpointRunner {
            client: self.clone(),
            handler,
            endpoint,
            query_values,
        }
    }

    pub fn run_request<'a, E: StdError + Send + 'a>(
        &'a self,
        request: Request,
    ) -> RequestRunner<'a, Error> {
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
    query_values: &'a Vec<(String, Option<String>)>,
}

impl<'a, E: Send> Debug for EndpointRunner<'a, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EndpointRunner").field("client", &self.client).field("handler", &"Handler<'a, E> = Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Response, E>> + Send + 'a>> + Send + Sync + 'a>;").field("endpoint", &self.endpoint).field("query_values", &self.query_values).finish()
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
            query_values: self.query_values,
        }
    }
}

impl<'a, E: StdError + Send + 'static> EndpointRunner<'a, E> {
    pub async fn run_resp(self) -> Result<Result<Response, E>, Error> {
        let req = self
            .client
            .build_req(&self.endpoint, self.query_values)
            .await?;

        println!("Running request: {:#?}", req);

        Ok((self.handler)(req).await) 
    }

    pub async fn run<T: Any + Send + Sync + 'static>(self) -> Result<Result<T, E>, Error> {
        let req = self
            .client
            .build_req(&self.endpoint, self.query_values)
            .await?;

        println!("Running request: {:#?}", req);

        match (self.handler)(req).await {
            Ok(resp) => Ok(Ok(self.endpoint.endpoint_output_specific::<T>(resp).await)),
            Err(e) => Ok(Err(e))
        }
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