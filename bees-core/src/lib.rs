use net::init_rate_limiter_duration;

use crate::{
    context::{init_context, init_context_if_needed}, 
    net::init_rate_limiter_duration_if_needed,
};

pub mod context;
pub mod endpoint_def;
pub mod net;
pub mod record_def;
pub mod resource_def;
pub mod utils;
pub mod capability;
pub mod request_decorator;


pub fn init(rate: usize) {
    init_rate_limiter_duration(rate);
    init_context();
}

pub fn init_default() {
    init_rate_limiter_duration(2);
    init_context();
}

pub fn init_default_if_needed() {
    init_rate_limiter_duration_if_needed(2);
    init_context_if_needed();
}

pub(crate) trait Sealed {}

pub mod prelude {
    pub use crate::net::client;
    pub use crate::net::client::{Client, EndpointRunner, HttpVerb};
    pub use crate::net::request::{Body, Request};

    pub use crate::endpoint_def::{Endpoint, no_op_processor};
    pub use crate::record_def::Record;
    
    pub use crate::resource_def::{Resource, resource_utils::static_res::StaticResource};

    pub use crate::{endpoint, init, record_def, resource};
}

pub mod re_exports {
    pub use dashmap;
    pub use reqwest;
}