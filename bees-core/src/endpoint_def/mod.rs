#[allow(clippy::module_inception)]
mod endpoint;
pub use endpoint::*;

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
    (noreg $record:expr => new $name:expr, $path:expr, $http_verb:expr, $($func:expr),+ $(,)? $(; [$($capability:expr),*] )? $(,)?) => {
        {
            let mut __endpoint_macro_endpoint_builder = $crate::endpoint_def::Endpoint::builder_template(
                $crate::endpoint_def::EndpointTemplate {
                    record_name: ::std::convert::Into::into($record),
                    name: ::std::convert::Into::into($name),
                    path: ::std::convert::Into::into($path),
                    http_verb: $http_verb,
                    capabilities: ::std::sync::Arc::new([$($($crate::capability::IntoBoxedCapability::into_boxed_capability($capability)),*)?]),
                }
            );

            
            $(
                __endpoint_macro_endpoint_builder.push_endpoint_output($func);
            )+

            __endpoint_macro_endpoint_builder.build()
        }
    };
    
    ($record:expr => new $name:expr, $path:expr, $http_verb:expr, $($func:expr),+ $(,)? $(; [$($capability:expr),*] )? $(,)?) => {
        {
            let __endpoint_macro_record = $crate::record!($record);

            let __endpoint_macro_endpoint = $crate::endpoint!(noreg $record => new 
                $name, 
                $path, 
                $http_verb, 
                $($func),+ 
                $(; [$($capability),*] )?
            );
         
            __endpoint_macro_record.add_endpoint(__endpoint_macro_endpoint.clone());

            __endpoint_macro_endpoint
        }
    };

    (noreg $record:expr =>  $(new $name:expr, $path:expr, $http_verb:expr, $($func:expr),+ $(,)? $(; [$($capability:expr),*] )? ),+ $(,)?) => {
        {
            let __endpoint_macro_record = $crate::record!($record);
            (
                $(
                    $crate::endpoint!(noreg $record => new 
                        $name, 
                        $path, 
                        $http_verb, 
                        $($func),+ 
                        $(; [$($capability),*] )?
                    )
                ),+
            )
        }
    };
    

    ($record:expr =>  $(new $name:expr, $path:expr, $http_verb:expr, $($func:expr),+ $(,)? $(; [$($capability:expr),*] )? ),+ $(,)?) => {
        {
            let __record = $crate::record!($record);
            (
                $(
                    {
                        let __endpoint = $crate::endpoint!(noreg $record => new 
                            $name, 
                            $path, 
                            $http_verb, 
                            $($func),+ 
                            $(; [$($capability),*] )?
                        );

                        __record.add_endpoint(__endpoint.clone());

                        __endpoint
                    }
                ),+
            )
        }
    };

    ($record:expr => $endpoint:expr) => {
        $crate::record_def::record_manager()
            .get_record(::std::convert::AsRef::as_ref($record))
            .expect("endpoint!: tried to access a non-existent record")
            .get_endpoint(::std::convert::AsRef::as_ref($endpoint))
            .expect("endpoint!: tried to access a non-existent endpoint")
    };

    (option $record:expr =>  $endpoint:expr) => {
        $crate::record_def::record_manager()
            .get_record(::std::convert::AsRef::as_ref($record))
            .and_then(|inner| inner.get_endpoint($endpoint))
    };
}

