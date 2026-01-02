use std::{pin::Pin, sync::Arc};

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

pub trait RequestDecorator<E: Send, G: Send>: Send + Sync {
    fn decorate<'a>(&self, f: Handler<'a, E>) -> Handler<'a, G>
    where
        E: 'a,
        G: 'a;
}

#[derive(Debug, Clone)]
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

impl<E: Send> RequestDecorator<E, E> for Retries {
    fn decorate<'a>(&self, f: Handler<'a, E>) -> Handler<'a, E> 
    where
        E: 'a,
    {
        // NOTE: `Retries` is so cheap that we can basically treat it as if it implemented Copy;
        // it takes up a comparable space to a `(usize, u32, u64)`; that's 20B, and this is also
        // just a very not hot path for the code to take, so don't try to optimize the two clones. 
        let clone = self.clone();
        Arc::new(move |req: Request| {
            let clone = clone.clone();
            let f: Handler<'a, E> = f.clone(); // arc!
            Box::pin(async move {
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
                Err(last_err.unwrap())
            })
        })
    }
}

#[allow(private_bounds)]
pub trait Decorate<'a, E: Send + 'a>: Sealed {
    fn decorate<G: Send, T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T)-> Handler<'a, G>
    where
        G: 'a;
      
}

impl<'a, E: std::marker::Send + 'a> Sealed for Handler<'a, E>{}
impl Sealed for Request{}


impl<'a, E: std::marker::Send + 'a> Decorate<'a, E> for Handler<'a, E> {
    fn decorate<G: Send + 'a, T: RequestDecorator<E, G> + 'a + ?Sized>(self, decorator: &'a T) -> Handler<'a, G> {
        decorator.decorate(self)
    }
}

pub struct MultipleDecorators<S, F> 
where
    S: Send + 'static,
    F: Send + 'static,
{
    func: Box<dyn (for<'a> Fn(Handler<'a, S>) -> Handler<'a, F>) + Send + Sync>,
}

impl<E, G> MultipleDecorators<E, G> 
where
    E: Send + 'static,
    G: Send + 'static,
{
    pub fn new<'a, RD>(request_decorator: RD) -> Self 
    where 
        RD: RequestDecorator<E, G> + 'static,
    {
        let func: Box<dyn for<'b> Fn(Handler<'b, E>) -> Handler<'b, G> + Send + Sync> = Box::new(move |handler| request_decorator.decorate(handler));
        MultipleDecorators { func }
    }

    pub fn add<'a, S, RD>(self, request_decorator: RD) -> MultipleDecorators<E, S> 
    where 
        S: Send,
        RD: RequestDecorator<G, S> + 'static,
    {
        let func: Box<dyn for<'b> Fn(Handler<'b, E>) -> Handler<'b, S> + Send + Sync> = Box::new(move |handler| request_decorator.decorate((self.func)(handler)));
        MultipleDecorators { func }
    }
}

impl<E, G> RequestDecorator<E, G> for MultipleDecorators<E, G>
where
    E: Send + 'static,
    G: Send + 'static,
{
    fn decorate<'a>(self: &MultipleDecorators<E, G>, f: Handler<'a, E>) -> Handler<'a, G>
    where
        E: 'a,
        G: 'a,
    {
        (self.func)(f)
    }
}