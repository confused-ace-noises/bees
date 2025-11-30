use std::sync::OnceLock;

use crate::core::context::Context;

pub mod context;
pub mod resource;
pub mod resources_utils;

pub(crate) static CONTEXT: OnceLock<Context> = OnceLock::new();

/// note: this must  be called *AFTER* `net::init_rate_limiter_duration()`
pub(crate) fn init_context() {
    CONTEXT
        .set(Context::new())
        .expect("CONTEXT was already set somehow???")
}

pub fn context() -> &'static Context {
    CONTEXT
        .get()
        .expect("this shoudln't happen. did you remember to init bees (`bees::init()`)?")
}

pub fn client() -> &'static crate::net::client::Client {
    &context().client
}

pub fn res_manager() -> &'static crate::core::resource::ResourceManager {
    &context().resources
}

pub fn records_manager() -> &'static crate::endpoint_record::record::RecordManager {
    &context().records
}

#[macro_export]
macro_rules! resource {
    // -------- REGISTER --------
    (reg $resource:expr) => {
        $crate::core::res_manager().add_resource($resource)
    };

    (reg $($resource:expr)+) => {
        (
            $(
                $crate::core::res_manager().add_resource($resource)
            )+
        )
    };

    // -------- GETTERS --------
    ($resource:expr) => {
        $crate::core::context().resources.get_resource(::std::convert::AsRef::as_ref($resource)).expect("resource!: tried to access a non-existant resource")
    };

    (option $resource:expr) => {
        $crate::core::context().resources.get_resource(::std::convert::AsRef::as_ref($resource))
    }
}
