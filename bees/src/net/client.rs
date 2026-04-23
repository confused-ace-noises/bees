use crate::{
    endpoint::{EndpointExt, EndpointInfo, Process, SupportsOutput},
    handler::{Handler, HandlerWrapper, WrapDecorate},
    net::{bodies::Body, net_error::NetError, rate_limiter::RateLimiter}, resources::resource_handler::ResourceManager
};
use derive_more::{Display, Error, From};
use futures::future::join;
use reqwest::{Client as ReqClient, Method, Response};
use std::{
    error::Error as StdError,
    fmt::Debug,
    sync::Arc,
};

use super::request::{Request, RequestBuilder};
// use super::net_error::NetError as Error;
use crate::utils::error::Error;

#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ReqClient>,
    rate_limiter: Arc<RateLimiter>,
    pub resource_manager: Arc<ResourceManager>,
}

// impl fmt::Debug for Client {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Client")
//             .field("inner", &self.inner)
//             .field("rate_limiter", &"async_rate_limiter internals")
//             .finish()
//     }
// }

impl Client {
    pub fn new(reqwest_client: ReqClient, rate_limiter: RateLimiter) -> Self {
        Self::__new(reqwest_client, Arc::new(rate_limiter), ResourceManager::new())
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
            ResourceManager::new()
        )
    }

    pub(crate) fn __new(client: ReqClient, rate_limiter: Arc<RateLimiter>, res_manager: ResourceManager) -> Self {
        Self {
            inner: Arc::new(client),
            rate_limiter,
            resource_manager: Arc::new(res_manager),
        }
    }

    // --------- DIRECT ---------
    ////// NO RATE LIMITER //////
    pub async fn reqwest_direct_no_rate_limit<Fut, E, F>(&self, f: F) -> Result<Response, E>
    where
        F: FnOnce(Arc<ReqClient>) -> Result<Fut, E>,
        Fut: Future<Output = Result<Response, E>>,
        E: StdError,
    {
        // self.rate_limiter.acquire().await;
        f(self.inner.clone())?.await
    }

    pub async fn execute_request_no_rate_limiter(&self, request: Request) -> Result<Response, Error> {
        // self.rate_limiter.acquire().await;
        Ok(self
            .inner
            .execute(request.inner)
            .await
            .map_err(NetError::ReqwestError)?)
    }

    //////// RATE LIMITER ////////
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

    pub async fn execute_reqwest_req(
        &self,
        request: reqwest::Request,
    ) -> Result<Response, Error> {
        self.rate_limiter.acquire().await;
        self.execute_reqwest_req_no_rate_limit(request).await
    }

    pub async fn execute_reqwest_req_no_rate_limit(
        &self,
        request: reqwest::Request,
    ) -> Result<Response, Error> {
        // self.rate_limiter.acquire().await;
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
        call_context: &E::CallContext,
    ) -> Result<RequestBuilder, Error> {

        // determine whether this makes sense, does it give enough of a speed boost to 
        // justify not guaranteeing order of operations?
        let (url, method) = join(E::full_url(&self.resource_manager, call_context), E::http_verb(call_context)).await;

        let url = url?;

        let request = self.get_raw_request_builder(method.verb.as_reqwest_method(), url);
        
        let mut request = match method.body {
            Some(body) => body.add_body(request).await?,
            None => request,
        };
        
        // let mut request = match verb {
        //     HttpVerb::GET | HttpVerb::DELETE(Option::None) | HttpVerb::OPTIONS | HttpVerb::HEAD => {
        //         request
        //     }

        //     HttpVerb::POST(body)
        //     | HttpVerb::PUT(body)
        //     | HttpVerb::PATCH(body)
        //     | HttpVerb::DELETE(Some(body)) => body.add_body(request).await?,
        // };

        let endpoint_caps = E::capabilities(call_context);
        let record_caps = E::record_capabilities();

        let capabilities = record_caps.iter().chain(endpoint_caps.iter());

        for capability in capabilities {
            request = capability.apply(request).await?
        }

        Ok(request)
    }

    pub async fn get_request<E>(&self, call_context: &E::CallContext) -> Result<Request, Error>
    where
        E: EndpointInfo,
    {
        self.request_builder::<E>(call_context)
            .await
            .and_then(|rb| rb.build().map_err(Error::from))
    }

    // --------- RUN HELPERS ---------
    pub fn run_endpoint_with<E: EndpointInfo>(
        &self,
        call_context: E::CallContext,
    ) -> EndpointRunner<E, E::EndpointHandler> {
        EndpointRunner::new(self.clone(), call_context)
    }

    pub fn run_endpoint_ref_with<'a, E: EndpointInfo>(
        &self,
        call_context: &'a mut E::CallContext
    ) -> EndpointRunnerRef<'a, E, E::EndpointHandler> {
        EndpointRunnerRef::new(self.clone(), call_context)
    }
    
    pub fn run_endpoint<E: EndpointInfo<CallContext = ()>>(
        &self,
    ) -> EndpointRunner<E, E::EndpointHandler>
    {
        EndpointRunner::new(self.clone(), ())
    }

    pub fn run_endpoint_ref<'a, E: EndpointInfo<CallContext = ()>>(
        &self,
        call_context: &'a mut E::CallContext
    ) -> EndpointRunnerRef<'a, E, E::EndpointHandler> {
        EndpointRunnerRef::new(self.clone(), call_context)
    }

    pub fn get_rate_limiter(&self) -> Arc<RateLimiter> {
        self.rate_limiter.clone()
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
    pub fn new(client: Client, call_context: E::CallContext) -> Self {
        let base_handler = E::endpoint_handler(&call_context);

        EndpointRunner {
            client,
            handler: base_handler,
            call_context,
        }
    }
}

