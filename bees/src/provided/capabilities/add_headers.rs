use std::{future::ready, str::FromStr};

use http::{HeaderMap, HeaderName, HeaderValue};


// use super::do_capability_impl;

#[cfg(not(feature = "async-trait"))]
use crate::capability::CapabilityOutput;

use crate::{capability::CapError, capability::Capability, utils::error::Error};

pub struct AddHeaderMap(pub HeaderMap);



#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Capability for AddHeaderMap {
    #[cfg(not(feature = "async-trait"))]
    fn apply<'a>(&'a self, mut request: crate::net::RequestBuilder) -> CapabilityOutput<'a> {
        request = request.headers(self.0.clone());
        CapabilityOutput::new(ready(Ok(request)))
    }

    #[cfg(feature = "async-trait")]
    async fn apply(&self, mut request: crate::net::RequestBuilder) -> Result<crate::net::RequestBuilder, CapError> {
        request = request.headers(self.0.clone());
        Ok(request)
    }
}

pub struct AddHeaders(pub Vec<(String, String)>);

impl AddHeaders {
    pub fn make_header_map(&self) -> Result<HeaderMap, CapError> {
        let mut header_map = HeaderMap::new();

        for (k, v) in &self.0 {
            let name = match HeaderName::from_str(k) {
                Ok(n) => n,
                Err(e) => {
                    return Err(Box::new(e) as CapError);
                }
            };

            let value = match HeaderValue::from_str(v) {
                Ok(v) => v,
                Err(e) => {
                    return Err(Box::new(e) as CapError);
                }
            };

            header_map.append(name, value);
        }

        Ok(header_map)
    }
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Capability for AddHeaders {
    #[cfg(not(feature = "async-trait"))]
    fn apply<'a>(&'a self, mut request: crate::net::RequestBuilder) -> CapabilityOutput<'a> {
        match self.make_header_map() {
            Ok(map) => {
                request = request.headers(map);
                CapabilityOutput::new(ready(Ok(request)))
            },
            Err(err) => {
                CapabilityOutput::new(ready(Err(err)))
            },
        }
    }

    #[cfg(feature = "async-trait")]
    async fn apply(&self, mut request: crate::net::RequestBuilder) -> Result<crate::net::RequestBuilder, CapError> {
        request = request.headers(self.make_header_map()?);

        Ok(request)
    }
}