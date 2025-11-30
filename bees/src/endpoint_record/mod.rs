use crate::{endpoint_record::endpoint::Capability};

pub mod endpoint;
pub mod record;
pub mod request_decorator;

#[macro_export]
macro_rules! record {

    // Basic constructor with no register and no endpoints
    (noreg $name:expr => $const_url:expr $(; [$( $capability:expr ),+])? $(,)? ) => {
        $crate::endpoint_record::record::Record::new(
            ::std::convert::Into::into($name),
            ::std::convert::Into::into($const_url),
            ::std::sync::Arc::new([ $( $($crate::endpoint_record::IntoBoxedCapability::into_boxed_capability($capability)),+ )? ])
        )
    };

    // register
    ($name:expr => $const_url:expr $(; [$( $capability:expr ),+] )? $(,)? ) => {{
        let __register_record = $crate::endpoint_record::record::Record::new(
            ::std::convert::Into::into($name),
            ::std::convert::Into::into($const_url),
            ::std::sync::Arc::new([ $( $($crate::endpoint_record::IntoBoxedCapability::into_boxed_capability($capability)),+ )? ])
        );

        $crate::core::records_manager().add_record(__register_record.clone());

        __register_record
    }};
    // ----- MULTIPLE ------

    // TODO: put all of the other branches up to the same standard

    ($( $name:expr => $const_url:expr $(; [ $( $capability:expr ),+ ] )? ),+ $(,)?) => { ( $( $crate::record!( $name => $const_url $(; [$($capability),+])? ) ),+ ) };

    // (noreg $((end $name:expr => $const_url:expr $(, [ $( $capability:expr ),+ ] )? $(; [ $($endpoint:expr),+ $(,)? ])? )),+ $(,)?) => { ( $( $crate::record!(noreg $name => $const_url $(, [$($capability),+])? $(; [$($endpoint),+])? ) ),+ ) };


    // expression lookup
    ($record:expr) => {
        $crate::core::records_manager()
            .get_record(::std::convert::AsRef::as_ref($record))
            .expect("record!: tried to access a non-existent record")
    };

    (option $record:expr) => {
        $crate::core::records_manager().get_record(::std::convert::AsRef::as_ref($record))
    };
}

