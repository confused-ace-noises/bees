use thiserror::Error;


#[derive(Debug, Error)]
pub enum NetError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    NotAValidUrl(#[from] url::ParseError),
}