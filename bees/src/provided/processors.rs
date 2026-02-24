use std::future::ready;

use reqwest::Response;

use crate::endpoint::Process;


pub struct NoOpProcess;
impl Process for NoOpProcess {
    type ProcessOutput = Response;

    fn process(resp: Response) -> impl Future<Output = Self::ProcessOutput> + Send {
        ready(resp)
    }
}

pub struct TextProcess;
impl Process for TextProcess {
    type ProcessOutput = String;

    async fn process(resp: Response) -> Self::ProcessOutput {
        resp.text().await.unwrap()
    }
}

#[cfg(feature = "reqwest-json")]
pub struct JsonProcessor;

#[cfg(feature = "reqwest-json")]
impl Process for JsonProcessor {
    type ProcessOutput = serde_json::Value;

    async fn process(resp: Response) -> Self::ProcessOutput {
        serde_json::from_str(TextProcess::process(resp))
    }
}