use bees::{
    self, endpoint,
    endpoint_record::endpoint::{Capability, HttpVerb},
    net::client::RequestBuilder,
    record,
};

pub struct DummyCap;

impl Capability for DummyCap {
    fn apply(
        &self,
        request: bees::net::client::RequestBuilder,
    ) -> bees::net::client::RequestBuilder {
        request.header("hello", "world")
    }
}

#[test]
pub fn make_record_simple() {
    bees::init_default_if_needed();

    record!("make_record_simple" => "https://my.api.com/api/");
}

#[test]
pub fn record_macro() {
    bees::init_default_if_needed();

    let simple = record!("simple" => "https://my.api.com/api/");
    let simple_caps = record!("simple_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);

    let noreg = record!("simple_noreg" => "https://my.api.com/api/");
    let noreg_caps =
        record!("simple_noreg_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);

    let (multiple_simple, multiple_simple_caps) = record!("multiple_simple" => "https://my.api.com/api/", "multiple_simple_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);
    let (multiple_noreg, multiple_noreg_caps) = record!(noreg "multiple_noreg" => "https://my.api.com/api/", "multiple_noreg_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);

    assert_eq!(record!("simple"), simple);
    assert_eq!(record!("simple_caps"), simple_caps);

    assert_eq!(record!(option "noreg"), None);
    assert_eq!(record!(option "noreg_caps"), None);

    assert_eq!(record!("multiple_simple"), multiple_simple);
    assert_eq!(record!("multiple_simple_caps"), multiple_simple_caps);

    assert_eq!(record!(option "multiple_noreg"), None);
    assert_eq!(record!(option "multiple_noreg_caps"), None);
}

#[test]
pub fn make_endpoint_simple() {
    bees::init_default_if_needed();

    record!("make_endpoint_simple_record" => "https://my.api.com/api/");
    endpoint!("make_endpoint_simple_record" => new "make_endpoint_simple", "ednpointpath", HttpVerb::GET, async |x| x.text().await);
}
