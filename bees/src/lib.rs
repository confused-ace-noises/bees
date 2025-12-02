use net::init_rate_limiter_duration;
use std::time::Duration;

use crate::core::init_context;

pub mod core;
pub mod endpoint_record;
pub mod net;

pub use dashmap;

// TODO: fix macros with EndpointTemplate and also maybe a post-response processing function on endpoints?

// idea: for making the post-response processing funcs work with general macros,
// make a special trait that already defines run() and then the func gets turned into
// a struct implementing that trait and `client` runs it through a specilized run_trait
// function

pub fn init(rate_limiter_duration: Duration) {
    init_rate_limiter_duration(rate_limiter_duration);
    init_context();
}
pub(crate) trait Sealed {}