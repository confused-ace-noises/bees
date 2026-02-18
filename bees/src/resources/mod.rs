pub mod resource_handler;
pub mod resource;
pub mod dyn_resource;

#[macro_export]
macro_rules! resource {
    (new $expr:expr) => {
        $crate::resource_manager().add_resource($expr)
    };

    (dyn new $expr:expr) => {
        $crate::resource_manager().add_dyn_resource($expr)
    };

    ($expr:expr) => {
        $crate::resource_manager().get_resource($expr)
    };
}