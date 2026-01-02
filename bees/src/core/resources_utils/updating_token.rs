#![allow(unused_parens)]

use std::{collections::HashMap, error::Error, fmt::Display};

use tokio::{sync::RwLock, time::Instant};

use crate::{
    core::{client, resource::Resource},
    endpoint,
    endpoint_record::{
        endpoint::Endpoint,
    },
};

#[derive(Debug)]
pub struct UpdatingToken {
    pub last_update: RwLock<tokio::time::Instant>,
    pub update_interval: tokio::time::Duration,
    pub name: String,
    pub value: RwLock<Token>,
    pub endpoint: Endpoint,
    pub parse_values: HashMap<String, String>,
    pub query_values: Vec<(String, Option<String>)>,
}

impl UpdatingToken {
    pub async fn new(
        record: impl AsRef<str>,
        endpoint_name: impl AsRef<str>,

        name: String,
        update_interval: tokio::time::Duration,

        parse_values: HashMap<String, String>,
        query_values: Vec<(String, Option<String>)>,
    ) -> Result<Self, Box<dyn Error>> {
        let endpoint = endpoint!(&record => &endpoint_name);
        let client = client();
        let out = client
            .run_endpoint(endpoint.clone(), &parse_values, &query_values)
            .run::<Token>()
            .await?;

        Ok(Self {
            last_update: RwLock::new(Instant::now()),
            update_interval,
            name,
            value: RwLock::new(out),
            endpoint,
            parse_values,
            query_values,
        })
    }

    pub fn new_starting_value(
        record: impl AsRef<str>,
        endpoint_name: impl AsRef<str>,

        name: String,
        update_interval: tokio::time::Duration,

        parse_values: HashMap<String, String>,
        query_values: Vec<(String, Option<String>)>,
        start_value: Token,
    ) -> Self {
        Self {
            last_update: RwLock::new(Instant::now()),
            update_interval,
            name,
            value: RwLock::new(start_value),
            endpoint: endpoint!(&record => &endpoint_name),
            parse_values,
            query_values,
        }
    }   

    pub async fn update(&self) -> Result<Token, Box<dyn Error>> {
        let client = client();
        client
            .run_endpoint(
                self.endpoint.clone(),
                &self.parse_values,
                &self.query_values,
            )
            .run::<Token>()
            .await
    }
}

#[derive(Debug, Clone)]
pub struct Token(pub String);

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait::async_trait]
impl Resource for UpdatingToken {
    fn ident(&self) -> &str {
        self.name.as_str()
    }

    async fn data(&self) -> Box<dyn Display> {
        if self.last_update.read().await.elapsed() >= self.update_interval {
            let mut token_value = self.value.write().await;
            let mut last_update = self.last_update.write().await;

            // double-check because like this we ensure only one update happens
            if self.last_update.read().await.elapsed() >= self.update_interval {
                let updated_token = self.update().await.expect("failed to update token");

                *token_value = updated_token;
                *last_update = tokio::time::Instant::now();
            }
        }

        Box::new(self.value.read().await.clone())
    }
}
