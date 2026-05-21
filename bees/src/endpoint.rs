use std::{
    any::TypeId, convert::Infallible, error::Error as StdError, fmt::Debug, future::ready, str::FromStr, sync::{Arc, OnceLock}
};

use dashmap::DashMap;
use reqwest::Response;
use url::Url;

use super::net::net_error::NetError;
use crate::{handlers::BaseHandler, net::HttpMethod, resources::resource_handler::ResourceManager, utils::error::Error};
use crate::{
    capability::Capability,
    handlers::Handler,
    record::Record,
    utils::resource_string::ResourceString,
};

pub trait EndpointInfo: Send + Debug + 'static {
    type Record: Record;
    type CallContext: Send + Sync;

    const PATH: &str;

    fn capabilities(ctx: &mut Self::CallContext) -> Arc<[Box<dyn Capability>]>;
    fn http_method(ctx: &mut Self::CallContext) -> impl Future<Output = HttpMethod> + Send;

    #[allow(unused_variables)]
    fn modify_url(url: Url, ctx: &mut Self::CallContext) -> impl Future<Output = Url> + Send {
        ready(url)
    }
}

pub trait EndpointExt: EndpointInfo {
    fn parsed_path(client: &Arc<ResourceManager>) -> &'static ResourceString;
    fn record_capabilities() -> Arc<[Box<dyn Capability>]>;
    fn full_url(
        res_manager: &Arc<ResourceManager>, 
        ctx: &mut <Self as EndpointInfo>::CallContext,
    ) -> impl Future<Output = Result<Url, Error>> + Send;
}

impl<E: EndpointInfo + 'static> EndpointExt for E {
    
    fn parsed_path(res_manager: &Arc<ResourceManager>) -> &'static ResourceString {
        static CACHE: OnceLock<DashMap<TypeId, &'static ResourceString>> = OnceLock::new();
        let cache = CACHE.get_or_init(DashMap::new);
        
        cache.entry(TypeId::of::<E>())
            .or_insert_with(|| {
                let mut record = Self::Record::SHARED_URL.trim_end_matches("/").to_string();
                let endpoint = Self::PATH.trim_start_matches("/");
                record.push('/');
                record.push_str(endpoint);

                let resource = ResourceString::new_res_manager(res_manager, record);
                Box::leak(Box::new(resource))
            })
            .value()
    }

    async fn full_url(res_manager: &Arc<ResourceManager>, ctx: &mut <Self as EndpointInfo>::CallContext) -> Result<Url, Error> {
        let parsed = Self::parsed_path(res_manager);
        let formatted = &parsed.to_formatted_now().await?;
        println!("formatted: {formatted}");
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

pub type HandlerStackError = Box<dyn StdError + Send + Sync>;

pub trait HandlerStack<O>: EndpointInfo {
    type Handlers: Handler<Input = crate::net::Request, Output = O> + Sync;

    fn handlers(ctx: &mut <Self as EndpointInfo>::CallContext) -> impl Future<Output = Result<Self::Handlers, HandlerStackError>> + Send;
}

impl<E: EndpointInfo> HandlerStack<Result<Response, NetError>> for E {
    type Handlers = BaseHandler;

    fn handlers(_: &mut <Self as EndpointInfo>::CallContext) -> impl Future<Output = Result<Self::Handlers, HandlerStackError>> + Send {
        ready(Ok(BaseHandler))
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