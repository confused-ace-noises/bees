use std::{sync::atomic::{AtomicU64, Ordering}};
use tokio::time::{Instant as TokioInstant, Duration as TokioDuration};

#[derive(Debug)]
pub struct RateLimiter {
    next_at: AtomicU64,
    start: TokioInstant,
    nanos_per_token: u64,
    burst_nanos: u64,
}

impl RateLimiter {
    pub fn new(rate_per_sec: f64, burst: u64) -> Self {
        assert_ne!(rate_per_sec, 0.0, "rate_per_sec may not be 0");
        assert!(rate_per_sec.is_finite(), "rate_per_sec may only be finite");
        assert!(burst > 0, "burst may not be 0");

        // ceil so you always stay juuust under to never get yourself yeeted from the api
        let nanos_per_token = (1_000_000_000.0 / rate_per_sec).ceil() as u64;
        Self {
            next_at: AtomicU64::new(0),
            start: TokioInstant::now(),
            nanos_per_token,
            burst_nanos: nanos_per_token * burst,
        }
    }

    pub async fn acquire(&self) {
        let now = self.start.elapsed().as_nanos() as u64;
        let earliest = now.saturating_sub(self.burst_nanos);

        let slot = self.next_at
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |next| {
                Some(next.max(earliest) + self.nanos_per_token)
            })
            .unwrap()
            .max(earliest);

        if slot > now {
            tokio::time::sleep(TokioDuration::from_nanos(slot - now)).await;
        }
    }
}