use reqwest::Response;
use crate::handlers::Handler;


#[derive(Debug)]
pub struct IntoText;

impl Handler for IntoText {
    type Input = Response;

    type Output = String;

    async fn execute(
        &self,
        req: Self::Input,
    ) -> Self::Output {
        req.text().await.unwrap()
    }
}

#[cfg(feature = "reqwest-json")]
#[derive(Debug)]
pub struct IntoJson;

#[cfg(feature = "reqwest-json")]
impl Handler for IntoJson {
    type Input = Response;
    
    type Output = Result<serde_json::Value, serde_json::Error>;
    
    async fn execute(
        &self,
        input: Self::Input,
    ) -> Self::Output {
        serde_json::from_str(&IntoText::execute(&IntoText, input).await)
    }
}