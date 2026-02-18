#[cfg(not(feature = "async-trait"))]
use crate::CapabilityOutput;
use crate::{capability::Capability, net::RequestBuilder, utils::format_string::FormatString};
use std::fmt::Debug;
use crate::utils::error::Error;

#[derive(Debug)]
pub struct Body(pub Box<dyn BodyAdder>);

impl Body {
    pub fn new<B: BodyAdder + 'static>(body_adder: B) -> Self {
        Self(Box::new(body_adder) as Box<dyn BodyAdder>)
    }

    pub async fn add_body(
        &self,
        request: RequestBuilder,
    ) -> Result<RequestBuilder, Error> {
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
        })
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Capability for TextBody {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, Error> {
        self.0
            .to_formatted_now()
            .await
            .map(|string| request.body(string))
    }
}

#[cfg(feature = "reqwest-json")]
#[derive(Debug)]
pub struct JsonBody(pub serde_json::Value);

#[cfg(all(feature = "reqwest-json", not(feature = "async-trait")))]
impl Capability for JsonBody {
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move {
            FormatString::new(self.0.to_string())
                .to_formatted_now()
                .await
                .map(|j| request.body(j))
        })
    }
}

#[cfg(all(feature = "reqwest-json", feature = "async-trait"))]
#[async_trait::async_trait]
impl Capability for JsonBody {
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, Error> {
        FormatString::new(self.0.to_string())
            .to_formatted_now()
            .await
            .map(|j| request.body(j))
    }
}

#[cfg(feature = "reqwest-json")]
impl BodyAdder for JsonBody {}

#[cfg(feature = "reqwest-multipart")]
pub struct MultiPartBody<F>(pub F)
where
    F: Fn() -> Result<reqwest::multipart::Form, Error> + Send + Sync + 'static;

#[cfg(all(feature = "reqwest-multipart", not(feature = "async-trait")))]
impl<F> Capability for MultiPartBody<F>
where
    F: Fn() -> Result<reqwest::multipart::Form, Error> + Send + Sync + 'static,
{
    fn apply<'a>(&'a self, request: RequestBuilder) -> CapabilityOutput<'a> {
        CapabilityOutput::new(async move { Ok(request.multipart((self.0)()?)) })
    }
}

#[cfg(all(feature = "reqwest-multipart", feature = "async-trait"))]
#[async_trait::async_trait]
impl<F> Capability for MultiPartBody<F>
where
    F: Fn() -> Result<reqwest::multipart::Form, Error> + Send + Sync + 'static,
{
    async fn apply(&self, request: RequestBuilder) -> Result<RequestBuilder, Error> {
        Ok(request.multipart((self.0)()?))
    }
}
