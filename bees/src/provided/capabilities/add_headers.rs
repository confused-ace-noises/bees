use std::{future::ready, str::FromStr};

use http::{HeaderMap, HeaderName, HeaderValue};

use crate::{CapabilityOutput, capability::Capability, utils::error::Error};

pub struct AddHeaderMap(pub HeaderMap);

impl Capability for AddHeaderMap {
    fn apply<'a>(&'a self, mut request: crate::net::RequestBuilder) -> crate::CapabilityOutput<'a> {
        request = request.headers(self.0.clone());
        CapabilityOutput::new(ready(Ok(request)))
    }
}

pub struct AddHeaders(pub Vec<(String, String)>);
impl Capability for AddHeaders {
    fn apply<'a>(&'a self, mut request: crate::net::RequestBuilder) -> crate::CapabilityOutput<'a> {
        let mut header_map = HeaderMap::new();


        for (k, v) in &self.0 {
            let name = match HeaderName::from_str(k) {
                Ok(n) => n,
                Err(e) => {
                    return CapabilityOutput::new(ready(Err(Error::CapabilityError(Box::new(e)))));
                }
            };


            let value = match HeaderValue::from_str(v) {
                Ok(v) => v,
                Err(e) => {
                    return CapabilityOutput::new(ready(Err(Error::CapabilityError(Box::new(e)))));
                }
            };


            header_map.append(name, value);
        }


        request = request.headers(header_map);


        CapabilityOutput::new(ready(Ok(request)))
    }
}