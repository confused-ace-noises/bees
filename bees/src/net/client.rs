use crate::{
    endpoint::{EndpointExt, EndpointInfo, EndpointProcessor, Process},
    handler::{Handler, HandlerWrapper, WrapDecorate},
    net::{bodies::Body, net_error::NetError}
};
use derive_more::{Display, Error, From};
use async_rate_limiter::RateLimiter;
use reqwest::{Client as ReqClient, Method, Response};
use std::{
    error::Error as StdError,
    fmt::{self, Debug},
    sync::Arc,
};

use super::request::{Request, RequestBuilder};
// use super::net_error::NetError as Error;
use crate::utils::error::Error;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ReqClient>,
    rate_limiter: Arc<RateLimiter>,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.inner)
            .field("rate_limiter", &"async_rate_limiter internals")
            .finish()
    }
}

impl Client {
    pub fn new(reqwest_client: ReqClient, rate_limiter: RateLimiter) -> Self {
        Self::__new(reqwest_client, Arc::new(rate_limiter))
    }

    pub(crate) fn _new(rate_limiter: Arc<RateLimiter>) -> Self {
        Self::__new(
            // ReqClient::builder()
            //     .proxy(Proxy::http("http://127.0.0.1:8080/").unwrap())
            //     .proxy(Proxy::https("https://127.0.0.1:8080/").unwrap())
            //     .build()
            //     .unwrap(),
            ReqClient::new(),
            rate_limiter,
        )
    }

    pub(crate) fn __new(client: ReqClient, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            inner: Arc::new(client),
            rate_limiter,
        }
    }

    // --------- DIRECT ---------
    pub async fn reqwest_direct<Fut, E, F>(&self, f: F) -> Result<Response, E>
    where
        F: FnOnce(Arc<ReqClient>) -> Result<Fut, E>,
        Fut: Future<Output = Result<Response, E>>,
        E: StdError,
    {
        self.rate_limiter.acquire().await;
        f(self.inner.clone())?.await
    }

    pub fn get_raw_request_builder(&self, method: Method, url: impl reqwest::IntoUrl) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
            client: self.clone(),
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn execute_reqwest_req(
        &self,
        request: reqwest::Request,
    ) -> Result<Response, Error> {
        self.rate_limiter.acquire().await;
        Ok(self.inner.execute(request).await.map_err(NetError::from)?)
    }

    pub async fn execute_request(&self, request: Request) -> Result<Response, Error> {
        self.rate_limiter.acquire().await;
        Ok(self
            .inner
            .execute(request.inner)
            .await
            .map_err(NetError::ReqwestError)?)
    }

    // --------- ENDPOINT ---------
    pub async fn request_builder<E: EndpointInfo>(
        &self,
        call_context: &mut E::CallContext,
    ) -> Result<RequestBuilder, Error> {
        let url = E::full_url(call_context).await?;
        let verb = E::http_verb(call_context).await;

        let request = self.get_raw_request_builder(verb.as_method(), url);
        let mut request = match verb {
            HttpVerb::GET | HttpVerb::DELETE(Option::None) | HttpVerb::OPTIONS | HttpVerb::HEAD => {
                request
            }

            HttpVerb::POST(body)
            | HttpVerb::PUT(body)
            | HttpVerb::PATCH(body)
            | HttpVerb::DELETE(Some(body)) => body.add_body(request).await?,
        };

        let endpoint_caps = E::caps(call_context);
        let record_caps = E::record_capabilities();

        let capabilities = record_caps.iter().chain(endpoint_caps.iter());

        for capability in capabilities {
            request = capability.apply(request).await?
        }

        Ok(request)
    }

    pub async fn get_request<E>(&self, call_context: &mut E::CallContext) -> Result<Request, Error>
    where
        E: EndpointInfo,
    {
        self.request_builder::<E>(call_context)
            .await
            .and_then(|rb| rb.build().map_err(Error::from))
    }

    // --------- RUN HELPERS ---------
    pub fn run_endpoint<E: EndpointInfo>(
        &self,
        call_context: E::CallContext,
    ) -> EndpointRunner<E, E::EndpointHandler> {
        EndpointRunner::new(self.clone(), call_context)
    }

    pub fn run_endpoint_ref<'a, E: EndpointInfo>(
        &self,
        call_context: &'a mut E::CallContext
    ) -> EndpointRunnerRef<'a, E, E::EndpointHandler> {
        EndpointRunnerRef::new(self.clone(), call_context)
    }
}

