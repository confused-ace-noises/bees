use std::any::Any;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::endpoint::{EndpointInfo, SupportsOutput};
use crate::handler::Handler;
use crate::net::{Client, EndpointRunnerError};
use crate::resources::resource::{Resource, ResourceResult};
use derive_more::From;
use tokio::sync::RwLock;

#[cfg(not(feature = "async-trait"))]
use crate::resources::resource::Resource;

use std::error::Error as StdError;

type EndpointOutput = Result<Token, Arc<dyn Any + Send + Sync>>;

pub trait CanBeUsed: StdError + Clone {}

impl<T: StdError + Clone> CanBeUsed for T {}

#[derive(Debug)]
pub struct UpdatingToken<E>
where
    E: EndpointInfo<CallContext = ()> + SupportsOutput<EndpointOutput> + Debug + Send + Sync,
{
    pub client: Client,
    pub ident: String,
    pub update_interval: Duration,

    pub value: RwLock<Token>,
    pub last_update: RwLock<Instant>,
    endpoint: PhantomData<E>,
}

#[derive(From)]
pub enum UpdatingTokenError<H: Handler> {
    TokenError(Arc<dyn Any + Send + Sync>),
    EndpointRunnerError(EndpointRunnerError<H>)
}

impl<E> UpdatingToken<E>
where
    E: EndpointInfo<CallContext = ()> + SupportsOutput<EndpointOutput> + Sync,
    E::EndpointHandler: Sync,
{
    pub async fn new(
        ident: impl AsRef<str>,
        update_interval: Duration,
        client: Client,
    ) -> Result<Self, UpdatingTokenError<E::EndpointHandler>> {
        let first_value = client.run_endpoint::<E>().run::<EndpointOutput>().await??;

        Ok(Self::new_start_with(
            ident,
            first_value,
            update_interval,
            client,
        ))
    }

    pub fn new_start_with(
        ident: impl AsRef<str>,
        starting_value: Token,
        update_interval: Duration,
        client: Client,
    ) -> Self {
        Self {
            client,
            ident: ident.as_ref().to_string(),
            update_interval,
            value: RwLock::new(starting_value),
            last_update: RwLock::new(Instant::now()),
            endpoint: PhantomData,
        }
    }

    pub async fn force_update(&self) -> Result<(), UpdatingTokenError<E::EndpointHandler>> {
        let token = self.get_new_token().await??;
        let mut lock_token = self.value.write().await;
        let mut lock_last_update = self.last_update.write().await;

        *lock_last_update = Instant::now();
        *lock_token = token;

        Ok(())
    }

    pub async fn get_new_token(
        &self,
    ) -> Result<EndpointOutput, EndpointRunnerError<E::EndpointHandler>> {
        self.client
            .run_endpoint::<E>()
            .run::<EndpointOutput>()
            .await
    }

    pub async fn is_expired(&self) -> bool {
        self.last_update.read().await.elapsed() >= self.update_interval
    }

    pub async fn get_token(&self) -> ResourceResult {
        if self.is_expired().await {
            let mut token_value = self.value.write().await;
            let mut last_update = self.last_update.write().await;

            if self.is_expired().await {
                let possible_token = self.get_new_token().await;

                match possible_token {
                    Ok(Ok(token)) => {
                        *last_update = Instant::now();
                        *token_value = token;
                    }
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                }
            }
        }

        let read_value = self.value.read().await;

        Ok(Box::new(read_value.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Token(pub String);

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(not(feature = "async-trait"))]
impl<E> Resource for UpdatingToken<E>
where
    E: EndpointInfo<CallContext = ()> + SupportsOutput<EndpointOutput> + Sync,
    E::EndpointHandler: Sync,
{
    fn ident(&self) -> &str {
        self.ident.as_str()
    }

    fn data<'a>(&'a self) -> ResourceOutput<'a> {
        ResourceOutput::new(self.get_token())
    }
}

#[cfg(feature = "async-trait")]
#[async_trait::async_trait]
impl<E> Resource for UpdatingToken<E>
where
    E: EndpointInfo<CallContext = ()> + SupportsOutput<EndpointOutput> + Sync,
    E::EndpointHandler: Sync,
{
    fn ident(&self) -> &str {
        self.ident.as_str()
    }

    async fn data(&self) -> ResourceResult {
        self.get_token().await
    }
}
