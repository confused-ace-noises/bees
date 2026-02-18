// use crate::context::context;
pub mod client;
pub mod request;
pub mod net_error;
pub mod bodies;

pub use client::*;
pub use request::*;

// static RATE_LIMITER_DURATION: OnceLock<RateLimiter> = OnceLock::new();

// pub(crate) fn get_rate_limiter() -> &'static RateLimiter {
//     RATE_LIMITER_DURATION.get().expect("this shouldn't happen. did you remember to init bees (`bees::init()`)?")
// }

// TEMP: move over to net
// static CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new(reqwest::Client::new()));

// pub(super) fn init_rate_limiter_duration(rate: usize) {
//     let Ok(_) = RATE_LIMITER_DURATION.set(RateLimiter::new(rate)) else {
//         panic!("RATE_LIMITER_DURATION was already set somehow???")
//     };
// }

// // TODO: add ability to decide a burst amount for the rate limiter

// pub(super) fn init_rate_limiter_duration_if_needed(rate: usize) {
//     let _ = RATE_LIMITER_DURATION.set(RateLimiter::new(rate));
// }

// pub fn client() -> &'static crate::net::client::Client {
//     &context().client
// }