#[derive(Debug)]
pub struct EndpointRunner<E: EndpointInfo, H: Handler> {
    client: Client,
    handler: H,
    call_context: E::CallContext,
}

impl<E: EndpointInfo, H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W>
    for EndpointRunner<E, H>
{
    type Output = EndpointRunner<E, W::Output>;

    fn wrap(self, wrapper: W) -> Self::Output {
        EndpointRunner {
            client: self.client,
            handler: self.handler.wrap(wrapper),
            call_context: self.call_context,
        }
    }
}

impl<E: EndpointInfo> EndpointRunner<E, E::EndpointHandler> {
    pub fn new(client: Client, mut call_context: E::CallContext) -> Self {
        let base_handler = E::endpoint_handler(&mut call_context);

        EndpointRunner {
            client,
            handler: base_handler,
            call_context,
        }
    }
}

impl<E: EndpointInfo, H: Handler> EndpointRunner<E, H> {
    pub async fn run_get_response(&mut self) -> Result<Response, EndpointRunnerError<H>> {
        self.handler.execute(self.client.get_request::<E>(&mut self.call_context).await?).await.map_err(EndpointRunnerError::HandlerError)
    }
    
    pub async fn run<O>(&mut self) -> Result<O, EndpointRunnerError<H>>
    where
        E: EndpointProcessor<O>,
    {
        let response = self.run_get_response().await?;
        let proc_output = <E::Process as Process>::process(response).await;

        Ok(E::refine(proc_output, &mut self.call_context).await)
    }
}

pub struct EndpointRunnerRef<'a, E: EndpointInfo, H: Handler> {
    client: Client,
    handler: H,
    call_context: &'a mut E::CallContext,
}

impl<'a, E: EndpointInfo> EndpointRunnerRef<'a, E, E::EndpointHandler> {
    pub fn new(client: Client, call_context: &'a mut E::CallContext) -> Self {
        let base_handler = E::endpoint_handler(call_context);

        EndpointRunnerRef {
            client,
            handler: base_handler,
            call_context,
        }
    }
}

impl<'a, E: EndpointInfo, H: Handler> EndpointRunnerRef<'a, E, H> {
    pub async fn run_get_response(&mut self) -> Result<Response, EndpointRunnerError<H>> {
        self.handler.execute(self.client.get_request::<E>(self.call_context).await?).await.map_err(EndpointRunnerError::HandlerError)
    }
    
    pub async fn run<O>(&mut self) -> Result<O, EndpointRunnerError<H>>
    where
        E: EndpointProcessor<O>,
    { 
        let response = self.run_get_response().await?;
        let proc_output = <E::Process as Process>::process(response).await;

        Ok(E::refine(proc_output, self.call_context).await)
    }
}

impl<'a, E: EndpointInfo, H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W>
    for EndpointRunnerRef<'a, E, H>
{
    type Output = EndpointRunnerRef<'a, E, W::Output>;

    fn wrap(self, wrapper: W) -> Self::Output {
        EndpointRunnerRef {
            client: self.client,
            handler: self.handler.wrap(wrapper),
            call_context: self.call_context,
        }
    }
}


#[derive(Debug, Display, Error, From)]
pub enum EndpointRunnerError<H: Handler> {
    FailedToBuildRequest(#[error(source)] Error),
    
    #[from(skip)]
    HandlerError(#[error(source)] H::Error)
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
