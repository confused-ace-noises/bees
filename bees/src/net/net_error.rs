use derive_more::{Display, Error, From};

#[derive(Debug, Display, Error, From)]
pub enum NetError {
    ReqwestError(#[error(source)] reqwest::Error),

    NotAValidUrl(#[error(source)] url::ParseError),
}