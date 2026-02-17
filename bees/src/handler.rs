use std::sync::LazyLock;

use crate::{net::{Client, Request}, utils::error::Error};


// ######## TRAITS ########
pub trait Handler {
    type Error; 

    fn execute(
        &self,
        req: Request,
    ) -> impl Future<Output = Result<reqwest::Response, Self::Error>>;
}

pub trait HandlerWrapper<H: Handler> {
    type Output: Handler;

    fn wrap_into(&self, from: H) -> Self::Output;
}

pub trait WrapDecorate<H: Handler, W: HandlerWrapper<H>>: Sized {
    type Output;

    fn wrap(self, wrapper: W) -> Self::Output;
}

impl<H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W> for H {
    type Output = W::Output;
    
    fn wrap(self, wrapper: W) -> Self::Output
    {
        wrapper.wrap_into(self)
    }
    
}

// ######## BASE_HANDLER ########
#[derive(Debug, Clone)]
pub struct BaseHandler {
    client: Client,
}

impl BaseHandler {
    pub fn new(client: Client) -> Self {
        Self {
            client
        }
    }
}

// impl Default for BaseHandler {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl Handler for BaseHandler {
    type Error = Error;

    async fn execute(&self, req: Request) -> Result<reqwest::Response, Error> {
        self.client.execute(req).await
    }
}


// ######## RETRIES ########
pub struct Retries<H: Handler, const N: usize> { inner: H }

pub enum RetriesError<E> {
    InnerError(E),
    CouldNotCloneRequest,
}

impl<H: Handler, const N: usize> Retries<H, N> {
    pub fn new(inner: H) -> Self {
        Self { inner }
    }
}

impl<E, H: Handler<Error = E>, const N: usize> Handler for Retries<H, N> {
    type Error = RetriesError<E>;

    async fn execute(&self, req: Request) -> Result<reqwest::Response, Self::Error> {
        const { assert!(N > 0, "`N` in Retries<H, N> must be greater than 0"); };

        // let req = Arc::new(req);
        for n in 0..N {
            let Some(req) = req.try_clone() else {
                return Err(RetriesError::CouldNotCloneRequest)
            };

            let x = self.inner.execute(req).await;

            match x {
                Ok(resp) => return Ok(resp),
                Err(e) if n == N-1 => {
                    return Err(RetriesError::InnerError(e));
                },

                _ => continue,
            }
        }

        unreachable!()
    }
}

pub struct RetriesWrapper<const N: usize>;

impl<H: Handler, const N: usize> HandlerWrapper<H> for RetriesWrapper<N> {
    type Output = Retries<H, N>;

    fn wrap_into(&self, from: H) -> Self::Output {
        Retries { inner: from }
    }
}