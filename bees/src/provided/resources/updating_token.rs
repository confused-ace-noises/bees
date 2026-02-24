use std::fmt::{Display, Debug};
use std::time::{Instant, Duration};
use std::marker::PhantomData;

use async_lock::RwLock;
use crate::endpoint::{EndpointInfo, EndpointProcessor};
use crate::net::{Client, EndpointRunnerError};
use crate::resources::resource::{Resource, ResourceOutput};

#[derive(Debug)]
pub struct UpdatingToken<E>
where
    E: EndpointInfo<CallContext = ()> + EndpointProcessor<Token> + Debug + Send + Sync,
{
    pub client: Client,
    pub ident: String,
    pub update_interval: Duration,

    pub value: RwLock<Token>,
    pub last_update: RwLock<Instant>,
    endpoint: PhantomData<E>,
}

impl<E> UpdatingToken<E> 
where 
    E: EndpointInfo<CallContext = ()> + EndpointProcessor<Token> + Sync,
    E::EndpointHandler: Sync,
{
    pub async fn new(
        ident: impl AsRef<str>,
        update_interval: Duration,
        client: Client,
    ) -> Result<Self, EndpointRunnerError<E::EndpointHandler>> {
        let first_value = client.run_endpoint::<E>().run::<Token>().await?;

        Ok(Self::new_start_with(ident, first_value, update_interval, client))
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

    pub async fn force_update(&self) -> Result<(), EndpointRunnerError<E::EndpointHandler>> {
        let token = self.get_new_token().await?;
        let mut lock_token = self.value.write().await;
        let mut lock_last_update = self.last_update.write().await;

        *lock_last_update = Instant::now();
        *lock_token = token;

        Ok(())
    }

    pub async fn get_new_token(&self) -> Result<Token, EndpointRunnerError<E::EndpointHandler>> {
        self.client.run_endpoint::<E>().run::<Token>().await
    }
}

#[derive(Debug, Clone)]
pub struct Token(pub String);

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<E> Resource for UpdatingToken<E> 
where 
    E: EndpointInfo<CallContext = ()> + EndpointProcessor<Token> + Sync,
    E::EndpointHandler: Sync,
{
    fn ident(&self) -> &str {
        self.ident.as_str()
    }

    fn data<'a>(&'a self) -> ResourceOutput<'a> {
        ResourceOutput::new(async move {
            if self.last_update.read().await.elapsed() >= self.update_interval {
                let mut token_value = self.value.write().await;
                let mut last_update = self.last_update.write().await;

                if self.last_update.read().await.elapsed()>= self.update_interval {
                    let token = self.get_new_token().await.expect("failed to update token");

                    *last_update = Instant::now();
                    *token_value = token; 
                }
            }
            
            Box::new(self.value.read().await.clone()) as Box<dyn Display + Send>
        })
    }
}
