use std::sync::LazyLock;

use crate::resources::resource_handler::ResourceManager;
pub mod endpoint;
pub mod record;
pub mod capability;
pub mod net;
pub mod handler;
pub mod utils;
pub mod resources;
pub mod provided;

// pub static RESOURCE_MANAGER: LazyLock<ResourceManager> = LazyLock::new(|| {
//     ResourceManager::new()
// });

// pub fn resource_manager() -> &'static ResourceManager {
//     &RESOURCE_MANAGER
// }

// half impl of a proc macro i'll make sometime
#[allow(unused)]
macro_rules! attach_processor {
    ($name:ident -> $ret:ty: $($endpoints:ty),+) => {
        $(
            impl EndpointProcessor<$ret> for $endpoints {
                fn process(&mut self, resp: Response, _: &Self::CallContext) -> impl Future<Output = $ret> {
                    $name(resp)
                }
            }
        )+
    };

    ($name:ident -> $ret:ty: all) => {
        impl<E: EndpointInfo> EndpointProcessor<$ret> for E {
            fn process(&mut self, resp: Response, _: &Self::CallContext) -> impl Future<Output = $ret> {
                $name(resp)
            }
        }
    };
}

#[cfg(feature = "derive")]
pub use bees_macros::*;

pub mod re_exports {
    pub use reqwest;
    pub use url;
    pub use dashmap;
}