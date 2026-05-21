pub mod endpoint;
pub mod record;
pub mod capability;
pub mod net;
pub mod handlers;
pub mod utils;
pub mod resources;
pub mod provided;


// half impl of a proc macro i'll make sometime
// TODO: check if this can be used with new handler structure
// #[allow(unused)]
// macro_rules! attach_processor {
//     ($name:ident -> $ret:ty: $($endpoints:ty),+) => {
//         $(
//             impl EndpointProcessor<$ret> for $endpoints {
//                 fn process(&mut self, resp: Response, _: &Self::CallContext) -> impl Future<Output = $ret> {
//                     $name(resp)
//                 }
//             }
//         )+
//     };

//     ($name:ident -> $ret:ty: all) => {
//         impl<E: EndpointInfo> EndpointProcessor<$ret> for E {
//             fn process(&mut self, resp: Response, _: &Self::CallContext) -> impl Future<Output = $ret> {
//                 $name(resp)
//             }
//         }
//     };
// }

#[cfg(feature = "derive")]
pub use bees_macros::*;

pub mod re_exports {
    pub use reqwest;
    pub use url;
    pub use dashmap;
}