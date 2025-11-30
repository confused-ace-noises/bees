use net::init_rate_limiter_duration;
use std::time::Duration;

use crate::core::init_context;

pub mod core;
pub mod endpoint_record;
pub mod net;

pub use dashmap;

// TODO: fix macros with EndpointTemplate and also maybe a post-response processing function on endpoints?

pub fn init(rate_limiter_duration: Duration) {
    init_rate_limiter_duration(rate_limiter_duration);
    init_context();
}
pub(crate) trait Sealed {}

// #[derive(Debug)]
// struct ResourceExample {
//     ident: String,
// }

// #[async_trait]
// impl Resource for ResourceExample {
//     fn ident(&self) -> &str {
//         &self.ident
//     }

//     async fn data(&self) -> Box<dyn Display> {
//         Box::new("hi im important data".to_string())
//     }
// }


// #[derive(Debug)]
// struct Cap;
// impl Capability for Cap {
//     fn apply(&self, request: net::client::RequestBuilder) -> net::client::RequestBuilder {
//         request.timeout(Duration::from_millis(1000))
//     }
// }

// #[tokio::test]
// pub async fn test() {
//     use crate::core::{records_manager, res_manager};
//     init(Duration::from_millis(1000));

//     res_manager().add_resource(ResourceExample {
//         ident: String::from("Cookie or something idk"),
//     });
//     records_manager().add_record(crate::endpoint_record::record::Record::new(
//         "something".to_string(),
//         "https://example.com/hello/".to_string(),
//     ));
//     let record = record!("something");

//     let endpoint = Endpoint::new(
//         &record,
//         "hi im an endpoint".to_string(),
//         String::from("hi/<hello>").into(),
//         HttpVerb::GET,
//         Arc::new([]),
//     );

//     let endpoint1 = endpoint!("something" => new "endpoint1", "/hi/<endpoint>", HttpVerb::GET; [Cap, Cap]);

//     let endpoint2 = Endpoint::new(
//         &record,
//         "hi im another endpoint".to_string(),
//         String::from("hi/<hello>").into(),
//         HttpVerb::GET,
//         Arc::new([]),
//     );

//     let (x, y, z) = record!("hi" => "https://hi.com"; [endpoint, endpoint2], "hi2" => "https://hi2.com", "hi3" => "https://hi3.com");

//     // let y = record!("hi" => "https://www.google.com/somehting"; [endpoint, endpoint2]);

//     let x = &*(crate::core::records_manager()
//         .get_record_ref("something")
//         .expect("endpoint!: tried to access a non-existant record")); //.get_endpoint("hi im an endpoint").expect("endpoint!: tried to access a non-existant endpoint");

//     // -------
//     let cookie = resource!("Cookie or something idk");
//     let x = resource!(option "I dont exsist").ok_or("error").unwrap_err();
//     // -------

//     let endpoint = endpoint!(option "something" => "hi im an endpoint");

//     assert_eq!(cookie.data().await.to_string(), "hi im important data");
//     assert_eq!(x, "error");
//     println!("All tests passed!");
// }

// #[tokio::test]
// pub async fn test2() {
//     init(Duration::from_millis(1000));
//     let record = record!("hewwo" => "https://askiiart.net/api/");

//     let record2 = record!("hewwo");

//     println!("1: {:#?}, \n 2: {:#?}", record, record2);

//     assert_eq!(record, record2)
// }
