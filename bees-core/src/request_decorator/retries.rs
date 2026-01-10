use std::time::Duration;
use super::request_decorator::RequestDecorator;

use crate::net:: {
    client::Handler,
    request::Request,
};

use crate::handler_helper;

#[derive(Debug, Clone)]
pub struct Retries {
    pub max_retries: usize,
    pub base_delay: Duration,
}

impl Retries {
    pub fn new(max_retries: usize, base_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
        }
    }

    fn backoff_duration(&self, attempt: usize) -> Duration {
        let exp = (2u64.pow(attempt as u32 - 1)) as f64;
        let secs = self.base_delay.as_secs_f64() * exp;
        Duration::from_secs_f64(secs)
    }
}

impl<E: Send> RequestDecorator<E, E> for Retries {
    fn decorate<'a>(&self, f: Handler<'a, E>) -> Handler<'a, E> 
    where
        E: 'a,
    {
        // NOTE: `Retries` is so cheap that we can basically treat it as if it implemented Copy;
        // it takes up a comparable space to a `(usize, u32, u64)`; that's 20B, and this is also
        // just a very not hot path for the code to take, so don't try to optimize the two clones. 
        let clone = self.clone();
        handler_helper!(
            clone, f; 
            async move |req: Request| {
                let mut attempt = 0;
                let mut last_err = None;
                while attempt < clone.max_retries {
                    let resp = (f)(req.try_clone().expect("Retries RequestDecorator: cannot clone this request, the body isn't known. it might be a stream.")).await;
                    match resp {
                        Ok(res) => return Ok(res),
                        Err(err) => {
                            last_err = Some(err);
                            attempt += 1;
                            if attempt <= clone.max_retries {
                                let delay = clone.backoff_duration(attempt);
                                //eprintln!("Attempt {attempt} failed, retrying in {:?}...", delay);
                                tokio::time::sleep(delay).await;
                            }

                            continue;
                        }
                    }
                }
                Err(last_err.unwrap()) // this cannot fail
            }
        )
    }
}
