use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::endpoint::{EndpointInfo, HandlerStack};
use crate::net::Client;
use crate::resources::resource::{Resource, ResourceResult};
use crate::utils::error::Error;
use tokio::sync::RwLock;

#[cfg(not(feature = "async-trait"))]
use crate::resources::resource::ResourceOutput;

use std::error::Error as StdError;

type EndpointOutput<Err> = Result<Token, Err>;

pub trait CanBeUsed: StdError + Clone {}

impl<T: StdError + Clone> CanBeUsed for T {}

#[derive(Debug)]
pub struct UpdatingToken<E, Err: Debug + Send + Sync>
where
    E: EndpointInfo<CallContext = ()> + HandlerStack<EndpointOutput<Err>> + Debug + Send + Sync,
{
    pub client: Client,
    pub ident: String,
    pub update_interval: Duration,

    pub value: RwLock<Token>,
    pub last_update: RwLock<Instant>,
    endpoint: PhantomData<(E, Err)>,
}

#[derive(Debug)]
pub enum UpdatingTokenError<Err> {
    TokenError(Err),
    BeesError(Error)
}

impl<Err> From<Error> for UpdatingTokenError<Err> {
    fn from(value: Error) -> Self {
        UpdatingTokenError::BeesError(value)
    }
}

impl<E, Err> UpdatingToken<E, Err>
where
    E: EndpointInfo<CallContext = ()> + HandlerStack<EndpointOutput<Err>> + Sync,
    E::Handlers: Sync,
    Err: Debug + Send + Sync
{
    pub async fn new(
        ident: impl AsRef<str>,
        update_interval: Duration,
        client: Client,
    ) -> Result<Self, UpdatingTokenError<Err>> {
        let first_value = client.run_endpoint::<E, EndpointOutput<Err>>().run().await?;

        let first_value = match first_value {
            Ok(ok) => ok,
            Err(e) => return Err(UpdatingTokenError::TokenError(e)),
        };

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

    pub async fn force_update(&self) -> Result<(), UpdatingTokenError<Err>> {
        let token = self.get_new_token().await?;

        let token = match token {
            Ok(ok) => ok,
            Err(e) => return Err(UpdatingTokenError::TokenError(e))
        };

        let mut lock_token = self.value.write().await;
        let mut lock_last_update = self.last_update.write().await;

        *lock_last_update = Instant::now();
        *lock_token = token;

        Ok(())
    }

    pub async fn get_new_token(
        &self,
    ) -> Result<EndpointOutput<Err>, Error> {
        self.client
            .run_endpoint::<E, EndpointOutput<Err>>()
            .run()
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

        Ok(read_value.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Token(pub Arc<String>);

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(not(feature = "async-trait"))]
impl<E, Err> Resource for UpdatingToken<E, Err>
where
    E: EndpointInfo<CallContext = ()> + HandlerStack<EndpointOutput<Err>> + Sync,
    E::Handlers: Sync,
    Err: Debug + Send + Sync
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
    E: EndpointInfo<CallContext = ()> + HandlerStack<EndpointOutput> + Sync,
    E::Handlers: Sync,
{
    fn ident(&self) -> &str {
        self.ident.as_str()
    }

    async fn data(&self) -> ResourceResult {
        self.get_token().await
    }
}
