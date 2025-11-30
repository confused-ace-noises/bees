use std::{error::Error, pin::Pin, sync::Arc};

use tokio::time::Duration;

use reqwest::Response;

use crate::{Sealed, net::client::Request};

pub type Handler<'a, E> = 
    Arc<
        dyn Fn(Request) -> Pin<
                Box<
                    dyn Future<
                        Output = Result<Response, E>
                > + Send + 'a>
            > + Send + Sync + 'a
    >;

pub trait RequestDecorator<E: Error + Send, G: Error + Send>: Send + Sync {
    fn decorate<'a>(&'a self, f: Handler<'a, E>) -> Handler<'a, G>
    where
        E: 'a,
        G: 'a;
}

pub struct Retries {
    pub max_retries: usize,
    pub base_delay: Duration,
}

impl Retries {
    fn backoff_duration(&self, attempt: usize) -> Duration {
        let exp = (2u64.pow(attempt as u32 - 1)) as f64;
        // let jitter = fastrand::u64(0..50);
        let secs = self.base_delay.as_secs_f64() * exp;
        Duration::from_secs_f64(secs)
    }
}

impl<E: Error + Send> RequestDecorator<E, E> for Retries {
    fn decorate<'a>(&'a self, f: Handler<'a, E>) -> Handler<'a, E> 
    where
        E: 'a,
    {
        Arc::new(move |req: Request| {
            let f = f.clone();
            Box::pin(async move {
                let mut attempt = 0;
                let mut last_err = None;

                while attempt < self.max_retries {
                    let resp = (f)(req.try_clone().expect("Retries RequestDecorator: cannot clone this request, the body isn't known. it might be a stream.")).await;
                    match resp {
                        Ok(res) => return Ok(res),
                        Err(err) => {
                            last_err = Some(err);
                            attempt += 1;
                            if attempt <= self.max_retries {
                                let delay = self.backoff_duration(attempt);
                                eprintln!("Attempt {attempt} failed, retrying in {:?}...", delay);
                                tokio::time::sleep(delay).await;
                            }

                            continue;
                        }
                    }
                }
                Err(last_err.unwrap())
            })
        })
    }
}

#[allow(private_bounds)]
pub trait Decorate<'a, E: Error + Send + 'a>: Sealed {
    fn decorate<G: Error + Send, T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T)-> Handler<'a, G>
    where
        G: 'a;
      
}

impl<'a, E: Error + std::marker::Send + 'a> Sealed for Handler<'a, E>{}
impl Sealed for Request{}


impl<'a, E: Error + std::marker::Send + 'a> Decorate<'a, E> for Handler<'a, E> {
    fn decorate<G: Error + Send + 'a, T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T) -> Handler<'a, G> {
        decorator.decorate(self)
    }
}