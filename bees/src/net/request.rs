use core::fmt;
use std::time::Duration;

use delegate::delegate;
use http::{HeaderName, HeaderValue};
use reqwest::Response;

use crate::{
    handler::{BaseHandler, Handler, HandlerWrapper, WrapDecorate},
    net::{Client, net_error::NetError},
};

pub struct RequestRunner<H: Handler> {
    pub(super) request: Request,
    pub(super) handler: H,
}

impl<H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W> for RequestRunner<H> {
    type Output = RequestRunner<W::Output>;

    fn wrap(self, wrapper: W) -> Self::Output
    where
        W: crate::handler::HandlerWrapper<H>,
    {
        RequestRunner {
            request: self.request,
            handler: self.handler.wrap(wrapper),
        }
    }
}

impl<H: Handler> RequestRunner<H> {
    pub async fn run(self) -> Result<Response, H::Error> {
        self.handler.execute(self.request).await
    }
}

pub struct RequestBuilder {
    pub(super) client: Client,
    pub(super) inner: reqwest::RequestBuilder,
}

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestBuilder")
            .field("inner", &self.inner)
            .field("rate_limiter", &"async_rate_limiter internals")
            .finish()
    }
}

impl RequestBuilder {
    delegate! {
        to self.inner {
            #[expr(Self { inner: $, client: self.client })]
            pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> RequestBuilder
            where
                U: std::fmt::Display,
                P: std::fmt::Display;

            #[expr(Self { inner: $, client: self.client })]
            pub fn bearer_auth<T>(self, token: T) -> RequestBuilder
            where
                T: std::fmt::Display;

            #[expr(Self { inner: $, client: self.client })]
            pub fn body<T>(self, body: T) -> RequestBuilder
            where
                T: Into<reqwest::Body>;


            #[expr(Self { inner: $, client: self.client })]
            pub fn header<K, V>(self, key: K, value: V) -> RequestBuilder
            where
                HeaderName: TryFrom<K>,
                <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
                HeaderValue: TryFrom<V>,
                <HeaderValue as TryFrom<V>>::Error: Into<http::Error>;

            #[expr(Self { inner: $, client: self.client })]
            pub fn headers(self, headers: reqwest::header::HeaderMap) -> RequestBuilder;

            #[cfg(feature = "reqwest-query")]
            #[expr(Self { inner: $, client: self.client })]
            pub fn query<T: ?Sized + serde::Serialize>(self, query: &T) -> RequestBuilder;

            #[cfg(feature = "reqwest-form")]
            #[expr(Self { inner: $, client: self.client })]
            pub fn form<T: ?Sized + serde::Serialize>(self, form: &T) -> RequestBuilder;

            #[cfg(feature = "reqwest-json")]
            #[expr(Self { inner: $, client: self.client })]
            pub fn json<T: ?Sized + serde::Serialize>(self, json: &T) -> RequestBuilder;

            #[cfg(feature = "reqwest-multipart")]
            #[expr(Self { inner: $, client: self.client })]
            pub fn multipart(self, form: reqwest::multipart::Form) -> RequestBuilder;

            #[expr(Self { inner: $, client: self.client })]
            pub fn version(self, version: reqwest::Version) -> RequestBuilder;

            #[expr(Self { inner: $, client: self.client })]
            pub fn timeout(self, timeout: std::time::Duration) -> RequestBuilder;

        }
    }

    pub fn build(self) -> Result<Request, NetError> {
        Ok(Request {
            inner: self.inner.build()?,
            client: self.client
        })
    }

    pub async fn prepare_send(self) -> Result<RequestRunner<BaseHandler>, NetError> {
        Ok(RequestRunner {
            handler: BaseHandler,
            request: Request {
                inner: self.inner.build()?,
                client: self.client
            },
        })
    }
}

// OK

pub struct Request {
    pub(crate) inner: reqwest::Request,
    pub client: Client,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("inner", &self.inner)
            .field("rate_limiter", &"async_rate_limiter internals")
            .finish()
    }
}

impl Request {
    delegate! {
        to self.inner {
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

            
            #[expr(Some(Self { inner: $?, client: self.client.clone() }))]
            pub fn try_clone(&self) -> Option<Request>;
        }
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

impl TryFrom<Request> for http::Request<reqwest::Body> {
    type Error = reqwest::Error;

    fn try_from(value: Request) -> Result<Self, Self::Error> {
        let http_request = http::Request::try_from(value.inner)?;
        Ok(http_request)
    }
}