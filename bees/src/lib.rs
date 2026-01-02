use net::init_rate_limiter_duration;

use crate::{core::{init_context, init_context_if_needed}, net::init_rate_limiter_duration_if_needed};

pub mod core;
pub mod endpoint_record;
pub mod net;

pub use dashmap;

// TODO: fix macros with EndpointTemplate and also maybe a post-response processing function on endpoints?

// idea: for making the post-response processing funcs work with general macros,
// make a special trait that already defines run() and then the func gets turned into
// a struct implementing that trait and `client` runs it through a specilized run_trait
// function

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