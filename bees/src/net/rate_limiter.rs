use std::{sync::atomic::{AtomicU64, Ordering}, time::{Duration, Instant}};
use tokio::time::{Instant as TokioInstant, sleep_until};

#[derive(Debug)]
pub struct RateLimiter {
    next_allowed_ms: AtomicU64, // val
    interval: Duration,
    start: TokioInstant,
}

impl RateLimiter {
    #[must_use]
    pub fn new(interval: &Duration) -> Self {
        let now = Instant::now();
        Self {
            next_allowed_ms: AtomicU64::new(now.elapsed().as_millis() as u64),
            interval: *interval,
            // name,
            start: TokioInstant::now(),
        }
    }

    pub async fn wait(&self) {
        loop {
            let now = TokioInstant::now().duration_since(self.start).as_millis() as u64;
            let next = self.next_allowed_ms.load(Ordering::Acquire);

            if now >= next {
                let new_next = now + self.interval.as_millis() as u64;

                if self
                    .next_allowed_ms
                    // .compare_exchange(next, new_next, Ordering::SeqCst, Ordering::SeqCst) // TODO: understand CAS
                    .compare_exchange(next, new_next, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    // debug!(
                    //     rate_limiter_name = name,
                    //     "going without waiting, already clear"
                    // );
                    return;
                } else {
                    // trace!(
                    //     soft_error = true,
                    //     rate_limiter_name = name,
                    //     "CAS failed, retrying"
                    // );
                }
            } else {
                let wait_duration = Duration::from_millis(next - now);

                // debug!(
                //     rate_limiter_name = name,
                //     wait_ms = wait_duration.as_millis(),
                //     "waiting for: {} millisecs",
                //     wait_duration.as_millis()
                // );

                sleep_until(TokioInstant::now() + wait_duration).await;
                // debug!(rate_limiter_name = name, "done waiting");
            }
        }
    }
}
