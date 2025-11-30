use std::{sync::OnceLock, time::Duration};

pub mod client;
pub mod rate_limiter;

static RATE_LIMITER_DURATION: OnceLock<std::time::Duration> = OnceLock::new();

pub(crate) fn get_rate_limiter_duration() -> &'static Duration {
    RATE_LIMITER_DURATION.get().expect("this shoudln't happen. did you remember to init bees (`bees::init()`)?")
}

pub(super) fn init_rate_limiter_duration(duration: Duration) {
    RATE_LIMITER_DURATION.set(duration).expect("RATE_LIMITER_DURATION was already set somehow???");
}
