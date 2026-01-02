use net::init_rate_limiter_duration;

use crate::{core::{init_context, init_context_if_needed}, net::init_rate_limiter_duration_if_needed};

pub mod core;
pub mod endpoint_record;
pub mod net;

pub use dashmap;

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
    pub use crate::net::client::{Client, EndpointRunner};
    pub use crate::{init, endpoint, record, resource};
    pub use crate::core::{client, resource::Resource, resources_utils::static_res::StaticResource};
    pub use crate::endpoint_record::endpoint::{Endpoint, FormatString, HttpVerb, Capability};
    pub use crate::endpoint_record::record::Record;
}