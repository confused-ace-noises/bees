use core::fmt;
use std::{error::Error as StdError, fmt::Debug, time::Duration};

use async_rate_limiter::RateLimiter;
use http::{HeaderName, HeaderValue};
use reqwest::Response;

use super::client::{Client, Handler};
#[cfg(not(feature = "async-trait"))]
use crate::capability::CapabilityOutput;
use crate::{
    Sealed,
    capability::Capability,
    net::get_rate_limiter,
    request_decorator::{Decorate, RequestDecorator},
    utils::{Error, FormatString},
};
use delegate::delegate;

pub struct RequestRunner<'a, E: Send + 'a> {
    pub(super) client: Client,
    pub(super) handler: Handler<'a, E>,
    pub(super) request: Request,
}

impl<'a, E: Send + 'a> Sealed for RequestRunner<'a, E> {}

impl<'a, E: Send + 'a, G: Send + 'a> Decorate<'a, E, G> for RequestRunner<'a, E> {
    type Output = RequestRunner<'a, G>;

    fn decorate<T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T) -> Self::Output {
        let new_handler = self.handler.decorate(decorator);
        RequestRunner {
            client: self.client,
            handler: new_handler,
            request: self.request,
        }
    }
}

impl<'a, E: StdError + Send + 'static> RequestRunner<'a, E>
where
    E: StdError + Send + 'static,
{
    pub async fn run(self) -> Result<Response, Box<dyn StdError>> {
        Ok((self.handler)(self.request).await?)
    }
}

pub struct RequestBuilder {
    pub(super) inner: reqwest::RequestBuilder,
    pub(super) rate_limiter: &'static RateLimiter,
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

            #[cfg(feature = "reqwest_query")]
            #[expr(Self { inner: $, rate_limiter: self.rate_limiter })]
            pub fn query<T: ?Sized + serde::Serialize>(self, query: &T) -> RequestBuilder;

            #[cfg(feature = "reqwest_form")]
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
        self.rate_limiter.acquire().await;
        self.inner.send().await
    }
}

pub struct Request {
    pub(super) inner: reqwest::Request,
    pub(super) rate_limiter: &'static RateLimiter,
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

            #[expr(Some(Self { inner: $?, rate_limiter: self.rate_limiter }))]
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
            rate_limiter: get_rate_limiter(),
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

#[derive(Debug)]
pub struct Body(Box<dyn BodyAdder>);

impl Body {
    pub fn new<B: BodyAdder + 'static>(body_adder: B) -> Self {
        Self(Box::new(body_adder) as Box<dyn BodyAdder>)
    }

    pub async fn add_body(
        &self,
        request: RequestBuilder,
    ) -> Result<RequestBuilder, crate::utils::Error> {
        self.0.apply(request).await
    }
}

pub trait BodyAdder: Capability + Debug + Send + Sync {}

#[derive(Debug)]
pub struct TextBody(pub FormatString);

#[cfg(not(feature = "async-trait"))]
impl Capability for TextBody {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move {
            self.0
                .to_formatted_now()
                .await
                .map(|string| request.body(string))
                .map_err(|s| Error::CapabilityError(Box::new(s)))
        })
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Capability for TextBody {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::utils::Error> {
        self.0
            .to_formatted_now()
            .await
            .map(|string| request.body(string))
            .map_err(|s| Error::CapabilityError(Box::new(s)))
    }
}

#[cfg(feature = "reqwest_json")]
#[derive(Debug)]
pub struct JsonBody(pub serde_json::Value);

#[cfg(all(feature = "reqwest_json", not(feature = "async-trait")))]
impl Capability for JsonBody {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move {
            FormatString::new(self.0.to_string())
                .to_formatted_now()
                .await
                .map(|j| request.body(j))
                .map_err(|s| Error::CapabilityError(Box::new(s)))
        })
    }
}

#[cfg(all(feature = "reqwest_json", feature = "async-trait"))]
#[async_trait::async_trait]
impl Capability for JsonBody {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, crate::utils::Error> {
        FormatString::new(self.0.to_string())
            .to_formatted_now()
            .await
            .map(|j| request.body(j))
            .map_err(|s| Error::CapabilityError(Box::new(s)))
    }
}

#[cfg(feature = "reqwest_json")]
impl BodyAdder for JsonBody {}

#[cfg(feature = "reqwest_multipart")]
pub struct MultiPartBody<F>(pub F)
where
    F: Fn() -> Result<reqwest::multipart::Form, Error> + Send + Sync + 'static;

#[cfg(all(feature = "reqwest_multipart", not(feature = "async-trait")))]
impl<F> Capability for MultiPartBody<F>
where
    F: Fn() -> Result<reqwest::multipart::Form, Error> + Send + Sync + 'static,
{
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move { Ok(request.multipart((self.0)()?)) })
    }
}

#[cfg(all(feature = "reqwest_multipart", feature = "async-trait"))]
#[async_trait::async_trait]
impl<F> Capability for MultiPartBody<F>
where
    F: Fn() -> Result<reqwest::multipart::Form, Error> + Send + Sync + 'static,
{
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, Error> {
        Ok(request.multipart((self.0)()?))
    }
}