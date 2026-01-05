#[allow(clippy::module_inception)]
mod request_decorator;
pub mod retries;
pub mod multiple_decorators;

pub use request_decorator::*;

#[macro_export]
macro_rules! handler_helper {
    ($($transfer:ident),* $(,)?; async move |$ident:ident $(: $in_ty:ty)?| $expr:expr) => {
        {
            ::std::sync::Arc::new(
                move |$ident $(: $in_ty)?| {
                    $(
                        let $transfer = $transfer.clone();
                    )*
                    Box::pin(
                        async move {
                            $expr
                        }
                    )
                }
            )
        }
    };

    ($($transfer:ident),*; async move |$ident:ident $(: $in_ty:ty)?| -> $out_ty:ty {$expr:expr}) => {
        {
            ::std::sync::Arc::new(
                move |$ident $(: $in_ty)?| -> $out_ty {
                    $(
                        let $transfer = $transfer.clone();
                    )*
                    Box::pin(
                        async move {
                            $expr
                        }
                    )
                }
            )
        }
    };
}
