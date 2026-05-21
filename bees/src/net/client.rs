use crate::{
    endpoint::{EndpointExt, EndpointInfo, HandlerStack, HandlerStackError},
    handlers::{BaseHandler, Handler},
    net::{bodies::Body, net_error::NetError, rate_limiter::RateLimiter},
    resources::resource_handler::ResourceManager,
};
use futures::future::join;
use reqwest::{Client as ReqClient, Method, Response};
use std::{error::Error as StdError, fmt::Debug, sync::Arc};

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
        Self::__new(
            reqwest_client,
            Arc::new(rate_limiter),
            ResourceManager::new(),
        )
    }

    pub(crate) fn _new(rate_limiter: Arc<RateLimiter>) -> Self {
        Self::__new(
            ReqClient::new(),
            rate_limiter,
            ResourceManager::new(),
        )
    }

    pub(crate) fn __new(
        client: ReqClient,
        rate_limiter: Arc<RateLimiter>,
        res_manager: ResourceManager,
    ) -> Self {
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

    pub async fn execute_request_no_rate_limiter(
        &self,
        request: Request,
    ) -> Result<Response, Error> {
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

    pub fn get_raw_request_builder(
        &self,
        method: Method,
        url: impl reqwest::IntoUrl,
    ) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
            client: self.clone(),
        }
    }

    pub async fn execute_reqwest_req(&self, request: reqwest::Request) -> Result<Response, NetError> {
        self.rate_limiter.acquire().await;
        self.execute_reqwest_req_no_rate_limit(request).await
    }

    pub async fn execute_reqwest_req_no_rate_limit(
        &self,
        request: reqwest::Request,
    ) -> Result<Response, NetError> {
        // self.rate_limiter.acquire().await;
        Ok(self.inner.execute(request).await?)
    }

    pub async fn execute_request(&self, request: Request) -> Result<Response, NetError> {
        self.rate_limiter.acquire().await;
        self
            .inner
            .execute(request.inner)
            .await
            .map_err(NetError::ReqwestError)
    }

    // --------- ENDPOINT ---------
    pub async fn request_builder<E: EndpointInfo + 'static>(
        &self,
        call_context: &mut E::CallContext,
    ) -> Result<RequestBuilder, Error> {
        // determine whether this makes sense, does it give enough of a speed boost to
        // justify not guaranteeing order of operations?

        let url = E::full_url(&self.resource_manager, call_context).await?;
        let method = E::http_method(call_context).await;


        let request = self.get_raw_request_builder(method.verb.as_reqwest_method(), url);

        let mut request = match method.body {
            Some(body) => body.add_body(request).await?,
            None => request,
        };
        
        let endpoint_caps = E::capabilities(call_context);
        let record_caps = E::record_capabilities();

        let capabilities = record_caps.iter().chain(endpoint_caps.iter());

        for capability in capabilities {
            request = capability.apply(request).await?
        }

        Ok(request)
    }

    pub async fn get_request<E>(&self, call_context: &mut E::CallContext) -> Result<Request, Error>
    where
        E: EndpointInfo + 'static,
    {
        self.request_builder::<E>(call_context)
            .await
            .and_then(|rb| rb.build().map_err(Error::from))
    }

    // --------- RUN HELPERS ---------
    // pub async fn run_endpoint_with<E: EndpointInfo + HandlerStack<O>, O>(
    //     &self,
    //     call_context: E::CallContext,
    // ) -> Result<EndpointRunner<E, <E as HandlerStack<O>>::Handlers>, HandlerStackError> {
    //     EndpointRunner::<E, <E as HandlerStack<O>>::Handlers>::new(self.clone(), call_context).await
    // }

    // pub async fn run_endpoint_ref_with<'a, E: EndpointInfo + HandlerStack<O>, O>(
    //     &self,
    //     call_context: &'a mut E::CallContext,
    // ) -> Result<EndpointRunnerRef<'a, E, E::Handlers>, HandlerStackError> {
    //     EndpointRunnerRef::<E, <E as HandlerStack<O>>::Handlers>::new(self.clone(), call_context).await
    // }

    // pub async fn run_endpoint<E: EndpointInfo<CallContext = ()> + HandlerStack<O>, O>(
    //     &self,
    // ) -> Result<EndpointRunner<E, <E as HandlerStack<O>>::Handlers>, HandlerStackError> {
    //     EndpointRunner::<E, <E as HandlerStack<O>>::Handlers>::new(self.clone(), ()).await
    // }

    // pub async fn run_endpoint_ref<'a, E: EndpointInfo<CallContext = ()> + HandlerStack<O>, O>(
    //     &self,
    //     call_context: &'a mut E::CallContext,
    // ) -> Result<EndpointRunnerRef<'a, E, E::Handlers>, HandlerStackError> {
    //     EndpointRunnerRef::<E, <E as HandlerStack<O>>::Handlers>::new(self.clone(), call_context).await
    // }

    pub async fn run_endpoint_with<E: EndpointInfo + HandlerStack<O>, O>(
        &self,
        mut call_context: E::CallContext,
    ) -> Result<O, Error> {
        let handlers = E::handlers(&mut call_context).await?;

        Ok(handlers.execute(self.get_request::<E>(&mut call_context).await?).await)
    }

    pub async fn run_endpoint_ref_with<'a, E: EndpointInfo + HandlerStack<O>, O>(
        &self,
        call_context: &'a mut E::CallContext,
    ) -> Result<O, Error> {
        let handlers = E::handlers(call_context).await?;

        Ok(handlers.execute(self.get_request::<E>(call_context).await?).await)
    }

    pub async fn run_endpoint<E: EndpointInfo<CallContext = ()> + HandlerStack<O>, O>(
        &self,
    ) -> Result<O, Error> {
        self.run_endpoint_with::<E, O>(()).await
    }

    pub async fn run_endpoint_ref<'a, E: EndpointInfo<CallContext = ()> + HandlerStack<O>, O>(
        &self,
    ) -> Result<O, Error>  {
        self.run_endpoint_ref_with::<E, O>(&mut ()).await
    }

    pub fn get_rate_limiter(&self) -> Arc<RateLimiter> {
        self.rate_limiter.clone()
    }
}

