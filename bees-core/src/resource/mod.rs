use crate::context::context;

#[allow(clippy::module_inception)]
mod resource;
pub mod resource_utils;

pub use resource::*;

pub fn res_manager() -> &'static crate::resource::resource::ResourceManager {
    &context().resources
}

// TODO: fix this macro
#[macro_export]
macro_rules! resource {
    // -------- REGISTER --------
    (reg $resource:expr) => {
        $crate::resource::res_manager().add_resource($resource)
    };

    (reg $($resource:expr)+) => {
        (
            $(
                $crate::resource::res_manager().add_resource($resource)
            )+
        )
    };

    // -------- GETTERS --------
    ($resource:expr) => {
        $crate::resource::res_manager().get_resource(::std::convert::AsRef::as_ref($resource)).expect("resource!: tried to access a non-existent resource")
    };

    (option $resource:expr) => {
        $crate::resource::res_manager().get_resource(::std::convert::AsRef::as_ref($resource))
    }
}