#[macro_export]
/// # WARNING
///
/// This macro **cannot** reliably distinguish between:
/// - an unboxed `Capability` value, and
/// - a value that is *already* boxed (e.g. `Box<C>`)
///
/// The **only** boxed form the macro can detect is `Box<dyn Capability>`.
/// This is because, strictly speaking, `dyn Capability` does *not* implement
/// `Capability` itself — only `Box<dyn Capability>` does.
///
/// As a result:
///
/// - If you pass an unboxed capability, it will be boxed normally.
/// - If you pass a `Box<dyn Capability>`, it will *not* be re-boxed.
/// - If you pass *any other* boxed capability (e.g. `Box<C>` where `C: Capability`),
///   it will be boxed **again**, producing a `Box<Box<C>>`.  
///
/// This double box will still coerce to `Box<dyn Capability>` correctly, because
/// `Box<T>` implements `Capability` whenever `T` does.
///
/// However, this adds an unnecessary heap indirection and therefore wastes memory.
///
/// **If your capability is already boxed**, and you want to avoid double boxing,
/// make sure you convert it to `Box<dyn Capability>` yourself before passing it
/// to the macro.
macro_rules! endpoint {
    ($record:expr => new $name:expr, $path:expr, $http_verb:expr, $func:expr $(; [$($capability:expr),*] )? $(,)?) => {
        {
            let __endpoint_macro_record = $crate::record!($record);

            let __endpoint_macro_endpoint = $crate::endpoint_record::endpoint::Endpoint::new_template(
                $crate::endpoint_record::endpoint::EndpointTemplate {
                    record_name: ::std::convert::Into::into($record),
                    name: ::std::convert::Into::into($name),
                    path: ::std::convert::Into::into($path),
                    http_verb: $http_verb,
                    capabilities: ::std::sync::Arc::new([$($($crate::endpoint_record::IntoBoxedCapability::into_boxed_capability($capability)),*)?]),
                    endpoint_output: $func,
                }
            );

            __endpoint_macro_record.add_endpoint(__endpoint_macro_endpoint.clone());

            __endpoint_macro_endpoint
        }
    };

    (noreg $record:expr => new $path:expr, $http_verb:expr, $func:expr $(; [$($capability:expr),*] )? $(,)?) => {
        {
            let __endpoint_macro_record = $crate::record!($record);

            $crate::endpoint_record::endpoint::Endpoint::new(&__endpoint_macro_record, ::std::convert::AsRef::as_ref($path), $http_verb, Arc::new<[($($crate::endpoint_record::IntoBoxedCapability::into_boxed_capability($capability)),*)?]>, $func)

            __endpoint_macro_endpoint
        }
    };

    ($record:expr =>  $(new $name:expr, $path:expr, $http_verb:expr, $func:expr $(; [$($capability:expr),*] )? ),+ $(,)?) => {
        {
            let __endpoint_macro_record = $crate::record!($record);
            (
                $(
                    {

                        let __endpoint_macro_endpoint = $crate::endpoint_record::endpoint::Endpoint::new_template(
                            $crate::endpoint_record::endpoint::EndpointTemplate {
                                record_name: ::std::convert::Into::into($record),
                                name: ::std::convert::Into::into($name),
                                path: ::std::convert::Into::into($path),
                                http_verb: $http_verb,
                                capabilities: ::std::sync::Arc::new([$($($crate::endpoint_record::IntoBoxedCapability::into_boxed_capability($capability)),*)?]),
                                endpoint_output: $func,
                            }
                        );

                        __endpoint_macro_record.add_endpoint(__endpoint_macro_endpoint.clone());

                        __endpoint_macro_endpoint
                    }
                ),+
            )
        }
    };

    (noreg $record:expr =>  $(new $name:expr, $path:expr, $http_verb:expr, $func:expr $(; [$($capability:expr),*] )? ),+ $(,)?) => {
        {
            let __endpoint_macro_record = $crate::record!($record);
            (
                $(
                    {

                        let __endpoint_macro_endpoint = $crate::endpoint_record::endpoint::Endpoint::new(&__endpoint_macro_record, ::std::convert::Into::into($name), ::std::convert::Into::into($path), $http_verb, Arc::new([$($($crate::endpoint_record::IntoBoxedCapability::into_boxed_capability($capability)),*)?]), $func);

                        __endpoint_macro_endpoint
                    }
                ),+
            )
        }
    };

    ($record:expr => $endpoint:expr) => {
        $crate::core::records_manager()
            .get_record(::std::convert::AsRef::as_ref($record))
            .expect("endpoint!: tried to access a non-existent record")
            .get_endpoint(::std::convert::AsRef::as_ref(($endpoint)))
            .expect("endpoint!: tried to access a non-existent endpoint")
    };

    (option $record:expr =>  $endpoint:expr) => {
        $crate::core::records_manager().get_record(::std::convert::AsRef::as_ref($record)).and_then(|inner| inner.get_endpoint($endpoint))
    };
}

pub trait IntoBoxedCapability {
    fn into_boxed_capability(self) -> Box<dyn Capability>;
}

impl IntoBoxedCapability for Box<dyn Capability> {
    fn into_boxed_capability(self) -> Box<dyn Capability> {
        self
    }
}

impl<T: Capability + 'static> IntoBoxedCapability for T {
    fn into_boxed_capability(self) -> Box<dyn Capability> {
        Box::new(self) as Box<dyn Capability>
    }
}

// impl<C: Capability> IntoBoxedCapability for Box<C> {

// }

#[test]
fn record_macro_compiles_all_branches() {
    use crate::record;

    // --- WITHOUT ENDPOINTS ---

    // noreg, no caps
    let _r1 = record!(noreg "noreg_no_caps" => "/a");

    // noreg, with caps
    // let _r2 = record!(noreg "noreg_caps" => "/b"; ["cap1", "cap2"]);

    // register, no caps
    let _r3 = record!("reg_no_caps" => "/c");

    // register, with caps
    // let _r4 = record!("reg_caps" => "/d"; ["cap1"]);

    // --- MULTIPLE ---

    let _multiple_reg = record!(
        "m1" => "/m1"; [Box::new(|x: crate::net::client::RequestBuilder| x) as Box<dyn Capability>],
        "m2" => "x"; [Box::new(|x: crate::net::client::RequestBuilder| x) as Box<dyn Capability>]
    );

    // let _multiple_noreg = record!(
    //     noreg
    //     "m3" => "/m3",
    //     "m4" => "/m4", ["cap1"]; ["epB", "epC"]
    // );

    // --- LOOKUPS ---

    let _expr_lookup = record!("reg_no_caps"); // (expr)
    let _expr_option = record!(option "reg_no_caps"); // (option expr)
}
