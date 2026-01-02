use std::{collections::HashMap, time::Duration};

use bees::{
    self, handler_helper,
    core::client,
    endpoint,
    endpoint_record::{
        endpoint::{Capability, HttpVerb, no_op_processor},
        request_decorator::{Decorate, MultipleDecorators, RequestDecorator, Retries},
    },
    net::client::Request,
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

    let _noreg = record!("simple_noreg" => "https://my.api.com/api/");
    let _noreg_caps = record!("simple_noreg_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);

    let (multiple_simple, multiple_simple_caps) = record!("multiple_simple" => "https://my.api.com/api/", "multiple_simple_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);
    let (_multiple_noreg, _multiple_noreg_caps) = record!(noreg "multiple_noreg" => "https://my.api.com/api/", "multiple_noreg_caps" => "https://my.api.com/api/"; [DummyCap, DummyCap]);

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

struct DummyDecorator1;
impl<E, G> RequestDecorator<E, G> for DummyDecorator1
where
    E: Send,
    G: Send + From<E>,
{
    fn decorate<'a>(
        &self,
        f: bees::endpoint_record::request_decorator::Handler<'a, E>,
    ) -> bees::endpoint_record::request_decorator::Handler<'a, G>
    where
        E: 'a,
        G: 'a,
    {
        handler_helper!(
            f;
            async move |req: Request| {
                match f(req).await {
                    Ok(ok) => Ok(ok),
                    Err(e) => Err(e.into()),
                }
            }
        )
    }
}

#[test]
pub fn decorators() {
    let client: &'static bees::net::client::Client = client();

    record!("decorators_record" => "https://my.api.com/api/");
    endpoint!("decorators_record" => new "decorators_endpoint", "endpoint/path", HttpVerb::GET, no_op_processor);

    let multiple_decs: MultipleDecorators<_, reqwest::Error> =
        MultipleDecorators::new(Retries::new(3, Duration::from_millis(200))).push(DummyDecorator1);

    client
        .run_endpoint(
            endpoint!("decorators_record" => "decorators_endpoint"),
            &HashMap::new(),
            &vec![],
        )
        .decorate(&multiple_decs);
}