impl<E: EndpointInfo, H: Handler> EndpointRunner<E, H> {
    pub async fn run_get_response(&self) -> Result<Response, EndpointRunnerError<H>> {
        self.handler.execute(self.client.get_request::<E>(&self.call_context).await?).await.map_err(EndpointRunnerError::HandlerError)
    }
    
    pub async fn run<O>(&self) -> Result<O, EndpointRunnerError<H>>
    where
        E: SupportsOutput<O>,
    {
        let response = self.run_get_response().await?;
        let proc_output = E::Process::process(response).await;

        Ok(proc_output)
    }

    pub async fn run_get_context<O>(self) -> Result<(O, E::CallContext), EndpointRunnerError<H>>
    where
        E: SupportsOutput<O>,
    {
        self.run::<O>().await.map(move |ok| (ok, self.call_context))
    }
}

pub struct EndpointRunnerRef<'a, E: EndpointInfo, H: Handler> {
    client: Client,
    handler: H,
    pub call_context: &'a mut E::CallContext,
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
        E: SupportsOutput<O>,
    { 
        let response = self.run_get_response().await?;
        let proc_output = <E as SupportsOutput<O>>::Process::process(response).await;

        Ok(proc_output)
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
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

pub struct HttpMethod {
    pub verb: HttpVerb,
    pub body: Option<Body>,
}

impl HttpMethod {
    pub fn new(verb: HttpVerb, body: Option<Body>) -> Self {
        Self {
            verb, 
            body
        }
    }

    pub fn new_no_body(verb: HttpVerb) -> Self {
        Self { verb, body: None }
    }
}


impl HttpVerb {
    pub fn as_reqwest_method(&self) -> Method {
        match self {
            HttpVerb::GET => Method::GET,
            HttpVerb::POST => Method::POST,
            HttpVerb::PUT => Method::PUT,
            HttpVerb::DELETE => Method::DELETE,
            HttpVerb::PATCH => Method::PATCH,
            HttpVerb::OPTIONS => Method::OPTIONS,
            HttpVerb::HEAD => Method::HEAD,
        }
    }
}
