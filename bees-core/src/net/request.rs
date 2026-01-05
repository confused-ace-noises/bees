use core::fmt;
use std::{borrow::Borrow, collections::HashMap, error::Error, time::Duration};

use async_rate_limiter::RateLimiter;
use http::{HeaderName, HeaderValue};
use reqwest::Response;

use crate::{Sealed, net::get_rate_limiter, request_decorator::{Decorate, RequestDecorator}, utils::FormatString};
use super::client::{Client, Handler};
use delegate::delegate;

pub struct RequestRunner<'a, E: Send + 'a> {
    pub(super) client: Client,
    pub(super) handler: Handler<'a, E>,
    pub(super) request: Request,
}

impl<'a, E: Send + 'a> Sealed for RequestRunner<'a, E>{}

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

impl<'a, E: Error + Send + 'static> RequestRunner<'a, E>
where
    E: Error + Send + 'static,
{
    pub async fn run(self) -> Result<Response, Box<dyn Error>> {
        Ok((self.handler)(self.request).await?)
    }
}

pub struct RequestBuilder {
    pub(super) inner: reqwest::RequestBuilder,
    pub(super) rate_limiter: &'static RateLimiter,
}

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestBuilder").field("inner", &self.inner).field("rate_limiter", &"async_rate_limiter internals").finish()
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
        f.debug_struct("Request").field("inner", &self.inner).field("rate_limiter", &"async_rate_limiter internals").finish()
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

#[derive(Clone)]
pub enum Body {
    Text(FormatString),
    #[cfg(feature = "reqwest_json")]
    Json(serde_json::Value),
    #[cfg(feature = "reqwest_multipart")]
    Multipart(Arc<Box<dyn Fn(&HashMap<String, String>) -> reqwest::multipart::Form + Send + Sync>>),
}
impl fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            #[cfg(feature = "reqwest_json")]
            Self::Json(arg0) => f.debug_tuple("Json").field(arg0).finish(),
            #[cfg(feature = "reqwest_multipart")]
            Self::Multipart(_) => f.debug_tuple("Multipart").field(&"Box<dyn Fn(&HashMap<String, String>) -> reqwest::multipart::Form + Send + Sync>").finish(),
        }
    }
}

impl Body {
    pub async fn to_formatted(&self, values: impl Borrow<HashMap<String, String>>) -> reqwest::Body {
        match self {
            Body::Text(format_string) => format_string.to_formatted_now(values).await.expect("TODO: make a decent error system; format values should include all values to be formatted").into(),
    
            #[cfg(feature = "reqwest_json")]
            Body::Json(value) => {
                FormatString::new(value.to_string()).to_formatted_now(values).await.expect("TODO: make a decent error system; format values should include all values to be formatted").into()
            }

            #[cfg(feature = "reqwest_multipart")]
            Body::Multipart(multipart) => {
                ((multipart)(values.borrow())).into()
            }
        }
    }
}
