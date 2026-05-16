use std::{
    fmt::Debug, future::ready, str::FromStr, sync::{Arc, OnceLock}
};

use reqwest::Response;
use url::Url;

use super::net::net_error::NetError;
use crate::{handler::BaseHandler, net::HttpMethod, resources::resource_handler::ResourceManager, utils::error::Error};
use crate::{
    capability::Capability,
    handler::Handler,
    record::Record,
    utils::resource_string::ResourceString,
};

pub trait EndpointInfo: Send + Debug {
    type Record: Record;
    type CallContext: Send + Sync;

    const PATH: &str;

    fn capabilities(ctx: &Self::CallContext) -> Arc<[Box<dyn Capability>]>;
    fn http_method(ctx: &Self::CallContext) -> impl Future<Output = HttpMethod> + Send;

    #[allow(unused_variables)]
    fn modify_url(url: Url, ctx: &Self::CallContext) -> impl Future<Output = Url> + Send {
        ready(url)
    }
}

pub trait EndpointExt: EndpointInfo {
    fn parsed_path(client: &Arc<ResourceManager>) -> &'static ResourceString;
    fn record_capabilities() -> Arc<[Box<dyn Capability>]>;
    fn full_url(
        res_manager: &Arc<ResourceManager>, 
        ctx: &<Self as EndpointInfo>::CallContext,
    ) -> impl Future<Output = Result<Url, Error>> + Send;
}

impl<E: EndpointInfo> EndpointExt for E {
    fn parsed_path(res_manager: &Arc<ResourceManager>) -> &'static ResourceString {
        static PARSED: OnceLock<ResourceString> = OnceLock::new();
        PARSED.get_or_init(|| ResourceString::new_res_manager(res_manager, E::PATH))
    }

    async fn full_url(res_manager: &Arc<ResourceManager>, ctx: &<Self as EndpointInfo>::CallContext) -> Result<Url, Error> {
        let parsed = Self::parsed_path(res_manager);
        let formatted = &parsed.to_formatted_now().await?;
        Ok(Self::modify_url(
            Url::from_str(formatted).map_err(NetError::NotAValidUrl)?,
            ctx,
        )
        .await)
    }

    fn record_capabilities() -> Arc<[Box<dyn Capability>]> {
        <<Self as EndpointInfo>::Record as Record>::shared_caps()
    }
}

pub trait HandlerStack<O>: EndpointInfo {
    type Handlers: Handler<Input = crate::net::Request, Output = O>;

    fn handlers(ctx: &<Self as EndpointInfo>::CallContext) -> Self::Handlers;
}

impl<E: EndpointInfo> HandlerStack<Result<Response, Error>> for E {
    type Handlers = BaseHandler;

    fn handlers(_: &<Self as EndpointInfo>::CallContext) -> Self::Handlers {
        BaseHandler
    }
}

// TODO: maybe add this?
// pub trait Pipeline {
//     type Output;
//     type Handlers: Handler<Input = crate::net::Request, Output = Output>;

//     fn handlers() -> Self::Handlers;
// }

// macro_rules! attach_pipeline {
//     ($endpoint:ty => $pipeline:ty) => {
//         impl ::bees::endpoint::HandlerStack<$pipeline::Output> for $endpoint {
//             type Handlers = $pipeline::Handlers;

//             fn execute(_: &<Self as EndpointInfo>::CallContext) -> Self::Handlers {
//                 $pipeline::handlers()
//             }
//         }
//     };
// }