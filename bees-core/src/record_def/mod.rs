use crate::context::context;

#[allow(clippy::module_inception)]
mod record;

pub use record::*;

pub fn record_manager() -> &'static crate::record_def::record::RecordManager {
    &context().records
}

#[macro_export]
macro_rules! record {
    // Basic constructor with no register and no endpoints
    (noreg $name:expr => $const_url:expr $(; [$( $capability:expr ),+])? $(,)? ) => {
        $crate::record_def::Record::new(
            ::std::convert::Into::into($name),
            ::std::convert::Into::into($const_url),
            ::std::sync::Arc::new([ $( $($crate::capability::IntoBoxedCapability::into_boxed_capability($capability)),+ )? ])
        )
    };

    // register
    ($name:expr => $const_url:expr $(; [$( $capability:expr ),+] )? $(,)? ) => {{
        let __register_record = $crate::record_def::Record::new(
            ::std::convert::Into::into($name),
            ::std::convert::Into::into($const_url),
            ::std::sync::Arc::new([ $( $($crate::capability::IntoBoxedCapability::into_boxed_capability($capability)),+ )? ])
        );

        $crate::record_def::record_manager().add_record(__register_record.clone());

        __register_record
    }};
    // ----- MULTIPLE ------

    ($( $name:expr => $const_url:expr $(; [ $( $capability:expr ),+ ] )? ),+ $(,)?) => { ( $( $crate::record!( $name => $const_url $(; [$($capability),+])? ) ),+ ) };

    (noreg $( $name:expr => $const_url:expr $(; [ $( $capability:expr ),+ ] )? ),+ $(,)?) => { ( $( $crate::record!(noreg $name => $const_url $(; [$($capability),+])? ) ),+ ) };

    // expression lookup
    ($record:expr) => {
        $crate::record_def::record_manager()
            .get_record(::std::convert::AsRef::as_ref($record))
            .expect("record!: tried to access a non-existent record")
    };

    (option $record:expr) => {
        $crate::record_def::record_manager().get_record(::std::convert::AsRef::as_ref($record))
    };
}