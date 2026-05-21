use reqwest::Response;

use crate::{net::{Request, net_error::NetError}, utils::error::Error};
use std::{fmt::Debug, num::NonZeroUsize};

// ######## TRAITS ########
pub trait Handler: Debug + Send {
    type Input;
    type Output;

    fn execute(
        &self,
        input: Self::Input,
    ) -> impl Future<Output = Self::Output> + Send;
}

#[derive(Debug)]
pub struct Chain<A, B>(
    pub A,
    pub B,
)
where
    A: Handler + Sync,
    B: Handler<Input = A::Output> + Sync,

    A::Output: Send,
    A::Input: Send,
    B::Output: Send;

impl<A: Handler, B: Handler> Handler for Chain<A, B>
where 
    A: Handler + Sync,
    B: Handler<Input = A::Output> + Sync,

    A::Output: Send,
    A::Input: Send,
    B::Output: Send
{
    type Input = A::Input;

    type Output = B::Output;

    async fn execute(
        &self,
        input: Self::Input,
    ) -> Self::Output {
        let output = self.0.execute(input).await;
        self.1.execute(output).await
    }
}

#[derive(Debug)]
pub struct TryChain<A, B>(pub A, pub B)
where
    A: Handler + Sync,

    A::Output: Send,
    A::Input: Send;

impl<A: Handler, B: Handler, Ok, Err> Handler for TryChain<A, B>
where 
    A: Handler<Output = Result<Ok, Err>> + Sync,
    B: Handler<Input = Ok> + Sync,

    A::Output: Send,
    A::Input: Send,
    B::Output: Send,
    Err: Send
{
    type Input = A::Input;

    type Output = Result<B::Output, Err>;

    async fn execute(
        &self,
        input: Self::Input,
    ) -> Self::Output {
        let output = self.0.execute(input).await?;
        Ok(self.1.execute(output).await)
    }
}

// #[macro_export]
// macro_rules! chain {
//     ($handler:ty) => {
//         $handler
//     };

//     ($handler:ty, $($remaining:ty),+) => {
//         $crate::handlers::Chain<$handler, $crate::chain!($($remaining),*)>
//     };
// }

// pub trait HandlerWrapper<H: Handler> {
//     type Output: Handler;

//     fn wrap_into(&self, from: H) -> Self::Output;
// }

// pub trait WrapDecorate<H: Handler, W: HandlerWrapper<H>>: Sized {
//     type Output;

//     fn wrap(self, wrapper: W) -> Self::Output;
// }

// impl<H: Handler, W: HandlerWrapper<H>> WrapDecorate<H, W> for H {
//     type Output = W::Output;
    
//     fn wrap(self, wrapper: W) -> Self::Output
//     {
//         wrapper.wrap_into(self)
//     }
    
// }

// ######## BASE_HANDLER ########
#[derive(Debug, Clone)]
pub struct NoRateLimiterBaseHandler;

impl Handler for NoRateLimiterBaseHandler {
    type Output = Result<reqwest::Response, NetError>;
    type Input = Request;

    async fn execute(
        &self,
        req: Self::Input,
    ) -> Self::Output {
        req.client.execute_reqwest_req_no_rate_limit(req.inner).await
    }
}

#[derive(Debug, Clone)]
pub struct BaseHandler;

impl Handler for BaseHandler {
    type Output = Result<reqwest::Response, NetError>;
    type Input = Request;

    async fn execute(
        &self,
        req: Self::Input,
    ) -> Self::Output {
        req.client.execute_reqwest_req(req.inner).await
    }
}



// ######## RETRIES ########
#[derive(Debug)]
pub struct Retries<H: Handler + Sync> { inner: H, n_retries: NonZeroUsize }

#[derive(Debug)]
pub enum RetriesError<E> {
    InnerError(E),
    CouldNotCloneRequest,
}

impl<H, E> Retries<H> 
where 
    E: Debug,
    H: Handler<Input = Request, Output = Result<Response, E>> + Sync
{
    pub fn new(inner: H, n_retries: usize) -> Self {
        assert_ne!(n_retries, 0, "n_retries in Retries<H> must be greater than 0");

        // ? safety: new_unchecked is fine because it's checked above ^
        Self { inner, n_retries: unsafe { NonZeroUsize::new_unchecked(n_retries) } }
    }
}

impl<E, H> Handler for Retries<H> 
where 
    E: Debug,
    H: Handler<Input = Request, Output = Result<Response, E>> + Sync,
{
    type Input = Request;
    type Output = Result<reqwest::Response, RetriesError<E>>;

    async fn execute(&self, req: Self::Input) -> Self::Output {
        // let req = Arc::new(req);
        let n_retries: usize = self.n_retries.into();
        for n in 0..n_retries{
            let Some(req) = req.try_clone() else {
                return Err(RetriesError::CouldNotCloneRequest)
            };

            let x = self.inner.execute(req).await;

            match x {
                Ok(resp) => return Ok(resp),
                Err(e) if n == n_retries => {
                    return Err(RetriesError::InnerError(e));
                },

                _ => continue,
            }
        }

        unreachable!()
    }
}