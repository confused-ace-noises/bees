#![allow(unused_parens)]

use std::fmt::Display;

use reqwest::Url;
use tokio::sync::RwLock;

use crate::{core::{client, resource::Resource}, net::client::Client};

#[derive(Debug)]
pub struct UpdatingCookie<Fut> 
where 
    Fut: std::future::Future<Output = String> + Send
{
    pub last_update: RwLock<tokio::time::Instant>,
    pub update_interval: tokio::time::Duration,
    pub cookie_name: String,
    pub cookie_value: RwLock<String>,
    pub credentials: Credentials,
    pub login_url: Url,
    pub update_fn: fn(&Client, &Credentials, Url) -> Fut,
}

impl<Fut: std::future::Future<Output = String> + Send> UpdatingCookie<Fut> {
    pub fn new(credentials: Credentials, login_url: Url, update_fn: fn(&Client, &Credentials, Url) -> Fut) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[async_trait::async_trait]
impl<Fut: std::future::Future<Output = String> + Send + std::fmt::Debug> Resource for UpdatingCookie<Fut> {
    fn ident(&self) ->  &str {
        self.cookie_name.as_str()
    }

    async fn data(&self) -> Box<dyn Display> {
        let client = client();
        if self.last_update.read().await.elapsed() >= self.update_interval {
            let mut cookie_value = self.cookie_value.write().await;
            let mut last_update = self.last_update.write().await;            
            
            // double-check because like this we ensure only one update happens 
            if self.last_update.read().await.elapsed() >= self.update_interval {
                let updated_cookie = (self.update_fn)(client, &self.credentials, self.login_url.clone()).await;

                *cookie_value = updated_cookie;
                *last_update = tokio::time::Instant::now();
            }
        }
        
        Box::new(self.cookie_value.read().await.clone())
    }
}