// #[derive(Debug)]
// pub struct EndpointRunner<E: EndpointInfo, H: Handler> {
//     client: Client,
//     handler: H,
//     call_context: E::CallContext,
// }

// impl<E: EndpointInfo, H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W>
//     for EndpointRunner<E, H>
// {
//     type Output = EndpointRunner<E, W::Output>;

//     fn wrap(self, wrapper: W) -> Self::Output {
//         EndpointRunner {
//             client: self.client,
//             handler: self.handler.wrap(wrapper),
//             call_context: self.call_context,
//         }
//     }
// }

// impl<E, H> EndpointRunner<E, H>
// where
//     E: EndpointInfo,
//     H: Handler,
// {
//     pub async fn new<O>(client: Client, call_context: E::CallContext) -> Result<EndpointRunner<E, E::Handlers>, HandlerStackError>
//     where
//         E: HandlerStack<O>,
//     {
//         let base_handler = <E as HandlerStack<O>>::handlers(&call_context).await?;

//         Ok(EndpointRunner {
//             client,
//             handler: base_handler,
//             call_context,
//         })
//     }
// }

// impl<E, H> EndpointRunner<E, H>
// where
//     E: EndpointInfo + 'static,
//     H: Handler<Input = Request>,
// {
//     pub async fn run(&self) -> Result<H::Output, Error> {
//         Ok(self
//             .handler
//             .execute(self.client.get_request::<E>(&self.call_context).await?)
//             .await)
//     }

//     pub async fn run_force_response(&self) -> Result<Response, Error> {
//         BaseHandler::execute(
//             &BaseHandler,
//             self.client.get_request::<E>(&self.call_context).await?,
//         )
//         .await
//     }

//     pub async fn run_get_context(self) -> Result<(H::Output, E::CallContext), Error> {
//         Ok((self
//             .handler
//             .execute(self.client.get_request::<E>(&self.call_context).await?)
//             .await, self.call_context))
//     }

//     pub async fn run_force_response_get_context(self) -> Result<(Response, E::CallContext), Error> {
//         Ok((BaseHandler::execute(
//             &BaseHandler,
//             self.client.get_request::<E>(&self.call_context).await?,
//         )
//         .await?, self.call_context))
//     }
// }

// pub struct EndpointRunnerRef<'a, E: EndpointInfo, H: Handler> {
//     client: Client,
//     handler: H,
//     pub call_context: &'a E::CallContext,
// }

// impl<'a, E: EndpointInfo, H: Handler> EndpointRunnerRef<'a, E, H> {
//     pub async fn new<O>(client: Client, call_context: &'a mut E::CallContext) -> Result<EndpointRunnerRef<'a, E, E::Handlers>, HandlerStackError> 
//     where 
//         E: HandlerStack<O>
//     {
//         let base_handler = E::handlers(call_context).await?;

//         Ok(EndpointRunnerRef {
//             client,
//             handler: base_handler,
//             call_context,
//         })
//     }
// }

// impl<'a, E: EndpointInfo + 'static, H: Handler<Input = Request>> EndpointRunnerRef<'a, E, H> {
//     pub async fn run(&self) -> Result<H::Output, Error> {
//         Ok(self
//             .handler
//             .execute(self.client.get_request::<E>(self.call_context).await?)
//             .await)
//     }

//     pub async fn run_force_response(&self) -> Result<Response, Error> {
//         BaseHandler::execute(
//             &BaseHandler,
//             self.client.get_request::<E>(self.call_context).await?,
//         )
//         .await
//     }
// }

// impl<'a, E: EndpointInfo, H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W>
//     for EndpointRunnerRef<'a, E, H>
// {
//     type Output = EndpointRunnerRef<'a, E, W::Output>;

//     fn wrap(self, wrapper: W) -> Self::Output {
//         EndpointRunnerRef {
//             client: self.client,
//             handler: self.handler.wrap(wrapper),
//             call_context: self.call_context,
//         }
//     }
// }

// #[derive(Debug, Display, Error, From)]
// pub enum EndpointRunnerError<H: Handler> {
//     FailedToBuildRequest(#[error(source)] Error),

//     #[from(skip)]
//     HandlerError(#[error(source)] H::Error),
// }

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
        Self { verb, body }
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
