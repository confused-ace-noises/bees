use std::{future::ready, str::FromStr};

use http::{HeaderMap, HeaderName, HeaderValue};


// use super::do_capability_impl;

#[cfg(not(feature = "async-trait"))]
use crate::capability::CapabilityOutput;

use crate::{CapError, capability::Capability, utils::error::Error};

pub struct AddHeaderMap(pub HeaderMap);



#[cfg(not(feature = "async-trait"))]
impl Capability for AddHeaderMap {
    fn apply<'a>(&'a self, mut request: crate::net::RequestBuilder) -> crate::CapabilityOutput<'a> {
        request = request.headers(self.0.clone());
        CapabilityOutput::new(ready(Ok(request)))
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Capability for AddHeaderMap {
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

#[cfg(not(feature = "async-trait"))]
impl Capability for AddHeaders {
    fn apply<'a>(&'a self, mut request: crate::net::RequestBuilder) -> crate::CapabilityOutput<'a> {
        request = request.headers(self.make_header_map()?);

        CapabilityOutput::new(ready(Ok(request)))
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl Capability for AddHeaders {
    async fn apply(&self, mut request: crate::net::RequestBuilder) -> Result<crate::net::RequestBuilder, CapError> {
        request = request.headers(self.make_header_map()?);

        Ok(request)
    }
}